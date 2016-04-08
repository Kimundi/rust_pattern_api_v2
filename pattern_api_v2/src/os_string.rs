/*
    notes to existing osstring RFC:

    - replace() should work with pattern due to consistency with string,
      or have another name
    -

*/

macro_rules! otry {
    ($e:expr) => {
        match $e {
            Some(o) => return Some(o),
            None => (),
        }
    }
}

macro_rules! impl_both_mutability {
    ($module:ident, $slice:ty,
                    $cursor:ty,
                    $cursor_elem:ty,
                    $cursors_to_haystack:expr,
                    $haystack_to_cursors:expr,
                    $str_slice:ty) => {
        pub mod $module {
            use core_traits::*;
            use std::ffi::OsStr;
            use std::mem;

            #[derive(Copy, Clone)]
            struct Iter<'a> {
                haystack: ($cursor, $cursor),
                start: $cursor,
                end: $cursor,
                _marker: ::std::marker::PhantomData<$slice>
            }

            impl<'a> Iter<'a> {
                #[inline]
                fn new(haystack: $slice) -> Self {
                    let (start, end) = $haystack_to_cursors(haystack);
                    Iter {
                        haystack: (start, end),
                        start: start,
                        end: end,
                        _marker: ::std::marker::PhantomData,
                    }
                }

                #[inline]
                fn next(&mut self) -> Option<$cursor_elem> {
                    while self.start != self.end {
                        unsafe {
                            let b = *self.start;
                            self.start = self.start.offset(1);
                            return Some(b);
                        }
                    }
                    None
                }

                #[inline]
                fn next_back(&mut self) -> Option<$cursor_elem> {
                    while self.start != self.end {
                        unsafe {
                            self.end = self.end.offset(-1);
                            let b = *self.end;
                            return Some(b);
                        }
                    }
                    None
                }

                #[inline]
                fn skip_non_utf8(&mut self) -> Option<($cursor, $cursor)> {
                    let original_start = self.start;
                    while self.start != self.end {
                        unsafe {
                            let current_start_is_valid =
                                utf8::ptr_range_starts_with_valid_utf8(self.start,
                                                                       self.end);

                            if current_start_is_valid {
                                break;
                            }

                            self.start = self.start.offset(1);
                        }
                    }

                    if original_start != self.start {
                        Some((original_start, self.start))
                    } else {
                        None
                    }
                }

                #[inline]
                fn skip_non_utf8_reverse(&mut self) -> Option<($cursor, $cursor)> {
                    let original_end = self.end;
                    while self.start != self.end {
                        unsafe {
                            let current_end_is_valid =
                                utf8::ptr_range_ends_with_valid_utf8(self.start,
                                                                     self.end);

                            if current_end_is_valid {
                                break;
                            }

                            self.end = self.end.offset(-1);
                        }
                    }

                    if original_end != self.end {
                        Some((self.end, original_end))
                    } else {
                        None
                    }
                }
            }

            impl<'a> SearchCursors for $slice {
                type Haystack = ($cursor, $cursor);
                type Cursor = $cursor;
                type MatchType = $slice;

                fn into_haystack(self) -> Self::Haystack {
                    $haystack_to_cursors(self)
                }

                fn offset_from_front(haystack: Self::Haystack,
                                     begin: Self::Cursor) -> usize {
                    begin as usize - haystack.0 as usize
                }

                unsafe fn range_to_self(_: Self::Haystack,
                                        start: Self::Cursor,
                                        end: Self::Cursor) -> Self::MatchType {
                    ($cursors_to_haystack)(start, end)
                }
                fn cursor_at_front(hs: Self::Haystack) -> Self::Cursor {
                    hs.0
                }
                fn cursor_at_back(hs: Self::Haystack) -> Self::Cursor {
                    hs.1
                }
                fn match_type_len(mt: &Self::MatchType) -> usize { mt.len() }
            }

            unsafe impl<'a> InverseMatchesAreValid for $slice {}

            //////////////////////////////////////////////////////////////////
            // Impl for a CharEq wrapper
            //////////////////////////////////////////////////////////////////

            use utf8::{self, CharEq, CharEqPattern};

            #[derive(Clone)]
            pub struct CharEqSearcher<'a, C: CharEq> {
                char_eq: C,
                iter: Iter<'a>,
                ascii_only: bool,
            }

            impl<'a, C: CharEq> Pattern<$slice> for CharEqPattern<C> {
                type Searcher = CharEqSearcher<'a, C>;

                #[inline]
                fn into_searcher(self, haystack: $slice) -> CharEqSearcher<'a, C> {
                    CharEqSearcher {
                        ascii_only: self.0.only_ascii(),
                        char_eq: self.0,
                        iter: Iter::new(haystack),
                    }
                }
            }

            unsafe impl<'a, C: CharEq> Searcher<$slice> for CharEqSearcher<'a, C> {
                #[inline]
                fn haystack(&self) -> ($cursor, $cursor) {
                    self.iter.haystack
                }

                #[inline]
                fn next_match(&mut self) -> Option<($cursor, $cursor)> {
                    if self.ascii_only {
                        while let Some(b) = {
                            self.iter.skip_non_utf8();
                            self.iter.next()
                        } {
                            if b < 128 && self.char_eq.matches(b as char) {
                                return Some(unsafe {
                                    (self.iter.start.offset(-1),
                                     self.iter.start)
                                })
                            }
                        }
                    } else {
                        while let Some(c) = {
                            self.iter.skip_non_utf8();
                            utf8::next_code_point(|| self.iter.next())
                        } {
                            if self.char_eq.matches(c) {
                                return Some(unsafe {
                                    (self.iter.start.offset(-(c.len_utf8() as isize)),
                                     self.iter.start)
                                })
                            }
                        }
                    }
                    None
                }

                #[inline]
                fn next_reject(&mut self) -> Option<($cursor, $cursor)> {
                    if self.ascii_only {
                        while let Some(b) = {
                            otry!(self.iter.skip_non_utf8());
                            self.iter.next()
                        } {
                            if b > 127 || !self.char_eq.matches(b as char) {
                                unsafe {
                                    let reject_start = self.iter.start.offset(-1);
                                    return Some((reject_start, self.iter.start))
                                }
                            }
                        }
                    } else {
                        while let Some(c) = {
                            otry!(self.iter.skip_non_utf8());
                            utf8::next_code_point(|| self.iter.next())
                        } {
                            if !self.char_eq.matches(c) {
                                return Some(unsafe {
                                    (self.iter.start.offset(-(c.len_utf8() as isize)),
                                     self.iter.start)
                                })
                            }
                        }
                    }
                    None
                }
            }

            unsafe impl<'a, C: CharEq> ReverseSearcher<$slice> for CharEqSearcher<'a, C> {
                #[inline]
                fn next_match_back(&mut self) -> Option<($cursor, $cursor)>  {
                    if self.ascii_only {
                        while let Some(b) = {
                            self.iter.skip_non_utf8_reverse();
                            self.iter.next_back()
                        } {
                            if b < 128 && self.char_eq.matches(b as char) {
                                return Some(unsafe {
                                    (self.iter.end,
                                     self.iter.end.offset(1))
                                })
                            }
                        }
                    } else {
                        while let Some(c) = {
                            self.iter.skip_non_utf8_reverse();
                            utf8::next_code_point_reverse(|| self.iter.next_back())
                        } {
                            if self.char_eq.matches(c) {
                                return Some(unsafe {
                                    (self.iter.end,
                                     self.iter.end.offset(c.len_utf8() as isize))
                                })
                            }
                        }
                    }
                    None
                }

                #[inline]
                fn next_reject_back(&mut self) -> Option<($cursor, $cursor)>  {
                    if self.ascii_only {
                        while let Some(b) = {
                            otry!(self.iter.skip_non_utf8_reverse());
                            self.iter.next_back()
                        } {
                            if b > 127 || !self.char_eq.matches(b as char) {
                                unsafe {
                                    let reject_end = self.iter.end.offset(1);
                                    return Some((self.iter.end, reject_end))
                                }
                            }
                        }
                    } else {
                        while let Some(c) = {
                            otry!(self.iter.skip_non_utf8_reverse());
                            utf8::next_code_point_reverse(|| self.iter.next_back())
                        } {
                            if !self.char_eq.matches(c) {
                                return Some(unsafe {
                                    (self.iter.end,
                                     self.iter.end.offset(c.len_utf8() as isize))
                                })
                            }
                        }
                    }
                    None
                }
            }

            impl<'a, C: CharEq> DoubleEndedSearcher<$slice> for CharEqSearcher<'a, C> {}

            /////////////////////////////////////////////////////////////////////////////
            // Impl for char
            /////////////////////////////////////////////////////////////////////////////

            /// Associated type for `<char as Pattern<&'a str>>::Searcher`.
            #[derive(Clone)]
            pub struct CharSearcher<'a>(CharEqSearcher<'a, char>);

            unsafe impl<'a> Searcher<$slice> for CharSearcher<'a> {
                searcher_methods!(forward, s, s.0, $cursor);
            }

            unsafe impl<'a> ReverseSearcher<$slice> for CharSearcher<'a> {
                searcher_methods!(reverse, s, s.0, $cursor);
            }

            impl<'a> DoubleEndedSearcher<$slice> for CharSearcher<'a> {}

            /// Searches for chars that are equal to a given char
            impl<'a> Pattern<$slice> for char {
                pattern_methods!(CharSearcher<'a>, CharEqPattern, CharSearcher, $slice);
            }

            /////////////////////////////////////////////////////////////////////////////
            // Impl for F: FnMut(char) -> bool
            /////////////////////////////////////////////////////////////////////////////

            /// Associated type for `<F as Pattern<&'a str>>::Searcher`.
            #[derive(Clone)]
            pub struct CharPredicateSearcher<'a, F>(CharEqSearcher<'a, F>)
                where F: FnMut(char) -> bool;

            unsafe impl<'a, F> Searcher<$slice> for CharPredicateSearcher<'a, F>
                where F: FnMut(char) -> bool
            {
                searcher_methods!(forward, s, s.0, $cursor);
            }

            unsafe impl<'a, F> ReverseSearcher<$slice> for CharPredicateSearcher<'a, F>
                where F: FnMut(char) -> bool
            {
                searcher_methods!(reverse, s, s.0, $cursor);
            }

            impl<'a, F> DoubleEndedSearcher<$slice> for CharPredicateSearcher<'a, F>
                where F: FnMut(char) -> bool {}

            /// Searches for chars that match the given predicate
            impl<'a, F> Pattern<$slice> for F where F: FnMut(char) -> bool {
                pattern_methods!(CharPredicateSearcher<'a, F>, CharEqPattern, CharPredicateSearcher, $slice);
            }

            /////////////////////////////////////////////////////////////////////////////
            // Impl for &[char]
            /////////////////////////////////////////////////////////////////////////////

            // Todo: Change / Remove due to ambiguity in meaning.

            /// Associated type for `<&[char] as Pattern<&'a str>>::Searcher`.
            #[derive(Clone)]
            pub struct CharSliceSearcher<'a, 'b>(CharEqSearcher<'a, &'b [char]>);

            unsafe impl<'a, 'b> Searcher<$slice> for CharSliceSearcher<'a, 'b> {
                searcher_methods!(forward, s, s.0, $cursor);
            }

            unsafe impl<'a, 'b> ReverseSearcher<$slice> for CharSliceSearcher<'a, 'b> {
                searcher_methods!(reverse, s, s.0, $cursor);
            }

            impl<'a, 'b> DoubleEndedSearcher<$slice> for CharSliceSearcher<'a, 'b> {}

            /// Searches for chars that are equal to any of the chars in the array
            impl<'a, 'b> Pattern<$slice> for &'b [char] {
                pattern_methods!(CharSliceSearcher<'a, 'b>, CharEqPattern, CharSliceSearcher, $slice);
            }

            ////////////////////////////////////////////////////////////////////
            // Impl for &OsStr
            ////////////////////////////////////////////////////////////////////

            use fast_sequence_search::{OrdSlice, OrdSlicePattern, OrdSeqSearcher};
            use fast_sequence_search::ByteOptimization;

            pub struct OsStrSearcher<'a, 'b>(OrdSeqSearcher<'b, $slice>);

            impl<'a> OrdSlice for $slice {
                type NeedleElement = u8;
                type FastSkipOptimization = ByteOptimization;

                fn next_valid_pos(hs: &Self::Haystack, pos: usize) -> Option<usize> {
                    let s = Self::haystack_as_slice(hs);
                    s[pos..].iter().next().map(|_| pos + 1)
                }

                fn next_valid_pos_back(hs: &Self::Haystack, pos: usize) -> Option<usize> {
                    let s = Self::haystack_as_slice(hs);
                    s[..pos].iter().next_back().map(|_| pos - 1)
                }

                fn haystack_as_slice<'t>(hs: &'t Self::Haystack) -> &'t [Self::NeedleElement] {
                    unsafe {
                        mem::transmute($cursors_to_haystack(hs.0, hs.1))
                    }
                }

                fn pos_is_valid(_: &Self::Haystack, _: usize) -> bool {
                    true
                }

                unsafe fn cursor_at_offset(hs: Self::Haystack, offset: usize) -> Self::Cursor {
                    hs.0.offset(offset as isize)
                }
            }

            /// Non-allocating substring search.
            ///
            /// Will handle the pattern `""` as returning empty matches at each character
            /// boundary.
            impl<'a, 'b> Pattern<$slice> for &'b OsStr {
                pattern_methods!{
                    OsStrSearcher<'a, 'b>,
                    |s: &'b OsStr| {
                        cfg_match! {
                            windows => {
                                // on windows lone surrogate pairs
                                // at the front/back
                                // need to be considered a contract violation
                                // since it is not possible
                                // to find them on a OsStr if they
                                // are encoded as part of a normal
                                // character

                                let (a, _, b) = super::split_loony_surrogates(s);

                                if a.len() > 0 || b.len() > 0 {
                                    panic!("The Pattern API does not support \
                                            searching for strings \
                                            starting or ending with \
                                            lone surrogate codepoints");
                                }
                            }
                            unix => {
                                // On this platform the surrogate issue
                                // does not exist
                            }
                        }

                        unsafe {
                            OrdSlicePattern(
                                mem::transmute::<&'b OsStr, &'b [u8]>(s))
                        }
                    },
                    OsStrSearcher,
                    $slice
                }
            }

            unsafe impl<'a, 'b> Searcher<$slice> for OsStrSearcher<'a, 'b> {
                searcher_methods!(forward, s, s.0, $cursor);
            }

            unsafe impl<'a, 'b> ReverseSearcher<$slice> for OsStrSearcher<'a, 'b> {
                searcher_methods!(reverse, s, s.0, $cursor);
            }

            ////////////////////////////////////////////////////////////////////
            // Impl for &str
            ////////////////////////////////////////////////////////////////////

            pub struct StrSearcher<'a, 'b>(OrdSeqSearcher<'b, $slice>);

            /// Non-allocating substring search.
            ///
            /// Will handle the pattern `""` as returning empty matches at each character
            /// boundary.
            impl<'a, 'b> Pattern<$slice> for &'b str {
                pattern_methods!(StrSearcher<'a, 'b>,
                                |s: &'b str| OrdSlicePattern(s.as_bytes()),
                                StrSearcher,
                                $slice);
            }

            unsafe impl<'a, 'b> Searcher<$slice> for StrSearcher<'a, 'b> {
                searcher_methods!(forward, s, s.0, $cursor);
            }

            unsafe impl<'a, 'b> ReverseSearcher<$slice> for StrSearcher<'a, 'b> {
                searcher_methods!(reverse, s, s.0, $cursor);
            }

            ////////////////////////////////////////////////////////////////////
            // Wrapper for returning &str matches
            ////////////////////////////////////////////////////////////////////

            pub struct PartialUnicode<'a> {
                pub os_str: $slice
            }

            impl<'a> SearchCursors for PartialUnicode<'a> {
                type Haystack = ($cursor, $cursor);
                type Cursor = $cursor;
                type MatchType = $str_slice;

                fn into_haystack(self) -> Self::Haystack {
                    self.os_str.into_haystack()
                }

                fn offset_from_front(haystack: Self::Haystack,
                                     begin: Self::Cursor) -> usize {
                    <$slice>::offset_from_front(haystack, begin)
                }

                unsafe fn range_to_self(h: Self::Haystack,
                                        start: Self::Cursor,
                                        end: Self::Cursor) -> Self::MatchType {
                    let s = <$slice>::range_to_self(h, start, end);

                    // cast &[mut]OsStr to &[mut]str
                    mem::transmute::<$slice, $str_slice>(s)
                }
                fn cursor_at_front(hs: Self::Haystack) -> Self::Cursor {
                    <$slice>::cursor_at_front(hs)
                }
                fn cursor_at_back(hs: Self::Haystack) -> Self::Cursor {
                    <$slice>::cursor_at_back(hs)
                }
                fn match_type_len(mt: &Self::MatchType) -> usize { mt.len() }
            }

            ////////////////////////////////////////////////////////////////////
            // PartialUnicode impl for char
            ////////////////////////////////////////////////////////////////////

            unsafe impl<'a> Searcher<PartialUnicode<'a>> for CharSearcher<'a> {
                searcher_methods!(forward, s, s.0, $cursor);
            }

            unsafe impl<'a> ReverseSearcher<PartialUnicode<'a>> for CharSearcher<'a> {
                searcher_methods!(reverse, s, s.0, $cursor);
            }

            impl<'a> DoubleEndedSearcher<PartialUnicode<'a>> for CharSearcher<'a> {}

            /// Searches for chars that are equal to a given char
            impl<'a> Pattern<PartialUnicode<'a>> for char {
                pattern_methods!(CharSearcher<'a>, CharEqPattern, CharSearcher,
                                 PartialUnicode<'a>, |s: PartialUnicode<'a>| s.os_str);
            }

            ////////////////////////////////////////////////////////////////////
            // PartialUnicode impl for FnMut(char) -> bool
            ////////////////////////////////////////////////////////////////////

            unsafe impl<'a, F> Searcher<PartialUnicode<'a>> for CharPredicateSearcher<'a, F>
                where F: FnMut(char) -> bool
            {
                searcher_methods!(forward, s, s.0, $cursor);
            }

            unsafe impl<'a, F> ReverseSearcher<PartialUnicode<'a>> for CharPredicateSearcher<'a, F>
                where F: FnMut(char) -> bool
            {
                searcher_methods!(reverse, s, s.0, $cursor);
            }

            impl<'a, F> DoubleEndedSearcher<PartialUnicode<'a>> for CharPredicateSearcher<'a, F>
                where F: FnMut(char) -> bool {}

            /// Searches for chars that match the given predicate
            impl<'a, F> Pattern<PartialUnicode<'a>> for F where F: FnMut(char) -> bool {
                pattern_methods!(CharPredicateSearcher<'a, F>, CharEqPattern, CharPredicateSearcher, PartialUnicode<'a>, |s: PartialUnicode<'a>| s.os_str);
            }

            ////////////////////////////////////////////////////////////////////
            // PartialUnicode impl for &[char]
            ////////////////////////////////////////////////////////////////////

            // Todo: Change / Remove due to ambiguity in meaning.

            unsafe impl<'a, 'b> Searcher<PartialUnicode<'a>> for CharSliceSearcher<'a, 'b> {
                searcher_methods!(forward, s, s.0, $cursor);
            }

            unsafe impl<'a, 'b> ReverseSearcher<PartialUnicode<'a>> for CharSliceSearcher<'a, 'b> {
                searcher_methods!(reverse, s, s.0, $cursor);
            }

            impl<'a, 'b> DoubleEndedSearcher<PartialUnicode<'a>> for CharSliceSearcher<'a, 'b> {}

            /// Searches for chars that are equal to any of the chars in the array
            impl<'a, 'b> Pattern<PartialUnicode<'a>> for &'b [char] {
                pattern_methods!(CharSliceSearcher<'a, 'b>, CharEqPattern, CharSliceSearcher, PartialUnicode<'a>, |s: PartialUnicode<'a>| s.os_str);
            }
            ////////////////////////////////////////////////////////////////////
            // PartialUnicode impl for &str
            ////////////////////////////////////////////////////////////////////

            /// Non-allocating substring search.
            ///
            /// Will handle the pattern `""` as returning empty matches at each character
            /// boundary.
            impl<'a, 'b> Pattern<PartialUnicode<'a>> for &'b str {
                pattern_methods!(StrSearcher<'a, 'b>,
                                |s: &'b str| OrdSlicePattern(s.as_bytes()),
                                StrSearcher,
                                PartialUnicode<'a>,
                                |s: PartialUnicode<'a>| s.os_str);
            }

            unsafe impl<'a, 'b> Searcher<PartialUnicode<'a>> for StrSearcher<'a, 'b> {
                searcher_methods!(forward, s, s.0, $cursor);
            }

            unsafe impl<'a, 'b> ReverseSearcher<PartialUnicode<'a>> for StrSearcher<'a, 'b> {
                searcher_methods!(reverse, s, s.0, $cursor);
            }
        }
    }
}

impl_both_mutability!(shared, &'a OsStr, *const u8, u8, |start, end| {
    let slice = ::std::slice::from_raw_parts(start,
        end as usize - start as usize);

    // TODO: Solve properly
    mem::transmute::<&'a [u8], &'a OsStr>(slice)
}, |haystack: &OsStr| {
    // TODO: Solve properly
    let haystack = unsafe {
        mem::transmute::<&OsStr, &[u8]>(haystack)
    };

    let begin = haystack.as_ptr();
    let end = unsafe {
        begin.offset(haystack.len() as isize)
    };
    (begin, end)
}, &'a str);

impl_both_mutability!(mutable, &'a mut OsStr, *mut u8, u8, |start, end| {
    let slice = ::std::slice::from_raw_parts_mut(start,
        end as usize - start as usize);

    // TODO: Solve properly
    mem::transmute::<&'a mut [u8], &'a mut OsStr>(slice)
}, |haystack: &mut OsStr| {
    // TODO: Solve properly
    let haystack = unsafe {
        mem::transmute::<&mut OsStr, &mut [u8]>(haystack)
    };

    let begin = haystack.as_mut_ptr();
    let end = unsafe {
        begin.offset(haystack.len() as isize)
    };
    (begin, end)
}, &'a mut str);

use ::Pattern;
use ::ReverseSearcher;
use std::ffi::OsStr;
use std::ffi::OsString;

impl<'a, 'b> Pattern<&'a OsStr> for &'b String {
    pattern_methods!(shared::StrSearcher<'a, 'b>, |s: &'b String| &**s, |s| s, &'a OsStr);
}

impl<'a, 'b, 'c> Pattern<&'a OsStr> for &'c &'b str {
    pattern_methods!(shared::StrSearcher<'a, 'b>, |&s| s, |s| s, &'a OsStr);
}

impl<'a, 'b> Pattern<&'a OsStr> for &'b OsString {
    pattern_methods!(shared::OsStrSearcher<'a, 'b>, |s: &'b OsString| &**s, |s| s, &'a OsStr);
}

impl<'a, 'b, 'c> Pattern<&'a OsStr> for &'c &'b OsStr {
    pattern_methods!(shared::OsStrSearcher<'a, 'b>, |&s| s, |s| s, &'a OsStr);
}

fn starts_with_surrogate(v: &[u8]) -> Option<u16> {
    let mut iter = v.iter().cloned();
    use utf8::next_code_point;

    if v.len() >= 3
    && v[0] == 237
    && v[1] >= 160
    && v[1] <= 191
    && v[2] >= 128
    && v[2] <= 191 {
        next_code_point(|| iter.next()).map(|c| c as u32 as u16)
    } else {
        None
    }
}

fn ends_with_surrogate(v: &[u8]) -> Option<u16> {
    if v.len() >= 3 {
        starts_with_surrogate(&v[v.len() - 3..])
    } else {
        None
    }
}

fn split_loony_surrogates(s: &OsStr) -> (&[u8], &[u8], &[u8]) {
    let s = unsafe {
        ::std::mem::transmute::<&OsStr, &[u8]>(s)
    };

    let mut front_len = 0;
    if let Some(s) = starts_with_surrogate(s) {
        if (0xDC00...0xDFFF).contains(s) {
            front_len = 3;
        }
    };

    let mut back_len = 0;
    if let Some(s) = ends_with_surrogate(s) {
        if (0xD800...0xDBFF).contains(s) {
            back_len = 3;
        }
    };

    let a = front_len;
    let b = s.len() - back_len;

    (&s[..a], &s[a..b], &s[b..])
}

#[test]
fn test_split_loony_surrogates() {
    fn check(s: &[u16]) -> (usize, usize, usize) {
        use std_integration::OsStringExtension;
        let s = OsString::from_wide(s);
        let (x, y, z) = split_loony_surrogates(&s);
        (x.len(), y.len(), z.len())
    }

    // regular surrogate pair
    assert_eq!(check(&[        0xD800, 0xDC00        ]), (0,4,0));
    // split-up at front/back
    assert_eq!(check(&[0xDC00, 0xD800, 0xDC00        ]), (3,4,0));
    assert_eq!(check(&[        0xD800, 0xDC00, 0xD800]), (0,4,3));
    assert_eq!(check(&[0xDC00, 0xD800, 0xDC00, 0xD800]), (3,4,3));
    assert_eq!(check(&[0xDC00,                 0xD800]), (3,0,3));
    assert_eq!(check(&[0xDC00,                       ]), (3,0,0));
    assert_eq!(check(&[                        0xD800]), (0,0,3));
}

