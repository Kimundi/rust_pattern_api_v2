#[macro_use]
extern crate pattern_api_v2_test_support;
extern crate pattern_api_v2;

use pattern_api_v2::slice::Elem;
use pattern_api_v2::iterators::{Matches, RMatches};
use pattern_api_v2::os_string::shared::PartialUnicode as UOsStr;
use pattern_api_v2::os_string::mutable::PartialUnicode as UMutOsStr;

use pattern_api_v2_test_support::{s};
use std::ffi::{OsStr};


iterator_cross_test! {
    forward-backward, Matches::new, RMatches::new, {
        str, &str: &s("abbcbbd"), _: "bb",
            ["bb", "bb"],
            ["bb", "bb"]
        str_mut, &mut str: &mut s("abbcbbd"), _: "bb",
            ["bb", "bb"],
            ["bb", "bb"]

        os_str_str, &OsStr: os!("abbcbbd"), _: "bb",
            [os!("bb"), os!("bb")],
            [os!("bb"), os!("bb")]
        os_str_mut_str, &mut OsStr: mos!("abbcbbd"), _: "bb",
            [os!("bb"), os!("bb")],
            [os!("bb"), os!("bb")]

        os_str_os, &OsStr: os!("abbcbbd"), _: os!("bb"),
            [os!("bb"), os!("bb")],
            [os!("bb"), os!("bb")]
        os_str_mut_os, &mut OsStr: mos!("abbcbbd"), _: os!("bb"),
            [os!("bb"), os!("bb")],
            [os!("bb"), os!("bb")]

        uos_str_str, UOsStr: uos!(b"abbcbbd"), _: "bb",
            ["bb", "bb"],
            ["bb", "bb"]
        uos_str_mut_str, UMutOsStr: muos!(b"abbcbbd"), _: "bb",
            ["bb", "bb"],
            ["bb", "bb"]

        u8, &[u8]: &{*b"abbcbbd"}, &[_]: b"bb",
            [b"bb", b"bb"],
            [b"bb", b"bb"]
        u8_mut, &mut [u8]: &mut{*b"abbcbbd"}, &[_]: b"bb",
            [b"bb", b"bb"],
            [b"bb", b"bb"]

        i32, &[i32]: &{[1,-2,-2,3,-2,-2,4]}, &[_]: &[-2, -2],
            [&[-2, -2], &[-2, -2]],
            [&[-2, -2], &[-2, -2]]
        i32_mut, &mut [i32]: &mut{[1,-2,-2,3,-2,-2,4]}, &[_]: &[-2, -2],
            [&[-2, -2], &[-2, -2]],
            [&[-2, -2], &[-2, -2]]
    }
    double, Matches::new, RMatches::new, {
        str_char,             _: "abcbd",              _: 'b',      ["b", "b"]
        str_pred,             _: "abcbd",              _: |_| true, ["a", "b", "c", "b", "d"]
        str_mut_char,  &mut str: &mut s("abcbd"),      _: 'b',      ["b", "b"]
        str_mut_pred,  &mut str: &mut s("abcbd"),      _: |_| true, ["a", "b", "c", "b", "d"]

        os_char,         &OsStr: os!(b"ab\xbebd"),     _: 'b',      ["b", "b"]
        os_pred,         &OsStr: os!(b"ab\xbebd"),     _: |_| true, ["a", "b", "b", "d"]
        os_mut_char, &mut OsStr: mos!(b"ab\xbebd"),    _: 'b',      ["b", "b"]
        os_mut_pred, &mut OsStr: mos!(b"ab\xbebd"),    _: |_| true, ["a", "b", "b", "d"]

        uos_char,        UOsStr: uos!(b"ab\xbebd"),    _: 'b',      ["b", "b"]
        uos_pred,        UOsStr: uos!(b"ab\xbebd"),    _: |_| true, ["a", "b", "b", "d"]
        uos_mut_char, UMutOsStr: muos!(b"ab\xbebd"),   _: 'b',      ["b", "b"]
        uos_mut_pred, UMutOsStr: muos!(b"ab\xbebd"),   _: |_| true, ["a", "b", "b", "d"]

        u8_elem,         &[u8]: b"abcbd",        _: Elem(b'b'),   [b"b", b"b"]
        u8_pred,         &[u8]: b"abcbd",        _: |_: &_| true, [b"a", b"b", b"c", b"b", b"d"]
        u8_mut_elem, &mut [u8]: &mut{*b"abcbd"}, _: Elem(b'b'),   [b"b", b"b"]
        u8_mut_pred, &mut [u8]: &mut{*b"abcbd"}, _: |_: &_| true, [b"a", b"b", b"c", b"b", b"d"]

        i32_elem,         &[i32]: &[1,-2,3,-2,4],      _: Elem(-2),     [&[-2], &[-2]]
        i32_pred,         &[i32]: &[1,-2,3,-2,4],      _: |_: &_| true, [&[1], &[-2], &[3], &[-2], &[4]]
        i32_mut_elem, &mut [i32]: &mut{[1,-2,3,-2,4]}, _: Elem(-2),     [&[-2], &[-2]]
        i32_mut_pred, &mut [i32]: &mut{[1,-2,3,-2,4]}, _: |_: &_| true, [&[1], &[-2], &[3], &[-2], &[4]]
    }
}
