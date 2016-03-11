extern crate pattern_api_v2 as pattern;

use pattern::Pattern;
use pattern::{Searcher, ReverseSearcher};
use pattern::SearchCursors;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum SearchResult {
    Match(usize, usize),
    Reject(usize, usize),
}

impl SearchResult {
    pub fn begin(&self) -> usize {
        match *self {
            Match(a, _) | Reject(a, _) => a,
        }
    }
    pub fn end(&self) -> usize {
        match *self {
            Match(_, b) | Reject(_, b) => b,
        }
    }
}

use SearchResult::{Match, Reject};

#[macro_export]
macro_rules! searcher_test {
    // For testing if the results of a double ended searcher exactly match what is expected
    ($name:ident, $p:expr, $h:expr, double, is exact [$($e:expr,)*]) => {
        #[allow(unused_imports)]
        mod $name {
            use $crate::SearchResult::{Match, Reject};
            use $crate::{cmp_search_to_vec};

            #[test]
            fn fwd_exact() {
                cmp_search_to_vec(false, || $p, $h, Some(vec![$($e),*]));
            }
            #[test]
            fn bwd_exact() {
                cmp_search_to_vec(true, || $p, $h, Some(vec![$($e),*]));
            }
        }
    };
    // For testing if the results of a double ended searcher are merely valid
    ($name:ident, $p:expr, $h:expr, double, is valid) => {
        #[allow(unused_imports)]
        mod $name {
            use $crate::SearchResult::{Match, Reject};
            use $crate::{cmp_search_to_vec, compare};

            #[test]
            fn fwd_and_bwd_valid() {
                let left = &cmp_search_to_vec(false, || $p, $h, None);
                let right = &cmp_search_to_vec(true, || $p, $h, None);
                compare(left, right, $h);
            }
        }
    };
    // For testing if the results of a forward-reverse searcher exactly match what is expected
    ($name:ident, $p:expr, $h:expr, reverse, is exact [$($e:expr,)*], [$($f:expr,)*]) => {
        #[allow(unused_imports)]
        mod $name {
            use $crate::SearchResult::{Match, Reject};
            use $crate::{cmp_search_to_vec};

            #[test]
            fn fwd_exact() {
                cmp_search_to_vec(false, || $p, $h, Some(vec![$($e),*]));
            }
            #[test]
            fn bwd_exact() {
                cmp_search_to_vec(true, || $p, $h, Some(vec![$($f),*]));
            }
        }
    };
    // For testing if the results of a forward-reverse searcher are merely valid
    ($name:ident, $p:expr, $h:expr, reverse, is valid) => {
        #[allow(unused_imports)]
        mod $name {
            use $crate::SearchResult::{Match, Reject};
            use $crate::{cmp_search_to_vec};

            #[test]
            fn fwd_and_bwd_valid() {
                let left = &cmp_search_to_vec(false, || $p, $h, None);
                let right = &cmp_search_to_vec(true, || $p, $h, None);
            }
        }
    };
}

pub fn cmp_search_to_vec<'a, P, F>(rev: bool,
                                   pat: F,
                                   haystack: &'a str,
                                   right: Option<Vec<SearchResult>>) -> Vec<SearchResult>
where P: Pattern<&'a str>,
      P::Searcher: ReverseSearcher<&'a str>,
      F: Fn() -> P,
{
    let mut matches = {
        let mut searcher = pat().into_searcher(haystack);
        let mut v = vec![];
        loop {
            match if !rev {searcher.next_match()} else {searcher.next_match_back()} {
                Some((a, b)) => v.push((a, b)),
                None => break,
            }
        }
        if rev {
            v.reverse();
        }
        v.into_iter().map(|(a, b)| {
            let haystack = haystack.into_haystack();
            (
                <&'a str>::offset_from_front(haystack, a),
                <&'a str>::offset_from_front(haystack, b),
            )
        })
    };

    let mut rejects = {
        let mut searcher = pat().into_searcher(haystack);
        let mut v = vec![];
        loop {
            match if !rev {searcher.next_reject()} else {searcher.next_reject_back()} {
                Some((a, b)) => v.push((a, b)),
                None => break,
            }
        }
        if rev {
            v.reverse();
        }
        v.into_iter().map(|(a, b)| {
            let haystack = haystack.into_haystack();
            (
                <&'a str>::offset_from_front(haystack, a),
                <&'a str>::offset_from_front(haystack, b),
            )
        })
    };

    let mut v = vec![];

    // Merge the two streams of results
    {
        let mut cur_match = matches.next();
        let mut cur_reject = rejects.next();

        loop {
            if cur_match.is_none() && cur_reject.is_none() {
                break;
            } else if cur_match.is_some() && cur_reject.is_some() {
                let m = cur_match.unwrap();
                let r = cur_reject.unwrap();

                if m.0 <= r.0 {
                    v.push(Match(m.0, m.1));
                    cur_match = matches.next();
                } else {
                    v.push(Reject(r.0, r.1));
                    cur_reject = rejects.next();
                }
            } else if cur_match.is_some() {
                let m = cur_match.unwrap();
                v.push(Match(m.0, m.1));
                cur_match = matches.next();
            } else if cur_reject.is_some() {
                let r = cur_reject.unwrap();
                v.push(Reject(r.0, r.1));
                cur_reject = rejects.next();
            }
        }
    }

    println!("");

    // Validate and emit diagnostics

    if is_malformed(&v, haystack) {
        panic!("searcher impl outputted invalid search results");
    }

    if let Some(right) = right {
        compare(&v, &right, haystack);
    }

    v
}

pub fn is_malformed<H: SearchCursors>(v: &[SearchResult], haystack: H) -> bool {
    let mut found = false;
    for (i, pair) in v.windows(2).enumerate() {
        if pair[0].end() < pair[1].begin() {
            println!("Gap detected at end of {:?}", &v[..i+2]);
            found = true;
        }
        if pair[0].end() > pair[1].begin() {
            println!("Overlap detected at end of {:?}", &v[..i+2]);
            found = true;
        }
    }

    for (i, &e) in v.iter().enumerate() {
        if let Reject(a, b) = e {
            if a == b {
                println!("Zero-length Reject detected at end of {:?}", &v[..i+1]);
                found = true;
            }
        }
    }

    if v.len() > 0 {
        if v[0].begin() != 0 {
            println!("First interval did not start at begin of haystack: [{:?}, ...]", &v[0]);
            found = true;
        }

        let haystack = haystack.into_haystack();
        if v[v.len() - 1].end() != H::haystack_len(haystack) {
            println!("Last interval did not end at end of haystack: [..., {:?}]", &v[v.len() - 1]);
            found = true;
        }
    }

    found
}

pub fn compare(left: &[SearchResult], right: &[SearchResult], haystack: &str) {
    if is_malformed(&right, haystack) {
        panic!("should-be search result test input is malformed, check test code for correctness");
    }

    assert!(left == right, "\n  searcher:  {:?}\n  should-be: {:?}\n", left, right);
}
