use super::*;

/// Asserts that all elements are compatible with themselves and
/// not equal to the other elements
fn assert_distinct_elements(array: &[Tag]) {
    let array = array
        .iter()
        .cloned()
        .map(Tag::to_checkable)
        .collect::<Vec<CheckableTag>>();

    for (index, value) in array.iter().enumerate() {
        assert_eq!(value.check_compatible(value), Ok(()));
        for (_, incomp) in array.iter().enumerate().filter(|(i, _)| *i != index) {
            assert_ne!(value.check_compatible(incomp), Ok(()),);
        }
    }
}

const TAG_SET_EMPTY: Tag = Tag::set(rslice![]);
const TAG_ARR_EMPTY: Tag = Tag::arr(rslice![]);

const TAG_SET_0: Tag = Tag::set(rslice![Tag::bool_(false)]);
const TAG_ARR_0: Tag = Tag::arr(rslice![Tag::bool_(false)]);

const TAG_1_ORDER_0_VALUE: RSlice<'static, Tag> = rslice![
    Tag::bool_(false),
    Tag::bool_(true),
    Tag::arr(rslice![Tag::uint(0), Tag::int(-100)]),
    Tag::int(-100),
    Tag::int(100),
    Tag::set(rslice![Tag::str("Cap'n Rogers"), Tag::str("ironman")]),
    Tag::uint(!0),
    Tag::uint(100),
];
const TAG_SET_1_ORDER_0: Tag = Tag::set(TAG_1_ORDER_0_VALUE);
const TAG_ARR_1_ORDER_0: Tag = Tag::arr(TAG_1_ORDER_0_VALUE);

const TAG_1_ORDER_1_VALUE: RSlice<'static, Tag> = rslice![
    Tag::uint(!0),
    Tag::int(100),
    Tag::bool_(true),
    Tag::set(rslice![Tag::str("ironman"), Tag::str("Cap'n Rogers")]),
    Tag::uint(100),
    Tag::int(-100),
    Tag::bool_(false),
    Tag::arr(rslice![Tag::uint(0), Tag::int(-100)]),
];
const TAG_SET_1_ORDER_1: Tag = Tag::set(TAG_1_ORDER_1_VALUE);
const TAG_ARR_1_ORDER_1: Tag = Tag::arr(TAG_1_ORDER_1_VALUE);

const TAG_2_VALUE: RSlice<'static, Tag> = rslice![
    Tag::uint(!0),
    Tag::int(100),
    Tag::arr(rslice![Tag::uint(0), Tag::int(-100)]),
    Tag::bool_(true),
    Tag::set(rslice![
        Tag::str("Cap'n Rogers"),
        Tag::str("ironman"),
        Tag::str("Fe-male")
    ]),
    Tag::uint(100),
    Tag::int(-100),
    Tag::bool_(false),
    Tag::str("what the [redacted_for_g]"),
];
const TAG_SET_2: Tag = Tag::set(TAG_2_VALUE);
const TAG_ARR_2: Tag = Tag::arr(TAG_2_VALUE);

const TAG_MAP_EMPTY: Tag = Tag::map(rslice![]);

const TAG_MAP_0A: Tag = tag!({
    Tag::null()=>"baz",
    "world"=>"hello",
    "hello"=>"huh?",
    "hello"=>"world",
});

const TAG_MAP_0B: Tag = tag!({
    "hello"=>"world",
    "world"=>"hello",
});

const TAG_MAP_0E: Tag = tag!({ 0=>"what", 1=>"foo" });
const TAG_MAP_0F: Tag = tag!({ 0=>"foo", 1=>"what" });
const TAG_MAP_0G: Tag = tag!({ 1=>"foo", 2=>"what" });

const TAG_MAP_1A: Tag = tag!({
    "world"=>"hello",
    "foo"=>"bar",
    "foo"=>"baz",
    "hello"=>"huh?",
    "hello"=>"world",
});

const TAG_MAP_1B: Tag = tag!({
    "hello"=>"world",
    "world"=>"hello",
    "foo"=>"baz",
});

const TAG_MAP_2A: Tag = tag!({
    Tag::null()=>"baz",
    "world"=>"hello",
    Tag::null()=>"baz",
    0=>"fun",
    0=>"house",
    Tag::uint(100)=>"what",
    Tag::uint(100)=>"the",
    "foo"=>"bar",
    "foo"=>"baz",
    "hello"=>"huh?",
    "hello"=>"world",
});

const TAG_MAP_2B: Tag = tag!({
    "hello"=>"world",
    "world"=>"hello",
    "foo"=>"baz",
    0=>"house",
    Tag::uint(100)=>"the",
});

fn assert_subsets(array: &[(u32, Tag)]) {
    let array = array
        .iter()
        .cloned()
        .map(|(i, v)| (i, Tag::to_checkable(v)))
        .collect::<Vec<_>>();

    for (l_index, (l_ident, l_value)) in array.iter().enumerate() {
        for (r_index, (r_ident, r_value)) in array.iter().enumerate() {
            let res = l_value.check_compatible(r_value);

            if l_index <= r_index || l_ident == r_ident {
                assert_eq!(res, Ok(()), "left:{}\n\nright:{}", l_value, r_value);
            } else {
                assert_ne!(res, Ok(()), "left:{}\n\nright:{}", l_value, r_value);
            }
        }
    }
}

#[test]
fn check_set_compatibility() {
    assert_subsets(&[
        (0, TAG_SET_EMPTY),
        (1, TAG_SET_0),
        (2, TAG_SET_1_ORDER_0),
        (2, TAG_SET_1_ORDER_1),
        (3, TAG_SET_2),
    ]);
}

#[test]
fn check_map_compatibility() {
    assert_subsets(&[
        (0, TAG_MAP_EMPTY),
        (1, TAG_MAP_0A),
        (1, TAG_MAP_0B),
        (2, TAG_MAP_1A),
        (2, TAG_MAP_1B),
        (3, TAG_MAP_2A),
        (3, TAG_MAP_2B),
    ]);

    assert_distinct_elements(&[TAG_MAP_0E, TAG_MAP_0F, TAG_MAP_0G]);
}

#[test]
fn check_arr_compatibility() {
    assert_distinct_elements(&[
        TAG_ARR_EMPTY,
        TAG_ARR_0,
        TAG_ARR_1_ORDER_0,
        TAG_ARR_1_ORDER_1,
        TAG_ARR_2,
    ]);
}

const TAG_BOOLS: &[Tag] = &[Tag::bool_(false), Tag::bool_(true)];

#[test]
fn check_bool() {
    assert_distinct_elements(TAG_BOOLS);
}

const TAG_UINTS: &[Tag] = &[Tag::uint(0), Tag::uint(1), Tag::uint(2)];

#[test]
fn check_uint() {
    assert_distinct_elements(TAG_UINTS);
}

const TAG_INTS: &[Tag] = &[
    Tag::int(-2),
    Tag::int(-1),
    Tag::int(0),
    Tag::int(1),
    Tag::int(2),
];

#[test]
fn check_int() {
    assert_distinct_elements(TAG_INTS);
}

const TAG_STRS: &[Tag] = &[
    Tag::str("what"),
    Tag::str("the"),
    Tag::str("is"),
    Tag::str("this"),
    Tag::str("Hello, world!"),
];

#[test]
fn check_str() {
    assert_distinct_elements(TAG_STRS);
}

#[test]
fn check_different_same_variant() {
    assert_distinct_elements(&[
        Tag::bool_(false),
        Tag::int(0),
        Tag::uint(0),
        Tag::str(""),
        Tag::arr(rslice![]),
        Tag::set(rslice![]),
        Tag::map(rslice![]),
    ]);
}

#[test]
fn check_null() {
    let mut list = vec![
        TAG_SET_EMPTY,
        TAG_SET_0,
        TAG_SET_1_ORDER_0,
        TAG_SET_1_ORDER_1,
        TAG_SET_2,
        TAG_ARR_EMPTY,
        TAG_ARR_0,
        TAG_ARR_1_ORDER_0,
        TAG_ARR_1_ORDER_1,
        TAG_ARR_2,
    ];
    list.extend_from_slice(TAG_BOOLS);
    list.extend_from_slice(TAG_UINTS);
    list.extend_from_slice(TAG_INTS);
    list.extend_from_slice(TAG_STRS);

    let null_checkable = Tag::null().to_checkable();

    let list = list
        .into_iter()
        .map(Tag::to_checkable)
        .collect::<Vec<CheckableTag>>();

    for elems in &list {
        assert_eq!(null_checkable.check_compatible(elems), Ok(()));

        assert_ne!(elems.check_compatible(&null_checkable), Ok(()));
    }
}
