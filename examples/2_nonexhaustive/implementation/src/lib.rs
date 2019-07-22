use std::{
    collections::HashMap,
    mem,
};

use abi_stable::{
    external_types::RawValueBox,
    nonexhaustive_enum::{NonExhaustiveFor,NonExhaustive},
    prefix_type::PrefixTypeTrait,
    sabi_trait::prelude::TU_Opaque,
    std_types::{RBox,RBoxError,RResult,RStr,RString,ROk,RErr,RVec},
    export_root_module,
    sabi_extern_fn,
    traits::IntoReprC,
    rtry,
};

use core_extensions::SelfOps;

use example_2_interface::{
    Cents,Command,Command_NE,Error,ItemId,ReturnVal,ReturnVal_NE,Shop,Shop_TO,ShopMod,ShopModVal,
    ParamCreateItem,RetRenameItem,
};


#[export_root_module]
fn instantiate_root_module()->&'static ShopMod{
    ShopModVal {
        new,
        deserialize_command,
        deserialize_ret_val,
        serialize_command,
        serialize_ret_val,
    }.leak_into_prefix()
}


#[sabi_extern_fn]
pub fn new()->Shop_TO<'static,RBox<()>>{
    Shop_TO::from_value(
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
        cmd:Command_NE,
    )->RResult<ReturnVal_NE,NonExhaustiveFor<Error>>{
        use std::collections::hash_map::Entry;

        match cmd.into_enum() {
            Ok(Command::CreateItem(inner_cmd))=>{
                let ParamCreateItem{name,initial_count:count,price}=
                    RBox::into_inner(inner_cmd);

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
                        RetRenameItem{id,old_name,new_name}
                            .piped(RBox::new)
                            .piped(ReturnVal::RenameItem)
                    }
                    None => {
                        return RErr(NonExhaustive::new(Error::ItemIdNotFound{id}));
                    }
                }
            }
            Ok(Command::Many{list})=>{
                let mut ret=RVec::with_capacity(list.len());
                for elem in list {
                    let ret_cmd=rtry!(self.run_command(elem));
                    ret.push(ret_cmd);
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
fn deserialize_command(s:RStr<'_>)->RResult<Command_NE,RBoxError>{
    deserialize_json::<Command>(s.into())
        .map(NonExhaustiveFor::new)
}

#[sabi_extern_fn]
fn deserialize_ret_val(s:RStr<'_>)->RResult<ReturnVal_NE,RBoxError>{
    deserialize_json::<ReturnVal>(s.into())
        .map(NonExhaustiveFor::new)
}


#[sabi_extern_fn]
fn serialize_command(s:&Command_NE)->RResult<RawValueBox,RBoxError>{
    s.as_enum().into_c()
        .map_err(RBoxError::from)
        .and_then(serialize_json)
}

#[sabi_extern_fn]
fn serialize_ret_val(s:&ReturnVal_NE)->RResult<RawValueBox,RBoxError>{
    s.as_enum().into_c()
        .map_err(RBoxError::from)
        .and_then(serialize_json)
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

fn serialize_json<'a, T>(value: &'a T) -> RResult<RawValueBox, RBoxError>
where
    T: serde::Serialize,
{
    match serde_json::to_string::<T>(&value) {
        Ok(v)=>unsafe{ ROk(RawValueBox::from_string_unchecked(v)) }
        Err(e)=>RErr(RBoxError::new(e)),
    }
}



#[cfg(test)]
mod tests{
    use super::*;

    use abi_stable::library::RootModule;

    fn remove_spaces(s:&str)->String{
        s.chars().filter(|x| !x.is_whitespace() ).collect()
    }

    fn setup(){
        let _=ShopMod::load_module_with(|| Ok::<_,()>(instantiate_root_module()) );
    }

    #[test]
    fn serde_roundtrip_command(){
        setup();

        let json_a=r##"{
            "Many":{"list":[
                {"CreateItem":{
                    "name":"Box of Void",
                    "initial_count":10000,
                    "price":{"cents":100}
                }},
                {"AddItem":{
                    "id":{"id":0},
                    "count":10
                }},
                {"RenameItem":{
                    "id":{"id":0},
                    "new_name":"bar"
                }}
            ]}
        }"##;

        let list=vec![
            json_a,
        ];

        for json0 in list.into_iter().map(remove_spaces) {
            let obj0=serde_json::from_str::<Command_NE>(&json0).unwrap();
            
            let mut json1=serde_json::to_string(&obj0).unwrap();
            json1.retain(|x| !x.is_whitespace() );

            let obj1=serde_json::from_str::<Command_NE>(&json1).unwrap();

            assert_eq!(json0, json1);
            assert_eq!(obj0, obj1);
        }
    }

    #[test]
    fn serde_roundtrip_return_val(){
        setup();

        let json_a=r##"{
            "Many":{"list":[
                {"CreateItem":{
                    "count":100,
                    "id":{"id":0}
                }},
                {"AddItem":{
                    "remaining":10,
                    "id":{"id":0}
                }},
                {"RenameItem":{
                    "id":{"id":0},
                    "new_name":"bar",
                    "old_name":"foo"
                }}
            ]}
        }"##;

        let list=vec![
            json_a,
        ];

        for json0 in list.into_iter().map(remove_spaces) {
            let obj0=serde_json::from_str::<ReturnVal_NE>(&json0).unwrap();
            
            let mut json1=serde_json::to_string(&obj0).unwrap();
            json1.retain(|x| !x.is_whitespace() );

            let obj1=serde_json::from_str::<ReturnVal_NE>(&json1).unwrap();

            assert_eq!(json0, json1);
            assert_eq!(obj0, obj1);
        }
    }
}