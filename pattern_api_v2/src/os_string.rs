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
                    $haystack_to_cursors:expr) => {
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
                pattern_methods!(OsStrSearcher<'a, 'b>,
                                |s: &'b OsStr| OrdSlicePattern(unsafe {
                                    mem::transmute::<&'b OsStr, &'b [u8]>(s)
                                }),
                                OsStrSearcher,
                                $slice);
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
});

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
});
