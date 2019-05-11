#![allow(dead_code)]



#[allow(unused_imports)]
use core_extensions::{matches, prelude::*};

use rand::{
    thread_rng,
    seq::SliceRandom,
};

use crate::{
    abi_stability::{
        abi_checking::{AbiInstability,CheckingGlobals,check_abi_stability_with_globals},
        stable_abi_trait::AbiInfo,
        AbiInfoWrapper, 
    },
    prefix_type::{PrefixTypeMetadata,PrefixTypeTrait},
    *,
    test_utils::must_panic,
    type_level::bools::*,
    utils::transmute_reference,
};


fn custom_default<T>()->T
where T:From<u8>
{
    101.into()
}


mod prefix0 {
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(inside_abi_stable_crate)]
    // #[sabi(debug_print)]
    #[sabi(kind(Prefix(prefix_struct="Prefix")))]
    pub struct PrefixVal {
        #[sabi(last_prefix_field)]
        pub field0: u8,
    }
}

mod prefix1 {
    use super::custom_default;
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(
        inside_abi_stable_crate,
        // debug_print,
        kind(Prefix(prefix_struct="Prefix")),
        missing_field(with="custom_default::<_>"),
    )]
    pub struct PrefixVal {
        #[sabi(last_prefix_field)]
        pub field0: u8,
        pub field1: u16,
    }
}

mod prefix2 {
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(inside_abi_stable_crate)]
    #[sabi(kind(Prefix(prefix_struct="Prefix")))]
    #[sabi(missing_field(default))]
    pub struct PrefixVal {
        #[sabi(last_prefix_field)]
        pub field0: u8,
        pub field1: u16,
        pub field2: u32,
    }
}

// PrefixVal types have to keep the same alignment when fields are added
mod prefix2_misaligned {
    #[repr(C,align(16))]
    #[derive(StableAbi)]
    // #[sabi(debug_print)]
    #[sabi(inside_abi_stable_crate)]
    #[sabi(kind(Prefix(prefix_struct="Prefix")))]
    pub struct PrefixVal {
        #[sabi(last_prefix_field)]
        pub field0: u8,
        pub field1: u16,
        pub field2: u32,
    }
}

mod prefix2_different_prefix {
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(inside_abi_stable_crate)]
    #[sabi(kind(Prefix(prefix_struct="Prefix")))]
    pub struct PrefixVal {
        pub field0: u8,
        #[sabi(last_prefix_field)]
        pub field1: u16,
        pub field2: u32,
    }
}

mod prefix3 {
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(inside_abi_stable_crate)]
    #[sabi(kind(Prefix(prefix_struct="Prefix")))]
    #[sabi(missing_field(panic))]
    pub struct PrefixVal {
        #[sabi(last_prefix_field)]
        pub field0: u8,
        pub field1: u16,
        pub field2: u32,
        pub field3: u64,
    }
}



/// Dereferences the AbiInfo of a `&T` to the layout of `T`
fn dereference_abi(abi:&'static AbiInfo)->&'static AbiInfo{
    abi.layout.phantom_fields[0].abi_info.get()
}



static PREF_0:&'static AbiInfoWrapper = <&prefix0::Prefix>::ABI_INFO;
static PREF_1:&'static AbiInfoWrapper = <&prefix1::Prefix>::ABI_INFO;
static PREF_2:&'static AbiInfoWrapper = <&prefix2::Prefix>::ABI_INFO;
static PREF_3:&'static AbiInfoWrapper = <&prefix3::Prefix>::ABI_INFO;


fn new_list()->Vec<&'static AbiInfoWrapper>{
    vec![PREF_0, PREF_1, PREF_2, PREF_3]
}

#[cfg_attr(not(miri),test)]
fn prefixes_test() {
    let list = new_list();

    let mut rng=thread_rng();

    fn gen_elem_from(
        abi_wrapper:&'static AbiInfoWrapper
    )->(&'static AbiInfoWrapper,PrefixTypeMetadata){
        let prefix=abi_wrapper.get()
            .piped(dereference_abi)
            .layout
            .piped(PrefixTypeMetadata::new);
        (abi_wrapper,prefix)
    }

    let mut gen_generation=|skip_first:usize|{
        let mut ret=Vec::<(&'static AbiInfoWrapper,PrefixTypeMetadata)>::new();
        for _ in 0..list.len() {
            let pushed=gen_elem_from(list.choose(&mut rng).unwrap().clone());
            ret.push(pushed);
        }
        let max_size=ret.iter().map(|(_,x)|x.fields.len()).max().unwrap();
        ret.extend(
            list.iter().cloned()
                .skip(skip_first)
                .take(max_size)
                .map(gen_elem_from)
        );
        ret
    };

    for _ in 0..200{
        let globals=CheckingGlobals::new();

        let t_list=gen_generation(0);
        let o_list=gen_generation(1);

        for ((this,t_prefix),(other,o_prefix)) in 
            t_list.iter().cloned().zip(o_list.iter().cloned())
        {
            let prefix_type_map=globals.prefix_type_map.lock().unwrap();
            let value_len=prefix_type_map.value_len();
            let key_len=prefix_type_map.key_len();
            // Not dropping it here causes a deadlock inside 
            // check_abi_stability_with_globals.
            drop(prefix_type_map);

            let res = check_abi_stability_with_globals(this, other,&globals);

            let prefix_type_map=globals.prefix_type_map.lock().unwrap();
            if t_prefix.fields.len() <= o_prefix.fields.len() {
                res.unwrap_or_else(|e| panic!("{:#?}",e) );


                let deref_this=dereference_abi(this .get());
                let deref_other=dereference_abi(other.get());
                let t_id=deref_this.get_utypeid();
                let o_id=deref_other.get_utypeid();

                let t_map_prefix=prefix_type_map.get(&t_id);
                let o_map_prefix=prefix_type_map.get(&o_id);

                let t_map_prefix=t_map_prefix.unwrap();
                let o_map_prefix=o_map_prefix.unwrap();

                for pre in vec![o_prefix.clone(),t_map_prefix.clone(),o_map_prefix.clone()] {
                    assert_eq!(
                        t_prefix.prefix_field_count,
                        pre.prefix_field_count, 
                    );
                    for (l_field,r_field) in t_prefix.fields.iter().zip(pre.fields.iter()) {
                        assert_eq!(l_field,r_field);
                    }
                }

                assert!(t_prefix.fields.len()<=t_map_prefix.fields.len());
                assert!(o_prefix.fields.len()<=o_map_prefix.fields.len());

                assert_eq!(t_map_prefix as *const _,o_map_prefix as *const _);
            } else {
                assert_eq!(value_len,prefix_type_map.value_len());
                assert_eq!(key_len,prefix_type_map.key_len());
                
                let errs = res.unwrap_err().flatten_errors();
                assert!(
                    errs.iter()
                        .any(|err| matches!(AbiInstability::FieldCountMismatch{..}=err)),
                );
            }
        }

        let prefix_type_map=globals.prefix_type_map.lock().unwrap();

        let max_prefix=t_list.iter().zip(o_list.iter())
            .map(|((_,l_prefix),(_,r_prefix))| (l_prefix.clone(),r_prefix.clone()) )
            .filter(|(l,r)| l.fields.len() <= r.fields.len() )
            .map(|(l,r)| PrefixTypeMetadata::max(l,r) )
            .max_by_key(|prefix| prefix.fields.len() )
            .unwrap();

        // Asserting that the layout they all map to is the one with the most fields
        for this in list.iter().cloned() {
            let id=dereference_abi(this.get()).get_utypeid();

            // The random sanpling did not include this type.
            let prefix=match prefix_type_map.get(&id) {
                Some(x)=>x,
                None=>continue,
            };

            for (l_field,r_field) in prefix.fields.iter().zip(max_prefix.fields.iter()) {
                assert_eq!(l_field,r_field);
            }
            assert_eq!(
                prefix.fields.len(), 
                max_prefix.fields.len(),
            );
        }
    }
}


fn check_interface_impl_pair(
    globals:&CheckingGlobals,
    this :&'static AbiInfoWrapper,
    other:&'static AbiInfoWrapper,
){
    let deref_this=dereference_abi(this.get());
    let deref_other=dereference_abi(other.get());

    let t_prefix=PrefixTypeMetadata::new(deref_this.layout);
    let o_prefix=PrefixTypeMetadata::new(deref_other.layout);

    if let Err(e)=check_abi_stability_with_globals(this,other,&globals) {
        if t_prefix.fields.len() <= o_prefix.fields.len() {
            panic!("{:#?}",e);
        }else{
            return;
        }
    }

    let prefix_type_map=globals.prefix_type_map.lock().unwrap();


    let t_id=deref_this.get_utypeid();
    let o_id=deref_other.get_utypeid();

    let t_map_prefix=prefix_type_map.get(&t_id);
    let o_map_prefix=prefix_type_map.get(&o_id);


    let t_map_prefix=t_map_prefix.unwrap();
    let o_map_prefix=o_map_prefix.unwrap();

    for pre in vec![o_prefix.clone(),t_map_prefix.clone(),o_map_prefix.clone()] {
        assert_eq!(
            t_prefix.prefix_field_count,
            pre.prefix_field_count, 
        );
        for (field_i,(l_field,r_field)) in 
            t_prefix.fields.iter().zip(pre.fields.iter()).enumerate() 
        {
            if t_prefix.accessible_fields.is_accessible(field_i)
                &&o_prefix.accessible_fields.is_accessible(field_i)
            {
                assert_eq!(l_field,r_field,"\nleft:{:#?}\n\nright:{:#?}\n",l_field,r_field);
            }
        }
    }

    assert!(t_prefix.fields.len()<=t_map_prefix.fields.len());
    assert!(o_prefix.fields.len()<=o_map_prefix.fields.len());

    assert_eq!(t_map_prefix as *const _,o_map_prefix as *const _);
}


#[cfg_attr(not(miri),test)]
fn hierarchical_prefix_test(){
    let library_00=PREF_2;
    let library_01=PREF_1;
    let library_02=PREF_0;

    let library_10=PREF_2;
    let library_11=PREF_2;
    let library_12=PREF_3;

    let library_0=PREF_0;
    let library_1=PREF_1;

    let binary=PREF_0;

    let mut rng=thread_rng();

    for _ in 0..100 {
        let globals=CheckingGlobals::new();
        let mut checks=vec![
            (binary,library_0),
            (binary,library_1),
            (library_0,library_00),
            (library_0,library_01),
            (library_0,library_02),
            (library_1,library_10),
            (library_1,library_11),
            (library_1,library_12),
        ];
        checks.shuffle(&mut rng);

        for (this,other) in checks {
            check_interface_impl_pair(&globals,this,other);
        }
    }
}


#[cfg_attr(not(miri),test)]
fn prefix_is_same_alignment(){
    let globals=CheckingGlobals::new();
    let misaligned = <&prefix2_misaligned::Prefix>::ABI_INFO;

    for pref in vec![ PREF_0,PREF_1 ] {
        let errs = check_abi_stability_with_globals(pref, misaligned,&globals)
            .unwrap_err()
            .flatten_errors();

        assert!(
            errs
            .iter()
            .any(|err| matches!(AbiInstability::Alignment{..}=err))
        );
    }
}


#[cfg_attr(not(miri),test)]
fn prefix_is_same_size(){
    let globals=CheckingGlobals::new();
    let list=new_list();

    for pref in list.iter().cloned() {
        let mismatched_prefix=<&prefix2_different_prefix::Prefix>::ABI_INFO;
        let errs = check_abi_stability_with_globals(pref,mismatched_prefix ,&globals)
            .unwrap_err()
            .flatten_errors();

        assert!(
            errs
            .iter()
            .any(|err| matches!(AbiInstability::MismatchedPrefixSize{..}=err))
        );
    }
}



#[cfg_attr(not(miri),test)]
fn prefix_on_nonexistent_field() {
    let prefix0=
        prefix0::PrefixVal{
            field0:1,
        }.leak_into_prefix();

    {
        let value1:&prefix1::Prefix=unsafe{ transmute_reference(prefix0) };
        assert_eq!(value1.field0(),1);
        assert_eq!(value1.field1(),custom_default::<u16>());
    }
    {
        let value2:&prefix2::Prefix=unsafe{ transmute_reference(prefix0) };
        assert_eq!(value2.field0(),1);
        assert_eq!(value2.field1(),0);
        assert_eq!(value2.field2(),0);
    }
    {
        let value3:&prefix3::Prefix=unsafe{ transmute_reference(prefix0) };
        assert_eq!(value3.field0(),1);
        must_panic(file_span!(),||value3.field1()).unwrap();
        must_panic(file_span!(),||value3.field2()).unwrap();
        must_panic(file_span!(),||value3.field3()).unwrap();
    }
}



/////////////////////////////////////////////////////////////////////////



pub trait EnabledFields{
    const ENABLE_FIELD_0:bool=true;
    const ENABLE_FIELD_1:bool=true;
    const ENABLE_FIELD_2:bool=true;
    const ENABLE_FIELD_3:bool=true;
}


impl<B0,B1,B2,B3> EnabledFields for (B0,B1,B2,B3)
where
    B0:Boolean,
    B1:Boolean,
    B2:Boolean,
    B3:Boolean,
{
    const ENABLE_FIELD_0:bool=<B0 as Boolean>::VALUE;
    const ENABLE_FIELD_1:bool=<B1 as Boolean>::VALUE;
    const ENABLE_FIELD_2:bool=<B2 as Boolean>::VALUE;
    const ENABLE_FIELD_3:bool=<B3 as Boolean>::VALUE;
}


macro_rules! declare_enabled_fields {
    ( 
        $struct:ident {
            $($trait_definition:tt)*
        }
    ) => {
        pub struct $struct;

        impl EnabledFields for $struct {
            $($trait_definition)*
        }
    }
}

declare_enabled_fields!{
    ACCESSIBLE_ALL {
        const ENABLE_FIELD_0:bool=true;
        const ENABLE_FIELD_1:bool=true;
        const ENABLE_FIELD_2:bool=true;
        const ENABLE_FIELD_3:bool=true;
    }
}

declare_enabled_fields!{
    ACCESSIBLE_ALL_EXCEPT_0 {
        const ENABLE_FIELD_0:bool=false;
        const ENABLE_FIELD_1:bool=true;
        const ENABLE_FIELD_2:bool=true;
        const ENABLE_FIELD_3:bool=true;
    }
}

declare_enabled_fields!{
    ACCESSIBLE_ALL_EXCEPT_1 {
        const ENABLE_FIELD_0:bool=true;
        const ENABLE_FIELD_1:bool=false;
        const ENABLE_FIELD_2:bool=true;
        const ENABLE_FIELD_3:bool=true;
    }
}


declare_enabled_fields!{
    ACCESSIBLE_ALL_EXCEPT_2 {
        const ENABLE_FIELD_0:bool=true;
        const ENABLE_FIELD_1:bool=true;
        const ENABLE_FIELD_2:bool=false;
        const ENABLE_FIELD_3:bool=true;
    }
}


declare_enabled_fields!{
    ACCESSIBLE_ALL_EXCEPT_3 {
        const ENABLE_FIELD_0:bool=true;
        const ENABLE_FIELD_1:bool=true;
        const ENABLE_FIELD_2:bool=true;
        const ENABLE_FIELD_3:bool=false;
    }
}

static COND_FIELD_0_ALL:&'static AbiInfoWrapper = 
    <&cond_fields_0::Prefix<ACCESSIBLE_ALL>>::ABI_INFO;

static COND_FIELD_1_ALL:&'static AbiInfoWrapper = 
    <&cond_fields_1::Prefix<ACCESSIBLE_ALL>>::ABI_INFO;

static COND_FIELD_2_ALL:&'static AbiInfoWrapper = 
    <&cond_fields_2::Prefix<ACCESSIBLE_ALL>>::ABI_INFO;

static COND_FIELD_3_ALL:&'static AbiInfoWrapper = 
    <&cond_fields_3::Prefix<ACCESSIBLE_ALL>>::ABI_INFO;


static COND_FIELD_0_EXCEPT_0:&'static AbiInfoWrapper = 
    <&cond_fields_0::Prefix<ACCESSIBLE_ALL_EXCEPT_0>>::ABI_INFO;

static COND_FIELD_1_EXCEPT_0:&'static AbiInfoWrapper = 
    <&cond_fields_1::Prefix<ACCESSIBLE_ALL_EXCEPT_0>>::ABI_INFO;

static COND_FIELD_2_EXCEPT_0:&'static AbiInfoWrapper = 
    <&cond_fields_2::Prefix<ACCESSIBLE_ALL_EXCEPT_0>>::ABI_INFO;

static COND_FIELD_3_EXCEPT_0:&'static AbiInfoWrapper = 
    <&cond_fields_3::Prefix<ACCESSIBLE_ALL_EXCEPT_0>>::ABI_INFO;


static COND_FIELD_1_EXCEPT_1:&'static AbiInfoWrapper = 
    <&cond_fields_1::Prefix<ACCESSIBLE_ALL_EXCEPT_1>>::ABI_INFO;

static COND_FIELD_2_EXCEPT_1:&'static AbiInfoWrapper = 
    <&cond_fields_2::Prefix<ACCESSIBLE_ALL_EXCEPT_1>>::ABI_INFO;

static COND_FIELD_3_EXCEPT_1:&'static AbiInfoWrapper = 
    <&cond_fields_3::Prefix<ACCESSIBLE_ALL_EXCEPT_1>>::ABI_INFO;


static COND_FIELD_2_EXCEPT_2:&'static AbiInfoWrapper = 
    <&cond_fields_2::Prefix<ACCESSIBLE_ALL_EXCEPT_2>>::ABI_INFO;

static COND_FIELD_3_EXCEPT_2:&'static AbiInfoWrapper = 
    <&cond_fields_3::Prefix<ACCESSIBLE_ALL_EXCEPT_2>>::ABI_INFO;


static COND_FIELD_3_EXCEPT_3:&'static AbiInfoWrapper = 
    <&cond_fields_3::Prefix<ACCESSIBLE_ALL_EXCEPT_3>>::ABI_INFO;







mod cond_fields_0 {
    use crate::marker_type::UnsafeIgnoredType;
    use super::EnabledFields;
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(
        inside_abi_stable_crate,
        kind(Prefix(prefix_struct="Prefix")),
        prefix_bound="C:EnabledFields",
        unconstrained(C),
    )]
    pub struct PrefixVal<C> {
        pub _marker:UnsafeIgnoredType<C>,
        #[sabi(accessible_if=" <C as EnabledFields>::ENABLE_FIELD_0 ")]
        #[sabi(last_prefix_field)]
        pub field0: u8,
    }
}

mod cond_fields_1 {
    use crate::marker_type::UnsafeIgnoredType;
    use super::EnabledFields;
    
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(
        inside_abi_stable_crate,
        kind(Prefix(prefix_struct="Prefix")),
        prefix_bound="C:EnabledFields",
        unconstrained(C),
    )]
    pub struct PrefixVal<C,T=u8,U=u16> {
        pub _marker:UnsafeIgnoredType<C>,
        
        #[sabi(accessible_if=" <C as EnabledFields>::ENABLE_FIELD_0 ")]
        #[sabi(last_prefix_field)]
        pub field0: T,
        
        #[sabi(accessible_if=" <C as EnabledFields>::ENABLE_FIELD_1 ")]
        pub field1: U,
    }
}

mod cond_fields_2 {
    use crate::marker_type::UnsafeIgnoredType;
    use super::EnabledFields;
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(
        inside_abi_stable_crate,
        kind(Prefix(prefix_struct="Prefix")),
        prefix_bound="C:EnabledFields",
        unconstrained(C),
    )]
    pub struct PrefixVal<C,T=u8,U=u16,V=u32> {
        pub _marker:UnsafeIgnoredType<C>,
        
        #[sabi(accessible_if=" <C as EnabledFields>::ENABLE_FIELD_0 ")]
        #[sabi(last_prefix_field)]
        pub field0: T,
        
        #[sabi(accessible_if=" <C as EnabledFields>::ENABLE_FIELD_1 ")]
        pub field1: U,
        
        #[sabi(accessible_if=" <C as EnabledFields>::ENABLE_FIELD_2 ")]
        pub field2: V,
    }
}

// PrefixVal types have to keep the same alignment when fields are added
mod cond_fields_2_misaligned {
    use crate::marker_type::UnsafeIgnoredType;
    use super::EnabledFields;
    #[repr(C,align(16))]
    #[derive(StableAbi)]
    #[sabi(
        inside_abi_stable_crate,
        kind(Prefix(prefix_struct="Prefix")),
        prefix_bound="C:EnabledFields",
        unconstrained(C),
    )]
    pub struct PrefixVal<C> {
        pub _marker:UnsafeIgnoredType<C>,
        
        #[sabi(accessible_if=" <C as EnabledFields>::ENABLE_FIELD_0 ")]
        #[sabi(last_prefix_field)]
        pub field0: u8,
        
        #[sabi(accessible_if=" <C as EnabledFields>::ENABLE_FIELD_1 ")]
        pub field1: u16,
        
        #[sabi(accessible_if=" <C as EnabledFields>::ENABLE_FIELD_2 ")]
        pub field2: u32,
    }
}

mod cond_fields_2_different_prefix {
    use crate::marker_type::UnsafeIgnoredType;
    use super::EnabledFields;
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(
        inside_abi_stable_crate,
        kind(Prefix(prefix_struct="Prefix")),
        prefix_bound="C:EnabledFields",
        unconstrained(C),
    )]
    pub struct PrefixVal<C,T=u8,U=u16,V=u32> {
        pub _marker:UnsafeIgnoredType<C>,
        
        #[sabi(accessible_if=" <C as EnabledFields>::ENABLE_FIELD_0 ")]
        pub field0: T,
        
        #[sabi(accessible_if=" <C as EnabledFields>::ENABLE_FIELD_1 ")]
        #[sabi(last_prefix_field)]
        pub field1: U,
        
        #[sabi(accessible_if=" <C as EnabledFields>::ENABLE_FIELD_2 ")]
        pub field2: V,
    }
}

mod cond_fields_3 {
    use crate::marker_type::UnsafeIgnoredType;
    use super::EnabledFields;
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(
        // debug_print,
        inside_abi_stable_crate,
        kind(Prefix(prefix_struct="Prefix")),
        prefix_bound="C:EnabledFields",
        unconstrained(C),
    )]
    pub struct PrefixVal<C,T=u8,U=u16,V=u32,W=u64> {
        pub _marker:UnsafeIgnoredType<(C,T,U,V,W)>,
        
        #[sabi(accessible_if=" <C as EnabledFields>::ENABLE_FIELD_0 ")]
        #[sabi(last_prefix_field)]
        pub field0: T,
        
        #[sabi(accessible_if=" <C as EnabledFields>::ENABLE_FIELD_1 ")]
        pub field1: U,
        
        #[sabi(accessible_if=" <C as EnabledFields>::ENABLE_FIELD_2 ")]
        pub field2: V,
        
        #[sabi(accessible_if=" <C as EnabledFields>::ENABLE_FIELD_3 ")]
        pub field3: W,
    }
}

mod cond_fields_3_uncond_prefix {
    use crate::marker_type::UnsafeIgnoredType;
    use super::EnabledFields;
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(
        // debug_print,
        inside_abi_stable_crate,
        kind(Prefix(prefix_struct="Prefix")),
        prefix_bound="C:EnabledFields",
        unconstrained(C),
    )]
    pub struct PrefixVal<C,T=u8,U=u16,V=u32,W=u64> {
        pub _marker:UnsafeIgnoredType<(C,T,U,V,W)>,
        
        #[sabi(last_prefix_field)]
        pub field0: T,
        
        #[sabi(accessible_if=" <C as EnabledFields>::ENABLE_FIELD_1 ")]
        pub field1: U,
        
        #[sabi(accessible_if=" <C as EnabledFields>::ENABLE_FIELD_2 ")]
        pub field2: V,
        
        #[sabi(accessible_if=" <C as EnabledFields>::ENABLE_FIELD_3 ")]
        pub field3: W,
    }
}



#[cfg_attr(not(miri),test)]
fn prefix_cond_field_test(){
    let mut rng=thread_rng();

    use crate::type_level::bools::{True as T,False as F};

    use self::cond_fields_2::Prefix as Prefix2;
    use self::cond_fields_3::Prefix as Prefix3;
    use self::cond_fields_3_uncond_prefix::Prefix as Prefix3UncondPrefix;

    type au32=[u32;1];
    type ai32=[i32;1];

    let mut valid_lists=vec![
        vec![
            <&Prefix3<(F,F,F,F),ai32,ai32,ai32,ai32>>::ABI_INFO,
            <&Prefix3<(T,F,F,F),i32 ,ai32,ai32,ai32>>::ABI_INFO,
            <&Prefix3<(T,T,F,F),i32 ,i32 ,ai32,ai32>>::ABI_INFO,
            <&Prefix3<(T,T,T,F),i32 ,i32 ,i32 ,ai32>>::ABI_INFO,
            <&Prefix3<(T,T,T,T),i32 ,i32 ,i32 ,i32 >>::ABI_INFO,
            <&Prefix3<(T,T,T,T),i32 ,i32 ,i32 ,i32 >>::ABI_INFO,
        ],
        vec![
            <&Prefix3<(T,T,T,T),i32 ,i32 ,i32 ,i32 >>::ABI_INFO,
            <&Prefix3<(F,T,T,T),ai32,i32 ,i32 ,i32 >>::ABI_INFO,
            <&Prefix3<(F,F,T,T),ai32,ai32,i32 ,i32 >>::ABI_INFO,
            <&Prefix3<(F,F,F,T),ai32,ai32,ai32,i32 >>::ABI_INFO,
            <&Prefix3<(F,F,F,F),ai32,ai32,ai32,ai32>>::ABI_INFO,
            <&Prefix3<(T,T,T,T),i32 ,i32 ,i32 ,i32 >>::ABI_INFO,
            <&Prefix3<(T,T,T,T),i32 ,i32 ,i32 ,i32 >>::ABI_INFO,
        ],
        vec![
            <&Prefix2<(F,T,F,F),au32,i32 ,au32>>::ABI_INFO,
            <&Prefix3<(T,F,T,T),i32 ,au32,i32 ,i32>>::ABI_INFO,
            <&Prefix3<(T,T,T,T),i32 ,i32 ,i32 ,i32>>::ABI_INFO,
            <&Prefix3<(T,T,T,T),i32 ,i32 ,i32 ,i32>>::ABI_INFO,
        ],
    ];


    let invalid=vec![
        (
            <&Prefix3<(T,F,T,F),i32 ,au32,i32 ,i32>>::ABI_INFO,
            <&Prefix2<(F,T,F,F),au32,i32 ,au32>>::ABI_INFO,
        ),
        (
            <&Prefix2<(F,T,F,F),au32,i32 ,au32>>::ABI_INFO,
            <&Prefix3<(T,T,T,T),i32 ,au32,i32 ,i32>>::ABI_INFO,
        ),
        (
            <&Prefix3<(T,T,T,T),i32 ,i32 ,i32 ,i32>>::ABI_INFO,
            <&Prefix3<(T,T,T,T),i32 ,au32,i32 ,i32>>::ABI_INFO,
        ),
        (
            <&Prefix3<(T,T,T,T),i32 ,i32 ,au32,i32>>::ABI_INFO,
            <&Prefix3<(F,F,T,F),i32 ,i32 ,i32 ,i32>>::ABI_INFO,
        ),
        (
            <&Prefix3UncondPrefix<(T,T,T,T),i32 ,i32 ,au32,i32>>::ABI_INFO,
            <&Prefix3            <(T,T,T,T),i32 ,i32 ,au32,i32>>::ABI_INFO,
        ),
    ];


    for valid_list in &valid_lists {
        let globals=CheckingGlobals::new();
        for window in valid_list.windows(2) {
            check_interface_impl_pair(&globals,window[0],window[1]);
        }
    }

    for _ in 0..50{
        for valid_list in &mut valid_lists {
            let globals=CheckingGlobals::new();
            valid_list.shuffle(&mut rng);

            for window in valid_list.windows(2) {
                check_interface_impl_pair(&globals,window[0],window[1]);
            }
        }
    }


    const CHECKED_ERRORS:usize=3;


    let mut err_counts=vec![0;CHECKED_ERRORS];


    {
        let mut inc_on_err=|conds:[bool;CHECKED_ERRORS]|->bool{
            let mut any=false;
            for (i,&cond) in conds.iter().enumerate() {
                any=any||cond;
                err_counts[i]+=cond as usize;
            }
            any
        };

        let globals=CheckingGlobals::new();
        for (interf,impl_) in invalid {
            let errs = check_abi_stability_with_globals(interf,impl_ ,&globals)
                .unwrap_err()
                .flatten_errors();

            assert!(
                errs
                .iter()
                .any(|e|{
                    inc_on_err([
                        matches!(AbiInstability::FieldCountMismatch{..}=e),
                        matches!(AbiInstability::Name{..}=e),
                        matches!(AbiInstability::MismatchedPrefixConditionality{..}=e),
                    ])
                }),
            );
        }
    }

    assert!(
        err_counts.iter().all(|&x| x!=0 ),
        "err_counts:{:#?}",
        err_counts,
    );

}


#[cfg_attr(not(miri),test)]
fn hierarchical_prefix_cond_field_test(){
    let mut rng=thread_rng();

    let fields_0=vec![
        COND_FIELD_0_ALL,
        COND_FIELD_0_EXCEPT_0,
    ];

    let fields_1=vec![
        COND_FIELD_1_ALL,
        COND_FIELD_1_EXCEPT_0,
        COND_FIELD_1_EXCEPT_1,
    ];

    let fields_2=vec![
        COND_FIELD_2_ALL,
        COND_FIELD_2_EXCEPT_0,
        COND_FIELD_2_EXCEPT_1,
        COND_FIELD_2_EXCEPT_2,
    ];

    let fields_3=vec![
        COND_FIELD_3_ALL,
        COND_FIELD_3_EXCEPT_0,
        COND_FIELD_3_EXCEPT_1,
        COND_FIELD_3_EXCEPT_2,
        COND_FIELD_3_EXCEPT_3,
    ];

    for _ in 0..500 {
        let globals=CheckingGlobals::new();
        
        let library_00=fields_2.choose(&mut rng).unwrap().clone();
        let library_01=fields_1.choose(&mut rng).unwrap().clone();
        let library_02=fields_0.choose(&mut rng).unwrap().clone();

        let library_10=fields_2.choose(&mut rng).unwrap().clone();
        let library_11=fields_2.choose(&mut rng).unwrap().clone();
        let library_12=fields_3.choose(&mut rng).unwrap().clone();

        let library_0=fields_0.choose(&mut rng).unwrap().clone();
        let library_1=fields_1.choose(&mut rng).unwrap().clone();

        let binary=fields_0.choose(&mut rng).unwrap().clone();


        let mut checks=vec![
            (binary,library_0),
            (binary,library_1),
            (library_0,library_00),
            (library_0,library_01),
            (library_0,library_02),
            (library_1,library_10),
            (library_1,library_11),
            (library_1,library_12),
        ];
        checks.shuffle(&mut rng);

        for (this,other) in checks {
            check_interface_impl_pair(&globals,this,other);
        }
    }
}


#[test]
fn prefix_on_conditional_fields() {

    use crate::{
        type_level::bools::{True as T,False as F},
        marker_type::UnsafeIgnoredType,
    };

    
    type Prefix1<AF>=cond_fields_1::Prefix<AF,i8,i32>;
    type Prefix2<AF>=cond_fields_2::Prefix<AF,i8,i32,i32>;
    type Prefix3<AF>=cond_fields_3::Prefix<AF,i8,i32,i32,i32>;
    type Prefix3UncondPrefix<AF>=
        cond_fields_3_uncond_prefix::Prefix<AF,i8,i32,i32,i32>;

    {// Casting Prefix0 to Prefix1 with different field accessibilities
        let prefix0=
            cond_fields_0::PrefixVal{
                _marker:UnsafeIgnoredType::<(T,T,T,T)>::DEFAULT,
                field0:1,
            }.leak_into_prefix();

        {// The field cannot be accessed even though it was initialized.
            let value:&Prefix1<(F,F,F,F)>=unsafe{ transmute_reference(prefix0) };

            assert_eq!(value.field0(),None);
            assert_eq!(value.field1(),None);
        }
        {// The first field can be accessed.
            let value:&Prefix1<(T,F,F,F)>=unsafe{ transmute_reference(prefix0) };

            assert_eq!(value.field0(),Some(1));
            assert_eq!(value.field1(),None);
        }
    }



    let prefix3=cond_fields_3::PrefixVal{
            _marker:UnsafeIgnoredType::<((T,T,T,T),_,_,_,_)>::DEFAULT,
            field0:1,
            field1:3,
            field2:7,
            field3:12,
    }.leak_into_prefix();

    {// Casting Prefix3 to Prefix2 with different field accessibilities	
        {
            let value:&Prefix2<(F,F,F,F)>=unsafe{ transmute_reference(prefix3) };

            assert_eq!(value.field0(),None);
            assert_eq!(value.field1(),None);
            assert_eq!(value.field2(),None);
        }
        {
            let value:&Prefix2<(T,F,F,F)>=unsafe{ transmute_reference(prefix3) };

            assert_eq!(value.field0(),Some(1));
            assert_eq!(value.field1(),None);
            assert_eq!(value.field2(),None);
        }
        {
            let value:&Prefix2<(F,T,F,F)>=unsafe{ transmute_reference(prefix3) };

            assert_eq!(value.field0(),None);
            assert_eq!(value.field1(),Some(3));
            assert_eq!(value.field2(),None);
        }
        {
            let value:&Prefix2<(F,F,T,F)>=unsafe{ transmute_reference(prefix3) };

            assert_eq!(value.field0(),None);
            assert_eq!(value.field1(),None);
            assert_eq!(value.field2(),Some(7));
        }
        {
            let value:&Prefix2<(T,T,T,T)>=unsafe{ transmute_reference(prefix3) };

            assert_eq!(value.field0(),Some(1));
            assert_eq!(value.field1(),Some(3));
            assert_eq!(value.field2(),Some(7));
        }
    }

    {// Casting Prefix3 to Prefix3UncondPrefix with different field accessibilities 
        {
            let value:&Prefix3UncondPrefix<(F,F,F,F)>=unsafe{ transmute_reference(prefix3) };

            assert_eq!(value.field0(),1);
            assert_eq!(value.field1(),None);
            assert_eq!(value.field2(),None);
            assert_eq!(value.field2(),None);
        }
        {
            let value:&Prefix3UncondPrefix<(F,F,T,F)>=unsafe{ transmute_reference(prefix3) };

            assert_eq!(value.field0(),1);
            assert_eq!(value.field1(),None);
            assert_eq!(value.field2(),Some(7));
            assert_eq!(value.field3(),None);
        }
        {
            let value:&Prefix3UncondPrefix<(T,T,T,T)>=unsafe{ transmute_reference(prefix3) };

            assert_eq!(value.field0(),1);
            assert_eq!(value.field1(),Some(3));
            assert_eq!(value.field2(),Some(7));
            assert_eq!(value.field3(),Some(12));
        }
    }
}
