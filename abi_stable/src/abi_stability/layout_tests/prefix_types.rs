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
    #[sabi(kind(Prefix(prefix_struct="Prefix_Prefix")))]
    pub struct Prefix {
        #[sabi(last_prefix_field)]
        pub field0: u8,
    }
}

mod prefix1 {
    use super::custom_default;
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(inside_abi_stable_crate)]
    #[sabi(kind(Prefix(prefix_struct="Prefix_Prefix")))]
    #[sabi(missing_field(with="custom_default::<_>"))]
    pub struct Prefix {
        #[sabi(last_prefix_field)]
        pub field0: u8,
        pub field1: u16,
    }
}

mod prefix2 {
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(inside_abi_stable_crate)]
    #[sabi(kind(Prefix(prefix_struct="Prefix_Prefix")))]
    #[sabi(missing_field(default))]
    pub struct Prefix {
        #[sabi(last_prefix_field)]
        pub field0: u8,
        pub field1: u16,
        pub field2: u32,
    }
}

// Prefix types have to keep the same alignment when fields are added
mod prefix2_misaligned {
    #[repr(C,align(16))]
    #[derive(StableAbi)]
    // #[sabi(debug_print)]
    #[sabi(inside_abi_stable_crate)]
    #[sabi(kind(Prefix(prefix_struct="Prefix_Prefix")))]
    pub struct Prefix {
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
    #[sabi(kind(Prefix(prefix_struct="Prefix_Prefix")))]
    pub struct Prefix {
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
    #[sabi(kind(Prefix(prefix_struct="Prefix_Prefix")))]
    #[sabi(missing_field(panic))]
    pub struct Prefix {
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



static PREF_0:&'static AbiInfoWrapper = <&prefix0::Prefix_Prefix>::ABI_INFO;
static PREF_1:&'static AbiInfoWrapper = <&prefix1::Prefix_Prefix>::ABI_INFO;
static PREF_2:&'static AbiInfoWrapper = <&prefix2::Prefix_Prefix>::ABI_INFO;
static PREF_3:&'static AbiInfoWrapper = <&prefix3::Prefix_Prefix>::ABI_INFO;


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

            // println!(
            //     "(fields_this:{},fields_other:{})",
            //     t_prefix.fields.len(),
            //     o_prefix.fields.len()
            // );

            let prefix_type_map=globals.prefix_type_map.lock().unwrap();
            if t_prefix.fields.len() <= o_prefix.fields.len() {
                res.unwrap_or_else(|e| panic!("{:#?}",e) );


                let deref_this=dereference_abi(this .get());
                let deref_other=dereference_abi(other.get());
                let t_id=deref_this.get_utypeid();
                let o_id=deref_other.get_utypeid();
                // println!("t_id:{:?}\no_id:{:?}\n",t_id,o_id);

                let t_map_prefix=prefix_type_map.get(&t_id);
                let o_map_prefix=prefix_type_map.get(&o_id);

                // fn format_opt_ref<T>(ref_:Option<&T>)->String{
                //     match ref_ {
                //         Some(x)=>format!("{:p}",x),
                //         None=>"null".into(),
                //     }
                // }
                // println!("t_map_prefix:{}",format_opt_ref(t_map_prefix) );
                // println!("o_map_prefix:{}",format_opt_ref(o_map_prefix) );

                let t_map_prefix=t_map_prefix.unwrap();
                let o_map_prefix=o_map_prefix.unwrap();

                for pre in vec![o_prefix,*t_map_prefix,*o_map_prefix] {
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
            .map(|((_,l_prefix),(_,r_prefix))| (*l_prefix,*r_prefix) )
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
            check_abi_stability_with_globals(this,other,&globals)
                .unwrap_or_else(|e| panic!("{:#?}",e) );

            let prefix_type_map=globals.prefix_type_map.lock().unwrap();

            let t_prefix=PrefixTypeMetadata::new(dereference_abi(this.get()).layout);
            let o_prefix=PrefixTypeMetadata::new(dereference_abi(other.get()).layout);

            let deref_this=dereference_abi(this .get());
            let deref_other=dereference_abi(other.get());
            let t_id=deref_this.get_utypeid();
            let o_id=deref_other.get_utypeid();

            let t_map_prefix=prefix_type_map.get(&t_id);
            let o_map_prefix=prefix_type_map.get(&o_id);


            let t_map_prefix=t_map_prefix.unwrap();
            let o_map_prefix=o_map_prefix.unwrap();

            for pre in vec![o_prefix,*t_map_prefix,*o_map_prefix] {
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
        }
    }
}


#[cfg_attr(not(miri),test)]
fn prefix_is_same_alignment(){
    let globals=CheckingGlobals::new();
    let misaligned = <&prefix2_misaligned::Prefix_Prefix>::ABI_INFO;

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
        let mismatched_prefix=<&prefix2_different_prefix::Prefix_Prefix>::ABI_INFO;
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


unsafe fn transmute_reference<T,U>(ref_:&T)->&U{
    &*(ref_ as *const _ as *const U)
}


#[cfg_attr(not(miri),test)]
fn prefix_on_nonexistent_field() {
    let prefix0=
        prefix0::Prefix{
            field0:1,
        }.leak_into_prefix();

    {
        let value1:&prefix1::Prefix_Prefix=unsafe{ transmute_reference(prefix0) };
        assert_eq!(value1.field0(),1);
        assert_eq!(value1.field1(),custom_default::<u16>());
    }
    {
        let value2:&prefix2::Prefix_Prefix=unsafe{ transmute_reference(prefix0) };
        assert_eq!(value2.field0(),1);
        assert_eq!(value2.field1(),0);
        assert_eq!(value2.field2(),0);
    }
    {
        let value3:&prefix3::Prefix_Prefix=unsafe{ transmute_reference(prefix0) };
        assert_eq!(value3.field0(),1);
        must_panic(file_span!(),||value3.field1()).unwrap();
        must_panic(file_span!(),||value3.field2()).unwrap();
        must_panic(file_span!(),||value3.field3()).unwrap();
    }
}