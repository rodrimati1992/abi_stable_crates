use std::{
    fmt::{self, Display},
    ops::Range,
    path::Path,
    rc::Rc,
};

use core_extensions::{SelfOps, StringExt};

use proc_macro2::TokenStream as TokenStream2;

use serde::Deserialize;

mod regex_wrapper;
mod text_replacement;
mod vec_from_map;

use self::regex_wrapper::RegexWrapper;
use self::text_replacement::replace_text;
use self::vec_from_map::deserialize_vec_pairs;

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Deserialize)]
pub struct Tests {
    cases: Vec<TestCase>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TestCase {
    name: String,
    code: String,
    subcase: Vec<Rc<Subcase>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Subcase {
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_vec_pairs")]
    replacements: Vec<(String, String)>,

    /// Tries to match all these searches on the string.
    #[serde(default)]
    find_all: Vec<Matcher>,

    /// Tries to find whether there is a match for any of these searches on the string.
    #[serde(default)]
    find_any: Vec<Matcher>,

    error_count: usize,
}

impl Tests {
    pub fn load(text_case_name: &str) -> Tests {
        let path = Path::new("./test_data/").join(format!("{}.ron", text_case_name));
        let file = std::fs::read_to_string(path).unwrap();
        ron::de::from_str(&file).unwrap()
    }

    pub fn run_test<F>(&self, mut f: F)
    where
        F: FnMut(&str) -> Result<TokenStream2, syn::Error>,
    {
        self.run_test_inner(&mut f);
    }
    fn run_test_inner(&self, f: &mut dyn FnMut(&str) -> Result<TokenStream2, syn::Error>) {
        let mut had_err = false;

        let mut input = String::new();

        for test_case in &self.cases {
            let mut test_errors = TestErrors {
                test_name: test_case.name.clone(),
                expected: Vec::new(),
            };

            for subcase in &test_case.subcase {
                replace_text(&test_case.code, &subcase.replacements, &mut input);

                let mut composite_strings = String::new();

                let result = match f(&input) {
                    Ok(x) => Ok(write_display(&mut composite_strings, &x)),
                    Err(e) => e
                        .into_iter()
                        .map(|x| {
                            composite_strings.push('\0');
                            write_display(&mut composite_strings, &x)
                        })
                        .collect::<Vec<Range<usize>>>()
                        .piped(Err),
                };

                let output = composite_strings;

                let error_count = match &result {
                    Ok(_) => 0,
                    Err(x) => x.len(),
                };

                let not_found_all = subcase
                    .find_all
                    .iter()
                    .filter(|s| !s.matches(&output))
                    .cloned()
                    .collect::<Vec<_>>();

                let not_found_any = !subcase.find_any.iter().any(|s| s.matches(&output));

                let is_success = {
                    subcase.error_count == error_count && not_found_all.is_empty() && not_found_any
                };

                if !is_success {
                    test_errors.expected.push(TestErr {
                        input: input.clone(),
                        result,
                        output,
                        not_found_all,
                        not_found_any,
                        subcase: subcase.clone(),
                    });
                }
            }

            if !test_errors.expected.is_empty() {
                eprintln!("{}", test_errors);
                had_err = true;
            }
        }

        if had_err {
            panic!()
        }
    }
}

fn write_display<D>(string: &mut String, disp: &D) -> std::ops::Range<usize>
where
    D: Display,
{
    use std::fmt::Write;
    let start = string.len();
    let _ = write!(string, "{}", disp);
    start..string.len()
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Deserialize)]
pub enum Matcher {
    #[serde(alias = "regex")]
    Regex(RegexWrapper),

    #[serde(alias = "str")]
    Str(String),

    #[serde(alias = "not")]
    Not(Box<Matcher>),
}

impl Matcher {
    fn matches(&self, text: &str) -> bool {
        match self {
            Matcher::Regex(regex) => regex.is_match(text),
            Matcher::Str(find) => text.contains(&*find),
            Matcher::Not(searcher) => !searcher.matches(text),
        }
    }
}

impl Display for Matcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Matcher::Regex(regex) => {
                writeln!(f, "Regex:\n{}", regex.to_string().left_padder(4))
            }
            Matcher::Str(find) => {
                writeln!(f, "String:\n{}", find.to_string().left_padder(4))
            }
            Matcher::Not(searcher) => {
                writeln!(f, "Not:\n{}", searcher.to_string().left_padder(4))
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct TestErrors {
    test_name: String,
    expected: Vec<TestErr>,
}

#[derive(Debug, Clone)]
pub struct TestErr {
    input: String,
    result: Result<Range<usize>, Vec<Range<usize>>>,
    output: String,
    not_found_all: Vec<Matcher>,
    not_found_any: bool,
    subcase: Rc<Subcase>,
}

macro_rules! dashes {
    () => {
        "--------------------"
    };
}
const DASHES: &str = concat!(dashes!(), dashes!(), dashes!(), dashes!());

impl Display for TestErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(DASHES)?;
        writeln!(f, "These cases from test '{}' failed:\n", self.test_name)?;

        for test in &self.expected {
            let output = &test.output;

            writeln!(f, "  Test:")?;

            writeln!(f, "    Test Input:\n{}", test.input.left_padder(6))?;

            if !test.not_found_all.is_empty() {
                writeln!(f, "    Expected all of these to match:")?;

                for search_for in &test.not_found_all {
                    Display::fmt(&search_for.to_string().left_padder(6), f)?;
                    writeln!(f)?;
                }
            }

            if test.not_found_any {
                for search_for in &test.subcase.find_any {
                    Display::fmt(&search_for.to_string().left_padder(6), f)?;
                }
            }

            match &test.result {
                Ok(output_r) => {
                    writeln!(
                        f,
                        "    Error Count:0    Expected:{}",
                        test.subcase.error_count,
                    )?;
                    writeln!(
                        f,
                        "    Test Output:\n{}",
                        output[output_r.clone()].left_padder(6)
                    )?;
                }
                Err(errors) => {
                    writeln!(
                        f,
                        "    Error Count:{}    Expected:{}",
                        errors.len(),
                        test.subcase.error_count,
                    )?;
                    for err_r in errors {
                        writeln!(f, "      Error:\n{}", output[err_r.clone()].left_padder(8))?;
                    }
                }
            }

            writeln!(f)?;
        }

        Ok(())
    }
}
