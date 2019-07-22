use std::{
    fs,
    path::{Path,PathBuf},
    io::{self,BufRead,Write,Read},
};


use core_extensions::SelfOps;

use structopt::StructOpt;

use abi_stable::{
    std_types::{RString,ROk,RErr},
    library::RootModule,
};

use example_2_interface::{
    Shop,ShopMod,Command_NE,ReturnVal,
};


/// Returns the path the library will be loaded from.
fn compute_library_path()->io::Result<PathBuf>{
    let debug_dir  ="../../../target/debug/"  .as_ref_::<Path>().into_(PathBuf::T);
    let release_dir="../../../target/release/".as_ref_::<Path>().into_(PathBuf::T);

    let debug_path  =ShopMod::get_library_path(&debug_dir);
    let release_path=ShopMod::get_library_path(&release_dir);

    match (debug_path.exists(),release_path.exists()) {
        (false,false)=>debug_dir,
        (true,false)=>debug_dir,
        (false,true)=>release_dir,
        (true,true)=>{
            if debug_path.metadata()?.modified()? < release_path.metadata()?.modified()? {
                release_dir
            }else{
                debug_dir
            }
        }
    }.piped(Ok)
}




fn main(){
    let library_path=compute_library_path().unwrap();
    let mods=ShopMod::load_from_directory(&library_path)
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


