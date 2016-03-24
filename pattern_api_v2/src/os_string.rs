/*
    notes to existing osstring RFC:

    - replace() should work with pattern due to consistency with string,
      or have another name
    -

*/

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

            ////////////////////////////////////////////////////////////////////
            // Impl for &OsStr
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
            impl<'a, 'b> Pattern<$slice> for &'b OsStr {
                pattern_methods!(StrSearcher<'a, 'b>,
                                |s: &'b OsStr| OrdSlicePattern(unsafe {
                                    mem::transmute::<&'b OsStr, &'b [u8]>(s)
                                }),
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
