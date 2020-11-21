use std::path::PathBuf;

use abi_stable::{
    std_types::{ROk,RErr},
    library::{development_utils::compute_library_path, RootModule},
};

use example_2_interface::{ShopMod_Ref,Command_NE};


fn main(){
    let target: &std::path::Path = "../../../target/".as_ref();
    let library_path=compute_library_path::<ShopMod_Ref>(target).unwrap();

    let mods=ShopMod_Ref::load_from_directory(&library_path)
        .unwrap_or_else(|e| panic!("{}", e) );

    let config_path=match std::env::args_os().nth(1) {
        Some(os)=>PathBuf::from(os),
        None=>{
            println!(
                "Help:You can pass a configuration's path as a command-line argument."
            );
            PathBuf::from("./data/app_config.json")
        }
    };

    let file=match std::fs::read_to_string(&*config_path) {
        Ok(x)=>x,
        Err(e)=>{
            eprintln!(
                "Could not load the configuration file at:\n\
                \t{}\n\
                Because of this error:\n{}\n", 
                config_path.display(),
                e
            );
            std::process::exit(1);
        }
    };

    let command=match serde_json::from_str::<Command_NE>(&file) {
        Ok(x) => x,
        Err(e) => {
            eprintln!(
                "Could not parse the configuration file at:\n\
                 \t{}\n\
                 Because of this error:\n\
                 {}\n", 
                config_path.display(),
                e
            );
            std::process::exit(1);
        },
    };

    let mut shop_trait_object=mods.new()();

    match shop_trait_object.run_command(command) {
        ROk(ret_val)=>{
            println!("Return value of running command:\n{:#?}\n", ret_val);
        }
        RErr(e)=>{
            eprintln!("Error from running command:\n{:?}\n",e);
            std::process::exit(1);
        }
    }
}


