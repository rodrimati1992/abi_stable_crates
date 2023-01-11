
use abi_stable::{
    sabi_trait::prelude::TD_Opaque,
    std_types::{RArc, RBox, Tuple1, Tuple2, Tuple3}
};


#[abi_stable::sabi_trait]
pub trait RFoo<'a, T: Copy + 'a> {
    fn get(&'a self) -> &'a T;
}

impl<'a, A: Copy + 'a> RFoo<'a, A> for Tuple1<A> {
    fn get(&'a self) -> &'a A {
        &self.0
    }
}

impl<'a, A: 'a, B: Copy + 'a> RFoo<'a, B> for Tuple2<A, B> {
    fn get(&'a self) -> &'a B {
        &self.1
    }
}

impl<'a, A: 'a, B: 'a, C: Copy + 'a> RFoo<'a, C> for Tuple3<A, B, C> {
    fn get(&'a self) -> &'a C {
        &self.2
    }
}

impl<'a, T> RFoo<'a, T> for RArc<T>
where
    T: 'a + Copy,
{
    fn get(&'a self) -> &'a T {
        &**self
    }
}


fn main() {
    let object = &RFoo_TO::from_ptr(RBox::new(RArc::new(76)), TD_Opaque);
    let tuple1_object = &RFoo_TO::from_ptr(RArc::new(Tuple1(100)), TD_Opaque);
    let tuple2_object = &RFoo_TO::from_value(Tuple2(101u32, 202_u32), TD_Opaque);
    let tuple3_object = &RFoo_TO::from_value(Tuple3(11, 22, 300_u32), TD_Opaque);

    assert_eq!(object.get(), &76);
    assert_eq!(tuple1_object.get(), &100);
    assert_eq!(tuple2_object.get(), &202);
    assert_eq!(tuple3_object.get(), &300);

    assert_eq!(RFoo::get(object), &76);
    assert_eq!(RFoo::get(tuple1_object), &100);
    assert_eq!(RFoo::get(tuple2_object), &202);
    assert_eq!(RFoo::get(tuple3_object), &300);

}