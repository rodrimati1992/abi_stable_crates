#[doc(hidden)]
#[macro_use]
pub mod test_utils;


use core_extensions::StringExt;

#[doc(hidden)]
pub fn mangle_ident<S>(kind:&str,name:S)->String
where S: ::std::fmt::Display
{

    let unmangled=format!("_as.{}.{}",kind,name);

    let mut mangled=String::with_capacity(unmangled.len()*3/2);

    for kv in unmangled.split_while(|c| c.is_alphanumeric() ) {
        if kv.key {
            mangled.push_str(kv.str);
            continue
        }
        for c in kv.str.chars() {
            mangled.push_str(match c {
                '.'=>"_0",
                '_'=>"_1",
                '-'=>"_2",
                '<'=>"_3",
                '>'=>"_4",
                '('=>"_5",
                ')'=>"_6",
                '['=>"_7",
                ']'=>"_8",
                '{'=>"_9",
                '}'=>"_a",
                ' '=>"_b",
                ','=>"_c",
                ':'=>"_d",
                ';'=>"_e",
                '!'=>"_f",
                '#'=>"_g",
                '$'=>"_h",
                '%'=>"_i",
                '/'=>"_j",
                '='=>"_k",
                '?'=>"_l",
                '¿'=>"_m",
                '¡'=>"_o",
                '*'=>"_p",
                '+'=>"_q",
                '~'=>"_r",
                '|'=>"_s",
                '°'=>"_t",
                '¬'=>"_u",
                '\''=>"_x",
                '\"'=>"_y",
                '`'=>"_z",
                c=>panic!("cannot currently mangle the '{}' character.", c),
            });
        }
    }

    mangled
}


/// Gets the name of the static that contains the LibHeader of an abi_stable library.
pub fn mangled_root_module_loader_name()->String{
    mangle_ident("lib_header","root module loader")
}


