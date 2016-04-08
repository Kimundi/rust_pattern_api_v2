fn ptr_range_len<T>(a: *const T, b: *const T) -> usize {
    (b as usize - a as usize) / ::std::mem::size_of::<T>()
}

#[derive(Copy, Clone)]
pub struct Elem<T>(pub T);

trait ElemEq<T> {
    fn matches(&mut self, &T) -> bool;
}

impl<T: Eq> ElemEq<T> for Elem<T> {
    #[inline]
    fn matches(&mut self, c: &T) -> bool { self.0 == *c }
}

impl<F, T> ElemEq<T> for F where F: FnMut(&T) -> bool {
    #[inline]
    fn matches(&mut self, c: &T) -> bool { (*self)(c) }
}

struct ElemEqPattern<P>(P);

macro_rules! impl_both_mutability {
    ($module:ident, $slice:ty,
                    $cursor:ty,
                    $cursor_elem:ty,
                    $raw_to_safe:expr,
                    $cursors_to_haystack:expr,
                    $haystack_to_cursors:expr) => {
        pub mod $module {
            use core_traits::*;
            use super::ptr_range_len;

            #[derive(Copy, Clone)]
            struct Iter<'a, T: 'a> {
                haystack: ($cursor, $cursor),
                start: $cursor,
                end: $cursor,
                _marker: ::std::marker::PhantomData<$slice>
            }

            impl<'a, T> Iter<'a, T> {
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
                            let b = $raw_to_safe(self.start);
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
                            let b = $raw_to_safe(self.end);
                            return Some(b);
                        }
                    }
                    None
                }
            }

            impl<'a, T> PatternHaystack for $slice {
                type Haystack = ($cursor, $cursor);
                type Cursor = $cursor;
                type MatchType = $slice;

                fn into_haystack(self) -> Self::Haystack {
                    $haystack_to_cursors(self)
                }

                fn offset_from_front(haystack: Self::Haystack,
                                     begin: Self::Cursor) -> usize {
                    ptr_range_len(haystack.0, begin)
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

            unsafe impl<'a, T> InverseMatchesAreValid for $slice {}

            //////////////////////////////////////////////////////////////////
            // Impl for a ElemEq wrapper
            //////////////////////////////////////////////////////////////////


            use super::{ElemEq, ElemEqPattern};

            #[derive(Clone)]
            struct ElemEqSearcher<'a, T: 'a, C> {
                elem_eq: C,
                iter: Iter<'a, T>,
            }

            impl<'a, T, C: ElemEq<T>> Pattern<$slice> for ElemEqPattern<C> {
                type Searcher = ElemEqSearcher<'a, T, C>;

                #[inline]
                fn into_searcher(self, haystack: $slice) -> ElemEqSearcher<'a, T, C> {
                    ElemEqSearcher {
                        elem_eq: self.0,
                        iter: Iter::new(haystack),
                    }
                }

                fn is_prefix_of(mut self, haystack: $slice) -> bool {
                    haystack.iter()
                            .next()
                            .map(|c| self.0.matches(c))
                            .unwrap_or(false)
                }

                fn is_suffix_of(mut self, haystack: $slice) -> bool
                    where Self::Searcher: ReverseSearcher<$slice>
                {
                    haystack.iter()
                            .next_back()
                            .map(|c| self.0.matches(c))
                            .unwrap_or(false)
                }
            }

            unsafe impl<'a, T, C: ElemEq<T>> Searcher<$slice> for ElemEqSearcher<'a, T, C> {
                #[inline]
                fn haystack(&self) -> ($cursor, $cursor) {
                    self.iter.haystack
                }

                #[inline]
                fn next_match(&mut self) -> Option<($cursor, $cursor)> {
                    while let Some(b) = self.iter.next() {
                        if self.elem_eq.matches(b) {
                            return Some(unsafe {
                                (self.iter.start.offset(-1),
                                    self.iter.start)
                            })
                        }
                    }

                    None
                }

                #[inline]
                fn next_reject(&mut self) -> Option<($cursor, $cursor)> {
                    while let Some(b) = self.iter.next() {
                        if !self.elem_eq.matches(b) {
                            unsafe {
                                let reject_start = self.iter.start.offset(-1);
                                return Some((reject_start, self.iter.start))
                            }
                        }
                    }
                    None
                }
            }

            unsafe impl<'a, T, C: ElemEq<T>> ReverseSearcher<$slice> for ElemEqSearcher<'a, T, C> {
                #[inline]
                fn next_match_back(&mut self) -> Option<($cursor, $cursor)>  {
                    while let Some(b) = self.iter.next_back() {
                        if self.elem_eq.matches(b) {
                            return Some(unsafe {
                                (self.iter.end,
                                    self.iter.end.offset(1))
                            })
                        }
                    }
                    None
                }

                #[inline]
                fn next_reject_back(&mut self) -> Option<($cursor, $cursor)>  {
                    while let Some(b) = self.iter.next_back() {
                        if !self.elem_eq.matches(b) {
                            unsafe {
                                let reject_end = self.iter.end.offset(1);
                                return Some((self.iter.end, reject_end))
                            }
                        }
                    }
                    None
                }
            }

            impl<'a, T, C: ElemEq<T>> DoubleEndedSearcher<$slice> for ElemEqSearcher<'a, T, C> {}

            /////////////////////////////////////////////////////////////////////////////
            // Impl for Elem
            /////////////////////////////////////////////////////////////////////////////

            use super::Elem;

            /// Associated type for `<char as Pattern<&'a str>>::Searcher`.
            #[derive(Clone)]
            pub struct ElemSearcher<'a, T: 'a>(ElemEqSearcher<'a, T, Elem<T>>);

            unsafe impl<'a, T: Eq> Searcher<$slice> for ElemSearcher<'a, T> {
                searcher_methods!(forward, s, s.0, $cursor);
            }

            unsafe impl<'a, T: Eq> ReverseSearcher<$slice> for ElemSearcher<'a, T> {
                searcher_methods!(reverse, s, s.0, $cursor);
            }

            impl<'a, T: Eq> DoubleEndedSearcher<$slice> for ElemSearcher<'a, T> {}

            /// Searches for chars that are equal to a given char
            impl<'a, T: Eq> Pattern<$slice> for Elem<T> {
                pattern_methods!(ElemSearcher<'a, T>, ElemEqPattern, ElemSearcher, $slice);
            }

            /////////////////////////////////////////////////////////////////////////////
            // Impl for F: FnMut(char) -> bool
            /////////////////////////////////////////////////////////////////////////////

            /// Associated type for `<F as Pattern<&'a str>>::Searcher`.
            #[derive(Clone)]
            pub struct ElemPredicateSearcher<'a, T: 'a, F>(ElemEqSearcher<'a, T, F>)
                where F: FnMut(&T) -> bool;

            unsafe impl<'a, T, F> Searcher<$slice> for ElemPredicateSearcher<'a, T, F>
                where F: FnMut(&T) -> bool
            {
                searcher_methods!(forward, s, s.0, $cursor);
            }

            unsafe impl<'a, T, F> ReverseSearcher<$slice> for ElemPredicateSearcher<'a, T, F>
                where F: FnMut(&T) -> bool
            {
                searcher_methods!(reverse, s, s.0, $cursor);
            }

            impl<'a, T, F> DoubleEndedSearcher<$slice> for ElemPredicateSearcher<'a, T, F>
                where F: FnMut(&T) -> bool {}

            /// Searches for chars that match the given predicate
            impl<'a, T, F> Pattern<$slice> for F where F: FnMut(&T) -> bool {
                pattern_methods!(ElemPredicateSearcher<'a, T, F>, ElemEqPattern, ElemPredicateSearcher, $slice);
            }

            ////////////////////////////////////////////////////////////////////
            // Impl for &str
            ////////////////////////////////////////////////////////////////////

            use fast_sequence_search::{OrdSlice, OrdSlicePattern, OrdSeqSearcher};
            use fast_sequence_search::{NoOptimization};

            pub struct SliceSearcher<'a, 'b, T: 'a + 'b + Ord>(OrdSeqSearcher<'b, $slice>);

            impl<'a, T: Ord + 'a> OrdSlice for $slice {
                type NeedleElement = T;
                type FastSkipOptimization = NoOptimization;

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
                        $cursors_to_haystack(hs.0, hs.1)
                    }
                }

                fn pos_is_valid(_: &Self::Haystack, _: usize) -> bool {
                    true
                }

                unsafe fn cursor_at_offset(hs: Self::Haystack, offset: usize) -> Self::Cursor {
                    hs.0.offset(offset as isize)
                }
            }

            // TODO: Specialize
            /*
            use fast_sequence_search::{ByteOptimization};
            type ByteSlice<'a, T> = $slice;
            impl<'a> OrdSlice for ByteSlice<'a, u8> {
                type FastSkipOptimization = ByteOptimization;
            }
            */

            /// Non-allocating substring search.
            ///
            /// Will handle the pattern `""` as returning empty matches at each character
            /// boundary.
            impl<'a, 'b, T: Ord> Pattern<$slice> for &'b [T] {
                pattern_methods!(SliceSearcher<'a, 'b, T>,
                                OrdSlicePattern,
                                SliceSearcher,
                                $slice);
            }

            unsafe impl<'a, 'b, T: Ord> Searcher<$slice> for SliceSearcher<'a, 'b, T> {
                searcher_methods!(forward, s, s.0, $cursor);
            }

            unsafe impl<'a, 'b, T: Ord> ReverseSearcher<$slice> for SliceSearcher<'a, 'b, T> {
                searcher_methods!(reverse, s, s.0, $cursor);
            }

        }
    }
}

impl_both_mutability!(shared, &'a [T], *const T, &'a T, |ptr: *const T| {
    &*ptr
}, |start, end| {
    ::std::slice::from_raw_parts(start, ptr_range_len(start, end))
}, |haystack: &'a [T]| {
    let begin = haystack.as_ptr();
    let end = unsafe {
        begin.offset(haystack.len() as isize)
    };
    (begin, end)
});

impl_both_mutability!(mutable, &'a mut [T], *mut T, &'a mut T, |ptr: *mut T| {
    &mut *ptr
}, |start, end| {
    ::std::slice::from_raw_parts_mut(start, ptr_range_len(start, end))
}, |haystack: &'a mut [T]| {
    let begin = haystack.as_mut_ptr();
    let end = unsafe {
        begin.offset(haystack.len() as isize)
    };
    (begin, end)
});
