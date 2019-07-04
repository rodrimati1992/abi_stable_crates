use std::{
    collections::HashMap,
    mem,
};

use abi_stable::{
    nonexhaustive_enum::{NonExhaustiveFor,NonExhaustive},
    prefix_type::PrefixTypeTrait,
    sabi_trait::prelude::TU_Opaque,
    std_types::{RBox,RBoxError,RResult,RStr,RString,ROk,RErr,RVec},
    export_root_module,
    sabi_extern_fn,
    rtry,
};

use core_extensions::SelfOps;

use example_2_interface::{
    Cents,Command,Error,ItemId,ReturnVal,Shop,Shop_TO,Shop_from_value,ShopMod,ShopModVal,
    SerdeWrapper,
};


#[export_root_module]
fn instantiate_root_module()->&'static ShopMod{
    ShopModVal {
        new,
        deserialize_command,
        deserialize_ret_val,
    }.leak_into_prefix()
}


#[sabi_extern_fn]
pub fn new()->Shop_TO<'static,RBox<()>>{
    Shop_from_value(
        ShopState{
            items_map:HashMap::default(),
            items:Vec::new(),
        },
        TU_Opaque
    )
}



#[derive(Debug,Clone)]
struct ShopState{
    items_map:HashMap<RString,usize>,
    items:Vec<Item>
}

#[derive(Debug,Clone)]
struct Item{
    name:RString,
    id:ItemId,
    price:Cents,
    count:u32,
}



impl Shop for ShopState{
    fn run_command(
        &mut self,
        cmd:NonExhaustiveFor<Command>,
    )->RResult<NonExhaustiveFor<ReturnVal>,NonExhaustiveFor<Error>>{
        use std::collections::hash_map::Entry;

        match cmd.into_enum() {
            Ok(Command::CreateItem{name,initial_count:count,price})=>{
                match self.items_map.entry(name.clone()) {
                    Entry::Occupied(entry)=>{
                        let id=ItemId{id:*entry.get()};
                        return RErr(NonExhaustive::new(Error::ItemAlreadyExists{id,name}))
                    }
                    Entry::Vacant(entry)=>{
                        let id=ItemId{id:self.items.len()};
                        entry.insert(self.items.len());
                        self.items.push(Item{name,id,price,count});
                        ReturnVal::CreateItem{count,id}
                    }
                }
            }
            Ok(Command::DeleteItem{id})=>{
                if id.id < self.items.len() {
                    self.items.remove(id.id);
                    ReturnVal::DeleteItem{id}
                }else{
                    return RErr(NonExhaustive::new(Error::ItemIdNotFound{id}));
                }
            }
            Ok(Command::AddItem{id,count})=>{
                match self.items.get_mut(id.id) {
                    Some(item) => {
                        item.count+=count;
                        ReturnVal::AddItem{remaining:item.count,id}
                    }
                    None => {
                        return RErr(NonExhaustive::new(Error::ItemIdNotFound{id}));
                    }
                }
            }
            Ok(Command::RemoveItem{id,count})=>{
                match self.items.get_mut(id.id) {
                    Some(item) => {
                        let prev_count=item.count;
                        item.count=item.count.saturating_sub(count);
                        ReturnVal::RemoveItem{
                            removed:prev_count-item.count,
                            remaining:item.count,
                            id
                        }
                    }
                    None => {
                        return RErr(NonExhaustive::new(Error::ItemIdNotFound{id}));
                    }
                }
            }
            Ok(Command::RenameItem{id,new_name})=>{
                match self.items.get_mut(id.id) {
                    Some(item) => {
                        let old_name=mem::replace(&mut item.name,new_name.clone());
                        self.items_map.remove(&old_name);
                        self.items_map.insert(new_name.clone(),id.id);
                        ReturnVal::RenameItem{id,old_name,new_name}
                    }
                    None => {
                        return RErr(NonExhaustive::new(Error::ItemIdNotFound{id}));
                    }
                }
            }
            Ok(Command::Many{list})=>{
                let mut ret=RVec::with_capacity(list.len());
                for elem in list {
                    let ret_cmd=rtry!(self.run_command(elem.inner));
                    ret.push(SerdeWrapper::new(ret_cmd));
                }
                ReturnVal::Many{list:ret}
            }
            Ok(Command::__NonExhaustive)=>{
                return 
                    Error::InvalidCommand{
                        cmd:RBox::new(NonExhaustive::new(Command::__NonExhaustive))
                    }.piped(NonExhaustive::new)
                    .piped(RErr);
            }
            Err(e)=>{
                return 
                    Error::InvalidCommand{
                        cmd:RBox::new(e.into_inner())
                    }.piped(NonExhaustive::new)
                    .piped(RErr);
            }
        }.piped(NonExhaustive::new)
            .piped(ROk)
    }
}



#[sabi_extern_fn]
fn deserialize_command(s:RStr<'_>)->RResult<NonExhaustiveFor<Command>,RBoxError>{
    deserialize_json::<Command>(s.into())
        .map(NonExhaustiveFor::new)
}

#[sabi_extern_fn]
fn deserialize_ret_val(s:RStr<'_>)->RResult<NonExhaustiveFor<ReturnVal>,RBoxError>{
    deserialize_json::<ReturnVal>(s.into())
        .map(NonExhaustiveFor::new)
}




fn deserialize_json<'a, T>(s: RStr<'a>) -> RResult<T, RBoxError>
where
    T: serde::Deserialize<'a>,
{
    match serde_json::from_str::<T>(s.into()) {
        Ok(x) => ROk(x),
        Err(e) => RErr(RBoxError::new(e)),
    }
}
