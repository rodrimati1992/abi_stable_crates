use super::*;

use abi_stable::library::RootModule;

use serde_json::value::RawValue;

fn setup() {
    let _ = TextOpsMod_Ref::load_module_with(|| Ok::<_, ()>(instantiate_root_module()));
}

#[test]
fn test_reverse_lines() {
    let mut state = new();
    assert_eq!(
        &*reverse_lines(&mut state, "hello\nbig\nworld".into()),
        "world\nbig\nhello\n"
    );
}

#[test]
fn test_remove_words() {
    let mut state = new();
    {
        let words = ["burrito", "like", "a"];
        let mut iter = words.iter().cloned().map(RCow::from);
        let param = RemoveWords {
            string: "Monads are like a burrito wrapper.".into(),
            words: DynTrait::from_borrowing_ptr(&mut iter),
        };
        assert_eq!(&*remove_words(&mut state, param), "Monads are wrapper.");
    }
    {
        let words = ["largest", "is"];
        let mut iter = words.iter().cloned().map(RCow::from);
        let param = RemoveWords {
            string: "The   largest planet  is    jupiter.".into(),
            words: DynTrait::from_borrowing_ptr(&mut iter),
        };
        assert_eq!(&*remove_words(&mut state, param), "The   planet  jupiter.");
    }
}

#[test]
fn deserializing() {
    setup();

    let json = r#"
        {
            "processed_bytes":101
        }
    "#;

    let rvref = serde_json::from_str::<&RawValue>(json).unwrap();
    let value0 = TOStateBox::deserialize_from_proxy(rvref.into()).unwrap();

    let value1 = serde_json::from_str::<TOStateBox>(&json).unwrap();

    assert_eq!(value0, value1);
}

#[test]
fn serializing() {
    setup();

    let this = TextOperationState {
        processed_bytes: 1337,
    }
    .piped(TOStateBox::from_value);

    let serialized_0 = this
        .serialize_into_proxy()
        .unwrap()
        .get()
        .split_whitespace()
        .collect::<String>();

    let expected_0 = r#"{"processed_bytes":1337}"#;

    assert_eq!(serialized_0, expected_0);

    assert_eq!(serde_json::to_string(&this).unwrap(), expected_0,);
}
