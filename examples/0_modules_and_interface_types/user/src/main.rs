use std::{
    fs,
    path::PathBuf,
    io::{self,BufRead,Write,Read},
};


use core_extensions::SelfOps;

use structopt::StructOpt;

use abi_stable::{
    std_types::{RString,RCow},
    DynTrait,
    library::{development_utils::compute_library_path, RootModule},
};

use example_0_interface::{CowStrIter,TextOpsMod_Ref,RemoveWords,TOStateBox};


mod tests;

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
#[structopt(author="_")]
enum Command {
    #[structopt(name = "reverse-line-order")]
    #[structopt(author="_")]
/**

Reverse the order of all lines from stdin into stdout once stdin disconnects.

Example:

Running this(on linux,don't know how it would work on windows or mac):
```
echo -e "A\nB\nC\nD" | cargo run -- reverse-line-order
```

Outputs this:
```
D
C
B
A
```

*/
    ReverseLineOrder ,

/**

Copies the stdin into stdout,removing the words passed as command line arguments.

Example:
   
Running this  
```
echo "This is an example phrase,try replacing this with some other sentence." | \
cargo run -- remove-words is an try this with 
```
Outputs this:
```
This example phrase,replacing some other sentence.
```

*/
    #[structopt(name = "remove-words")]
    #[structopt(author="_")]
    RemoveWords{
        words:Vec<RString>
    },

    #[structopt(name = "run-tests")]
    #[structopt(author="_")]
    /**

Runs some tests that require a dynamic library.
This is how some integration tests are done,may be replaced with a 
dedicated test suite eventually.
    */
    RunTests,

    /**

Runs some json encoded commands,outputting the json encoded return value to stdout.
The command can come from either from stdin or from a file
For some examples of json commands please look in the `data/` directory.

Examples:
    
    `cargo run -- json-command data/0_reverse_lines.json`
    
    `cargo run -- json-command data/1_remove_words.json`

    `cargo run -- json-command data/2_get_processed_bytes.json`

*/
    #[structopt(name = "json-command")]
    #[structopt(author="_")]
    Json{
        /// The file to load the json command from.
        file:Option<PathBuf>
    },
    /// Does nothing,used to check the startup duration.
    #[structopt(name = "nothing")]
    #[structopt(author="_")]
    Nothing,
}



fn main()-> io::Result<()> {
    let target: &std::path::Path = "../../../target/".as_ref();
    let library_path=compute_library_path::<TextOpsMod_Ref>(target).unwrap();

    let mods=TextOpsMod_Ref::load_from_directory(&library_path)
        .unwrap_or_else(|e| panic!("{}", e) );
    
    let opts =  Command::clap()
        .get_matches()
        .piped_ref(Command::from_clap);

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
                let mut words=words.iter().map(RCow::from);
                let params=RemoveWords{
                    string:line.into(),
                    words:DynTrait::from_borrowing_ptr(&mut words,CowStrIter),
                };

                mods.remove_words()(&mut state,params)
            })?;
        }
        Command::RunTests=>{
            tests::run_dynamic_library_tests(mods);
        }
        Command::Json{file}=>{
            fn run_command(mods:TextOpsMod_Ref,state:&mut TOStateBox,s:&str)->RString{
                let command=serde_json::from_str(s)
                    .unwrap_or_else(|e| panic!("\n{}\n",e) );
                
                let ret=mods.run_command()(state,command);
                serde_json::to_string(&ret)
                    .unwrap_or_else(|e| panic!("\n{}\n",e) )
                    .into()
            }

            match file.as_ref().map(|f| (f,fs::read_to_string(f)) ) {
                Some((_,Ok(file)))=>{
                    println!("{}",run_command(mods,&mut state,&*file));
                }
                Some((path,Err(e)))=>{
                    println!("Could not load file at:\n\t{}\nBecause:\n\t{}",path.display(),e);
                }
                None=>{
                    process_stdin(|line| run_command(mods,&mut state,line) )?;
                }
            }
        }
        Command::Nothing=>{}
    }

    Ok(())
}


