use abi_stable::{
    std_types::RCow,
    DynTrait,
};

use example_0_interface::{
    CowStrIter,
};


use super::*;

/// This tests that a type coming from a dynamic library 
/// cannot be converted back to its std-library equivalent
/// while reusing the heap allocation.
///
/// The reason why they can't reuse the heap allocation is because they might
/// be using a different global allocator that this binary is using.
///
/// There is no way that I am aware to check at compile-time what allocator
/// the type is using,so this is the best I can do while staying safe.
pub fn run_dynamic_library_tests(mods:&'static TextOpsMod){
    test_reverse_lines(mods);
    test_remove_words(mods);
    
    println!();
    println!(".-------------------------.");
    println!("|     tests succeeded!    |");
    println!("'-------------------------'");

}


fn test_reverse_lines(mods:&'static TextOpsMod) {
    let text_ops=mods;

    let mut state = text_ops.new()();
    assert_eq!(
        &* text_ops.reverse_lines()(&mut state, "hello\nbig\nworld".into()),
        "world\nbig\nhello\n"
    );
}


fn test_remove_words(mods:&'static TextOpsMod) {
    let text_ops=mods;

    let mut state = text_ops.new()();
    {
        let words = &mut vec!["burrito", "like","a"].into_iter().map(RCow::from);
        
        let param = RemoveWords {
            string: "Monads are like a burrito wrapper.".into(),
            words: DynTrait::from_borrowing_ptr(words,CowStrIter),
        };
        assert_eq!(&*text_ops.remove_words()(&mut state, param), "Monads are wrapper.");
    }
    {
        let words = &mut vec!["largest","is"].into_iter().map(RCow::from);
        let param = RemoveWords {
            string: "The   largest planet  is    jupiter.".into(),
            words: DynTrait::from_borrowing_ptr(words,CowStrIter),
        };
        assert_eq!(&*text_ops.remove_words()(&mut state, param), "The   planet  jupiter.");
    }
}
