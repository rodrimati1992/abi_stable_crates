use crate::{
    StableAbi,
    abi_stability::check_layout_compatibility,
    type_layout::{TypeLayout,TLData},
};


#[repr(C)]
#[derive(StableAbi)]
#[sabi(tag="tag![ <T as crate::const_utils::AssocStr>::STR ]")]
struct FieldBound<T>{
    #[sabi(bound="crate::const_utils::AssocStr")]
    value:T,
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(bound="T:crate::const_utils::AssocStr")]
#[sabi(tag="tag![ <T as crate::const_utils::AssocStr>::STR ]")]
struct TypeBound<T>{
    value:T,
}



#[repr(C)]
#[derive(StableAbi)]
#[sabi(unsafe_opaque_fields)]
pub struct UnsafeOpaqueFields0<T,U>{
    hello:T,
    world:U,
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub struct UnsafeSabiOpaqueFields0<T,U>{
    hello:T,
    world:U,
}

#[repr(C)]
#[derive(StableAbi)]
pub struct UnsafeOpaqueField0<T,U>{
    hello:T,
    #[sabi(unsafe_opaque_field)]
    world:U,
}
    
#[repr(C)]
#[derive(StableAbi)]
pub struct UnsafeSabiOpaqueField0<T,U>{
    hello:T,
    #[sabi(unsafe_sabi_opaque_field)]
    world:U,
}



////////////////////////////////////////////////////////////////////////////////


#[test]
fn is_sabi_opaque_fields(){
    let list:Vec<(&'static TypeLayout,Vec<Option<&'static str>>)>=vec![
        (
            UnsafeOpaqueFields0::<u32,u32>::LAYOUT,
            vec![Some("OpaqueField"),Some("OpaqueField")]
        ),
        (
            UnsafeSabiOpaqueFields0::<u32,u32>::LAYOUT,
            vec![Some("SabiOpaqueField"),Some("SabiOpaqueField")]
        ),
        (
            UnsafeOpaqueField0::<u32,u32>::LAYOUT,
            vec![None,Some("OpaqueField")]
        ),
        (
            UnsafeSabiOpaqueField0::<u32,u32>::LAYOUT,
            vec![None,Some("SabiOpaqueField")]
        ),
    ];

    for (layout,field_typenames) in list {
        let fields=match layout.data {
            TLData::Struct{fields}=>fields,
            _=>unreachable!()
        };

        for (field,field_typename) in fields.into_iter().zip(field_typenames) {
            if let Some(typename)=field_typename {
                assert_eq!( field.layout.get().name(), typename );
            }
        }
    }
}


#[test]
fn same_opaque_fields() {
    let lists=vec![
        vec![
            UnsafeOpaqueFields0::<u32,u32>::LAYOUT,
            UnsafeOpaqueFields0::<u32,i32>::LAYOUT,
            UnsafeOpaqueFields0::<i32,u32>::LAYOUT,
            UnsafeOpaqueFields0::<i32,i32>::LAYOUT,
        ],
        vec![
            UnsafeSabiOpaqueFields0::<u32,u32>::LAYOUT,
            UnsafeSabiOpaqueFields0::<u32,i32>::LAYOUT,
            UnsafeSabiOpaqueFields0::<i32,u32>::LAYOUT,
            UnsafeSabiOpaqueFields0::<i32,i32>::LAYOUT,
        ],
        vec![
            UnsafeOpaqueField0::<u32,u32>::LAYOUT,
            UnsafeOpaqueField0::<u32,i32>::LAYOUT,
        ],
        vec![
            UnsafeSabiOpaqueField0::<u32,u32>::LAYOUT,
            UnsafeSabiOpaqueField0::<u32,i32>::LAYOUT,
        ],
    ];

    for list in lists {
        for window in list.windows(2) {
            check_layout_compatibility( window[0], window[1] ).unwrap();
        }
    }
}


#[test]
fn different_opaque_fields() {
    let list = vec![
        UnsafeOpaqueFields0::<u32,u32>::LAYOUT,
        UnsafeOpaqueFields0::<u32,u64>::LAYOUT,
        
        UnsafeSabiOpaqueFields0::<u32,u32>::LAYOUT,
        UnsafeSabiOpaqueFields0::<u32,u64>::LAYOUT,

        UnsafeOpaqueField0::<u32,u32>::LAYOUT,
        UnsafeOpaqueField0::<i32,u32>::LAYOUT,
        UnsafeOpaqueField0::<u32,u64>::LAYOUT,

        UnsafeSabiOpaqueField0::<u32,u32>::LAYOUT,
        UnsafeSabiOpaqueField0::<i32,u32>::LAYOUT,
        UnsafeSabiOpaqueField0::<u32,u64>::LAYOUT,
    ];

    let (_dur, ()) = core_extensions::measure_time::measure(|| {
        for (i, this) in list.iter().cloned().enumerate() {
            for (j, other) in list.iter().cloned().enumerate() {
                let res=check_layout_compatibility(this, other);
                if i == j {
                    res.unwrap();
                } else {
                    res.unwrap_err();
                }
            }
        }
    });

    // println!("taken {} to check all listed layouts", dur);
}