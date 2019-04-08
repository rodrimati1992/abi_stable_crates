use std::io::{self,BufRead,Write};

use abi_stable::{
    std_types::{RString,RVec},
    library::LibraryTrait,
};

use abi_stable_example_interface::{RemoveWords,TOLib};

use once_cell::{
    sync::Lazy,
    sync_lazy,
};

static TEXT_OPS_LIB:Lazy<&'static TOLib>=sync_lazy!{
    TOLib::new("../target/release/".as_ref())
        .or_else(|_| TOLib::new("../target/debug/".as_ref()) )
        .unwrap()
};


fn main()-> io::Result<()> {
    let deleted_words=::std::env::args_os()
        .skip(1)
        .map(|s|->RString{ s.to_string_lossy().into_owned().into() })
        .collect::<RVec<RString>>();

    let first:Option<&str>=deleted_words.first().map(|x| &**x );

    if first==Some("-h") || first==Some("--help") {
        println!(
"
This program echoes stdin into stdout while deleting the words that were passed as arguments.

`<program_path> the` would delete all instances of `the` in the output.

Example:
`echo 'This is the best thing that has ever existed in the world.' | <program_path> the is in ever`
Outputs:\"This best thing that has existed world.\"

");

        ::std::process::exit(1);
    }

    let lib=&*TEXT_OPS_LIB;
    let mut state=(lib.new)();

    let mut line_buffer=String::new();

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    while stdin.read_line(&mut line_buffer)?!=0 {
        let params=RemoveWords{
            string:line_buffer.as_str().into(),
            words:deleted_words.as_rslice(),
        };

        let replaced=(lib.remove_words_string)(&mut state,params);
        stdout.write_all(replaced.as_bytes())?;
        writeln!(stdout)?;
    }

    Ok(())
}
