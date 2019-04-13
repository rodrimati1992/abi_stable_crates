use std::{
    path::{Path,PathBuf},
    io::{self,BufRead,Write,Read},
    sync::Arc,
};


use core_extensions::SelfOps;

use structopt::StructOpt;

use abi_stable::{
    std_types::{RString,RVec,RArc,RBox},
    library::{Library,ModuleTrait,LibraryTrait,LibraryError},
    StableAbi,
    traits::{IntoReprC},
};

use abi_stable_example_interface::{TextOpsMod,Modules,RemoveWords,load_library_in};

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

/// Returns the path the library will be loaded from.
fn compute_library_path()->io::Result<PathBuf>{
    use std::io;

    let debug_dir  ="../target/debug/"  .as_ref_::<Path>().into_(PathBuf::T);
    let release_dir="../target/release/".as_ref_::<Path>().into_(PathBuf::T);

    let debug_path  =TextOpsMod::get_library_path(&debug_dir);
    let release_path=TextOpsMod::get_library_path(&release_dir);

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
    /// Reverse the order of all lines from stdin into stdout once stdin disconnects.
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
    RunTests
}



fn main()-> io::Result<()> {
    let library_path=compute_library_path().unwrap();
    let mods=load_library_in(&library_path)
        .unwrap_or_else(|e| panic!("{}", e) );
    
    let opts = Command::from_args();

    let mut state=(mods.text_operations.new)();

    match opts {
        Command::ReverseLineOrder=>{
            let mut buffer=String::new();
            io::stdin().read_to_string(&mut buffer)?;
            let reversed=
                (mods.text_operations.reverse_lines)(&mut state,buffer.as_str().into(),());
            io::stdout().write_all(reversed.as_bytes())?;
        }
        Command::RemoveWords{words}=>{
            process_stdin(|line|{
                let params=RemoveWords{
                    string:line.into(),
                    words:words[..].into(),
                };

                (mods.text_operations.remove_words_string)(&mut state,params)
            })?;
        }
        Command::Greet{name}=>{
            (mods.hello_world.greeter)(name.as_str().into());
        }
        Command::RunTests=>{
            run_dynamic_library_tests(mods);
        }
    }

    Ok(())
}


/// This tests that a type coming from a dynamic library 
/// cannot be converted back to its std-library equivalent
/// while reusing the heap allocation.
///
/// The reason why they can't reuse the heap allocation is because they might
/// be using a different global allocator that this binary is using.
///
/// There is no way that I am aware to check at compile-time what allocator
/// the type is using,so this is the best I can do while staying safe.
fn run_dynamic_library_tests(mods:&'static Modules){
    test_reverse_lines(mods);
    test_remove_words(mods);

    let val=(mods.hello_world.for_tests)();
    {
        let arc_std=val.arc.piped(RArc::into_arc);
        assert_eq!(Arc::strong_count(&arc_std),1);
        assert_ne!(
            (&*arc_std) as *const _ as usize,
            val.arc_address
        );
    }
    {
        let box_std=val.box_.piped(RBox::into_box);
        assert_ne!(
            (&*box_std) as *const _ as usize,
            val.box_address
        );
    }
    {
        let vec_std=val.vec_.piped(RVec::into_vec);
        assert_ne!(
            vec_std.as_ptr() as usize,
            val.vec_address
        );
    }
    {
        let string_std=val.string.piped(RString::into_string);
        assert_ne!(
            string_std.as_ptr() as usize,
            val.string_address
        );
    }
    
    println!("tests succeeded!");

}


fn test_reverse_lines(mods:&'static Modules) {
    let text_ops=mods.text_operations;

    let mut state = (text_ops.new)();
    assert_eq!(
        &* (text_ops.reverse_lines)(&mut state, "hello\nbig\nworld".into(),()),
        "world\nbig\nhello\n"
    );
}
fn test_remove_words(mods:&'static Modules) {
    let text_ops=mods.text_operations;

    let mut state = (text_ops.new)();
    {
        let words = ["burrito".into_c(), "like".into(),"a".into()];
        let param = RemoveWords {
            string: "Monads are like a burrito wrapper.".into(),
            words: words[..].into_c(),
        };
        assert_eq!(&*(text_ops.remove_words_str)(&mut state, param), "Monads are wrapper.");
    }
    {
        let words = ["largest".into_c(),"is".into()];
        let param = RemoveWords {
            string: "The   largest planet  is    jupiter.".into(),
            words: words[..].into_c(),
        };
        assert_eq!(&*(text_ops.remove_words_str)(&mut state, param), "The   planet  jupiter.");
    }
}
