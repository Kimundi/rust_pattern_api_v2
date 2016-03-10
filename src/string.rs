
// TODO: This is mostly stolen from std::str
mod utf8 {
    /// Mask of the value bits of a continuation byte
    const CONT_MASK: u8 = 0b0011_1111;
    /// Value of the tag bits (tag mask is !CONT_MASK) of a continuation byte
    const TAG_CONT_U8: u8 = 0b1000_0000;

    /// Return the initial codepoint accumulator for the first byte.
    /// The first byte is special, only want bottom 5 bits for width 2, 4 bits
    /// for width 3, and 3 bits for width 4.
    #[inline]
    fn utf8_first_byte(byte: u8, width: u32) -> u32 { (byte & (0x7F >> width)) as u32 }

    /// Return the value of `ch` updated with continuation byte `byte`.
    #[inline]
    fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 { (ch << 6) | (byte & CONT_MASK) as u32 }

    /// Checks whether the byte is a UTF-8 continuation byte (i.e. starts with the
    /// bits `10`).
    #[inline]
    fn utf8_is_cont_byte(byte: u8) -> bool { (byte & !CONT_MASK) == TAG_CONT_U8 }

    #[inline]
    fn unwrap_or_0(opt: Option<u8>) -> u8 {
        match opt {
            Some(byte) => byte,
            None => 0,
        }
    }

    /// Reads the next code point out of a byte iterator (assuming a
    /// UTF-8-like encoding).
    pub fn next_code_point<F>(mut next_byte: F) -> Option<char>
        where F: FnMut() -> Option<u8>
    {
        // Decode UTF-8
        let x = match next_byte() {
            None => return None,
            Some(next_byte) if next_byte < 128 => return Some(next_byte as char),
            Some(next_byte) => next_byte,
        };

        // Multibyte case follows
        // Decode from a byte combination out of: [[[x y] z] w]
        // NOTE: Performance is sensitive to the exact formulation here
        let init = utf8_first_byte(x, 2);
        let y = unwrap_or_0(next_byte());
        let mut ch = utf8_acc_cont_byte(init, y);
        if x >= 0xE0 {
            // [[x y z] w] case
            // 5th bit in 0xE0 .. 0xEF is always clear, so `init` is still valid
            let z = unwrap_or_0(next_byte());
            let y_z = utf8_acc_cont_byte((y & CONT_MASK) as u32, z);
            ch = init << 12 | y_z;
            if x >= 0xF0 {
                // [x y z w] case
                // use only the lower 3 bits of `init`
                let w = unwrap_or_0(next_byte());
                ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, w);
            }
        }

        Some(unsafe { ::std::mem::transmute(ch) })
    }

    /// Reads the last code point out of a byte iterator (assuming a
    /// UTF-8-like encoding).
    pub fn next_code_point_reverse<F>(mut next_byte_back: F) -> Option<char>
        where F: FnMut() -> Option<u8>
    {
        // Decode UTF-8
        let w = match next_byte_back() {
            None => return None,
            Some(next_byte) if next_byte < 128 => return Some(next_byte as char),
            Some(back_byte) => back_byte,
        };

        // Multibyte case follows
        // Decode from a byte combination out of: [x [y [z w]]]
        let mut ch;
        let z = unwrap_or_0(next_byte_back());
        ch = utf8_first_byte(z, 2);
        if utf8_is_cont_byte(z) {
            let y = unwrap_or_0(next_byte_back());
            ch = utf8_first_byte(y, 3);
            if utf8_is_cont_byte(y) {
                let x = unwrap_or_0(next_byte_back());
                ch = utf8_first_byte(x, 4);
                ch = utf8_acc_cont_byte(ch, y);
            }
            ch = utf8_acc_cont_byte(ch, z);
        }
        ch = utf8_acc_cont_byte(ch, w);

        Some(unsafe { ::std::mem::transmute(ch) })
    }
}

macro_rules! pattern_methods {
    ($t:ty, $pmap:expr, $smap:expr, $slice:ty) => {
        type Searcher = $t;

        #[inline]
        fn into_searcher(self, haystack: $slice) -> $t {
            ($smap)(($pmap)(self).into_searcher(haystack))
        }

        #[inline]
        fn is_contained_in(self, haystack: $slice) -> bool {
            ($pmap)(self).is_contained_in(haystack)
        }

        #[inline]
        fn is_prefix_of(self, haystack: $slice) -> bool {
            ($pmap)(self).is_prefix_of(haystack)
        }

        #[inline]
        fn is_suffix_of(self, haystack: $slice) -> bool
            where $t: ReverseSearcher<$slice>
        {
            ($pmap)(self).is_suffix_of(haystack)
        }
    }
}

macro_rules! searcher_methods {
    (forward, $cursor:ty) => {
        #[inline]
        fn haystack(&self) -> ($cursor, $cursor) {
            self.0.haystack()
        }
        #[inline]
        fn next_match(&mut self) -> Option<($cursor, $cursor)> {
            self.0.next_match()
        }
        #[inline]
        fn next_reject(&mut self) -> Option<($cursor, $cursor)> {
            self.0.next_reject()
        }
    };
    (reverse, $cursor:ty) => {
        #[inline]
        fn next_match_back(&mut self) -> Option<($cursor, $cursor)> {
            self.0.next_match_back()
        }
        #[inline]
        fn next_reject_back(&mut self) -> Option<($cursor, $cursor)> {
            self.0.next_reject_back()
        }
    }
}

pub struct Ascii(pub u8);

trait CharEq {
    fn matches(&mut self, char) -> bool;
    fn only_ascii(&self) -> bool;
}

impl CharEq for char {
    #[inline]
    fn matches(&mut self, c: char) -> bool { *self == c }

    #[inline]
    fn only_ascii(&self) -> bool { (*self as u32) < 128 }
}

impl<F> CharEq for F where F: FnMut(char) -> bool {
    #[inline]
    fn matches(&mut self, c: char) -> bool { (*self)(c) }

    #[inline]
    fn only_ascii(&self) -> bool { false }
}

impl<'a> CharEq for &'a [char] {
    #[inline]
    fn matches(&mut self, c: char) -> bool {
        self.iter().any(|&m| { let mut m = m; m.matches(c) })
    }

    #[inline]
    fn only_ascii(&self) -> bool {
        self.iter().all(|m| m.only_ascii())
    }
}

struct CharEqPattern<C: CharEq>(C);

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

                unsafe fn offset_from_start(haystack: Self::Haystack,
                                            begin: Self::Cursor) -> usize {
                    begin as usize - haystack.0 as usize
                }

                unsafe fn range_to_self(_: Self::Haystack,
                                        start: Self::Cursor,
                                        end: Self::Cursor) -> Self {
                    ($cursors_to_haystack)(start, end)
                }
                unsafe fn cursor_at_front(hs: Self::Haystack) -> Self::Cursor {
                    hs.0
                }
                unsafe fn cursor_at_back(hs: Self::Haystack) -> Self::Cursor {
                    hs.1
                }
            }

            use super::Ascii;

            pub struct AsciiSearcher<'a> {
                iter: Iter<'a>,
                ascii: u8,
            }

            unsafe impl<'a> Searcher<$slice> for AsciiSearcher<'a> {
                fn haystack(&self) -> ($cursor, $cursor) {
                    self.iter.haystack
                }

                fn next_match(&mut self) -> Option<($cursor, $cursor)> {
                    while self.iter.start != self.iter.end {
                        unsafe {
                            let p = self.iter.start;
                            self.iter.start = self.iter.start.offset(1);

                            if *p == self.ascii {
                                return Some((p, self.iter.start));
                            }
                        }
                    }
                    None
                }

                fn next_reject(&mut self) -> Option<($cursor, $cursor)> {
                    while self.iter.start != self.iter.end {
                        unsafe {
                            let p = self.iter.start;
                            self.iter.start = self.iter.start.offset(1);

                            if *p != self.ascii {
                                return Some((p, self.iter.start));
                            }
                        }
                    }
                    None
                }
            }

            impl<'a> Pattern<$slice> for Ascii {
                type Searcher = AsciiSearcher<'a>;

                fn into_searcher(self, haystack: $slice) -> Self::Searcher {
                    assert!(self.0 < 128, "Ascii is only defined on byte values 0 - 127");

                    AsciiSearcher {
                        iter: Iter::new(haystack),
                        ascii: self.0,
                    }
                }

                fn is_prefix_of(self, haystack: $slice) -> bool {
                    haystack.bytes().next() == Some(self.0)
                }

                fn is_suffix_of(self, haystack: $slice) -> bool
                    where Self::Searcher: ReverseSearcher<$slice>
                {
                    haystack.bytes().next_back() == Some(self.0)
                }
            }

            //////////////////////////////////////////////////////////////////
            // Impl for a CharEq wrapper
            //////////////////////////////////////////////////////////////////


            use super::{CharEq, CharEqPattern};

            #[derive(Clone)]
            struct CharEqSearcher<'a, C: CharEq> {
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

            use super::utf8;

            unsafe impl<'a, C: CharEq> Searcher<$slice> for CharEqSearcher<'a, C> {
                #[inline]
                fn haystack(&self) -> ($cursor, $cursor) {
                    self.iter.haystack
                }

                #[inline]
                fn next_match(&mut self) -> Option<($cursor, $cursor)> {
                    if self.ascii_only {
                        while let Some(b) = self.iter.next() {
                            if self.char_eq.matches(b as char) {
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
                            if !self.char_eq.matches(b as char) {
                                return Some(unsafe {
                                    (self.iter.start.offset(-1),
                                     self.iter.start)
                                })
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
                        while let Some(b) = self.iter.next() {
                            if self.char_eq.matches(b as char) {
                                return Some(unsafe {
                                    (self.iter.end,
                                     self.iter.end.offset(1))
                                })
                            }
                        }
                    } else {
                        while let Some(c) = utf8::next_code_point_reverse(|| self.iter.next()) {
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
                        while let Some(b) = self.iter.next() {
                            if !self.char_eq.matches(b as char) {
                                return Some(unsafe {
                                    (self.iter.end,
                                     self.iter.end.offset(1))
                                })
                            }
                        }
                    } else {
                        while let Some(c) = utf8::next_code_point_reverse(|| self.iter.next()) {
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

            /// Associated type for `<char as Pattern<'a>>::Searcher`.
            #[derive(Clone)]
            pub struct CharSearcher<'a>(CharEqSearcher<'a, char>);

            unsafe impl<'a> Searcher<$slice> for CharSearcher<'a> {
                searcher_methods!(forward, $cursor);
            }

            unsafe impl<'a> ReverseSearcher<$slice> for CharSearcher<'a> {
                searcher_methods!(reverse, $cursor);
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

            /// Associated type for `<&[char] as Pattern<'a>>::Searcher`.
            #[derive(Clone)]
            pub struct CharSliceSearcher<'a, 'b>(CharEqSearcher<'a, &'b [char]>);

            unsafe impl<'a, 'b> Searcher<$slice> for CharSliceSearcher<'a, 'b> {
                searcher_methods!(forward, $cursor);
            }

            unsafe impl<'a, 'b> ReverseSearcher<$slice> for CharSliceSearcher<'a, 'b> {
                searcher_methods!(reverse, $cursor);
            }

            impl<'a, 'b> DoubleEndedSearcher<$slice> for CharSliceSearcher<'a, 'b> {}

            /// Searches for chars that are equal to any of the chars in the array
            impl<'a, 'b> Pattern<$slice> for &'b [char] {
                pattern_methods!(CharSliceSearcher<'a, 'b>, CharEqPattern, CharSliceSearcher, $slice);
            }

            /////////////////////////////////////////////////////////////////////////////
            // Impl for F: FnMut(char) -> bool
            /////////////////////////////////////////////////////////////////////////////

            /// Associated type for `<F as Pattern<'a>>::Searcher`.
            #[derive(Clone)]
            pub struct CharPredicateSearcher<'a, F>(CharEqSearcher<'a, F>)
                where F: FnMut(char) -> bool;

            unsafe impl<'a, F> Searcher<$slice> for CharPredicateSearcher<'a, F>
                where F: FnMut(char) -> bool
            {
                searcher_methods!(forward, $cursor);
            }

            unsafe impl<'a, F> ReverseSearcher<$slice> for CharPredicateSearcher<'a, F>
                where F: FnMut(char) -> bool
            {
                searcher_methods!(reverse, $cursor);
            }

            impl<'a, F> DoubleEndedSearcher<$slice> for CharPredicateSearcher<'a, F>
                where F: FnMut(char) -> bool {}

            /// Searches for chars that match the given predicate
            impl<'a, F> Pattern<$slice> for F where F: FnMut(char) -> bool {
                pattern_methods!(CharPredicateSearcher<'a, F>, CharEqPattern, CharPredicateSearcher, $slice);
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
    ::std::mem::transmute(slice)
}, |haystack: &mut str| {
    let begin = haystack.as_ptr() as *mut u8;
    let end = unsafe {
        begin.offset(haystack.len() as isize)
    };
    (begin, end)
});


