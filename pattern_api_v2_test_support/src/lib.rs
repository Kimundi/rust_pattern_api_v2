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
    ($name:ident, $p:expr, $h:expr, double: [$($e:expr,)*]) => {
        #[allow(unused_imports)]
        mod $name {
            use $crate::SearchResult::{Match, Reject};
            use $crate::{cmp_search_to_vec};
            use super::*;

            #[test]
            fn fwd_exact() {
                cmp_search_to_vec(false, || $p, || $h, Some(vec![$($e),*]));
            }
            #[test]
            fn bwd_exact() {
                cmp_search_to_vec(true, || $p, || $h, Some(vec![$($e),*]));
            }
        }
    };
    // For testing if the results of a forward-reverse searcher exactly match what is expected
    ($name:ident, $p:expr, $h:expr, forward: [$($e:expr,)*], backward: [$($f:expr,)*]) => {
        #[allow(unused_imports)]
        mod $name {
            use $crate::SearchResult::{Match, Reject};
            use $crate::{cmp_search_to_vec};

            #[test]
            fn fwd_exact() {
                cmp_search_to_vec(false, || $p, || $h, Some(vec![$($e),*]));
            }
            #[test]
            fn bwd_exact() {
                cmp_search_to_vec(true, || $p, || $h, Some(vec![$($f),*]));
            }
        }
    };
}

pub fn cmp_search_to_vec<'a, H, P, F, HF>(rev: bool,
                                          mut pat: F,
                                          mut haystack: HF,
                                          right: Option<Vec<SearchResult>>) -> Vec<SearchResult>
where H: SearchCursors,
      P: Pattern<H>,
      P::Searcher: ReverseSearcher<H>,
      F: FnMut() -> P,
      HF: FnMut() -> H,
{

    cmp_search_to_vec2(rev, |f: &mut Callback| {
        let h = haystack();
        let p = pat();
        f.call(h, p);
    }, right)
}

#[macro_export]
macro_rules! searcher_cross_test {
    ($tname:ident {
        double: [$($res:expr,)*];
        for:
        $($cname:ident, $hty:ty: $h:expr, $pty:ty: $p:expr;)*
    }) => {
        #[allow(unused_imports)]
        mod $tname {
            use $crate::SearchResult::{self, Match, Reject};

            fn build() -> Vec<SearchResult> {
                vec![$($res),*]
            }

            $(
                mod $cname {
                    use $crate::SearchResult::{Match, Reject};
                    use $crate::{cmp_search_to_vec2, Callback};
                    use super::build;

                    #[test]
                    fn fwd_exact() {
                        cmp_search_to_vec2(false, |f: &mut Callback| {
                            let h: $hty = $h;
                            let p: $pty = $p;
                            f.call(h, p);
                        }, Some(build()));
                    }
                    #[test]
                    fn bwd_exact() {
                        cmp_search_to_vec2(true, |f: &mut Callback| {
                            let h: $hty = $h;
                            let p: $pty = $p;
                            f.call(h, p);
                        }, Some(build()));
                    }
                }
            )*
        }
    }
}

enum CallbackMode {
    Gather {
        rev: bool,
        matches: bool,
    },
}

pub struct Callback {
    mode: CallbackMode,
    result: Vec<(usize, usize)>,
    hs_len: Option<usize>,
}
impl Callback {
    pub fn call<H, P>(&mut self, haystack: H, pattern: P)
        where H: SearchCursors,
              P: Pattern<H>,
              P::Searcher: ReverseSearcher<H>,
    {
        match self.mode {
            CallbackMode::Gather { rev, matches } => {
                let r = {
                    let mut searcher = pattern.into_searcher(haystack);
                    let mut v = vec![];
                    loop {
                        let next = match (rev, matches) {
                            (false, true) =>  searcher.next_match(),
                            (false, false) =>  searcher.next_reject(),
                            (true, true) =>  searcher.next_match_back(),
                            (true, false) =>  searcher.next_reject_back(),
                        };
                        match next {
                            Some((a, b)) => v.push((a, b)),
                            None => break,
                        }
                    }
                    if rev {
                        v.reverse();
                    }
                    self.hs_len = Some(H::haystack_len(searcher.haystack()));
                    v.into_iter().map(|(a, b)| {
                        let haystack = searcher.haystack();
                        (
                            H::offset_from_front(haystack, a),
                            H::offset_from_front(haystack, b),
                        )
                    }).collect::<Vec<_>>()
                };
                self.result = r;
            }
        }

    }

    fn new(rev: bool, matches: bool) -> Self {
        Callback {
            mode: CallbackMode::Gather {
                rev: rev,
                matches: matches,
            },
            result: vec![],
            hs_len: None,
        }
    }
}


pub fn cmp_search_to_vec2<F>(rev: bool,
                             mut f: F,
                             right: Option<Vec<SearchResult>>)
                             ->  Vec<SearchResult>
where F: FnMut(&mut Callback),
{
    let mut matches = Callback::new(rev, true);
    f(&mut matches);

    let mut rejects = Callback::new(rev, false);
    f(&mut rejects);

    let hs_len = matches.hs_len.unwrap();

    let mut matches = matches.result.into_iter();
    let mut rejects = rejects.result.into_iter();

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

    if is_malformed(&v, hs_len) {
        panic!("searcher impl outputted invalid search results");
    }

    if let Some(right) = right {
        compare(&v, &right, hs_len);
    }

    v
}

pub fn is_malformed(v: &[SearchResult], haystack_len: usize) -> bool {
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

        if v[v.len() - 1].end() != haystack_len {
            println!("Last interval did not end at end of haystack: [..., {:?}]", &v[v.len() - 1]);
            found = true;
        }
    }

    found
}

pub fn compare(left: &[SearchResult], right: &[SearchResult], haystack_len: usize) {
    if is_malformed(&right, haystack_len) {
        panic!("should-be search result test input is malformed, check test code for correctness");
    }

    assert!(left == right, "\n  searcher:  {:?}\n  should-be: {:?}\n", left, right);
}
