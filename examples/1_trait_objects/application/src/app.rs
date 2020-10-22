use super::*;

use std::time::{Duration,Instant};

pub struct TheApplication{
    pub(super) plugins:Vec<PluginType>,
    pub(super) state:ApplicationState,
}


pub struct ApplicationState{
    // This is separate so that I can use the address 
    // of a PluginType to find its index inside of plugins.
    pub(super) plugin_ids:Vec<PluginId>,
    pub(super) id_map:HashMap<RString,PluginModAndIndices>,
    pub(super) delayed_commands:VecDeque<DelayedCommand>,
    pub(super) responses:VecDeque<DelayedResponse>,
    pub(super) sender:RSender<AsyncCommand>,
    pub(super) receiver:RReceiver<AsyncCommand>,
    pub(super) last_run_at:Instant,
}



fn print_response(
    plugin_id:&PluginId,
    response:&str,
){
    println!(
        "reponse:\n{}\nfrom:\n    {:?}\n\n",
        response.left_pad(4),
        plugin_id,
    );
}

impl TheApplication{
    /// Runs a command,
    pub fn run_command(&mut self,which_plugin:WhichPlugin,command:RStr<'_>)->Result<(),AppError>{
        let list=self.state.expand_which_plugin(which_plugin)?;
        for index in list {
            let state=Application_TO::from_ptr(&mut self.state,TU_Opaque);
            let plugin=&mut self.plugins[index as usize];
            println!("command:\n{}", command.left_pad(4));
            let resp=plugin.json_command(command,state).into_result()?;
            self.state.register_command_run();
            print_response(&self.state.plugin_ids[index],&resp);
        }
        Ok(())
    }

    pub fn tick(&mut self)->Result<(),AppError>{
        if let Ok(ac)=self.state.receiver.try_recv() {
            self.state.send_command_to_plugin(&ac.from,ac.which_plugin,ac.command).into_result()?;
        }

        if let Some(dc)=self.state.delayed_commands.pop_front() {
            self.run_command_(&dc.from,dc.plugin_index,&dc.command)?;
        }

        let mut responses=mem::replace(&mut self.state.responses,VecDeque::new());
        for DelayedResponse{to,from,response} in responses.drain(..) {
            let response=PluginResponse::owned_response(from,response);
            let state=Application_TO::from_ptr(&mut self.state,TU_Opaque);
            if let RSome(res)=self.plugins[to]
                .handle_response(response, state)
                .into_result()?
            {
                print_response(&res.plugin_id,&res.response);
            }
        }
        self.state.responses=responses;

        Ok(())
    }

    pub fn is_finished(&self)->bool{
        self.state.last_run_at.elapsed()>=Duration::from_secs(5)
    }

    fn run_command_(&mut self,from:&PluginId,to:usize,command:&str)->Result<(),AppError>{
        let state=Application_TO::from_ptr(&mut self.state,TU_Opaque);
        let response=self.plugins[to].json_command(command.into(),state).into_result()?;
        let to=self.state.index_for_plugin_id(from)?;

        self.state.register_command_run();

        let response=DelayedResponse{
            from:self.state.plugin_ids[to].clone(),
            to,
            response,
        };

        self.state.responses.push_back(response);
        Ok(())
    }
}


impl ApplicationState{
    pub(crate) fn new()->Self{
        let (sender,receiver)=crossbeam_channel::unbounded();

        Self{
            plugin_ids:Vec::new(),
            id_map:HashMap::new(),
            delayed_commands:VecDeque::new(),
            responses:VecDeque::new(),
            sender,
            receiver,
            last_run_at:Instant::now(),
        }
    }

    fn register_command_run(&mut self){
        self.last_run_at=Instant::now();
    }

    fn index_for_plugin_id(&self,id:&PluginId)->Result<usize,AppError>{
        self.id_map.get(&*id.named)
            .and_then(|x| x.indices.get(id.instance as usize).cloned() )
            .ok_or_else(|| AppError::InvalidPlugin(WhichPlugin::Id(id.clone())) )
    }

    fn expand_which_plugin(&self,which_plugin:WhichPlugin)->Result<PluginIndices,AppError>{
        match which_plugin {
            WhichPlugin::Id(id)=>{
                self.index_for_plugin_id(&id)
                    .map(|i| PluginIndices::from([i]) )
            },
             WhichPlugin::First{ref named}
            |WhichPlugin::Last{ref named}
            |WhichPlugin::Every{ref named}=>{
                self.id_map.get(&**named)
                    .and_then(|x|{
                        let list=&x.indices;
                        match which_plugin {
                            WhichPlugin::First{..}=>{
                                PluginIndices::from([*list.first()?])
                            },
                            WhichPlugin::Last{..}=>{
                                PluginIndices::from([*list.last()?])
                            },
                            WhichPlugin::Every{..}=>{
                                PluginIndices::from(&**list)
                            },
                            _=>unreachable!(),
                        }.piped(Some)
                    })
                    .ok_or_else(|| AppError::InvalidPlugin(which_plugin.clone()) )
            },
            WhichPlugin::Many(list)=>{
                let mut plugin_indices=PluginIndices::new();
                for elem in list {
                    plugin_indices.extend(self.expand_which_plugin(elem)?);
                }
                Ok(plugin_indices)   
            }
        }
    }
}


impl Application for ApplicationState{
    fn send_command_to_plugin(
        &mut self,
        from:&PluginId,
        which_plugin:WhichPlugin,
        command:RString,
    )->RResult<(),AppError>{
        self.expand_which_plugin(which_plugin)
            .map(|plugin_indices|{
                let command=Arc::new(command);
                for plugin_index in plugin_indices {
                    let from=from.clone();
                    self.delayed_commands.push_back(DelayedCommand{
                        from,
                        plugin_index,
                        command:command.clone(),
                    });
                }
            }).into()
    }

    fn get_plugin_id(&self,which_plugin:WhichPlugin)->RResult<RVec<PluginId>,AppError>{
        self.expand_which_plugin(which_plugin)
            .map(|list|{
                list.into_iter()
                    .map(|i| self.plugin_ids[i].clone() )
                    .collect::<RVec<PluginId>>()
            })
            .into()
    }

    fn sender(&self)->RSender<AsyncCommand>{
        self.sender.clone()
    }

    fn loaded_plugins(&self)->RVec<PluginId>{
        self.plugin_ids.clone().into()
    }
}



////////////////////////////////////////////////////////////////////////////////




