#![no_implicit_prelude]

extern crate pattern_api_v2;
use pattern_api_v2::std_integration::IteratorConstructors;
use pattern_api_v2::Pattern;

use std::vec::Vec;
use std::option::Option::{self, Some, None};
use std::string::String;
use std::convert::From;
use std::iter::Iterator;
use std::string::ToString;

#[test]
fn test_find() {
    assert_eq!("hello".find('l'), Some(2));
    assert_eq!("hello".find(|c:char| c == 'o'), Some(4));
    assert!("hello".find('x').is_none());
    assert!("hello".find(|c:char| c == 'x').is_none());
    assert_eq!("ประเทศไทย中华Việt Nam".find('华'), Some(30));
    assert_eq!("ประเทศไทย中华Việt Nam".find(|c: char| c == '华'), Some(30));
}

#[test]
fn test_rfind() {
    assert_eq!("hello".rfind('l'), Some(3));
    assert_eq!("hello".rfind(|c:char| c == 'o'), Some(4));
    assert!("hello".rfind('x').is_none());
    assert!("hello".rfind(|c:char| c == 'x').is_none());
    assert_eq!("ประเทศไทย中华Việt Nam".rfind('华'), Some(30));
    assert_eq!("ประเทศไทย中华Việt Nam".rfind(|c: char| c == '华'), Some(30));
}

#[test]
fn test_find_str() {
    // byte positions
    assert_eq!("".find(""), Some(0));
    assert!("banana".find("apple pie").is_none());

    let data = "abcabc";
    assert_eq!(data[0..6].find("ab"), Some(0));
    assert_eq!(data[2..6].find("ab"), Some(3 - 2));
    assert!(data[2..4].find("ab").is_none());

    let string = "ประเทศไทย中华Việt Nam";
    let mut data = String::from(string);
    data.push_str(string);
    assert!(data.find("ไท华").is_none());
    assert_eq!(data[0..43].find(""), Some(0));
    assert_eq!(data[6..43].find(""), Some(6 - 6));

    assert_eq!(data[0..43].find("ประ"), Some( 0));
    assert_eq!(data[0..43].find("ทศไ"), Some(12));
    assert_eq!(data[0..43].find("ย中"), Some(24));
    assert_eq!(data[0..43].find("iệt"), Some(34));
    assert_eq!(data[0..43].find("Nam"), Some(40));

    assert_eq!(data[43..86].find("ประ"), Some(43 - 43));
    assert_eq!(data[43..86].find("ทศไ"), Some(55 - 43));
    assert_eq!(data[43..86].find("ย中"), Some(67 - 43));
    assert_eq!(data[43..86].find("iệt"), Some(77 - 43));
    assert_eq!(data[43..86].find("Nam"), Some(83 - 43));

    // find every substring -- assert that it finds it, or an earlier occurrence.
    let string = "Việt Namacbaabcaabaaba";
    for (i, ci) in string.char_indices() {
        let ip = i + ci.len_utf8();
        for j in string[ip..].char_indices()
                             .map(|(i, _)| i)
                             .chain(Some(string.len() - ip))
        {
            let pat = &string[i..ip + j];
            assert!(match string.find(pat) {
                None => false,
                Some(x) => x <= i,
            });
            assert!(match string.rfind(pat) {
                None => false,
                Some(x) => x >= i,
            });
        }
    }
}

#[test]
fn test_starts_with() {
    assert!(("".starts_with("")));
    assert!(("abc".starts_with("")));
    assert!(("abc".starts_with("a")));
    assert!((!"a".starts_with("abc")));
    assert!((!"".starts_with("abc")));
    assert!((!"ödd".starts_with("-")));
    assert!(("ödd".starts_with("öd")));
}

#[test]
fn test_ends_with() {
    assert!(("".ends_with("")));
    assert!(("abc".ends_with("")));
    assert!(("abc".ends_with("c")));
    assert!((!"a".ends_with("abc")));
    assert!((!"".ends_with("abc")));
    assert!((!"ddö".ends_with("-")));
    assert!(("ddö".ends_with("dö")));
}

#[test]
fn test_trim_left_matches() {
    let v: &[char] = &[];
    assert_eq!(" *** foo *** ".trim_left_matches(v), " *** foo *** ");
    let chars: &[char] = &['*', ' '];
    assert_eq!(" *** foo *** ".trim_left_matches(chars), "foo *** ");
    assert_eq!(" ***  *** ".trim_left_matches(chars), "");
    assert_eq!("foo *** ".trim_left_matches(chars), "foo *** ");

    assert_eq!("11foo1bar11".trim_left_matches('1'), "foo1bar11");
    let chars: &[char] = &['1', '2'];
    assert_eq!("12foo1bar12".trim_left_matches(chars), "foo1bar12");
    assert_eq!("123foo1bar123".trim_left_matches(|c: char| c.is_numeric()), "foo1bar123");
}

#[test]
fn test_trim_right_matches() {
    let v: &[char] = &[];
    assert_eq!(" *** foo *** ".trim_right_matches(v), " *** foo *** ");
    let chars: &[char] = &['*', ' '];
    assert_eq!(" *** foo *** ".trim_right_matches(chars), " *** foo");
    assert_eq!(" ***  *** ".trim_right_matches(chars), "");
    assert_eq!(" *** foo".trim_right_matches(chars), " *** foo");

    assert_eq!("11foo1bar11".trim_right_matches('1'), "11foo1bar");
    let chars: &[char] = &['1', '2'];
    assert_eq!("12foo1bar12".trim_right_matches(chars), "12foo1bar");
    assert_eq!("123foo1bar123".trim_right_matches(|c: char| c.is_numeric()), "123foo1bar");
}

#[test]
fn test_trim_matches() {
    let v: &[char] = &[];
    assert_eq!(" *** foo *** ".trim_matches(v), " *** foo *** ");
    let chars: &[char] = &['*', ' '];
    assert_eq!(" *** foo *** ".trim_matches(chars), "foo");
    assert_eq!(" ***  *** ".trim_matches(chars), "");
    assert_eq!("foo".trim_matches(chars), "foo");

    assert_eq!("11foo1bar11".trim_matches('1'), "foo1bar");
    let chars: &[char] = &['1', '2'];
    assert_eq!("12foo1bar12".trim_matches(chars), "foo1bar");
    assert_eq!("123foo1bar123".trim_matches(|c: char| c.is_numeric()), "foo1bar");
}

#[test]
fn test_trim_left() {
    assert_eq!("".trim_left(), "");
    assert_eq!("a".trim_left(), "a");
    assert_eq!("    ".trim_left(), "");
    assert_eq!("     blah".trim_left(), "blah");
    assert_eq!("   \u{3000}  wut".trim_left(), "wut");
    assert_eq!("hey ".trim_left(), "hey ");
}

#[test]
fn test_trim_right() {
    assert_eq!("".trim_right(), "");
    assert_eq!("a".trim_right(), "a");
    assert_eq!("    ".trim_right(), "");
    assert_eq!("blah     ".trim_right(), "blah");
    assert_eq!("wut   \u{3000}  ".trim_right(), "wut");
    assert_eq!(" hey".trim_right(), " hey");
}

#[test]
fn test_trim() {
    assert_eq!("".trim(), "");
    assert_eq!("a".trim(), "a");
    assert_eq!("    ".trim(), "");
    assert_eq!("    blah     ".trim(), "blah");
    assert_eq!("\nwut   \u{3000}  ".trim(), "wut");
    assert_eq!(" hey dude ".trim(), "hey dude");
}

#[test]
fn test_contains() {
    assert!("abcde".contains("bcd"));
    assert!("abcde".contains("abcd"));
    assert!("abcde".contains("bcde"));
    assert!("abcde".contains(""));
    assert!("".contains(""));
    assert!(!"abcde".contains("def"));
    assert!(!"".contains("a"));

    let data = "ประเทศไทย中华Việt Nam";
    assert!(data.contains("ประเ"));
    assert!(data.contains("ะเ"));
    assert!(data.contains("中华"));
    assert!(!data.contains("ไท华"));
}

#[test]
fn test_contains_char() {
    assert!("abc".contains('b'));
    assert!("a".contains('a'));
    assert!(!"abc".contains('d'));
    assert!(!"".contains('a'));
}

#[test]
fn test_char_indicesator() {
    let s = "ศไทย中华Việt Nam";
    let p = [0, 3, 6, 9, 12, 15, 18, 19, 20, 23, 24, 25, 26, 27];
    let v = ['ศ','ไ','ท','ย','中','华','V','i','ệ','t',' ','N','a','m'];

    let mut pos = 0;
    let it = s.char_indices();

    for c in it {
        assert_eq!(c, (p[pos], v[pos]));
        pos += 1;
    }
    assert_eq!(pos, v.len());
    assert_eq!(pos, p.len());
}

#[test]
fn test_char_indices_revator() {
    let s = "ศไทย中华Việt Nam";
    let p = [27, 26, 25, 24, 23, 20, 19, 18, 15, 12, 9, 6, 3, 0];
    let v = ['m', 'a', 'N', ' ', 't', 'ệ','i','V','华','中','ย','ท','ไ','ศ'];

    let mut pos = 0;
    let it = s.char_indices().rev();

    for c in it {
        assert_eq!(c, (p[pos], v[pos]));
        pos += 1;
    }
    assert_eq!(pos, v.len());
    assert_eq!(pos, p.len());
}

#[test]
fn test_splitn_char_iterator() {
    let data = "\nMäry häd ä little lämb\nLittle lämb\n";

    let split: Vec<&str> = data.splitn(4, ' ').collect();
    assert_eq!(split, ["\nMäry", "häd", "ä", "little lämb\nLittle lämb\n"]);

    let split: Vec<&str> = data.splitn(4, |c: char| c == ' ').collect();
    assert_eq!(split, ["\nMäry", "häd", "ä", "little lämb\nLittle lämb\n"]);

    // Unicode
    let split: Vec<&str> = data.splitn(4, 'ä').collect();
    assert_eq!(split, ["\nM", "ry h", "d ", " little lämb\nLittle lämb\n"]);

    let split: Vec<&str> = data.splitn(4, |c: char| c == 'ä').collect();
    assert_eq!(split, ["\nM", "ry h", "d ", " little lämb\nLittle lämb\n"]);
}

#[test]
fn test_split_char_iterator_no_trailing() {
    let data = "\nMäry häd ä little lämb\nLittle lämb\n";

    let split: Vec<&str> = data.split('\n').collect();
    assert_eq!(split, ["", "Märy häd ä little lämb", "Little lämb", ""]);

    let split: Vec<&str> = data.split_terminator('\n').collect();
    assert_eq!(split, ["", "Märy häd ä little lämb", "Little lämb"]);
}

#[test]
fn test_rsplit() {
    let data = "\nMäry häd ä little lämb\nLittle lämb\n";

    let split: Vec<&str> = data.rsplit(' ').collect();
    assert_eq!(split, ["lämb\n", "lämb\nLittle", "little", "ä", "häd", "\nMäry"]);

    let split: Vec<&str> = data.rsplit("lämb").collect();
    assert_eq!(split, ["\n", "\nLittle ", "\nMäry häd ä little "]);

    let split: Vec<&str> = data.rsplit(|c: char| c == 'ä').collect();
    assert_eq!(split, ["mb\n", "mb\nLittle l", " little l", "d ", "ry h", "\nM"]);
}

#[test]
fn test_rsplitn() {
    let data = "\nMäry häd ä little lämb\nLittle lämb\n";

    let split: Vec<&str> = data.rsplitn(2, ' ').collect();
    assert_eq!(split, ["lämb\n", "\nMäry häd ä little lämb\nLittle"]);

    let split: Vec<&str> = data.rsplitn(2, "lämb").collect();
    assert_eq!(split, ["\n", "\nMäry häd ä little lämb\nLittle "]);

    let split: Vec<&str> = data.rsplitn(2, |c: char| c == 'ä').collect();
    assert_eq!(split, ["mb\n", "\nMäry häd ä little lämb\nLittle l"]);
}

#[test]
fn test_splitator() {
    fn t(s: &str, sep: &str, u: &[&str]) {
        let v: Vec<&str> = s.split(sep).collect();
        assert_eq!(v, u);
    }
    t("--1233345--", "12345", &["--1233345--"]);
    t("abc::hello::there", "::", &["abc", "hello", "there"]);
    t("::hello::there", "::", &["", "hello", "there"]);
    t("hello::there::", "::", &["hello", "there", ""]);
    t("::hello::there::", "::", &["", "hello", "there", ""]);
    t("ประเทศไทย中华Việt Nam", "中华", &["ประเทศไทย", "Việt Nam"]);
    t("zzXXXzzYYYzz", "zz", &["", "XXX", "YYY", ""]);
    t("zzXXXzYYYz", "XXX", &["zz", "zYYYz"]);
    t(".XXX.YYY.", ".", &["", "XXX", "YYY", ""]);
    t("", ".", &[""]);
    t("zz", "zz", &["",""]);
    t("ok", "z", &["ok"]);
    t("zzz", "zz", &["","z"]);
    t("zzzzz", "zz", &["","","z"]);
}

#[test]
fn test_pattern_deref_forward() {
    let data = "aabcdaa";
    assert!(data.contains("bcd"));
    assert!(data.contains(&"bcd"));
    assert!(data.contains(&"bcd".to_string()));
}

#[test]
fn test_empty_match_indices() {
    let data = "aä中!";
    let vec: Vec<_> = data.match_indices("").collect();
    assert_eq!(vec, [(0, ""), (1, ""), (3, ""), (6, ""), (7, "")]);
}

fn check_contains_all_substrings(s: &str) {
    assert!(s.contains(""));
    for i in 0..s.len() {
        for j in i+1..s.len() + 1 {
            assert!(s.contains(&s[i..j]));
        }
    }
}

#[test]
fn strslice_issue_16589() {
    assert!("bananas".contains("nana"));

    // prior to the fix for #16589, x.contains("abcdabcd") returned false
    // test all substrings for good measure
    check_contains_all_substrings("012345678901234567890123456789bcdabcdabcd");
}

#[test]
fn strslice_issue_16878() {
    assert!(!"1234567ah012345678901ah".contains("hah"));
    assert!(!"00abc01234567890123456789abc".contains("bcabc"));
}


#[test]
fn test_strslice_contains() {
    let x = "There are moments, Jeeves, when one asks oneself, 'Do trousers matter?'";
    check_contains_all_substrings(x);
}

#[test]
fn test_rsplitn_char_iterator() {
    let data = "\nMäry häd ä little lämb\nLittle lämb\n";

    let mut split: Vec<&str> = data.rsplitn(4, ' ').collect();
    split.reverse();
    assert_eq!(split, ["\nMäry häd ä", "little", "lämb\nLittle", "lämb\n"]);

    let mut split: Vec<&str> = data.rsplitn(4, |c: char| c == ' ').collect();
    split.reverse();
    assert_eq!(split, ["\nMäry häd ä", "little", "lämb\nLittle", "lämb\n"]);

    // Unicode
    let mut split: Vec<&str> = data.rsplitn(4, 'ä').collect();
    split.reverse();
    assert_eq!(split, ["\nMäry häd ", " little l", "mb\nLittle l", "mb\n"]);

    let mut split: Vec<&str> = data.rsplitn(4, |c: char| c == 'ä').collect();
    split.reverse();
    assert_eq!(split, ["\nMäry häd ", " little l", "mb\nLittle l", "mb\n"]);
}

#[test]
fn test_split_char_iterator() {
    let data = "\nMäry häd ä little lämb\nLittle lämb\n";

    let split: Vec<&str> = data.split(' ').collect();
    assert_eq!( split, ["\nMäry", "häd", "ä", "little", "lämb\nLittle", "lämb\n"]);

    let mut rsplit: Vec<&str> = data.split(' ').rev().collect();
    rsplit.reverse();
    assert_eq!(rsplit, ["\nMäry", "häd", "ä", "little", "lämb\nLittle", "lämb\n"]);

    let split: Vec<&str> = data.split(|c: char| c == ' ').collect();
    assert_eq!( split, ["\nMäry", "häd", "ä", "little", "lämb\nLittle", "lämb\n"]);

    let mut rsplit: Vec<&str> = data.split(|c: char| c == ' ').rev().collect();
    rsplit.reverse();
    assert_eq!(rsplit, ["\nMäry", "häd", "ä", "little", "lämb\nLittle", "lämb\n"]);

    // Unicode
    let split: Vec<&str> = data.split('ä').collect();
    assert_eq!( split, ["\nM", "ry h", "d ", " little l", "mb\nLittle l", "mb\n"]);

    let mut rsplit: Vec<&str> = data.split('ä').rev().collect();
    rsplit.reverse();
    assert_eq!(rsplit, ["\nM", "ry h", "d ", " little l", "mb\nLittle l", "mb\n"]);

    let split: Vec<&str> = data.split(|c: char| c == 'ä').collect();
    assert_eq!( split, ["\nM", "ry h", "d ", " little l", "mb\nLittle l", "mb\n"]);

    let mut rsplit: Vec<&str> = data.split(|c: char| c == 'ä').rev().collect();
    rsplit.reverse();
    assert_eq!(rsplit, ["\nM", "ry h", "d ", " little l", "mb\nLittle l", "mb\n"]);
}

#[test]
fn test_rev_split_char_iterator_no_trailing() {
    let data = "\nMäry häd ä little lämb\nLittle lämb\n";

    let mut split: Vec<&str> = data.split('\n').rev().collect();
    split.reverse();
    assert_eq!(split, ["", "Märy häd ä little lämb", "Little lämb", ""]);

    let mut split: Vec<&str> = data.split_terminator('\n').rev().collect();
    split.reverse();
    assert_eq!(split, ["", "Märy häd ä little lämb", "Little lämb"]);
}

#[test]
fn starts_with_in_unicode() {
    assert!(!"├── Cargo.toml".starts_with("# "));
}

#[test]
fn starts_short_long() {
    assert!(!"".starts_with("##"));
    assert!(!"##".starts_with("####"));
    assert!("####".starts_with("##"));
    assert!(!"##ä".starts_with("####"));
    assert!("####ä".starts_with("##"));
    assert!(!"##".starts_with("####ä"));
    assert!("##ä##".starts_with("##ä"));

    assert!("".starts_with(""));
    assert!("ä".starts_with(""));
    assert!("#ä".starts_with(""));
    assert!("##ä".starts_with(""));
    assert!("ä###".starts_with(""));
    assert!("#ä##".starts_with(""));
    assert!("##ä#".starts_with(""));
}

#[test]
fn contains_weird_cases() {
    assert!("* \t".contains(' '));
    assert!(!"* \t".contains('?'));
    assert!(!"* \t".contains('\u{1F4A9}'));
}

#[test]
fn trim_ws() {
    assert_eq!(" \t  a \t  ".trim_left_matches(|c: char| c.is_whitespace()),
                    "a \t  ");
    assert_eq!(" \t  a \t  ".trim_right_matches(|c: char| c.is_whitespace()),
               " \t  a");
    assert_eq!(" \t  a \t  ".trim_matches(|c: char| c.is_whitespace()),
                    "a");
    assert_eq!(" \t   \t  ".trim_left_matches(|c: char| c.is_whitespace()),
                         "");
    assert_eq!(" \t   \t  ".trim_right_matches(|c: char| c.is_whitespace()),
               "");
    assert_eq!(" \t   \t  ".trim_matches(|c: char| c.is_whitespace()),
               "");
}

macro_rules! generate_iterator_test {
    {
        $name:ident {
            $(
                ($($arg:expr),*) -> [$($t:tt)*];
            )*
        }
        with $fwd:expr, $bwd:expr;
    } => {
        #[test]
        fn $name() {
            $(
                {
                    let res = vec![$($t)*];

                    let fwd_vec: Vec<_> = ($fwd)($($arg),*).collect();
                    assert_eq!(fwd_vec, res);

                    let mut bwd_vec: Vec<_> = ($bwd)($($arg),*).collect();
                    bwd_vec.reverse();
                    assert_eq!(bwd_vec, res);
                }
            )*
        }
    };
    {
        $name:ident {
            $(
                ($($arg:expr),*) -> [$($t:tt)*];
            )*
        }
        with $fwd:expr;
    } => {
        #[test]
        fn $name() {
            $(
                {
                    let res = vec![$($t)*];

                    let fwd_vec: Vec<_> = ($fwd)($($arg),*).collect();
                    assert_eq!(fwd_vec, res);
                }
            )*
        }
    }
}

generate_iterator_test! {
    double_ended_split {
        ("foo.bar.baz", '.') -> ["foo", "bar", "baz"];
        ("foo::bar::baz", "::") -> ["foo", "bar", "baz"];
    }
    with str::split, str::rsplit;
}

generate_iterator_test! {
    double_ended_split_terminator {
        ("foo;bar;baz;", ';') -> ["foo", "bar", "baz"];
    }
    with str::split_terminator, str::rsplit_terminator;
}

generate_iterator_test! {
    double_ended_matches {
        ("a1b2c3", char::is_numeric) -> ["1", "2", "3"];
    }
    with str::matches, str::rmatches;
}

generate_iterator_test! {
    double_ended_match_indices {
        ("a1b2c3", char::is_numeric) -> [(1, "1"), (3, "2"), (5, "3")];
    }
    with str::match_indices, str::rmatch_indices;
}

generate_iterator_test! {
    not_double_ended_splitn {
        ("foo::bar::baz", 2, "::") -> ["foo", "bar::baz"];
    }
    with str::splitn;
}

generate_iterator_test! {
    not_double_ended_rsplitn {
        ("foo::bar::baz", 2, "::") -> ["baz", "foo::bar"];
    }
    with str::rsplitn;
}

#[test]
fn different_str_pattern_forwarding_lifetimes() {
    use pattern_api_v2::{SearchCursors, Searcher};

    trait RealFind {
        fn real_find<'a, P: Pattern<&'a str>>(&'a self, pat: P) -> Option<usize>;
    }
    impl RealFind for str {
        fn real_find<'a, P: Pattern<&'a str>>(&'a self, pat: P) -> Option<usize> {
            let mut searcher = pat.into_searcher(self);
            let h = searcher.haystack();
            searcher.next_match().map(|(i, _)| <&'a str>::offset_from_front(h, i))
        }
    }

    fn foo<'a, P>(p: P) where for<'b> &'b P: Pattern<&'a str> {
        for _ in 0..3 {
            "asdf".real_find(&p);
        }
    }

    foo("x");
}
