use std::borrow::Cow;

use abi_stable::{
    traits::IntoReprC,
    std_types::{RCow,RStr},
    DynTrait,
};

use example_0_interface::{
    CowStrIter,
    PrefixTypeMod0_Prefix,
    PrefixTypeMod1_Prefix,
};


/// For transmuting a reference.
unsafe fn transmute_reference<T,U>(ref_:&T)->&U{
    &*(ref_ as *const T as *const U)
}


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
pub fn run_dynamic_library_tests(mods:&'static TextOpsMod_Prefix){
    test_reverse_lines(mods);
    test_remove_words(mods);

    {
        let hw=mods.prefix_types_tests();
        let hw=unsafe{
            // This only works because I know that both structs have the same alignment,
            // if either struct alignment changed that conversion would be unsound.
            transmute_reference::<PrefixTypeMod0_Prefix,PrefixTypeMod1_Prefix>(hw)
        };


        assert_eq!(hw.field_a(),123);
        assert_eq!(hw.field_b(),None);
        assert_eq!(hw.field_c(),None);
        let res=::std::panic::catch_unwind(||{
            let _=hw.field_d();
        });
        assert!(res.is_err(),"value:{:#?}",res);
    }

    let val=mods.hello_world().for_tests()();
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
    
    println!();
    println!(".-------------------------.");
    println!("|     tests succeeded!    |");
    println!("'-------------------------'");

}


fn test_reverse_lines(mods:&'static TextOpsMod_Prefix) {
    let text_ops=mods;

    let mut state = text_ops.new()();
    assert_eq!(
        &* text_ops.reverse_lines()(&mut state, "hello\nbig\nworld".into()),
        "world\nbig\nhello\n"
    );
}


fn test_remove_words(mods:&'static TextOpsMod_Prefix) {
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
