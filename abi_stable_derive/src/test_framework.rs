use std::{
    collections::HashMap,
    fmt::{self,Display},
    ops::Range,
    path::Path,
    rc::Rc,
};

use core_extensions::prelude::*;

use proc_macro2::TokenStream as TokenStream2;

use serde::Deserialize;

mod regex_wrapper;

use self::regex_wrapper::RegexWrapper;


////////////////////////////////////////////////////////////////////////////////


type CompositeString=crate::composite_collections::CompositeString<usize>;


////////////////////////////////////////////////////////////////////////////////



#[derive(Debug,Clone,Deserialize)]
pub(crate) struct TestCase{
    name:String,
    code:String,
    subcase:Vec<Rc<Subcase>>,
}

#[derive(Debug,Clone,Deserialize)]
pub(crate) struct Subcase{
    #[serde(default)]
    replacements:HashMap<RegexWrapper,String>,
    
    /// Searches for these regexes in the output,which must be found for the test to pass.
    #[serde(default)]
    regex_search_for:Vec<RegexWrapper>,
    
    /// Searches for these strings in the output,which must be found for the test to pass.
    #[serde(default)]
    search_for:Vec<String>,

    error_count:usize,
}

impl TestCase{
    pub(crate) fn load(text_case_name:&str)->Vec<TestCase>{
        let path=Path::new("./test_data/").join(format!("{}.ron",text_case_name));
        let file=std::fs::read_to_string(path).unwrap();
        ron::de::from_str(&file).unwrap()
    }

    pub(crate) fn new(name:String,code:String)->Self{
        Self{
            name,
            code,
            subcase:Vec::new(),
        }
    }

    pub(crate) fn add_subcase(mut self,subcase:Subcase)->Self{
        self.subcase.push(Rc::new(subcase));
        self
    }

    pub(crate) fn check_with<F>(&self,mut f:F)->Result<(),TestErrors>
    where
        F:FnMut(&str)->Result<TokenStream2,syn::Error>
    {
        let mut test_errors=TestErrors{
            test_name:self.name.clone(),
            expected:Vec::new(),
        };

        for subcase in &self.subcase {
            let input:String=subcase.replacements
                .iter()
                .fold(self.code.clone(),|input,(regex,replacement)|->String{
                    regex.replace_all(&input,&**replacement).into_owned()
                });
            
            let mut composite_strings=CompositeString::new();

            let result=match f(&input) {
                Ok(x)=>Ok(composite_strings.push_display(&x).into_range()),
                Err(e)=>{
                    e.into_iter()
                        .map(|x|{
                            let _=composite_strings.push_str("\0");
                            composite_strings.push_display(&x).into_range()
                        })
                        .collect::<Vec<Range<usize>>>()
                        .piped(Err)
                },
            };

            let output=composite_strings.into_inner();

            let error_count=match &result {
                Ok(_) => 0,
                Err(x) => x.len(),
            };

            let not_found_regexes=
                subcase.regex_search_for.iter() 
                    .filter(|r| !r.is_match(&output) )
                    .cloned()
                    .collect::<Vec<_>>();

            let not_found_text=
                subcase.search_for.iter()
                    .filter(|s| !output.contains(&**s) )
                    .cloned()
                    .collect::<Vec<_>>();

            let is_success={
                subcase.error_count==error_count&&
                not_found_regexes.is_empty()&&
                not_found_text.is_empty()
            };

            if !is_success {
                test_errors.expected.push(TestErr{
                    input,
                    result,
                    output,
                    not_found_regexes,
                    not_found_text,
                    subcase: subcase.clone(),
                });
            }
        }

        if test_errors.expected.is_empty() {
            Ok(())
        }else{
            Err(test_errors)
        }
    }

    pub fn run_test<F>(list:&[Self],mut f:F)
    where
        F:FnMut(&str)->Result<TokenStream2,syn::Error>
    {
        let mut had_err=false;
        for test_case in list {
            if let Err(e)=test_case.check_with(&mut f) {
                eprintln!("{}",e);
                had_err=true;
            }
        }
        if had_err {
            panic!()
        }
    }
}



////////////////////////////////////////////////////////////////////////////////

#[derive(Debug,Clone)]
pub(crate) struct TestErrors{
    test_name:String,
    expected:Vec<TestErr>,
}

#[derive(Debug,Clone)]
pub(crate) struct TestErr{
    input:String,
    result:Result<Range<usize>,Vec<Range<usize>>>,
    output:String,
    not_found_regexes:Vec<RegexWrapper>,
    not_found_text:Vec<String>,
    subcase:Rc<Subcase>,
}


impl Display for TestErrors{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        writeln!(f,"{0}{0}{0}{0}","--------------------")?;
        writeln!(f,"These cases from test '{}' failed:\n",self.test_name)?;

        for test in &self.expected {
            let output=&test.output;

            writeln!(f,"  Test:")?;

            writeln!(f,"    Test Input:\n{}",test.input.left_padder(6))?;

            for regex in &test.not_found_regexes {
                let regex=format!("\"{}\"",&**regex);
                writeln!(f,"    Expected regex to match:\n{}",regex.left_padder(6))?;
            }
            
            for search_for in &test.not_found_text {
                let search_for=format!("\"{}\"",search_for);
                writeln!(f,"    Expected string to match:\n{}",search_for.left_padder(6))?;
            }

            match &test.result {
                Ok(output_r)=>{
                    writeln!(
                        f,
                        "    Error Count:0    Expected:{}",
                        test.subcase.error_count,
                    )?;
                    writeln!(f,"    Test Output:\n{}",output[output_r.clone()].left_padder(6))?;
                }
                Err(errors)=>{
                    writeln!(
                        f,
                        "    Error Count:{}    Expected:{}",
                        errors.len(),test.subcase.error_count,
                    )?;
                    for err_r in errors {
                        writeln!(f,"      Error:\n{}",output[err_r.clone()].left_padder(8))?;
                    }
                }
            }
            
            writeln!(f)?;
        }

        Ok(())
    }
}