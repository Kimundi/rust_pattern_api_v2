#![no_implicit_prelude]

#[macro_use]
extern crate pattern_api_v2_test_support;

extern crate pattern_api_v2;
use pattern_api_v2::std_integration::OsStrExtension;
use pattern_api_v2::std_integration::OsStringExtension;
use pattern_api_v2::std_integration::IteratorConstructors;
use pattern_api_v2::Pattern;

use std::vec::Vec;
use std::option::Option::{self, Some, None};
use std::string::String;
use std::convert::From;
use std::iter::Iterator;
use std::string::ToString;
use std::ffi::OsString;
use std::borrow::ToOwned;
use std::convert::AsRef;
use std::char;
use std::clone::Clone;
use std::ffi::OsStr;
use std::iter::DoubleEndedIterator;

macro_rules! is_windows { () => { false } }
macro_rules! if_unix_windows { (unix $u:block windows $w:block) => { $u } }

fn split_char() -> (OsString, OsString) {
    if_unix_windows! {
        unix {
            use std::os::unix::ffi::OsStringExt;

            (OsString::from_vec(vec![0xC2]), OsString::from_vec(vec![0xA2]))
        }
        windows {
            use windows::OsStringExt;
            (OsString::from_wide(&[0xD83D]), OsString::from_wide(&[0xDE3A]))
        }
    }
}

#[test]
fn osstr() {
    let string = OsString::from("hello");

    // Not yet implemented
    //assert_eq!(os!("\nHello  World").split_whitespace().collect::<Vec<_>>(),
    //           [os!("Hello"), os!("World")]);

    assert!(string.contains_os(os!("ll")));
    assert!(string.starts_with_os(os!("he")));
    assert!(string.ends_with_os(os!("lo")));
    assert!(string.contains("ll"));
    assert!(string.starts_with("he"));
    assert!(string.ends_with("lo"));
    assert_eq!(string.split('l').collect::<Vec<_>>(),
                [os!("he"), os!(""), os!("o")]);
    assert_eq!(string.rsplit('l').collect::<Vec<_>>(),
                [os!("o"), os!(""), os!("he")]);
    assert_eq!(string.split_terminator('o').collect::<Vec<_>>(),
                [os!("hell")]);
    assert_eq!(string.rsplit_terminator('o').collect::<Vec<_>>(),
                [os!("hell")]);
    assert_eq!(string.splitn(2, 'l').collect::<Vec<_>>(),
                [os!("he"), os!("lo")]);
    assert_eq!(string.rsplitn(2, 'l').collect::<Vec<_>>(),
                [os!("o"), os!("hel")]);
    assert_eq!(string.matches('l').collect::<Vec<_>>(), ["l"; 2]);
    assert_eq!(string.rmatches('l').collect::<Vec<_>>(), ["l"; 2]);
    assert_eq!(os!("aabcaa").trim_matches('a'), os!("bc"));
    assert_eq!(os!("aabcaa").trim_left_matches('a'), os!("bcaa"));
    assert_eq!(os!("aabcaa").trim_right_matches('a'), os!("aabc"));
}

#[test]
fn osstr_contains_os() {
    assert!(os!("").contains_os(""));
    assert!(os!("aé 💩").contains_os(""));
    assert!(os!("aé 💩").contains_os("aé"));
    assert!(os!("aé 💩").contains_os("é "));
    assert!(os!("aé 💩").contains_os(" 💩"));
    assert!(os!("aé 💩").contains_os("aé 💩"));
    assert!(!os!("aé 💩").contains_os("b"));
    assert!(!os!("").contains_os("a"));

    let (start, end) = split_char();
    let mut full = start.to_owned();
    full.push(&end);
    // Sanity check
    assert!(start.to_str().is_none() && end.to_str().is_none() &&
            full.to_str().is_some());

    assert!(!os!("").contains_os(&start));
    assert!(!os!("").contains_os(&end));

    assert!(start.contains_os(""));
    assert!(start.contains_os(&start));
    assert!(!start.contains_os(&end));
    assert!(!start.contains_os(&full));
    assert!(end.contains_os(""));
    assert!(!end.contains_os(&start));
    assert!(end.contains_os(&end));
    assert!(!end.contains_os(&full));
    assert!(full.contains_os(""));
    assert!(full.contains_os(&start));
    assert!(full.contains_os(&end));
    assert!(full.contains_os(&full));
}

#[test]
fn osstr_starts_with_os() {
    assert!(os!("").starts_with_os(""));
    assert!(os!("aé 💩").starts_with_os(""));
    assert!(os!("aé 💩").starts_with_os("aé"));
    assert!(!os!("aé 💩").starts_with_os(" 💩"));
    assert!(os!("aé 💩").starts_with_os("aé 💩"));
    assert!(!os!("").starts_with_os("a"));

    let (start, end) = split_char();
    let mut full = start.to_owned();
    full.push(&end);
    // Sanity check
    assert!(start.to_str().is_none() && end.to_str().is_none() &&
            full.to_str().is_some());

    assert!(!os!("").starts_with_os(&start));
    assert!(!os!("").starts_with_os(&end));

    assert!(start.starts_with_os(""));
    assert!(start.starts_with_os(&start));
    assert!(!start.starts_with_os(&end));
    assert!(!start.starts_with_os(&full));
    assert!(end.starts_with_os(""));
    assert!(!end.starts_with_os(&start));
    assert!(end.starts_with_os(&end));
    assert!(!end.starts_with_os(&full));
    assert!(full.starts_with_os(""));
    assert!(full.starts_with_os(&start));
    assert!(!full.starts_with_os(&end));
    assert!(full.starts_with_os(&full));
}

#[test]
fn osstr_ends_with_os() {
    assert!(os!("").ends_with_os(""));
    assert!(os!("aé 💩").ends_with_os(""));
    assert!(!os!("aé 💩").ends_with_os("aé"));
    assert!(os!("aé 💩").ends_with_os(" 💩"));
    assert!(os!("aé 💩").ends_with_os("aé 💩"));
    assert!(!os!("").ends_with_os("a"));

    let (start, end) = split_char();
    let mut full = start.to_owned();
    full.push(&end);
    // Sanity check
    assert!(start.to_str().is_none() && end.to_str().is_none() &&
            full.to_str().is_some());

    assert!(!os!("").ends_with_os(&start));
    assert!(!os!("").ends_with_os(&end));

    assert!(start.ends_with_os(""));
    assert!(start.ends_with_os(&start));
    assert!(!start.ends_with_os(&end));
    assert!(!start.ends_with_os(&full));
    assert!(end.ends_with_os(""));
    assert!(!end.ends_with_os(&start));
    assert!(end.ends_with_os(&end));
    assert!(!end.ends_with_os(&full));
    assert!(full.ends_with_os(""));
    assert!(!full.ends_with_os(&start));
    assert!(full.ends_with_os(&end));
    assert!(full.ends_with_os(&full));
}
