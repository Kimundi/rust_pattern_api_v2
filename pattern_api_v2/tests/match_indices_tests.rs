#[macro_use]
extern crate pattern_api_v2_test_support;
extern crate pattern_api_v2;

use pattern_api_v2::slice::Elem;
use pattern_api_v2::iterators::{MatchIndices, RMatchIndices};
use pattern_api_v2::os_string::shared::PartialUnicode as UOsStr;
use pattern_api_v2::os_string::mutable::PartialUnicode as UMutOsStr;

use pattern_api_v2_test_support::{os, s};
use std::ffi::{OsStr};

iterator_cross_test! {
    forward-backward, MatchIndices::new, RMatchIndices::new, {
        str, &str: &s("abbcbbd"), _: "bb",
            [(1, "bb"), (4, "bb")],
            [(1, "bb"), (4, "bb")]
        str_mut, &mut str: &mut s("abbcbbd"), _: "bb",
            [(1, ms!("bb")), (4, ms!("bb"))],
            [(1, ms!("bb")), (4, ms!("bb"))]

        os_str_str, &OsStr: &os(b"abbcbbd"), _: "bb",
            [(1, os!(b"bb")), (4, os!(b"bb"))],
            [(1, os!(b"bb")), (4, os!(b"bb"))]
        os_str_mut_str, &mut OsStr: &mut os(b"abbcbbd"), _: "bb",
            [(1, mos!(b"bb")), (4, mos!(b"bb"))],
            [(1, mos!(b"bb")), (4, mos!(b"bb"))]
        os_str_os, &OsStr: &os(b"abbcbbd"), _: os!(b"bb"),
            [(1, os!(b"bb")), (4, os!(b"bb"))],
            [(1, os!(b"bb")), (4, os!(b"bb"))]
        os_str_mut_os, &mut OsStr: &mut os(b"abbcbbd"), _: os!(b"bb"),
            [(1, mos!(b"bb")), (4, mos!(b"bb"))],
            [(1, mos!(b"bb")), (4, mos!(b"bb"))]

        uos_str_str, UOsStr: uos!(b"abbcbbd"), _: "bb",
            [(1, s!("bb")), (4, s!("bb"))],
            [(1, s!("bb")), (4, s!("bb"))]
        uos_str_mut_str, UMutOsStr: muos!(b"abbcbbd"), _: "bb",
            [(1, ms!("bb")), (4, ms!("bb"))],
            [(1, ms!("bb")), (4, ms!("bb"))]

        u8, &[u8]: sl!(b"abbcbbd"), &[_]: b"bb",
            [(1, sl!(b"bb")), (4, sl!(b"bb"))],
            [(1, sl!(b"bb")), (4, sl!(b"bb"))]
        u8_mut, &mut [u8]: msl!(b"abbcbbd"), &[_]: b"bb",
            [(1, msl!(b"bb")), (4, msl!(b"bb"))],
            [(1, msl!(b"bb")), (4, msl!(b"bb"))]
        i32, &[i32]: sl![1,-2,-2,3,-2,-2,4], &[_]: &[-2, -2],
            [(1, sl![-2, -2]), (4, sl![-2, -2])],
            [(1, sl![-2, -2]), (4, sl![-2, -2])]
        i32_mut, &mut [i32]: msl![1,-2,-2,3,-2,-2,4], &[_]: &[-2, -2],
            [(1, msl![-2, -2]), (4, msl![-2, -2])],
            [(1, msl![-2, -2]), (4, msl![-2, -2])]
    }
    double, MatchIndices::new, RMatchIndices::new, {
        str_char, _: "abcbd", _: 'b',
            [(1, "b"), (3, "b")]
        str_pred, _: "abcbd", _: |_| true,
            [(0, "a"), (1, "b"), (2, "c"), (3, "b"), (4, "d")]
        str_mut_char, &mut str: &mut s("abcbd"), _: 'b',
            [(1, ms!("b")), (3, ms!("b"))]
        str_mut_pred, &mut str: &mut s("abcbd"), _: |_| true,
            [(0, ms!("a")), (1, ms!("b")), (2, ms!("c")), (3, ms!("b")), (4, ms!("d"))]

        os_char, &OsStr: &os(b"ab\xbebd"), _: 'b',
            [(1, os!(b"b")), (3, os!(b"b"))]
        os_pred, &OsStr: &os(b"ab\xbebd"), _: |_| true,
            [(0, os!(b"a")), (1, os!(b"b")), (3, os!(b"b")), (4, os!(b"d"))]
        os_mut_char, &mut OsStr: &mut os(b"ab\xbebd"), _: 'b',
            [(1, mos!(b"b")), (3, mos!(b"b"))]
        os_mut_pred, &mut OsStr: &mut os(b"ab\xbebd"), _: |_| true,
            [(0, mos!(b"a")), (1, mos!(b"b")), (3, mos!(b"b")), (4, mos!(b"d"))]

        u8_elem, &[u8]: b"abcbd", _: Elem(b'b'),
            [(1, sl!(b"b")), (3, sl!(b"b"))]
        u8_pred, &[u8]: b"abcbd", _: |_: &_| true,
            [(0, sl!(b"a")), (1, sl!(b"b")), (2, sl!(b"c")), (3, sl!(b"b")), (4, sl!(b"d"))]
        u8_mut_elem, &mut [u8]: &mut{*b"abcbd"}, _: Elem(b'b'),
            [(1, msl!(b"b")), (3, msl!(b"b"))]
        u8_mut_pred, &mut [u8]: &mut{*b"abcbd"}, _: |_: &_| true,
            [(0, msl!(b"a")), (1, msl!(b"b")), (2, msl!(b"c")), (3, msl!(b"b")), (4, msl!(b"d"))]

        i32_elem, &[i32]: &[1,-2,3,-2,4], _: Elem(-2),
            [(1, sl![-2,]), (3, sl![-2,])]
        i32_pred, &[i32]: &[1,-2,3,-2,4], _: |_: &_| true,
            [(0, sl![1,]), (1, sl![-2,]), (2, sl![3,]), (3, sl![-2,]), (4, sl![4,])]
        i32_mut_elem, &mut [i32]: &mut{[1,-2,3,-2,4]}, _: Elem(-2),
            [(1, msl![-2,]), (3, msl![-2,])]
        i32_mut_pred, &mut [i32]: &mut{[1,-2,3,-2,4]}, _: |_: &_| true,
            [(0, msl![1,]), (1, msl![-2,]), (2, msl![3,]), (3, msl![-2,]), (4, msl![4,])]
    }
}
