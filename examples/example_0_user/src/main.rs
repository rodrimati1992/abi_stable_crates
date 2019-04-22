use std::{
    fs,
    path::{Path,PathBuf},
    io::{self,BufRead,Write,Read},
    sync::Arc,
};


use core_extensions::SelfOps;

use structopt::StructOpt;

use abi_stable::{
    std_types::{RString,RVec,RArc,RBox},
    library::RootModule,
};

use example_0_interface::{
    TextOpsMod_Prefix,RemoveWords,load_library_in,
    TOCommandBox,TOReturnValueArc,TOStateBox,
};


mod tests;


#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

/// Returns the path the library will be loaded from.
fn compute_library_path()->io::Result<PathBuf>{
    let debug_dir  ="../../target/debug/"  .as_ref_::<Path>().into_(PathBuf::T);
    let release_dir="../../target/release/".as_ref_::<Path>().into_(PathBuf::T);

    let debug_path  =TextOpsMod_Prefix::get_library_path(&debug_dir);
    let release_path=TextOpsMod_Prefix::get_library_path(&release_dir);

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


/// Processes stdin,outputting the transformed line to stdout.
fn process_stdin<F>(mut f:F)->io::Result<()>
where F:FnMut(&str)->RString
{
    let mut line_buffer=String::new();

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    while stdin.read_line(&mut line_buffer)?!=0 {
        let returned=f(&line_buffer);
        line_buffer.clear();
        stdout.write_all(returned.as_bytes())?;
        writeln!(stdout)?;
    }

    Ok(())
}



#[derive(StructOpt)]
enum Command {
    #[structopt(name = "reverse-line-order")]
    /// Reverse the order of all lines from stdin into stdout once stdin disconnects.
    ReverseLineOrder ,

    #[structopt(name = "remove-words")]
    /// Copies the stdin into stdout,removing the words passed as command line arguments.
    RemoveWords{
        words:Vec<RString>
    },

    #[structopt(name = "greet")]
    /// Says `Hello, <name_here>!`
    Greet{
        name:String
    },

    #[structopt(name = "run-tests")]
    /// Runs some tests.
    RunTests,

    /**
Runs some json encoded commands,outputting the json encoded return value to stdout.
The command can come from either from stdin or from a file
For some examples of json commands please look in the `data/` directory.
    **/
    #[structopt(name = "json-command")]
    JsonCommand{
        /// The file to load the json command from.
        file:Option<PathBuf>
    }
}



fn main()-> io::Result<()> {
    let library_path=compute_library_path().unwrap();
    let mods=load_library_in(&library_path)
        .unwrap_or_else(|e| panic!("{}", e) );
    
    let opts = Command::from_args();

    let mut state=mods.new()();

    match opts {
        Command::ReverseLineOrder=>{
            let mut buffer=String::new();
            io::stdin().read_to_string(&mut buffer)?;
            let reversed=
                mods.reverse_lines()(&mut state,buffer.as_str().into());
            io::stdout().write_all(reversed.as_bytes())?;
        }
        Command::RemoveWords{words}=>{
            process_stdin(|line|{
                let params=RemoveWords{
                    string:line.into(),
                    words:words[..].into(),
                };

                mods.remove_words_string()(&mut state,params)
            })?;
        }
        Command::Greet{name}=>{
            mods.hello_world().greeter()(name.as_str().into());
        }
        Command::RunTests=>{
            tests::run_dynamic_library_tests(mods);
        }
        Command::JsonCommand{file}=>{
            fn run_command(mods:&TextOpsMod_Prefix,state:&mut TOStateBox,s:&str)->RString{
                let command=TOCommandBox::deserialize_from_str(s)
                    .unwrap_or_else(|e| panic!("\n{}\n",e) );
                
                let ret=mods.run_command()(state,command);
                ret.serialized()
                    .unwrap_or_else(|e| panic!("\n{}\n",e) )
                    .into_owned()
            }

            match file.as_ref().map(|f| (f,fs::read_to_string(f)) ) {
                Some((_,Ok(file)))=>{
                    println!("{}",run_command(mods,&mut state,&*file));
                }
                Some((path,Err(e)))=>{
                    println!("Could not load file at:\n\t{}\nBecause:\n\t{}",path.display(),e);
                }
                None=>{
                    process_stdin(|line| run_command(mods,&mut state,line) );
                }
            }

        }
    }

    Ok(())
}


