use super::*;

use crate::{
    std_types::string::FromUtf8Error as OtherErr,
    test_utils::{check_formatting_equivalence, deref_address, Stringy},
};

///////////////////////////////////////////////////////////////////////////////

#[test]
fn new() {
    let err = Stringy::new("hello\n\rworld");

    let e0 = RBoxError::new(err.clone());

    check_formatting_equivalence(&err, &e0);
}

/// Testing that converting back and forth between
/// `RBoxError` and `Box<dyn Error>` gives back the object it started with.
#[test]
fn identity_conversion() {
    let err = Stringy::new("hello\n\rworld");

    {
        let e0 = RBoxError::new(err.clone());

        let addr = e0.heap_address();

        let e1 = e0.piped(RBoxError::into_box).piped(RBoxError::from_box);

        assert_eq!(addr, e1.heap_address());
    }
    {
        let e0 = Box::new(err);

        let addr = e0.piped_ref(deref_address);

        let e1 = e0
            .piped(|x| RBoxError::from_box(x))
            .piped(RBoxError::into_box);

        assert_eq!(addr, e1.piped_ref(deref_address));
    }
}

#[test]
fn from_to_box() {
    let err = Stringy::new("hello\n\rworld");

    {
        let e0 = err
            .clone()
            .piped(RBoxError::new)
            .piped(RBoxError::into_box)
            .piped(RBoxError::from_box);

        check_formatting_equivalence(&err, &e0);
    }
    {
        let e0 = err.clone().piped(RBoxError::new).piped(RBoxError::into_box);

        check_formatting_equivalence(&err, &e0);
    }
}

#[test]
fn downcast() {
    let err = Stringy::new("hello\n\rworld");

    macro_rules! downcast_ {
        (
            method = $method: ident,
            conv = $conv: expr
        ) => {{
            let res0 = RBoxError::new(err.clone())
                .$method::<Stringy>()
                .piped($conv)
                .is_some();
            let res1 = RBoxError::new(err.clone())
                .$method::<OtherErr>()
                .piped($conv)
                .is_none();

            assert!(res0, "This RBoxError could not be downcasted to Stringy.");

            assert!(res1, "This RBoxError should only downcast to Stringy.");
        }};
    }

    downcast_! {method = downcast    , conv = |x| x.ok() }
    downcast_! {method = downcast_ref, conv=::std::convert::identity}
    downcast_! {method = downcast_mut, conv=::std::convert::identity}
}

#[test]
fn casts_among_rboxerrors() {
    let err = Stringy::new("hello\n\rworld");

    macro_rules! casts_among_rboxerrors_ {
        (
            err_ty = $err: ty;
            methods = [$($method: ident),* $(,)*];
        ) => ({
            $(
                let e0=<$err>::new(err.clone());
                let addr = e0.heap_address();
                let e1 = e0.$method();
                assert_eq!(addr, e1.heap_address());

                check_formatting_equivalence(&err, &e1);

            )*
        })
    }

    casts_among_rboxerrors_! {
        err_ty = UnsyncRBoxError;
        methods = [into_unsync, as_unsync];
    }

    casts_among_rboxerrors_! {
        err_ty = SendRBoxError;
        methods = [into_unsync, as_unsync];
    }

    casts_among_rboxerrors_! {
        err_ty = RBoxError;
        methods = [into_unsync, as_unsync, as_send, into_send];
    }
}

fn check_formatted_debug_display(str_err: &Stringy, rerr: &RBoxError) {
    let as_dd = rerr.as_debug_display().unwrap();
    assert_eq!(format!("{:#?}", str_err), as_dd.debug.as_str());
    assert_eq!(format!("{:#}", str_err), as_dd.display.as_str());
    assert_eq!(str_err.str, as_dd.display.as_str());

    check_formatting_equivalence(&str_err, rerr);
}

#[test]
fn to_formatted() {
    let str_err = Stringy::new("hello\n\rworld");

    {
        let rerr = RBoxError::new(str_err.clone());

        assert_eq!(rerr.as_debug_display(), None);

        check_formatting_equivalence(&str_err, &rerr);
        let rerr: RBoxError = rerr.to_formatted_error();

        check_formatted_debug_display(&str_err, &rerr);
        check_formatted_debug_display(&str_err, &rerr.to_formatted_error());
    }
}

#[test]
fn from_fmt_or_debug() {
    let str_err = Stringy::new("hello\n\rworld");

    {
        let rerr = RBoxError::from_fmt(&str_err);

        check_formatted_debug_display(&str_err, &rerr);
        check_formatted_debug_display(&str_err, &rerr.to_formatted_error());
    }
    {
        let rerr = RBoxError::from_debug(&str_err);

        let as_dd = rerr.as_debug_display().unwrap();
        assert_eq!(format!("{:#?}", str_err), as_dd.debug.as_str());
        assert_eq!(format!("{:#?}", str_err), as_dd.display.as_str());

        assert_eq!(format!("{:#?}", str_err), format!("{:#?}", rerr));
        assert_eq!(format!("{:#?}", str_err), format!("{}", rerr));
    }
}
