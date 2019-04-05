
macro_rules! to_stream {
    ( $stream:ident ; $($expr:expr),* $(,)* ) => {{
        // use quote::TokenStreamExt;

        $( $expr.to_tokens($stream); )*
    }}
}


macro_rules! measure {
    ( $e:expr ) => ({
        $e
        // let (dur,val)= ::core_extensions::measure_time::measure(||$e);
        // println!("{}-{}:taken {} to run",file!(),line!(),dur);
        // val
    })
}
