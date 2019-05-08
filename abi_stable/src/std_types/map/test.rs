use super::*;

#[test]
fn new_map(){
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

    assert_eq!(
        map.remove_entry("the"),
        RSome(Tuple2("the".to_string(),5)),
    );
    assert_eq!(
        map.remove_entry("the"),
        RNone,
    );
    assert_eq!(
        map.remove_entry("what"),
        RSome(Tuple2("what".to_string(),10)),
    );
    assert_eq!(
        map.remove_entry("what"),
        RNone,
    );

}


#[test]
fn get(){
    let mut map=RHashMap::<String,_>::new();
    map.insert("what".into(),10);
    map.insert("the".into(),5);
    map.insert("oof".into(),33);
    map.insert("you".into(),55);

    assert_eq!(map.get("what"),Some(&10));
    assert_eq!(map.get("the"),Some(&5));
    assert_eq!(map.get("oof"),Some(&33));
    assert_eq!(map.get("you"),Some(&55));

    assert_eq!(map.contains_key("what"),true);
    assert_eq!(map.contains_key("the"),true);
    assert_eq!(map.contains_key("oof"),true);
    assert_eq!(map.contains_key("you"),true);


    assert_eq!(map.get("wasdat"),None);
    assert_eq!(map.get("thasdae"),None);
    assert_eq!(map.get("ofwwf"),None);
    assert_eq!(map.get("youeeeee"),None);

    assert_eq!(map.contains_key("wasdat"),false);
    assert_eq!(map.contains_key("thasdae"),false);
    assert_eq!(map.contains_key("ofwwf"),false);
    assert_eq!(map.contains_key("youeeeee"),false);


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