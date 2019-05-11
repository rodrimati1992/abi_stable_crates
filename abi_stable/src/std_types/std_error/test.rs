use super::*;

use crate::{
    test_utils::{check_formatting_equivalence,deref_address,Stringy},
};


///////////////////////////////////////////////////////////////////////////////


#[test]
fn new(){

    let err=Stringy::new("hello\n\rworld");

    let e0=RBoxError::new(err.clone());

    check_formatting_equivalence(&err,&e0);
}


/// Testing that converting back and forth between 
/// `RBoxError` and `Box<dyn Error>` gives back the object it started with.
#[test]
fn identity_conversion(){

    let err=Stringy::new("hello\n\rworld");

    {
        let e0=RBoxError::new(err.clone());

        let addr=e0.heap_address();

        let e1=e0.piped(RBoxError::into_box).piped(RBoxError::from_box);

        assert_eq!(
            addr, 
            e1.heap_address()
        );
    }
    {
        let e0=Box::new(err.clone());

        let addr=e0.piped_ref(deref_address);

        let e1=e0.piped(|x|RBoxError::from_box(x)).piped(RBoxError::into_box);
        
        assert_eq!(
            addr, 
            e1.piped_ref(deref_address)
        );
    }
}

#[test]
fn from_to_box(){
    let err=Stringy::new("hello\n\rworld");

    {
        let e0=err.clone()
            .piped(RBoxError::new)
            .piped(RBoxError::into_box)
            .piped(RBoxError::from_box);
        
        check_formatting_equivalence(&err,&e0);
    }
    {
        let e0=err.clone().piped(RBoxError::new).piped(RBoxError::into_box);
        
        check_formatting_equivalence(&err,&e0);
    }
}


#[test]
fn downcast() {
    let err=Stringy::new("hello\n\rworld");

    macro_rules! downcast_ {
        (
            method=$method:ident,
            conv=$conv:expr
        ) => ({
            let res0=err.clone().piped(RBoxError::new).$method::<Stringy>().piped($conv).is_some();
            let res1=err.clone().piped(RBoxError::new).$method::<u32>()    .piped($conv).is_none();

            assert!(res0,"This RBoxError could not be downcasted to Stringy.");

            assert!(res1,"This RBoxError should only downcast to Stringy.");
        })
    }

    downcast_!{method=downcast    ,conv=|x| x.ok() }
    downcast_!{method=downcast_ref,conv=::std::convert::identity}
    downcast_!{method=downcast_mut,conv=::std::convert::identity}
}


#[test]
fn casts_among_rboxerrors(){
    let err=Stringy::new("hello\n\rworld");
    
    macro_rules! casts_among_rboxerrors_ {
        (
            err_ty=$err:ty;
            methods=[$($method:ident),* $(,)*];
        ) => ({
            $(
                let e0=<$err>::new(err.clone());
                let addr=e0.heap_address();
                let e1=e0.$method();
                assert_eq!(addr, e1.heap_address());

                check_formatting_equivalence(&err,&e1);

            )*
        })
    }


    casts_among_rboxerrors_!{
        err_ty=UnsyncRBoxError;
        methods=[into_unsync,as_unsync];
    }

    casts_among_rboxerrors_!{
        err_ty=SendRBoxError;
        methods=[into_unsync,as_unsync];
    }

    casts_among_rboxerrors_!{
        err_ty=RBoxError;
        methods=[into_unsync,as_unsync,as_send,into_send];
    }
}

