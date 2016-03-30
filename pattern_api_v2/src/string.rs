macro_rules! impl_both_mutability {
    ($module:ident, $slice:ty,
                    $cursor:ty,
                    $cursor_elem:ty,
                    $cursors_to_haystack:expr,
                    $haystack_to_cursors:expr) => {
        pub mod $module {
            use core_traits::*;

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
            }

            impl<'a> SearchCursors for $slice {
                type Haystack = ($cursor, $cursor);
                type Cursor = $cursor;

                fn into_haystack(self) -> Self::Haystack {
                    $haystack_to_cursors(self)
                }

                fn offset_from_front(haystack: Self::Haystack,
                                     begin: Self::Cursor) -> usize {
                    begin as usize - haystack.0 as usize
                }

                unsafe fn range_to_self(_: Self::Haystack,
                                        start: Self::Cursor,
                                        end: Self::Cursor) -> Self {
                    ($cursors_to_haystack)(start, end)
                }
                fn cursor_at_front(hs: Self::Haystack) -> Self::Cursor {
                    hs.0
                }
                fn cursor_at_back(hs: Self::Haystack) -> Self::Cursor {
                    hs.1
                }
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

                fn is_prefix_of(mut self, haystack: $slice) -> bool {
                    haystack.chars()
                            .next()
                            .map(|c| self.0.matches(c))
                            .unwrap_or(false)
                }

                fn is_suffix_of(mut self, haystack: $slice) -> bool
                    where Self::Searcher: ReverseSearcher<$slice>
                {
                    haystack.chars()
                            .next_back()
                            .map(|c| self.0.matches(c))
                            .unwrap_or(false)
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
                        while let Some(b) = self.iter.next() {
                            if b < 128 && self.char_eq.matches(b as char) {
                                return Some(unsafe {
                                    (self.iter.start.offset(-1),
                                     self.iter.start)
                                })
                            }
                        }
                    } else {
                        while let Some(c) = utf8::next_code_point(|| self.iter.next()) {
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
                        while let Some(b) = self.iter.next() {
                            if b > 127 || !self.char_eq.matches(b as char) {
                                unsafe {
                                    let reject_start = self.iter.start.offset(-1);
                                    while !utf8::byte_is_char_boundary(
                                            *self.iter.start) {
                                        self.iter.start = self.iter.start.offset(1);
                                    }
                                    return Some((reject_start, self.iter.start))
                                }
                            }
                        }
                    } else {
                        while let Some(c) = utf8::next_code_point(|| self.iter.next()) {
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
                        while let Some(b) = self.iter.next_back() {
                            if b < 128 && self.char_eq.matches(b as char) {
                                return Some(unsafe {
                                    (self.iter.end,
                                     self.iter.end.offset(1))
                                })
                            }
                        }
                    } else {
                        while let Some(c) = utf8::next_code_point_reverse(|| self.iter.next_back()) {
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
                        while let Some(b) = self.iter.next_back() {
                            if b > 127 || !self.char_eq.matches(b as char) {
                                unsafe {
                                    let reject_end = self.iter.end.offset(1);
                                    while !utf8::byte_is_char_boundary(
                                            *self.iter.end) {
                                        self.iter.end = self.iter.end.offset(-1);
                                    }
                                    return Some((self.iter.end, reject_end))
                                }
                            }
                        }
                    } else {
                        while let Some(c) = utf8::next_code_point_reverse(|| self.iter.next_back()) {
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
            // Impl for &str
            ////////////////////////////////////////////////////////////////////

            use fast_sequence_search::{OrdSlice, OrdSlicePattern, OrdSeqSearcher};
            use fast_sequence_search::ByteOptimization;

            pub struct StrSearcher<'a, 'b>(OrdSeqSearcher<'b, $slice>);

            impl<'a> OrdSlice for $slice {
                type NeedleElement = u8;
                type FastSkipOptimization = ByteOptimization;

                fn next_valid_pos(hs: &Self::Haystack, pos: usize) -> Option<usize> {
                    let s = unsafe {
                        ::std::str::from_utf8_unchecked(Self::haystack_as_slice(hs))
                    };
                    s[pos..].chars().next().map(|c| pos + c.len_utf8())
                }

                fn next_valid_pos_back(hs: &Self::Haystack, pos: usize) -> Option<usize> {
                    let s = unsafe {
                        ::std::str::from_utf8_unchecked(Self::haystack_as_slice(hs))
                    };
                    s[..pos].chars().next_back().map(|c| pos - c.len_utf8())
                }

                fn haystack_as_slice<'t>(hs: &'t Self::Haystack) -> &'t [Self::NeedleElement] {
                    unsafe {
                        ::std::slice::from_raw_parts(hs.0, hs.1 as usize - hs.0 as usize)
                    }
                }

                fn pos_is_valid(hs: &Self::Haystack, pos: usize) -> bool {
                    let s = unsafe {
                        ::std::str::from_utf8_unchecked(Self::haystack_as_slice(hs))
                    };
                    s.is_char_boundary(pos)
                }

                unsafe fn cursor_at_offset(hs: Self::Haystack, offset: usize) -> Self::Cursor {
                    hs.0.offset(offset as isize)
                }
            }

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

impl_both_mutability!(shared, &'a str, *const u8, u8, |start, end| {
    let slice = ::std::slice::from_raw_parts(start,
        end as usize - start as usize);
    ::std::str::from_utf8_unchecked(slice)
}, |haystack: &str| {
    let begin = haystack.as_ptr();
    let end = unsafe {
        begin.offset(haystack.len() as isize)
    };
    (begin, end)
});

impl_both_mutability!(mutable, &'a mut str, *mut u8, u8, |start, end| {
    let slice = ::std::slice::from_raw_parts_mut(start,
        end as usize - start as usize);

    // TODO: This should probably be just library support in std
    ::std::mem::transmute::<&mut [u8], &mut str>(slice)
}, |haystack: &mut str| {
    let begin = haystack.as_ptr() as *mut u8;
    let end = unsafe {
        begin.offset(haystack.len() as isize)
    };
    (begin, end)
});

use ::Pattern;
use ::SearchCursors;
use ::ReverseSearcher;

/*impl<'b, H, P> Pattern<H> for &'b P
    where P: Pattern<H>,
          P: Copy,
          H: SearchCursors,
{}*/

impl<'a, 'b, 'c> Pattern<&'a str> for &'c &'b str {
    pattern_methods!(shared::StrSearcher<'a, 'b>, |&s| s, |s| s, &'a str);
}
