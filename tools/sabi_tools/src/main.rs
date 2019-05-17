use abi_stable::{
    reflection::export_module::MRItem,
    library::with_layout_from_path,
};

///////////////////////////////////////////////////////////////////////////////


fn main() {

    let path=::std::env::args_os().nth(1)
        .expect("\n\nMust pass the path to the abi_stanle dynamic library\n\n");

    let with_layout=with_layout_from_path(path.as_ref()).unwrap();

    let abi_info=with_layout.layout();

    let root_mod=MRItem::from_abi_info(abi_info.layout);

    let json=serde_json::to_string_pretty(&root_mod).unwrap();
    serde_json::from_str::<MRItem>(&json).unwrap();

    println!("{}", json );
}
