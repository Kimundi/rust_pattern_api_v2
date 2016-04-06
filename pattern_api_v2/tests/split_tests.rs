#[macro_use]
extern crate pattern_api_v2_test_support;
extern crate pattern_api_v2;

use pattern_api_v2::slice::Elem;
use pattern_api_v2::iterators::{Split, RSplit};

use pattern_api_v2_test_support::{s};
use std::ffi::{OsStr};

iterator_cross_test! {
    forward-backward, Split::new, RSplit::new, {
        str, &str: &s("abbcbbd"), _: "bb",
            ["a", "c", "d"],
            ["a", "c", "d"]
        str_mut, &mut str: &mut s("abbcbbd"), _: "bb",
            ["a", "c", "d"],
            ["a", "c", "d"]

        os_str_str, &OsStr: os!("abbcbbd"), _: "bb",
            [os!(b"a"), os!(b"c"), os!(b"d")],
            [os!(b"a"), os!(b"c"), os!(b"d")]
        os_str_mut_str, &mut OsStr: mos!("abbcbbd"), _: "bb",
            [os!(b"a"), os!(b"c"), os!(b"d")],
            [os!(b"a"), os!(b"c"), os!(b"d")]

        os_str_os, &OsStr: os!("abbcbbd"), _: os!(b"bb"),
            [os!(b"a"), os!(b"c"), os!(b"d")],
            [os!(b"a"), os!(b"c"), os!(b"d")]
        os_str_mut_os, &mut OsStr: mos!("abbcbbd"), _: os!(b"bb"),
            [os!(b"a"), os!(b"c"), os!(b"d")],
            [os!(b"a"), os!(b"c"), os!(b"d")]

        u8, &[u8]: &{*b"abbcbbd"}, &[_]: b"bb",
            [b"a", b"c", b"d"],
            [b"a", b"c", b"d"]
        u8_mut, &mut [u8]: &mut{*b"abbcbbd"}, &[_]: b"bb",
            [b"a", b"c", b"d"],
            [b"a", b"c", b"d"]

        i32, &[i32]: &{[1,-2,-2,3,-2,-2,4]}, &[_]: &[-2, -2],
            [&[1], &[3], &[4]],
            [&[1], &[3], &[4]]
        i32_mut, &mut [i32]: &mut{[1,-2,-2,3,-2,-2,4]}, &[_]: &[-2, -2],
            [&[1], &[3], &[4]],
            [&[1], &[3], &[4]]
    }
    double, Split::new, RSplit::new, {
        str_char,             _: "abcbd",              _: 'b',      ["a", "c", "d"]
        str_pred,             _: "abcbd",              _: |_| true, ["", "", "", "", "", ""]
        str_mut_char,  &mut str: &mut s("abcbd"),      _: 'b',      ["a", "c", "d"]
        str_mut_pred,  &mut str: &mut s("abcbd"),      _: |_| true, ["", "", "", "", "", ""]
        os_char,         &OsStr: os!(b"ab\xbebd"),     _: 'b',      [os!(b"a"), os!(b"\xbe"), os!(b"d")]
        os_pred,         &OsStr: os!(b"ab\xbebd"),     _: |_| true, [os!(b""), os!(b""), os!(b"\xbe"), os!(b""), os!(b"")]
        os_mut_char, &mut OsStr: mos!(b"ab\xbebd"),    _: 'b',      [os!(b"a"), os!(b"\xbe"), os!(b"d")]
        os_mut_pred, &mut OsStr: mos!(b"ab\xbebd"),    _: |_| true, [os!(b""), os!(b""), os!(b"\xbe"), os!(b""), os!(b"")]

        u8_elem,         &[u8]: b"abcbd",        _: Elem(b'b'),   [b"a", b"c", b"d"]
        u8_pred,         &[u8]: b"abcbd",        _: |_: &_| true, [b"", b"", b"", b"", b"", b""]
        u8_mut_elem, &mut [u8]: &mut{*b"abcbd"}, _: Elem(b'b'),   [b"a", b"c", b"d"]
        u8_mut_pred, &mut [u8]: &mut{*b"abcbd"}, _: |_: &_| true, [b"", b"", b"", b"", b"", b""]

        i32_elem,         &[i32]: &[1,-2,3,-2,4],      _: Elem(-2),     [&[1], &[3], &[4]]
        i32_pred,         &[i32]: &[1,-2,3,-2,4],      _: |_: &_| true, [&[], &[], &[], &[], &[], &[]]
        i32_mut_elem, &mut [i32]: &mut{[1,-2,3,-2,4]}, _: Elem(-2),     [&[1], &[3], &[4]]
        i32_mut_pred, &mut [i32]: &mut{[1,-2,3,-2,4]}, _: |_: &_| true, [&[], &[], &[], &[], &[], &[]]
    }
}

#[test]
fn inverse_match_is_valid() {
    use pattern_api_v2_test_support::InverseMatchesAreValidIsImplemented;
    use pattern_api_v2::os_string::shared::PartialUnicode as UOsStr;
    use pattern_api_v2::os_string::mutable::PartialUnicode as UMutOsStr;

    assert!(<&str>::inverse_match_is_valid());
    assert!(<&mut str>::inverse_match_is_valid());

    assert!(<&OsStr>::inverse_match_is_valid());
    assert!(<&mut OsStr>::inverse_match_is_valid());

    assert!(<&[u8]>::inverse_match_is_valid());
    assert!(<&mut [u8]>::inverse_match_is_valid());

    assert!(<&[i32]>::inverse_match_is_valid());
    assert!(<&mut [i32]>::inverse_match_is_valid());

    assert!(!<UOsStr>::inverse_match_is_valid());
    assert!(!<UMutOsStr>::inverse_match_is_valid());
}
