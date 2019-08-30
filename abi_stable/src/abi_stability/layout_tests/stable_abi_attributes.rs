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