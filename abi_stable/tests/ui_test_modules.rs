#![cfg(feature = "__ui")]

mod ui_tests {
    #[test]
    fn ui() {
        let t = trybuild::TestCases::new();

        for dir in ["nonexhaustive_ui_tests", "sabi_trait_ui_tests"] {
            t.compile_fail(format!("tests/ui_tests/{}/*err.rs", dir));
            t.pass(format!("tests/ui_tests/{}/*ok.rs", dir));
        }
    }
}
