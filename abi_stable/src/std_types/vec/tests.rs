use super::*;

use std::{iter, sync::Arc};

#[allow(unused_imports)]
use core_extensions::SelfOps;

use crate::{
    test_utils::{must_panic, ShouldHavePanickedAt},
    traits::IntoReprC,
};

#[cfg(feature = "rust_1_64")]
#[test]
fn const_as_slice_test() {
    const RV: &RVec<u8> = &RVec::new();
    const SLICE: &[u8] = RV.as_slice();

    assert_eq!(SLICE, [0u8; 0]);
}

#[test]
#[allow(clippy::drop_non_drop)]
fn test_equality_between_vecs() {
    struct F<T>(T);

    fn eq<'a, 'b, T>(left: &RVec<&'a T>, right: &RVec<&'b T>) -> bool
    where
        T: std::cmp::PartialEq,
    {
        left == right
    }

    let aaa = F(3);
    let bbb = F(5);
    let ccc = F(8);
    let ddd = F(13);

    {
        let v0 = rvec![&aaa.0, &bbb.0];
        let v1 = rvec![&ccc.0, &ddd.0];

        assert!(!eq(&v0, &v1));
    }

    // forcing the lifetime to extend to the end of the scope
    drop(ccc);
    drop(ddd);
    drop(aaa);
    drop(bbb);
}

fn _assert_covariant<'a: 'b, 'b, T>(x: RVec<&'a T>) -> RVec<&'b T> {
    x
}
fn _assert_covariant_vec<'a: 'b, 'b, T>(x: Vec<&'a T>) -> Vec<&'b T> {
    x
}

fn typical_list(upto: u8) -> (Vec<u8>, RVec<u8>) {
    let orig = (b'a'..=upto).collect::<Vec<_>>();
    (orig.clone(), orig.iter().cloned().collect())
}

#[test]
fn vec_drain() {
    let (original, list) = typical_list(b'j');
    let pointer = Arc::new(());

    macro_rules! assert_eq_drain {
        ($range: expr, $after_drain: expr) => {
            let range = $range;
            {
                let after_drain: Vec<u8> = $after_drain;
                let mut list = list.clone();
                let list_2 = list.drain(range.clone()).collect::<Vec<_>>();
                assert_eq!(&*list_2, &original[range.clone()], "");
                assert_eq!(&*list, &*after_drain,);
            }
            {
                let length = 10;
                let mut list_b = iter::repeat(pointer.clone())
                    .take(length)
                    .collect::<Vec<_>>();
                let range_len = list[range.clone()].len();
                list_b.drain(range.clone());
                assert_eq!(list_b.len(), length - range_len);
                assert_eq!(Arc::strong_count(&pointer), 1 + length - range_len);
            }
        };
    }

    assert_eq_drain!(.., vec![]);
    assert_eq_drain!(..3, vec![b'd', b'e', b'f', b'g', b'h', b'i', b'j']);
    assert_eq_drain!(3.., vec![b'a', b'b', b'c']);
    assert_eq_drain!(3..5, vec![b'a', b'b', b'c', b'f', b'g', b'h', b'i', b'j']);
}

#[test]
fn insert_remove() {
    let (original, list) = typical_list(b'd');

    let assert_insert_remove = |position: usize, expected: Vec<u8>| {
        let mut list = list.clone();

        let val = list.try_remove(position).unwrap();
        assert_eq!(&*list, &*expected);

        list.insert(position, val);
        assert_eq!(&*list, &*original);
    };

    //vec![b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j']

    assert_insert_remove(0, vec![b'b', b'c', b'd']);
    assert_insert_remove(1, vec![b'a', b'c', b'd']);
    assert_insert_remove(2, vec![b'a', b'b', b'd']);
    assert_insert_remove(3, vec![b'a', b'b', b'c']);

    {
        let mut list = RVec::new();
        list.insert(0, b'a');
        list.insert(1, b'b');
        list.insert(1, b'c');
        list.insert(0, b'd');
        assert_eq!(&*list, &[b'd', b'a', b'c', b'b']);
    }
}

#[test]
fn remove_panics() -> Result<(), ShouldHavePanickedAt> {
    let mut list = RVec::new();
    for (i, elem) in (10..20).enumerate() {
        must_panic(|| list.remove(i))?;
        list.push(elem);
        list.remove(i);
        list.push(elem);
    }
    Ok(())
}

#[test]
fn swap_remove() {
    let mut list = vec![10, 11, 12, 13, 14, 15].into_c();

    assert_eq!(list.swap_remove(5), 15);
    assert_eq!(&*list, &*vec![10, 11, 12, 13, 14]);
    assert_eq!(list.swap_remove(0), 10);
    assert_eq!(&*list, &*vec![14, 11, 12, 13]);
    assert_eq!(list.swap_remove(1), 11);
    assert_eq!(&*list, &*vec![14, 13, 12]);
}

#[test]
fn push_pop() {
    let mut list = RVec::<u32>::new();

    assert_eq!(list.pop(), None);

    for elem in 10..=13 {
        list.push(elem);
    }

    assert_eq!(&*list, &[10, 11, 12, 13]);

    assert_eq!(list.pop(), Some(13));
    assert_eq!(&*list, &[10, 11, 12]);

    assert_eq!(list.pop(), Some(12));
    assert_eq!(&*list, &[10, 11]);

    assert_eq!(list.pop(), Some(11));
    assert_eq!(&*list, &[10]);

    assert_eq!(list.pop(), Some(10));
    assert_eq!(&*list, <&[u32]>::default());

    assert_eq!(list.pop(), None);
}

#[test]
fn truncate() {
    {
        let orig = vec![0, 1, 2, 3, 4];
        let mut list = orig.clone().into_c();

        list.truncate(6);
        assert_eq!(&*list, &*orig);
        for i in (0..5).rev() {
            list.truncate(i);
            assert_eq!(&*list, &orig[..i]);
        }
    }
    {
        let pointer = Arc::new(());

        let length = 10;
        let mut list = iter::repeat(pointer.clone())
            .take(length)
            .collect::<Vec<_>>();

        assert_eq!(Arc::strong_count(&pointer), 1 + length);
        for i in (0..list.len()).rev() {
            list.truncate(i);
            assert_eq!(Arc::strong_count(&pointer), 1 + i);
        }
        assert_eq!(Arc::strong_count(&pointer), 1);
    }
}

#[test]
fn retain() {
    let orig = vec![2, 3, 4, 5, 6, 7, 8];
    let copy = orig.clone().piped(RVec::from);
    {
        let mut copy = copy.clone();
        copy.retain(|&v| v % 2 == 0);
        assert_eq!(&*copy, &[2, 4, 6, 8][..]);
    }
    {
        let mut copy = copy.clone();
        copy.retain(|&v| v % 2 == 1);
        assert_eq!(&*copy, &[3, 5, 7][..]);
    }
    {
        let mut copy = copy.clone();
        copy.retain(|_| true);
        assert_eq!(&*copy, &*orig);
    }
    {
        let mut copy = copy.clone();
        copy.retain(|_| false);
        assert_eq!(&*copy, <&[i32]>::default());
    }
    {
        let mut copy = copy.clone();
        let mut i = 0;
        copy.retain(|_| {
            let cond = i % 2 == 0;
            i += 1;
            cond
        });
        assert_eq!(&*copy, &[2, 4, 6, 8][..]);
    }
    {
        let mut copy = copy.clone();
        let mut i = 0;
        copy.retain(|_| {
            let cond = i % 3 == 0;
            i += 1;
            cond
        });
        assert_eq!(&*copy, &[2, 5, 8][..]);
    }
    {
        let mut copy = copy;
        let mut i = 0;
        must_panic(|| {
            copy.retain(|_| {
                i += 1;
                if i == 4 {
                    panic!()
                }
                true
            });
        })
        .unwrap();
        assert_eq!(&copy[..], &orig[..]);
    }
}

#[test]
fn resize() {
    let full = vec![1, 2, 3, 4, 5];
    let mut list = RVec::new();

    for i in 1..=5 {
        list.resize(i, i);
        assert_eq!(list[i - 1], i);
    }
    assert_eq!(&*list, &*full);

    for i in (1..=5).rev() {
        list.resize(i, 0);
        assert_eq!(&*list, &full[..i]);
    }
}

#[test]
fn extend_from_slice() {
    let mut list = RVec::new();
    let from: Vec<String> = vec!["hello 0".into(), "hello 1".into(), "hello 2".into()];
    list.extend_from_slice(&[]);
    list.extend_from_slice(&from);
    assert_eq!(&*list, &*from);

    let from2: Vec<String> = vec!["fuck".into(), "that".into()];
    let from_upto2 = from.iter().chain(from2.iter()).cloned().collect::<Vec<_>>();
    list.extend_from_slice(&from2);
    assert_eq!(&*list, &*from_upto2);
}

#[test]
fn extend_from_copy_slice() {
    let mut list = RVec::new();
    let from: Vec<&str> = vec!["hello 0", "hello 1", "hello 2"];
    list.extend_from_copy_slice(&[]);
    list.extend_from_copy_slice(&from);
    assert_eq!(&*list, &*from);

    let from2: Vec<&str> = vec!["fuck", "that"];
    let from_upto2 = from.iter().chain(from2.iter()).cloned().collect::<Vec<_>>();
    list.extend_from_copy_slice(&from2);
    assert_eq!(&*list, &*from_upto2);
}

#[test]
fn extend() {
    let mut list = RVec::new();
    let from: Vec<&str> = vec!["hello 0", "hello 1", "hello 2"];
    list.extend(from.iter().cloned());
    let from_empty: &[&str] = &[];
    list.extend(from_empty.iter().cloned());
    assert_eq!(&*list, &*from);

    let from2: Vec<&str> = vec!["fuck", "that"];
    let from_upto2 = from.iter().chain(from2.iter()).cloned().collect::<Vec<_>>();
    list.extend(from2.iter().cloned());
    assert_eq!(&*list, &*from_upto2);
}

#[test]
fn append() {
    let mut into = RVec::<u16>::new();

    into.append(&mut RVec::new());
    assert_eq!(into, Vec::<u16>::new());

    {
        let mut from = rvec![3u16, 5, 8];
        into.append(&mut from);
        assert_eq!(into, [3u16, 5, 8][..]);
        assert_eq!(from, Vec::<u16>::new());
    }

    into.append(&mut RVec::new());
    assert_eq!(into, [3u16, 5, 8][..]);

    {
        let mut from = rvec![13u16];
        into.append(&mut from);
        assert_eq!(into, [3u16, 5, 8, 13][..]);
        assert_eq!(from, Vec::<u16>::new());
    }
}

#[test]
fn into_iter() {
    assert_eq!(RVec::<()>::new().into_iter().next(), None);

    let arc = Arc::new(0);

    let orig = vec![arc.clone(), arc.clone(), arc.clone(), arc.clone()];
    let mut list = orig.clone().into_c();
    assert_eq!(list.clone().into_iter().collect::<Vec<_>>(), orig);

    assert_eq!(Arc::strong_count(&arc), 9);

    assert_eq!((&list).into_iter().cloned().collect::<Vec<_>>(), orig);
    assert_eq!(
        (&mut list)
            .into_iter()
            .map(|v: &mut Arc<i32>| v.clone())
            .collect::<Vec<_>>(),
        orig
    );
}

#[test]
fn into_iter_as_str() {
    let mut orig = vec![10, 11, 12, 13];
    let mut iter = orig.clone().into_c().into_iter();
    let mut i = 0;

    loop {
        assert_eq!(&orig[i..], iter.as_slice());
        assert_eq!(&mut orig[i..], iter.as_mut_slice());
        i += 1;
        if iter.next().is_none() {
            break;
        }
    }
}

#[test]
fn clone() {
    let orig = vec![10, 11, 12, 13];
    let clon = orig.clone();
    assert_ne!(orig.as_ptr(), clon.as_ptr());
    assert_eq!(orig, clon);
}

#[test]
fn from_vec() {
    let orig = vec![10, 11, 12, 13];
    let buffer_ptr = orig.as_ptr();
    let list = orig.into_c();
    assert_eq!(buffer_ptr, list.as_ptr());
}

#[test]
fn test_drop() {
    let pointer = Arc::new(());
    let length = 10;
    let list = iter::repeat(pointer.clone())
        .take(length)
        .collect::<Vec<_>>();
    assert_eq!(Arc::strong_count(&pointer), 1 + length);
    drop(list);
    assert_eq!(Arc::strong_count(&pointer), 1);
}

#[test]
fn into_vec() {
    let orig = vec![10, 11, 12, 13];
    let list = orig.clone().into_c();
    {
        let list = list.clone();
        let list_ptr = list.as_ptr();
        let list_1 = list.into_vec();
        assert_eq!(list_ptr, list_1.as_ptr());
        assert_eq!(orig, list_1);
    }
    {
        let list = list.set_vtable_for_testing();
        let list_ptr = list.as_ptr() as usize;
        let list_1 = list.into_vec();
        // No, MIR interpreter,
        // I'm not dereferencing a pointer here, I am comparing their adresses.
        assert_ne!(list_ptr, list_1.as_ptr() as usize);
        assert_eq!(orig, list_1);
    }
}

#[test]
fn rvec_macro() {
    assert_eq!(RVec::<u32>::new(), rvec![]);
    assert_eq!(RVec::from(vec![0]), rvec![0]);
    assert_eq!(RVec::from(vec![0, 3]), rvec![0, 3]);
    assert_eq!(RVec::from(vec![0, 3, 6]), rvec![0, 3, 6]);
    assert_eq!(RVec::from(vec![1; 10]), rvec![1;10]);
}

// Adapted from Vec tests
// (from rustc 1.50.0-nightly (eb4fc71dc 2020-12-17))
#[test]
fn retain_panic() {
    use std::{panic::AssertUnwindSafe, rc::Rc, sync::Mutex};

    struct Check {
        index: usize,
        drop_counts: Rc<Mutex<RVec<usize>>>,
    }

    impl Drop for Check {
        fn drop(&mut self) {
            self.drop_counts.lock().unwrap()[self.index] += 1;
            println!("drop: {}", self.index);
        }
    }

    let check_count = 10;
    let drop_counts = Rc::new(Mutex::new(rvec![0_usize; check_count]));
    let mut data: RVec<Check> = (0..check_count)
        .map(|index| Check {
            index,
            drop_counts: Rc::clone(&drop_counts),
        })
        .collect();

    let _ = std::panic::catch_unwind(AssertUnwindSafe(move || {
        let filter = |c: &Check| {
            if c.index == 2 {
                panic!("panic at index: {}", c.index);
            }
            // Verify that if the filter could panic again on another element
            // that it would not cause a double panic and all elements of the
            // vec would still be dropped exactly once.
            if c.index == 4 {
                panic!("panic at index: {}", c.index);
            }
            c.index < 6
        };
        data.retain(filter);
    }));

    let drop_counts = drop_counts.lock().unwrap();
    assert_eq!(check_count, drop_counts.len());

    for (index, count) in drop_counts.iter().cloned().enumerate() {
        assert_eq!(
            1, count,
            "unexpected drop count at index: {} (count: {})",
            index, count
        );
    }
}

#[test]
fn test_index() {
    let s = rvec![1, 2, 3, 4, 5];
    assert_eq!(s.index(0), &1);
    assert_eq!(s.index(4), &5);
    assert_eq!(s.index(..2), rvec![1, 2]);
    assert_eq!(s.index(1..2), rvec![2]);
    assert_eq!(s.index(3..), rvec![4, 5]);
}

#[test]
fn test_index_mut() {
    let mut s = rvec![1, 2, 3, 4, 5];

    assert_eq!(s.index_mut(0), &mut 1);
    assert_eq!(s.index_mut(4), &mut 5);
    assert_eq!(s.index_mut(..2), &mut rvec![1, 2]);
    assert_eq!(s.index_mut(1..2), &mut rvec![2]);
    assert_eq!(s.index_mut(3..), &mut rvec![4, 5]);
}

#[test]
fn test_slice() {
    let s = rvec![1, 2, 3, 4, 5];

    assert_eq!(s.slice(..), rslice![1, 2, 3, 4, 5]);
    assert_eq!(s.slice(..2), rslice![1, 2]);
    assert_eq!(s.slice(1..2), rslice![2]);
    assert_eq!(s.slice(3..), rslice![4, 5]);
}

#[test]
fn test_slice_mut() {
    let mut s = rvec![1, 2, 3, 4, 5];

    assert_eq!(
        s.slice_mut(..),
        RSliceMut::from_mut_slice(&mut [1, 2, 3, 4, 5])
    );
    assert_eq!(s.slice_mut(..2), RSliceMut::from_mut_slice(&mut [1, 2]));
    assert_eq!(s.slice_mut(1..2), RSliceMut::from_mut_slice(&mut [2]));
    assert_eq!(s.slice_mut(3..), RSliceMut::from_mut_slice(&mut [4, 5]));
}
