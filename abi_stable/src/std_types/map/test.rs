use super::*;

use std::str::FromStr;

use crate::std_types::RString;


fn new_stdmap()->HashMap<u32,u32>{
    vec![
        (90,40),
        (10,20),
        (88,30),
        (77,22),
    ].into_iter()
     .collect()
}

fn new_map<K,V>()->RHashMap<K,V>
where
    K:FromStr+Hash+Eq,
    V:FromStr,
    K::Err:Debug,
    V::Err:Debug,
{
    vec![
        ("90","40"),
        ("10","20"),
        ("88","30"),
        ("77","22"),
    ].into_iter()
     .map(|(k,v)| (k.parse::<K>().unwrap(),v.parse::<V>().unwrap()) )
     .collect()
}


#[test]
fn test_new_map(){
    let mut map=RHashMap::new();
    map.insert(10,100);
    assert_eq!(map.get(&10), Some(&100));
}

#[test]
fn insert(){
    let mut map=RHashMap::<String,_>::new();
    map.insert("what".into(),10);
    map.insert("the".into(),5);

    assert_eq!(
        map.insert("what".into(),33),
        RSome(10),
    );
    assert_eq!(
        map.insert("the".into(),77),
        RSome(5),
    );

}


#[test]
fn remove(){
    let mut map=RHashMap::<String,_>::new();
    map.insert("what".into(),10);
    map.insert("the".into(),5);
    map.insert("is".into(),14);
    map.insert("that".into(),54);

    assert_eq!(
        map.remove_entry("the"),
        RSome(Tuple2("the".to_string(),5)),
    );
    assert_eq!(
        map.remove_entry("the"),
        RNone,
    );
    
    assert_eq!(
        map.remove_entry_p(&"what".into()),
        RSome(Tuple2("what".to_string(),10)),
    );
    assert_eq!(
        map.remove_entry_p(&"what".into()),
        RNone,
    );
    
    assert_eq!(
        map.remove_entry_p(&"is".into()),
        RSome(Tuple2("is".to_string(),14)),
    );
    assert_eq!(
        map.remove_entry_p(&"is".into()),
        RNone,
    );

    assert_eq!(
        map.remove_entry("that"),
        RSome(Tuple2("that".to_string(),54)),
    );
    assert_eq!(
        map.remove_entry("that"),
        RNone,
    );

}


fn check_get<K,V>(map:&mut RHashMap<K,V>,key:K,value:Option<V>)
where
    K:Eq+Hash+Clone+Debug,
    V:PartialEq+Clone+Debug,
{
    assert_eq!(map.get(&key).cloned(),value.clone());
    assert_eq!(map.get_p(&key).cloned(),value.clone());
    assert_eq!(map.get_mut(&key).cloned(),value.clone());
    assert_eq!(map.get_mut_p(&key).cloned(),value.clone());
    
    assert_eq!(map.contains_key(&key),value.is_some(),"\nkey:{:?} value:{:?}\n",key,value );
    assert_eq!(map.contains_key_p(&key),value.is_some(),"\nkey:{:?} value:{:?}\n",key,value );
    
    if let Some(mut value)=value.clone() {
        assert_eq!(&map[&key],&value);
        assert_eq!(map.index_p(&key),&value);
    
        assert_eq!((&mut map[&key]),&mut value);
        assert_eq!(map.index_mut_p(&key),&mut value);
    }
    
}

#[test]
fn get(){
    let mut map=RHashMap::<String,_>::new();
    map.insert("what".into(),10);
    map.insert("the".into(),5);
    map.insert("oof".into(),33);
    map.insert("you".into(),55);

    check_get(&mut map,"what".into(),Some(10));
    check_get(&mut map,"the".into(),Some(5));
    check_get(&mut map,"oof".into(),Some(33));
    check_get(&mut map,"you".into(),Some(55));

    check_get(&mut map,"wasdat".into(),None);
    check_get(&mut map,"thasdae".into(),None);
    check_get(&mut map,"ofwwf".into(),None);
    check_get(&mut map,"youeeeee".into(),None);

    if let Some(x)=map.get_mut("what") {
        *x=*x*2;
    }
    if let Some(x)=map.get_mut("the") {
        *x=*x*2;
    }
    if let Some(x)=map.get_mut("oof") {
        *x=*x*2;
    }
    if let Some(x)=map.get_mut("you") {
        *x=*x*2;
    }

    assert_eq!(map.get("what"),Some(&20));
    assert_eq!(map.get("the"),Some(&10));
    assert_eq!(map.get("oof"),Some(&66));
    assert_eq!(map.get("you"),Some(&110));

}




#[test]
fn clear(){
    let mut map=RHashMap::<String,_>::new();
    map.insert("what".into(),10);
    map.insert("the".into(),5);
    map.insert("oof".into(),33);
    map.insert("you".into(),55);

    assert_eq!(map.get("what"),Some(&10));
    assert_eq!(map.get("the"),Some(&5));
    assert_eq!(map.get("oof"),Some(&33));
    assert_eq!(map.get("you"),Some(&55));

    map.clear();

    assert_eq!(map.get("what"),None);
    assert_eq!(map.get("the"),None);
    assert_eq!(map.get("oof"),None);
    assert_eq!(map.get("you"),None);
}



#[test]
fn len_is_empty(){
    let mut map=RHashMap::<String,_>::new();

    assert!(map.is_empty());
    assert_eq!(map.len(),0);
    
    map.insert("what".into(),10);
    assert!(!map.is_empty());
    assert_eq!(map.len(),1);
    
    map.insert("the".into(),5);
    assert!(!map.is_empty());
    assert_eq!(map.len(),2);
    
    map.insert("oof".into(),33);
    assert!(!map.is_empty());
    assert_eq!(map.len(),3);
    
    map.insert("you".into(),55);
    assert!(!map.is_empty());
    assert_eq!(map.len(),4);

    map.clear();
    
    assert!(map.is_empty());
    assert_eq!(map.len(),0);
}



#[test]
fn from_hashmap(){
    let mut stdmap=new_stdmap();

    let mut map:RHashMap<u32,u32>=stdmap.clone().into();

    assert_eq!(map.len(), 4);
    
    for Tuple2(key,val) in map.drain() {
        assert_eq!(stdmap.remove(&key),Some(val),"key:{:?} value:{:?}",key,val);
    }
    assert_eq!(stdmap.len(), 0);

    assert!(map.is_empty(),"map length:{:?}",map.len());

}


#[test]
fn into_hashmap(){
    let stdmap=new_stdmap();

    let map:RHashMap<u32,u32>=stdmap.clone().into();

    let stdmap2:HashMap<_,_>=map.into();

    assert_eq!(stdmap2,stdmap);
}


#[test]
fn from_iter(){
    let mut stdmap=new_stdmap();

    let map:RHashMap<u32,u32>=stdmap.clone().into_iter().collect();

    assert_eq!(map.len(), 4);
    
    for Tuple2(key,val) in map.iter() {
        assert_eq!(stdmap.remove(&key).as_ref(),Some(val),"key:{:?} value:{:?}",key,val);
    }
    assert_eq!(stdmap.len(), 0);

}


#[test]
fn into_iter(){
    let mut stdmap=new_stdmap();

    let map:RHashMap<u32,u32>=stdmap.clone().into_iter().collect();

    assert_eq!(map.len(), 4);
    
    for Tuple2(key,val) in map.into_iter() {
        assert_eq!(stdmap.remove(&key).as_ref(),Some(&val),"key:{:?} value:{:?}",key,val);
    }
    assert_eq!(stdmap.len(), 0);

}


#[test]
fn iter_mut(){
    let mut stdmap=new_stdmap();
    let mut map:RHashMap<_,_>=new_stdmap().into();

    for Tuple2(key,val) in map.iter_mut() {
        assert_eq!(stdmap.remove(&*key).as_ref(),Some(&*val),"key:{:?} value:{:?}",key,val);
        *val=*val+key;
    }
    assert_eq!(stdmap.len(), 0);

    assert_eq!(map.get(&90),Some(&130));
    assert_eq!(map.get(&10),Some(&30));
    assert_eq!(map.get(&88),Some(&118));
    assert_eq!(map.get(&77),Some(&99));
}


fn assert_is_occupied<K,V>(map:&mut RHashMap<K,V>,k:K,mut v:V)
where
    K:Eq+Hash+Clone+Debug,
    V:Clone+Debug+PartialEq,
{
    let mut entry=map.entry(k.clone());
    assert_matches!(REntry::Occupied{..}=entry);
    assert_eq!(entry.key(), &k);
    assert_eq!(entry.get().cloned(), Some(v.clone()));
    assert_eq!(entry.get_mut().cloned(), Some(v.clone()));
}

fn assert_is_vacant<K,V>(map:&mut RHashMap<K,V>,k:K)
where
    K:Eq+Hash+Clone+Debug,
    V:Clone+Debug+PartialEq,
{
    let mut entry=map.entry(k.clone());
    assert_matches!(REntry::Vacant{..}=entry);
    assert_eq!(entry.key(), &k);
    assert_eq!(entry.get().cloned(), None);
    assert_eq!(entry.get_mut().cloned(), None);
}


#[test]
fn existing_is_occupied(){
    let mut map=new_map::<RString,RString>();

    assert_is_occupied(&mut map,"90".into(),"40".into());
    assert_is_occupied(&mut map,"10".into(),"20".into());
    assert_is_occupied(&mut map,"88".into(),"30".into());
    assert_is_occupied(&mut map,"77".into(),"22".into());

    assert_is_vacant(&mut map,"13".into());
    assert_is_vacant(&mut map,"14".into());
}


#[test]
fn entry_or_insert(){
    let mut map=new_map::<RString,RString>();
    
    assert_is_vacant(&mut map,"12".into());

    assert_eq!(
        *map.entry("12".into()).or_insert("100".into()),
        "100".into_(RString::T)
    );
    assert_is_occupied(&mut map,"12".into(),"100".into());
    
    assert_eq!(
        *map.entry("12".into()).or_insert("105".into()), 
        "100".into_(RString::T)
    );
    assert_is_occupied(&mut map,"12".into(),"100".into());

}


#[test]
fn entry_or_insert_with(){
    let mut map=new_map::<RString,RString>();
    
    assert_is_vacant(&mut map,"12".into());

    assert_eq!(
        *map.entry("12".into()).or_insert_with(||"100".into()),
        "100".into_(RString::T)
    );
    assert_is_occupied(&mut map,"12".into(),"100".into());
    
    assert_eq!(
        *map.entry("12".into()).or_insert_with(||unreachable!()), 
        "100".into_(RString::T)
    );
    assert_is_occupied(&mut map,"12".into(),"100".into());
}


#[test]
fn entry_and_modify(){
    let mut map=new_map::<RString,RString>();
    
    assert_is_vacant(&mut map,"12".into());

    assert_matches!(
        REntry::Vacant{..}=
            map.entry("12".into()).and_modify(|_| unreachable!() )
    );

    assert_eq!(
        *map.entry("12".into())
            .and_modify(|_| unreachable!() )
            .or_insert_with(||"100".into()),
        "100".into_(RString::T)
    );
    assert_is_occupied(&mut map,"12".into(),"100".into());
    
    assert_eq!(
        *map.entry("12".into())
            .and_modify(|v| *v="what".into() )
            .or_insert_with(||unreachable!()), 
        "what".into_(RString::T)
    );
    assert_is_occupied(&mut map,"12".into(),"what".into());
}



#[test]
fn entry_or_default(){
    let mut map=new_map::<RString,RString>();
    
    assert_is_vacant(&mut map,"12".into());

    assert_eq!(
        *map.entry("12".into())
            .and_modify(|_| unreachable!() )
            .or_default(),
        "".into_(RString::T)
    );

    assert_eq!(
        *map.entry("12".into())
            .and_modify(|v| *v="hello".into() )
            .or_default(),
        "hello".into_(RString::T)
    );
}
