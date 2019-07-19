use std::{
    fs,
    path::PathBuf,
};


use abi_stable::{
    //abi_stability::check_layout_compatibility,
    reflection::export_module::MRItem,
    library::lib_header_from_path,
};

use core_extensions::SelfOps;

use structopt::StructOpt;


///////////////////////////////////////////////////////////////////////////////


#[derive(StructOpt)]
#[structopt(author="_")]
enum Command {

/**
Extracts the module structure of an abi_stable library,
with the typenames of each function parameter/return type.
*/
    #[structopt(name = "mods")]
    #[structopt(author="_")]
    Modules {
        /// The path to the library.
        library_path:PathBuf,
        
        /// Which file to output the module structure to.
        #[structopt(short = "o",long="out-file")]
        #[structopt(parse(from_os_str))]
        output_file:Option<PathBuf>,
        
        /// Whether to output the module structure to stdout.
        #[structopt(short = "s",)]
        output_stdout:bool,

        /// Whether to outputed json is compact
        #[structopt(long = "--compact",)]
        compact_json:bool
    },
}




fn main() {

    let lib_header=abi_stable::LIB_HEADER;
    let minor_s=lib_header.abi_minor.to_string();

    let short_about=
        "A program to extract a variety of information from an abi_stable dynamic library.";
    let long_about=format!("
{short_about}

This program uses the {major}.{minor} version of abi_stable.

The loaded dynamic library must use a {major}.{dep_minor}.* \
version of abi_stable to be loaded successfully.
",
    short_about=short_about,
    major=lib_header.abi_major,
    minor=lib_header.abi_minor,
    dep_minor=if lib_header.abi_major!=0 { "*" }else{ &*minor_s },
);

    let opts =  Command::clap()
        .about(short_about)
        .about(&*long_about)
        .get_matches()
        .piped_ref(Command::from_clap);

    match opts {
        Command::Modules{library_path,output_file,output_stdout,compact_json}=>{
            let lib_header=lib_header_from_path(library_path.as_ref()).unwrap();

            let abi_info=lib_header.layout().unwrap_or_else(||{
                println!(
                    "The dynamic library does not support reflection:\n    {}",
                    library_path.display(),
                );
                std::process::exit(1);
            });

            let root_mod=MRItem::from_abi_info(abi_info.get().layout);

            let ref json=if compact_json {
                serde_json::to_string(&root_mod).unwrap()
            }else{
                serde_json::to_string_pretty(&root_mod).unwrap()
            };
            
            if let Some(output_file)=&output_file {
                if let Err(e)=fs::write(output_file,json) {
                    panic!(
                        "Error writing to file:\n{}\nError:\n{}\n", 
                        output_file.display(),
                        e,
                    );
                }
            }
            if output_file.is_none() || output_stdout {
                println!("{}", json );
            }
        }
    }


}
