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

fn from_cp(cp: u16) -> OsString {
    OsString::from_wide(&[cp])
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

#[test]
fn wtf8_contains() {
    assert!(os!("").contains(os!("")));
    assert!(os!("aé 💩").contains(os!("")));
    assert!(os!("aé 💩").contains(os!("aé 💩")));
    assert!(!os!("").contains(os!("aé 💩")));
    assert!(os!("aé 💩").contains(os!("aé")));
    assert!(os!("aé 💩").contains(os!("é ")));
    assert!(os!("aé 💩").contains(os!(" 💩")));

    // Non UTF-8 cases
    fn check(haystack: &[u16], needle: &[u16]) -> bool {
        OsString::from_wide(haystack).contains(&OsString::from_wide(needle)[..])
    }

    // No surrogates in needle
    assert!( check(&[        0xD83D, 0xDE3A        ], &[0xD83D, 0xDE3A]));
    assert!( check(&[0x0020, 0xD83D, 0xDE3A        ], &[0xD83D, 0xDE3A]));
    assert!( check(&[0xD83D, 0xD83D, 0xDE3A        ], &[0xD83D, 0xDE3A]));
    assert!( check(&[0xD83E, 0xD83D, 0xDE3A        ], &[0xD83D, 0xDE3A]));
    assert!( check(&[0xDE3A, 0xD83D, 0xDE3A        ], &[0xD83D, 0xDE3A]));
    assert!( check(&[0xDE3B, 0xD83D, 0xDE3A        ], &[0xD83D, 0xDE3A]));
    assert!( check(&[        0xD83D, 0xDE3A, 0x0020], &[0xD83D, 0xDE3A]));
    assert!( check(&[        0xD83D, 0xDE3A, 0xD83D], &[0xD83D, 0xDE3A]));
    assert!( check(&[        0xD83D, 0xDE3A, 0xD83E], &[0xD83D, 0xDE3A]));
    assert!( check(&[        0xD83D, 0xDE3A, 0xDE3A], &[0xD83D, 0xDE3A]));
    assert!( check(&[        0xD83D, 0xDE3A, 0xDE3B], &[0xD83D, 0xDE3A]));
    assert!(!check(&[        0xD83E, 0xDE3A        ], &[0xD83D, 0xDE3A]));
    assert!(!check(&[        0xD83D, 0xDE3B        ], &[0xD83D, 0xDE3A]));
    assert!(!check(&[        0xD83E, 0xDE3B        ], &[0xD83D, 0xDE3A]));
    assert!(!check(&[        0xD83D                ], &[0xD83D, 0xDE3A]));
    assert!(!check(&[                0xDE3A        ], &[0xD83D, 0xDE3A]));
    assert!(!check(&[                              ], &[0xD83D, 0xDE3A]));

    // needle is just a lead surrogate
    assert!( check(&[        0xD83D        ], &[0xD83D]));
    assert!( check(&[0x0020, 0xD83D        ], &[0xD83D]));
    assert!( check(&[0xD83E, 0xD83D        ], &[0xD83D]));
    assert!( check(&[0xDE3A, 0xD83D        ], &[0xD83D]));
    assert!( check(&[        0xD83D, 0x0020], &[0xD83D]));
    assert!( check(&[        0xD83D, 0xD83E], &[0xD83D]));
    assert!( check(&[        0xD83D, 0xDE3A], &[0xD83D]));
    assert!(!check(&[        0xD83E        ], &[0xD83D]));
    assert!(!check(&[        0xDE3A        ], &[0xD83D]));
    assert!(!check(&[        0xD83E, 0xDE3A], &[0xD83D]));
    assert!(!check(&[                      ], &[0xD83D]));

    // needle is just a trail surrogate
    assert!( check(&[        0xDE3A        ], &[0xDE3A]));
    assert!( check(&[0x0020, 0xDE3A        ], &[0xDE3A]));
    assert!( check(&[0xD83D, 0xDE3A        ], &[0xDE3A]));
    assert!( check(&[0xDE3B, 0xDE3A        ], &[0xDE3A]));
    assert!( check(&[        0xDE3A, 0x0020], &[0xDE3A]));
    assert!( check(&[        0xDE3A, 0xD83D], &[0xDE3A]));
    assert!( check(&[        0xDE3A, 0xDE3B], &[0xDE3A]));
    assert!(!check(&[        0xDE3B        ], &[0xDE3A]));
    assert!(!check(&[        0xD83D        ], &[0xDE3A]));
    assert!(!check(&[0xD83D, 0xDE3B        ], &[0xDE3A]));
    assert!(!check(&[                      ], &[0xDE3A]));

    // needle is a trail and lead surrogate
    assert!( check(&[        0xDE3A, 0xD83D        ], &[0xDE3A, 0xD83D]));
    assert!( check(&[0x0020, 0xDE3A, 0xD83D        ], &[0xDE3A, 0xD83D]));
    assert!( check(&[0xD83D, 0xDE3A, 0xD83D        ], &[0xDE3A, 0xD83D]));
    assert!( check(&[0xD83E, 0xDE3A, 0xD83D        ], &[0xDE3A, 0xD83D]));
    assert!( check(&[0xDE3A, 0xDE3A, 0xD83D        ], &[0xDE3A, 0xD83D]));
    assert!( check(&[0xDE3B, 0xDE3A, 0xD83D        ], &[0xDE3A, 0xD83D]));
    assert!( check(&[        0xDE3A, 0xD83D, 0x0020], &[0xDE3A, 0xD83D]));
    assert!( check(&[        0xDE3A, 0xD83D, 0xD83D], &[0xDE3A, 0xD83D]));
    assert!( check(&[        0xDE3A, 0xD83D, 0xD83E], &[0xDE3A, 0xD83D]));
    assert!( check(&[        0xDE3A, 0xD83D, 0xDE3A], &[0xDE3A, 0xD83D]));
    assert!( check(&[        0xDE3A, 0xD83D, 0xDE3B], &[0xDE3A, 0xD83D]));
    assert!( check(&[0xD83D, 0xDE3A, 0xD83D, 0xDE3A], &[0xDE3A, 0xD83D]));

    assert!(!check(&[        0xDE3A, 0xD83E        ], &[0xDE3A, 0xD83D]));
    assert!(!check(&[0xD83D, 0xDE3A, 0xD83E        ], &[0xDE3A, 0xD83D]));
    assert!(!check(&[        0xDE3A, 0xD83E, 0xDE3A], &[0xDE3A, 0xD83D]));
    assert!(!check(&[0xD83D, 0xDE3A, 0xD83E, 0xDE3A], &[0xDE3A, 0xD83D]));

    assert!(!check(&[        0xDE3B, 0xD83D        ], &[0xDE3A, 0xD83D]));
    assert!(!check(&[0xD83D, 0xDE3B, 0xD83D        ], &[0xDE3A, 0xD83D]));
    assert!(!check(&[        0xDE3B, 0xD83D, 0xDE3A], &[0xDE3A, 0xD83D]));
    assert!(!check(&[0xD83D, 0xDE3B, 0xD83D, 0xDE3A], &[0xDE3A, 0xD83D]));

    assert!(!check(&[        0xDE3A                ], &[0xDE3A, 0xD83D]));
    assert!(!check(&[                0xD83D        ], &[0xDE3A, 0xD83D]));
    assert!(!check(&[                              ], &[0xDE3A, 0xD83D]));

    // needle is a trail surrogate and other stuff
    assert!( check(&[        0xDE3A, 0x0020        ], &[0xDE3A, 0x0020]));
    assert!( check(&[0x0020, 0xDE3A, 0x0020        ], &[0xDE3A, 0x0020]));
    assert!( check(&[0xD83D, 0xDE3A, 0x0020        ], &[0xDE3A, 0x0020]));
    assert!( check(&[0xDE3A, 0xDE3A, 0x0020        ], &[0xDE3A, 0x0020]));
    assert!( check(&[        0xDE3A, 0x0020, 0x0020], &[0xDE3A, 0x0020]));
    assert!( check(&[        0xDE3A, 0x0020, 0xD83D], &[0xDE3A, 0x0020]));
    assert!( check(&[        0xDE3A, 0x0020, 0xDE3A], &[0xDE3A, 0x0020]));
    assert!( check(&[0xD83D, 0xDE3A, 0x0020, 0x0020], &[0xDE3A, 0x0020]));
    assert!(!check(&[0xD83D, 0xDE3A, 0x0021        ], &[0xDE3A, 0x0020]));
    assert!(!check(&[0xD83D, 0xDE3B, 0x0020        ], &[0xDE3A, 0x0020]));
    assert!(!check(&[        0xDE3A                ], &[0xDE3A, 0x0020]));
    assert!(!check(&[                0x0020        ], &[0xDE3A, 0x0020]));
    assert!(!check(&[                              ], &[0xDE3A, 0x0020]));

    assert!( check(&[        0xDE3A, 0xDE3A        ], &[0xDE3A, 0xDE3A]));
    assert!( check(&[0x0020, 0xDE3A, 0xDE3A        ], &[0xDE3A, 0xDE3A]));
    assert!( check(&[0xD83D, 0xDE3A, 0xDE3A        ], &[0xDE3A, 0xDE3A]));
    assert!( check(&[0xDE3B, 0xDE3A, 0xDE3A        ], &[0xDE3A, 0xDE3A]));
    assert!( check(&[        0xDE3A, 0xDE3A, 0x0020], &[0xDE3A, 0xDE3A]));
    assert!( check(&[        0xDE3A, 0xDE3A, 0xD83D], &[0xDE3A, 0xDE3A]));
    assert!( check(&[        0xDE3A, 0xDE3A, 0xDE3B], &[0xDE3A, 0xDE3A]));
    assert!( check(&[0xD83D, 0xDE3A, 0xDE3A, 0xDE3B], &[0xDE3A, 0xDE3A]));
    assert!(!check(&[0xD83D, 0xDE3A, 0xDE3B        ], &[0xDE3A, 0xDE3A]));
    assert!(!check(&[0xD83D, 0xDE3B, 0xDE3A        ], &[0xDE3A, 0xDE3A]));
    assert!(!check(&[        0xDE3A                ], &[0xDE3A, 0xDE3A]));
    assert!(!check(&[                              ], &[0xDE3A, 0xDE3A]));

    // needle is a other stuff and a lead surrogate
    assert!( check(&[        0x0020, 0xD83D        ], &[0x0020, 0xD83D]));
    assert!( check(&[0x0020, 0x0020, 0xD83D        ], &[0x0020, 0xD83D]));
    assert!( check(&[0xD83D, 0x0020, 0xD83D        ], &[0x0020, 0xD83D]));
    assert!( check(&[0xDE3A, 0x0020, 0xD83D        ], &[0x0020, 0xD83D]));
    assert!( check(&[        0x0020, 0xD83D, 0x0020], &[0x0020, 0xD83D]));
    assert!( check(&[        0x0020, 0xD83D, 0xD83D], &[0x0020, 0xD83D]));
    assert!( check(&[        0x0020, 0xD83D, 0xDE3A], &[0x0020, 0xD83D]));
    assert!( check(&[0x0020, 0x0020, 0xD83D, 0x0020], &[0x0020, 0xD83D]));
    assert!(!check(&[        0x0020, 0xD83E, 0xDE3A], &[0x0020, 0xD83D]));
    assert!(!check(&[        0x0021, 0xD83D, 0xDE3A], &[0x0020, 0xD83D]));
    assert!(!check(&[        0x0020                ], &[0x0020, 0xD83D]));
    assert!(!check(&[                0xD83D        ], &[0x0020, 0xD83D]));
    assert!(!check(&[                              ], &[0x0020, 0xD83D]));

    assert!( check(&[        0xD83D, 0xD83D        ], &[0xD83D, 0xD83D]));
    assert!( check(&[0x0020, 0xD83D, 0xD83D        ], &[0xD83D, 0xD83D]));
    assert!( check(&[0xD83E, 0xD83D, 0xD83D        ], &[0xD83D, 0xD83D]));
    assert!( check(&[0xDE3A, 0xD83D, 0xD83D        ], &[0xD83D, 0xD83D]));
    assert!( check(&[        0xD83D, 0xD83D, 0x0020], &[0xD83D, 0xD83D]));
    assert!( check(&[        0xD83D, 0xD83D, 0xD83E], &[0xD83D, 0xD83D]));
    assert!( check(&[        0xD83D, 0xD83D, 0xDE3A], &[0xD83D, 0xD83D]));
    assert!( check(&[0xD83E, 0xD83D, 0xD83D, 0xDE3A], &[0xD83D, 0xD83D]));
    assert!(!check(&[        0xD83D, 0xD83E, 0xDE3A], &[0xD83D, 0xD83D]));
    assert!(!check(&[        0xD83E, 0xD83D, 0xDE3A], &[0xD83D, 0xD83D]));
    assert!(!check(&[        0xD83D                ], &[0xD83D, 0xD83D]));
    assert!(!check(&[                              ], &[0xD83D, 0xD83D]));

    // needle is a trail surrogate, other stuff, and a lead surrogate
    assert!( check(&[        0xDE3A, 0x0020, 0xD83D        ], &[0xDE3A, 0x0020, 0xD83D]));
    assert!( check(&[0x0020, 0xDE3A, 0x0020, 0xD83D        ], &[0xDE3A, 0x0020, 0xD83D]));
    assert!( check(&[0xD83D, 0xDE3A, 0x0020, 0xD83D        ], &[0xDE3A, 0x0020, 0xD83D]));
    assert!( check(&[0xDE3A, 0xDE3A, 0x0020, 0xD83D        ], &[0xDE3A, 0x0020, 0xD83D]));
    assert!( check(&[        0xDE3A, 0x0020, 0xD83D, 0x0020], &[0xDE3A, 0x0020, 0xD83D]));
    assert!( check(&[        0xDE3A, 0x0020, 0xD83D, 0xD83D], &[0xDE3A, 0x0020, 0xD83D]));
    assert!( check(&[        0xDE3A, 0x0020, 0xD83D, 0xDE3A], &[0xDE3A, 0x0020, 0xD83D]));
    assert!( check(&[0xD83D, 0xDE3A, 0x0020, 0xD83D, 0xDE3A], &[0xDE3A, 0x0020, 0xD83D]));
    assert!(!check(&[0xD83D, 0xDE3B, 0x0020, 0xD83D, 0xDE3A], &[0xDE3A, 0x0020, 0xD83D]));
    assert!(!check(&[0xD83D, 0xDE3A, 0x0021, 0xD83D, 0xDE3A], &[0xDE3A, 0x0020, 0xD83D]));
    assert!(!check(&[0xD83D, 0xDE3A, 0x0020, 0xD83E, 0xDE3A], &[0xDE3A, 0x0020, 0xD83D]));
    assert!(!check(&[        0xDE3A, 0x0020                ], &[0xDE3A, 0x0020, 0xD83D]));
    assert!(!check(&[        0xDE3A,         0xD83D        ], &[0xDE3A, 0x0020, 0xD83D]));
    assert!(!check(&[        0xDE3A,                       ], &[0xDE3A, 0x0020, 0xD83D]));
    assert!(!check(&[                0x0020, 0xD83D        ], &[0xDE3A, 0x0020, 0xD83D]));
    assert!(!check(&[                0x0020                ], &[0xDE3A, 0x0020, 0xD83D]));
    assert!(!check(&[                        0xD83D        ], &[0xDE3A, 0x0020, 0xD83D]));
    assert!(!check(&[                                      ], &[0xDE3A, 0x0020, 0xD83D]));

    // Non-surrogate part matches two overlapping times
    assert!(check(&[0xD83D, 0xDE3A, 0xD83D, 0xDE3A, 0xD83D, 0xDE3A],
                    &[        0xDE3A, 0xD83D, 0xDE3A, 0xD83D, 0xDE3A]));
    assert!(check(&[0xD83D, 0xDE3A, 0xD83D, 0xDE3A, 0xD83D, 0xDE3A],
                    &[0xD83D, 0xDE3A, 0xD83D, 0xDE3A, 0xD83D        ]));
}

#[test]
fn wtf8_starts_with() {
    assert!(os!("aé 💩").starts_with(os!("aé")));
    assert!(os!("aé 💩").starts_with(os!("aé 💩")));
    assert!(os!("aé 💩").starts_with(os!("")));
    assert!(!os!("aé 💩").starts_with(os!("é")));
    assert!(os!("").starts_with(os!("")));
    assert!(!os!("").starts_with(os!("a")));

    fn check_surrogates(prefix: &[u16]) {
        let mut lead = prefix.to_owned();
        lead.push(0xD83D);

            let mut full = lead.clone();
            full.push(0xDE3A);
            let full = OsString::from_wide(&full);

        let lead = OsString::from_wide(&lead);

        let mut other_lead = prefix.to_owned();
        other_lead.push(0xD83E);
        let other_lead = OsString::from_wide(&other_lead);

        let trail = OsString::from_wide(&[0xDE3A]);

        let prefix = &OsString::from_wide(prefix);

        assert_eq!(full, {
            let mut x = prefix.to_owned();
            x.push_str("😺");
            x
        });

        assert!(full.starts_with(&full));
        assert!(lead.starts_with(&lead));
        assert!(trail.starts_with(&trail));
        assert!(lead.starts_with(prefix));
        assert!(full.starts_with(&lead));
        assert!(!full.starts_with(&trail));
        assert!(!full.starts_with(&other_lead));
        assert!(!lead.starts_with(&full));
    }

    check_surrogates(&[]);
    check_surrogates(&[b'a' as u16]);
    check_surrogates(&[0xD83D]);
}

#[test]
fn wtf8_ends_with() {
    assert!(os!("aé 💩").ends_with(os!(" 💩")));
    assert!(os!("aé 💩").ends_with(os!("aé 💩")));
    assert!(os!("aé 💩").ends_with(os!("")));
    assert!(!os!("aé 💩").ends_with(os!("é")));
    assert!(os!("").ends_with(os!("")));
    assert!(!os!("").ends_with(os!("a")));

    fn check_surrogates(suffix: &[u16]) {
        let lead = vec!(0xD83D);
        let mut trail = vec!(0xDE3A);
        trail.extend(suffix);

        let mut other_trail = vec!(0xDE3B);
        other_trail.extend(suffix);
        let mut full = lead.clone();
        full.extend(&trail);



        assert_eq!(full, {
            let mut x = os!("😺").to_owned();
            x.push_os_str(suffix);
            x
        });

        assert!(full.ends_with(&full));
        assert!(lead.ends_with(&lead));
        assert!(trail.ends_with(&trail));
        assert!(trail.ends_with(suffix));
        assert!(full.ends_with(&trail));
        assert!(!full.ends_with(&lead));
        assert!(!full.ends_with(&other_trail));
        assert!(!trail.ends_with(&full));
    }

    check_surrogates(&[]);
    check_surrogates(&[b'a' as u16]);
    check_surrogates(&[0xDE3A]);
}

#[test]
fn wtf8_split() {
    assert_eq!(os!("").split('a').collect::<Vec<_>>(),
                [os!("")]);

    let mut non_utf8 = OsString::new();
    non_utf8.push_codepoint(0xD800);
    let mut string = os!("aΓ").to_owned();
    string.push_os_str(&non_utf8);
    string.push_str("aΓaΓaé 💩aΓ");
    string.push_os_str(&non_utf8);
    string.push_str("aΓ");
    assert_eq!(string.split("aΓ").collect::<Vec<_>>(),
                [os!(""), &non_utf8[..], os!(""),
                os!("aé 💩"), &non_utf8[..], os!("")]);

    assert_eq!(os!("aaa").split("aa").collect::<Vec<_>>(),
                [os!(""), os!("a")]);
}

#[test]
fn wtf8_rsplit() {
    assert_eq!(os!("").rsplit('a').collect::<Vec<_>>(),
                [os!("")]);

    let mut non_utf8 = OsString::new();
    non_utf8.push_codepoint(0xD800);
    let mut string = os!("aΓ").to_owned();
    string.push_os_str(&non_utf8);
    string.push_str("aΓaΓaé 💩aΓ");
    string.push_os_str(&non_utf8);
    string.push_str("aΓ");
    assert_eq!(string.rsplit("aΓ").collect::<Vec<_>>(),
                [os!(""), &non_utf8[..], os!("aé 💩"),
                os!(""), &non_utf8[..], os!("")]);

    assert_eq!(os!("aaa").rsplit("aa").collect::<Vec<_>>(),
                [os!(""), os!("a")]);
}

#[test]
fn wtf8_split_terminator() {
    assert!(os!("").split_terminator('a').next().is_none());
    assert!(os!("").split_terminator('a').next_back().is_none());
    assert_eq!(os!("a").split_terminator('a').collect::<Vec<_>>(),
                [os!("")]);
    assert_eq!(os!("a").split_terminator('a').rev().collect::<Vec<_>>(),
                [os!("")]);

    let mut non_utf8 = OsString::new();
    non_utf8.push_codepoint(0xD800);
    let mut string = os!("aΓ").to_owned();
    string.push_os_str(&non_utf8);
    string.push_str("Γ");
    assert_eq!(string.split_terminator("Γ").collect::<Vec<_>>(),
                [os!("a"), &non_utf8[..]]);
    string.push_str("aé 💩");
    assert_eq!(string.split_terminator("Γ").collect::<Vec<_>>(),
                [os!("a"), &non_utf8[..], os!("aé 💩")]);

    let string = os!("xΓΓx");
    let mut split = string.split_terminator('Γ');
    assert_eq!(split.next(), Some(os!("x")));
    assert_eq!(split.next_back(), Some(os!("x")));
    assert_eq!(split.clone().next(), Some(os!("")));
    assert_eq!(split.next_back(), Some(os!("")));
}

#[test]
fn wtf8_rsplit_terminator() {
    assert!(os!("").rsplit_terminator('a').next().is_none());
    assert!(os!("").rsplit_terminator('a').next_back().is_none());
    assert_eq!(os!("a").rsplit_terminator('a').collect::<Vec<_>>(),
                [os!("")]);
    assert_eq!(os!("a").rsplit_terminator('a').rev().collect::<Vec<_>>(),
                [os!("")]);

    let mut non_utf8 = OsString::new();
    non_utf8.push_codepoint(0xD800);
    let mut string = os!("aΓ").to_owned();
    string.push_os_str(&non_utf8);
    string.push_str("Γ");
    assert_eq!(string.rsplit_terminator("Γ").collect::<Vec<_>>(),
                [&non_utf8[..], os!("a")]);
    string.push_str("aé 💩");
    assert_eq!(string.rsplit_terminator("Γ").collect::<Vec<_>>(),
                [os!("aé 💩"), &non_utf8[..], os!("a")]);

    let string = os!("xΓΓx");
    let mut split = string.rsplit_terminator('Γ');
    assert_eq!(split.next(), Some(os!("x")));
    assert_eq!(split.next_back(), Some(os!("x")));
    assert_eq!(split.clone().next(), Some(os!("")));
    assert_eq!(split.next_back(), Some(os!("")));
}

#[test]
fn wtf8_splitn() {
    assert_eq!(os!("").splitn(2, 'a').collect::<Vec<_>>(),
                [os!("")]);
    assert!(os!("a").splitn(0, 'a').next().is_none());
    assert_eq!(os!("a").splitn(1, 'a').collect::<Vec<_>>(),
                [os!("a")]);

    let mut non_utf8 = OsString::new();
    non_utf8.push_codepoint(0xD800);
    let mut string = os!("aΓ").to_owned();
    string.push_os_str(&non_utf8);
    string.push_str("aΓaΓaé 💩aΓ");
    string.push_os_str(&non_utf8);
    string.push_str("aΓ");
    let mut end = non_utf8.clone();
    end.push_str("aΓ");
    assert_eq!(string.splitn(5, "aΓ").collect::<Vec<_>>(),
                [os!(""), &non_utf8[..], os!(""),
                os!("aé 💩"), &end[..]]);
}

#[test]
fn wtf8_rsplitn() {
    assert_eq!(os!("").rsplitn(2, 'a').collect::<Vec<_>>(),
                [os!("")]);
    assert!(os!("a").rsplitn(0, 'a').next().is_none());
    assert_eq!(os!("a").rsplitn(1, 'a').collect::<Vec<_>>(),
                [os!("a")]);

    let mut non_utf8 = OsString::new();
    non_utf8.push_codepoint(0xD800);
    let mut string = os!("aΓ").to_owned();
    string.push_os_str(&non_utf8);
    let beginning = string.clone();
    string.push_str("aΓaΓaé 💩aΓ");
    string.push_os_str(&non_utf8);
    string.push_str("aΓ");
    assert_eq!(string.rsplitn(5, "aΓ").collect::<Vec<_>>(),
                [os!(""), &non_utf8[..], os!("aé 💩"),
                os!(""), &beginning[..]]);
}

#[test]
fn wtf8_matches() {
    assert!(os!("").matches('a').next().is_none());

    let mut non_utf8 = OsString::new();
    non_utf8.push_codepoint(0xD800);
    let mut string = os!("aΓ").to_owned();
    string.push_os_str(&non_utf8);
    string.push_str("aΓaΓaé 💩aΓ");
    string.push_os_str(&non_utf8);
    string.push_str("aΓ");
    assert_eq!(string.matches("aΓ").collect::<Vec<_>>(), ["aΓ"; 5]);
    assert_eq!(string.matches(&['é', '💩'] as &[_]).collect::<Vec<_>>(), ["é", "💩"]);
}

#[test]
fn wtf8_rmatches() {
    assert!(os!("").rmatches('a').next().is_none());

    let mut non_utf8 = OsString::new();
    non_utf8.push_codepoint(0xD800);
    let mut string = os!("aΓ").to_owned();
    string.push_os_str(&non_utf8);
    string.push_str("aΓaΓaé 💩aΓ");
    string.push_os_str(&non_utf8);
    string.push_str("aΓ");
    assert_eq!(string.rmatches("aΓ").collect::<Vec<_>>(), ["aΓ"; 5]);
    assert_eq!(string.rmatches(&['é', '💩'] as &[_]).collect::<Vec<_>>(), ["💩", "é"]);
}

#[test]
fn wtf8_matches_replacement() {
    let mut non_utf8 = OsString::new();
    non_utf8.push_codepoint(0xD800);
    let replacement = non_utf8.to_string_lossy().into_owned();
    assert!(non_utf8.matches(&replacement).next().is_none());
}

#[test]
fn wtf8_trim_matches() {
    assert_eq!(os!("").trim_matches('a'), os!(""));
    assert_eq!(os!("b").trim_matches('a'), os!("b"));
    assert_eq!(os!("a").trim_matches('a'), os!(""));
    assert_eq!(os!("ab").trim_matches('a'), os!("b"));
    assert_eq!(os!("ba").trim_matches('a'), os!("b"));
    assert_eq!(os!("aba").trim_matches('a'), os!("b"));
    assert_eq!(os!("bab").trim_matches('a'), os!("bab"));

    let mut non_utf8 = OsString::new();
    non_utf8.push_codepoint(0xD800);
    let mut string = non_utf8.clone();
    string.push_str("x");
    assert_eq!(string.trim_matches('x'), &non_utf8[..]);
    let mut string = os!("x").to_owned();
    string.push_os_str(&non_utf8);
    assert_eq!(string.trim_matches('x'), &non_utf8[..]);
}

#[test]
fn wtf8_trim_left_matches() {
    assert_eq!(os!("").trim_left_matches('a'), os!(""));
    assert_eq!(os!("b").trim_left_matches('a'), os!("b"));
    assert_eq!(os!("a").trim_left_matches('a'), os!(""));
    assert_eq!(os!("ab").trim_left_matches('a'), os!("b"));
    assert_eq!(os!("ba").trim_left_matches('a'), os!("ba"));

    let mut non_utf8 = OsString::new();
    non_utf8.push_codepoint(0xD800);
    let mut string = non_utf8.clone();
    string.push_str("x");
    assert_eq!(string.trim_left_matches('x'), &string[..]);
    let mut string = os!("x").to_owned();
    string.push_os_str(&non_utf8);
    assert_eq!(string.trim_left_matches('x'), &non_utf8[..]);
}

#[test]
fn wtf8_trim_right_matches() {
    assert_eq!(os!("").trim_right_matches('a'), os!(""));
    assert_eq!(os!("b").trim_right_matches('a'), os!("b"));
    assert_eq!(os!("a").trim_right_matches('a'), os!(""));
    assert_eq!(os!("ab").trim_right_matches('a'), os!("ab"));
    assert_eq!(os!("ba").trim_right_matches('a'), os!("b"));

    let mut non_utf8 = OsString::new();
    non_utf8.push_codepoint(0xD800);
    let mut string = non_utf8.clone();
    string.push_str("x");
    assert_eq!(string.trim_right_matches('x'), &non_utf8[..]);
    let mut string = os!("x").to_owned();
    string.push_os_str(&non_utf8);
    assert_eq!(string.trim_right_matches('x'), &string[..]);
}
