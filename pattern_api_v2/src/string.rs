use super::*;

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
                        while let Some(b) = self.iter.next_back() {
                            if self.char_eq.matches(b as char) {
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
                            if !self.char_eq.matches(b as char) {
                                return Some(unsafe {
                                    (self.iter.end,
                                     self.iter.end.offset(1))
                                })
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

            /// Associated type for `<&[char] as Pattern<&'a str>>::Searcher`.
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

            /// Associated type for `<F as Pattern<&'a str>>::Searcher`.
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
    ::std::mem::transmute::<&mut [u8], &mut str>(slice)
}, |haystack: &mut str| {
    let begin = haystack.as_ptr() as *mut u8;
    let end = unsafe {
        begin.offset(haystack.len() as isize)
    };
    (begin, end)
});

/////////////////////////////////////////////////////////////////////////////
// Impl for &str
/////////////////////////////////////////////////////////////////////////////

use std::cmp;
use std::usize;

// Only temporary used to make the old code work without major changes
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum SearchStep {
    Match(usize, usize),
    Reject(usize, usize),
    Done
}

/// Non-allocating substring search.
///
/// Will handle the pattern `""` as returning empty matches at each character
/// boundary.
impl<'a, 'b> Pattern<&'a str> for &'b str {
    type Searcher = StrSearcher<'a, 'b>;

    #[inline]
    fn into_searcher(self, haystack: &'a str) -> StrSearcher<'a, 'b> {
        StrSearcher::new(haystack, self)
    }

    /// Checks whether the pattern matches at the front of the haystack
    #[inline]
    fn is_prefix_of(self, haystack: &'a str) -> bool {
        haystack.is_char_boundary(self.len()) &&
            self == &haystack[..self.len()]
    }

    /// Checks whether the pattern matches at the back of the haystack
    #[inline]
    fn is_suffix_of(self, haystack: &'a str) -> bool {
        self.len() <= haystack.len() &&
            haystack.is_char_boundary(haystack.len() - self.len()) &&
            self == &haystack[haystack.len() - self.len()..]
    }
}


/////////////////////////////////////////////////////////////////////////////
// Two Way substring searcher
/////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
/// Associated type for `<&str as Pattern<&'a str>>::Searcher`.
pub struct StrSearcher<'a, 'b> {
    haystack: &'a str,
    needle: &'b str,

    searcher: StrSearcherImpl,
}

#[derive(Clone, Debug)]
enum StrSearcherImpl {
    Empty(EmptyNeedle),
    TwoWay(TwoWaySearcher),
}

#[derive(Clone, Debug)]
struct EmptyNeedle {
    position: usize,
    end: usize,
    is_match_fw: bool,
    is_match_bw: bool,
}

impl<'a, 'b> StrSearcher<'a, 'b> {
    fn new(haystack: &'a str, needle: &'b str) -> StrSearcher<'a, 'b> {
        if needle.is_empty() {
            StrSearcher {
                haystack: haystack,
                needle: needle,
                searcher: StrSearcherImpl::Empty(EmptyNeedle {
                    position: 0,
                    end: haystack.len(),
                    is_match_fw: true,
                    is_match_bw: true,
                }),
            }
        } else {
            StrSearcher {
                haystack: haystack,
                needle: needle,
                searcher: StrSearcherImpl::TwoWay(
                    TwoWaySearcher::new(needle.as_bytes(), haystack.len())
                ),
            }
        }
    }
}

impl<'a, 'b> StrSearcher<'a, 'b> {
    #[inline]
    fn next(&mut self) -> SearchStep {
        match self.searcher {
            StrSearcherImpl::Empty(ref mut searcher) => {
                // empty needle rejects every char and matches every empty string between them
                let is_match = searcher.is_match_fw;
                searcher.is_match_fw = !searcher.is_match_fw;
                let pos = searcher.position;
                match self.haystack[pos..].chars().next() {
                    _ if is_match => SearchStep::Match(pos, pos),
                    None => SearchStep::Done,
                    Some(ch) => {
                        searcher.position += ch.len_utf8();
                        SearchStep::Reject(pos, searcher.position)
                    }
                }
            }
            StrSearcherImpl::TwoWay(ref mut searcher) => {
                // TwoWaySearcher produces valid *Match* indices that split at char boundaries
                // as long as it does correct matching and that haystack and needle are
                // valid UTF-8
                // *Rejects* from the algorithm can fall on any indices, but we will walk them
                // manually to the next character boundary, so that they are utf-8 safe.
                if searcher.position == self.haystack.len() {
                    return SearchStep::Done;
                }
                let is_long = searcher.memory == usize::MAX;
                match searcher.next::<RejectAndMatch>(self.haystack.as_bytes(),
                                                      self.needle.as_bytes(),
                                                      is_long)
                {
                    SearchStep::Reject(a, mut b) => {
                        // skip to next char boundary
                        while !self.haystack.is_char_boundary(b) {
                            b += 1;
                        }
                        searcher.position = cmp::max(b, searcher.position);
                        SearchStep::Reject(a, b)
                    }
                    otherwise => otherwise,
                }
            }
        }
    }

    #[inline]
    fn next_back(&mut self) -> SearchStep {
        match self.searcher {
            StrSearcherImpl::Empty(ref mut searcher) => {
                let is_match = searcher.is_match_bw;
                searcher.is_match_bw = !searcher.is_match_bw;
                let end = searcher.end;
                match self.haystack[..end].chars().next_back() {
                    _ if is_match => SearchStep::Match(end, end),
                    None => SearchStep::Done,
                    Some(ch) => {
                        searcher.end -= ch.len_utf8();
                        SearchStep::Reject(searcher.end, end)
                    }
                }
            }
            StrSearcherImpl::TwoWay(ref mut searcher) => {
                if searcher.end == 0 {
                    return SearchStep::Done;
                }
                let is_long = searcher.memory == usize::MAX;
                match searcher.next_back::<RejectAndMatch>(self.haystack.as_bytes(),
                                                           self.needle.as_bytes(),
                                                           is_long)
                {
                    SearchStep::Reject(mut a, b) => {
                        // skip to next char boundary
                        while !self.haystack.is_char_boundary(a) {
                            a -= 1;
                        }
                        searcher.end = cmp::min(a, searcher.end);
                        SearchStep::Reject(a, b)
                    }
                    otherwise => otherwise,
                }
            }
        }
    }
}

unsafe impl<'a, 'b> Searcher<&'a str> for StrSearcher<'a, 'b> {
    fn haystack(&self) -> (*const u8, *const u8) {
        let p = self.haystack.as_ptr();
        unsafe {
            (p, p.offset(self.haystack.len() as isize))
        }
    }

    #[inline(always)]
    fn next_match(&mut self) -> Option<(*const u8, *const u8)> {
        (|| match self.searcher {
            StrSearcherImpl::Empty(..) => {
                loop {
                    match self.next() {
                        SearchStep::Match(a, b) => return Some((a, b)),
                        SearchStep::Done => return None,
                        SearchStep::Reject(..) => { }
                    }
                }
            }
            StrSearcherImpl::TwoWay(ref mut searcher) => {
                let is_long = searcher.memory == usize::MAX;
                // write out `true` and `false` cases to encourage the compiler
                // to specialize the two cases separately.
                if is_long {
                    searcher.next::<MatchOnly>(self.haystack.as_bytes(),
                                               self.needle.as_bytes(),
                                               true)
                } else {
                    searcher.next::<MatchOnly>(self.haystack.as_bytes(),
                                               self.needle.as_bytes(),
                                               false)
                }
            }
        })().map(|(a, b)| unsafe {
            let p = self.haystack().0;
            (p.offset(a as isize), p.offset(b as isize))
        })
    }

    #[inline(always)]
    fn next_reject(&mut self) -> Option<(*const u8, *const u8)> {
        (|| loop {
            match self.next() {
                SearchStep::Reject(a, b) => return Some((a, b)),
                SearchStep::Done => return None,
                _ => (),
            }
        })().map(|(a, b)| unsafe {
            let p = self.haystack().0;
            (p.offset(a as isize), p.offset(b as isize))
        })
    }
}

unsafe impl<'a, 'b> ReverseSearcher<&'a str> for StrSearcher<'a, 'b> {
    #[inline]
    fn next_match_back(&mut self) -> Option<(*const u8, *const u8)> {
        (|| match self.searcher {
            StrSearcherImpl::Empty(..) => {
                loop {
                    match self.next_back() {
                        SearchStep::Match(a, b) => return Some((a, b)),
                        SearchStep::Done => return None,
                        SearchStep::Reject(..) => { }
                    }
                }
            }
            StrSearcherImpl::TwoWay(ref mut searcher) => {
                let is_long = searcher.memory == usize::MAX;
                // write out `true` and `false`, like `next_match`
                if is_long {
                    searcher.next_back::<MatchOnly>(self.haystack.as_bytes(),
                                                    self.needle.as_bytes(),
                                                    true)
                } else {
                    searcher.next_back::<MatchOnly>(self.haystack.as_bytes(),
                                                    self.needle.as_bytes(),
                                                    false)
                }
            }
        })().map(|(a, b)| unsafe {
            let p = self.haystack().0;
            (p.offset(a as isize), p.offset(b as isize))
        })
    }

    #[inline(always)]
    fn next_reject_back(&mut self) -> Option<(*const u8, *const u8)> {
        (|| loop {
            match self.next_back() {
                SearchStep::Reject(a, b) => return Some((a, b)),
                SearchStep::Done => return None,
                _ => (),
            }
        })().map(|(a, b)| unsafe {
            let p = self.haystack().0;
            (p.offset(a as isize), p.offset(b as isize))
        })
    }

}

/// The internal state of the two-way substring search algorithm.
#[derive(Clone, Debug)]
struct TwoWaySearcher {
    // constants
    /// critical factorization index
    crit_pos: usize,
    /// critical factorization index for reversed needle
    crit_pos_back: usize,
    period: usize,
    // TODO #1: Re-add with specialization for [u8] and str cases
    // /// `byteset` is an extension (not part of the two way algorithm);
    // /// it's a 64-bit "fingerprint" where each set bit `j` corresponds
    // /// to a (byte & 63) == j present in the needle.
    // byteset: u64,

    // variables
    position: usize,
    end: usize,
    /// index into needle before which we have already matched
    memory: usize,
    /// index into needle after which we have already matched
    memory_back: usize,
}

/*
    This is the Two-Way search algorithm, which was introduced in the paper:
    Crochemore, M., Perrin, D., 1991, Two-way string-matching, Journal of the ACM 38(3):651-675.

    Here's some background information.

    A *word* is a string of symbols. The *length* of a word should be a familiar
    notion, and here we denote it for any word x by |x|.
    (We also allow for the possibility of the *empty word*, a word of length zero).

    If x is any non-empty word, then an integer p with 0 < p <= |x| is said to be a
    *period* for x iff for all i with 0 <= i <= |x| - p - 1, we have x[i] == x[i+p].
    For example, both 1 and 2 are periods for the string "aa". As another example,
    the only period of the string "abcd" is 4.

    We denote by period(x) the *smallest* period of x (provided that x is non-empty).
    This is always well-defined since every non-empty word x has at least one period,
    |x|. We sometimes call this *the period* of x.

    If u, v and x are words such that x = uv, where uv is the concatenation of u and
    v, then we say that (u, v) is a *factorization* of x.

    Let (u, v) be a factorization for a word x. Then if w is a non-empty word such
    that both of the following hold

      - either w is a suffix of u or u is a suffix of w
      - either w is a prefix of v or v is a prefix of w

    then w is said to be a *repetition* for the factorization (u, v).

    Just to unpack this, there are four possibilities here. Let w = "abc". Then we
    might have:

      - w is a suffix of u and w is a prefix of v. ex: ("lolabc", "abcde")
      - w is a suffix of u and v is a prefix of w. ex: ("lolabc", "ab")
      - u is a suffix of w and w is a prefix of v. ex: ("bc", "abchi")
      - u is a suffix of w and v is a prefix of w. ex: ("bc", "a")

    Note that the word vu is a repetition for any factorization (u,v) of x = uv,
    so every factorization has at least one repetition.

    If x is a string and (u, v) is a factorization for x, then a *local period* for
    (u, v) is an integer r such that there is some word w such that |w| = r and w is
    a repetition for (u, v).

    We denote by local_period(u, v) the smallest local period of (u, v). We sometimes
    call this *the local period* of (u, v). Provided that x = uv is non-empty, this
    is well-defined (because each non-empty word has at least one factorization, as
    noted above).

    It can be proven that the following is an equivalent definition of a local period
    for a factorization (u, v): any positive integer r such that x[i] == x[i+r] for
    all i such that |u| - r <= i <= |u| - 1 and such that both x[i] and x[i+r] are
    defined. (i.e. i > 0 and i + r < |x|).

    Using the above reformulation, it is easy to prove that

        1 <= local_period(u, v) <= period(uv)

    A factorization (u, v) of x such that local_period(u,v) = period(x) is called a
    *critical factorization*.

    The algorithm hinges on the following theorem, which is stated without proof:

    **Critical Factorization Theorem** Any word x has at least one critical
    factorization (u, v) such that |u| < period(x).

    The purpose of maximal_suffix is to find such a critical factorization.

    If the period is short, compute another factorization x = u' v' to use
    for reverse search, chosen instead so that |v'| < period(x).

*/
impl TwoWaySearcher {
    fn new<T: Ord>(needle: &[T], end: usize) -> TwoWaySearcher {
        let (crit_pos_false, period_false) = TwoWaySearcher::maximal_suffix(needle, false);
        let (crit_pos_true, period_true) = TwoWaySearcher::maximal_suffix(needle, true);

        let (crit_pos, period) =
            if crit_pos_false > crit_pos_true {
                (crit_pos_false, period_false)
            } else {
                (crit_pos_true, period_true)
            };

        // A particularly readable explanation of what's going on here can be found
        // in Crochemore and Rytter's book "Text Algorithms", ch 13. Specifically
        // see the code for "Algorithm CP" on p. 323.
        //
        // What's going on is we have some critical factorization (u, v) of the
        // needle, and we want to determine whether u is a suffix of
        // &v[..period]. If it is, we use "Algorithm CP1". Otherwise we use
        // "Algorithm CP2", which is optimized for when the period of the needle
        // is large.
        if &needle[..crit_pos] == &needle[period.. period + crit_pos] {
            // short period case -- the period is exact
            // compute a separate critical factorization for the reversed needle
            // x = u' v' where |v'| < period(x).
            //
            // This is sped up by the period being known already.
            // Note that a case like x = "acba" may be factored exactly forwards
            // (crit_pos = 1, period = 3) while being factored with approximate
            // period in reverse (crit_pos = 2, period = 2). We use the given
            // reverse factorization but keep the exact period.
            let crit_pos_back = needle.len() - cmp::max(
                TwoWaySearcher::reverse_maximal_suffix(needle, period, false),
                TwoWaySearcher::reverse_maximal_suffix(needle, period, true));

            TwoWaySearcher {
                crit_pos: crit_pos,
                crit_pos_back: crit_pos_back,
                period: period,
                // TODO #1: Re-add with specialization for [u8] and str cases
                // byteset: Self::byteset_create(&needle[..period]),

                position: 0,
                end: end,
                memory: 0,
                memory_back: needle.len(),
            }
        } else {
            // long period case -- we have an approximation to the actual period,
            // and don't use memorization.
            //
            // Approximate the period by lower bound max(|u|, |v|) + 1.
            // The critical factorization is efficient to use for both forward and
            // reverse search.

            TwoWaySearcher {
                crit_pos: crit_pos,
                crit_pos_back: crit_pos,
                period: cmp::max(crit_pos, needle.len() - crit_pos) + 1,
                // TODO #1: Re-add with specialization for [u8] and str cases
                // byteset: Self::byteset_create(needle),

                position: 0,
                end: end,
                memory: usize::MAX, // Dummy value to signify that the period is long
                memory_back: usize::MAX,
            }
        }
    }

    // TODO #1: Re-add with specialization for [u8] and str cases
    /*
    #[inline]
    fn byteset_create(bytes: &[u8]) -> u64 {
        bytes.iter().fold(0, |a, &b| (1 << (b & 0x3f)) | a)
    }

    #[inline(always)]
    fn byteset_contains(&self, byte: u8) -> bool {
        (self.byteset >> ((byte & 0x3f) as usize)) & 1 != 0
    }
    */

    // One of the main ideas of Two-Way is that we factorize the needle into
    // two halves, (u, v), and begin trying to find v in the haystack by scanning
    // left to right. If v matches, we try to match u by scanning right to left.
    // How far we can jump when we encounter a mismatch is all based on the fact
    // that (u, v) is a critical factorization for the needle.
    #[inline(always)]
    fn next<S>(&mut self, haystack: &[u8], needle: &[u8], long_period: bool)
        -> S::Output
        where S: TwoWayStrategy
    {
        // `next()` uses `self.position` as its cursor
        let old_pos = self.position;
        let needle_last = needle.len() - 1;
        'search: loop {
            // Check that we have room to search in
            // position + needle_last can not overflow if we assume slices
            // are bounded by isize's range.
            let _tail_byte = match haystack.get(self.position + needle_last) {
                Some(&b) => b,
                None => {
                    self.position = haystack.len();
                    return S::rejecting(old_pos, self.position);
                }
            };

            if S::use_early_reject() && old_pos != self.position {
                return S::rejecting(old_pos, self.position);
            }

            // TODO #1: Re-add with specialization for [u8] and str cases
            /*
            // Quickly skip by large portions unrelated to our substring
            if !self.byteset_contains(tail_byte) {
                self.position += needle.len();
                if !long_period {
                    self.memory = 0;
                }
                continue 'search;
            }
            */

            // See if the right part of the needle matches
            let start = if long_period { self.crit_pos }
                        else { cmp::max(self.crit_pos, self.memory) };
            for i in start..needle.len() {
                if needle[i] != haystack[self.position + i] {
                    self.position += i - self.crit_pos + 1;
                    if !long_period {
                        self.memory = 0;
                    }
                    continue 'search;
                }
            }

            // See if the left part of the needle matches
            let start = if long_period { 0 } else { self.memory };
            for i in (start..self.crit_pos).rev() {
                if needle[i] != haystack[self.position + i] {
                    self.position += self.period;
                    if !long_period {
                        self.memory = needle.len() - self.period;
                    }
                    continue 'search;
                }
            }

            // We have found a match!
            let match_pos = self.position;

            // Note: add self.period instead of needle.len() to have overlapping matches
            self.position += needle.len();
            if !long_period {
                self.memory = 0; // set to needle.len() - self.period for overlapping matches
            }

            return S::matching(match_pos, match_pos + needle.len());
        }
    }

    // Follows the ideas in `next()`.
    //
    // The definitions are symmetrical, with period(x) = period(reverse(x))
    // and local_period(u, v) = local_period(reverse(v), reverse(u)), so if (u, v)
    // is a critical factorization, so is (reverse(v), reverse(u)).
    //
    // For the reverse case we have computed a critical factorization x = u' v'
    // (field `crit_pos_back`). We need |u| < period(x) for the forward case and
    // thus |v'| < period(x) for the reverse.
    //
    // To search in reverse through the haystack, we search forward through
    // a reversed haystack with a reversed needle, matching first u' and then v'.
    #[inline]
    fn next_back<S>(&mut self, haystack: &[u8], needle: &[u8], long_period: bool)
        -> S::Output
        where S: TwoWayStrategy
    {
        // `next_back()` uses `self.end` as its cursor -- so that `next()` and `next_back()`
        // are independent.
        let old_end = self.end;
        'search: loop {
            // Check that we have room to search in
            // end - needle.len() will wrap around when there is no more room,
            // but due to slice length limits it can never wrap all the way back
            // into the length of haystack.
            let _front_byte = match haystack.get(self.end.wrapping_sub(needle.len())) {
                Some(&b) => b,
                None => {
                    self.end = 0;
                    return S::rejecting(0, old_end);
                }
            };

            if S::use_early_reject() && old_end != self.end {
                return S::rejecting(self.end, old_end);
            }

            // TODO #1: Re-add with specialization for [u8] and str cases
            /*
            // Quickly skip by large portions unrelated to our substring
            if !self.byteset_contains(front_byte) {
                self.end -= needle.len();
                if !long_period {
                    self.memory_back = needle.len();
                }
                continue 'search;
            }
            */

            // See if the left part of the needle matches
            let crit = if long_period { self.crit_pos_back }
                       else { cmp::min(self.crit_pos_back, self.memory_back) };
            for i in (0..crit).rev() {
                if needle[i] != haystack[self.end - needle.len() + i] {
                    self.end -= self.crit_pos_back - i;
                    if !long_period {
                        self.memory_back = needle.len();
                    }
                    continue 'search;
                }
            }

            // See if the right part of the needle matches
            let needle_end = if long_period { needle.len() }
                             else { self.memory_back };
            for i in self.crit_pos_back..needle_end {
                if needle[i] != haystack[self.end - needle.len() + i] {
                    self.end -= self.period;
                    if !long_period {
                        self.memory_back = self.period;
                    }
                    continue 'search;
                }
            }

            // We have found a match!
            let match_pos = self.end - needle.len();
            // Note: sub self.period instead of needle.len() to have overlapping matches
            self.end -= needle.len();
            if !long_period {
                self.memory_back = needle.len();
            }

            return S::matching(match_pos, match_pos + needle.len());
        }
    }

    // Compute the maximal suffix of `arr`.
    //
    // The maximal suffix is a possible critical factorization (u, v) of `arr`.
    //
    // Returns (`i`, `p`) where `i` is the starting index of v and `p` is the
    // period of v.
    //
    // `order_greater` determines if lexical order is `<` or `>`. Both
    // orders must be computed -- the ordering with the largest `i` gives
    // a critical factorization.
    //
    // For long period cases, the resulting period is not exact (it is too short).
    #[inline]
    fn maximal_suffix<T: Ord>(arr: &[T], order_greater: bool) -> (usize, usize) {
        let mut left = 0; // Corresponds to i in the paper
        let mut right = 1; // Corresponds to j in the paper
        let mut offset = 0; // Corresponds to k in the paper, but starting at 0
                            // to match 0-based indexing.
        let mut period = 1; // Corresponds to p in the paper

        while let Some(a) = arr.get(right + offset) {
            // `left` will be inbounds when `right` is.
            let b = &arr[left + offset];
            if (a < b && !order_greater) || (a > b && order_greater) {
                // Suffix is smaller, period is entire prefix so far.
                right += offset + 1;
                offset = 0;
                period = right - left;
            } else if a == b {
                // Advance through repetition of the current period.
                if offset + 1 == period {
                    right += offset + 1;
                    offset = 0;
                } else {
                    offset += 1;
                }
            } else {
                // Suffix is larger, start over from current location.
                left = right;
                right += 1;
                offset = 0;
                period = 1;
            }
        }
        (left, period)
    }

    // Compute the maximal suffix of the reverse of `arr`.
    //
    // The maximal suffix is a possible critical factorization (u', v') of `arr`.
    //
    // Returns `i` where `i` is the starting index of v', from the back;
    // returns immedately when a period of `known_period` is reached.
    //
    // `order_greater` determines if lexical order is `<` or `>`. Both
    // orders must be computed -- the ordering with the largest `i` gives
    // a critical factorization.
    //
    // For long period cases, the resulting period is not exact (it is too short).
    fn reverse_maximal_suffix<T: Ord>(arr: &[T], known_period: usize,
                                      order_greater: bool) -> usize
    {
        let mut left = 0; // Corresponds to i in the paper
        let mut right = 1; // Corresponds to j in the paper
        let mut offset = 0; // Corresponds to k in the paper, but starting at 0
                            // to match 0-based indexing.
        let mut period = 1; // Corresponds to p in the paper
        let n = arr.len();

        while right + offset < n {
            let a = &arr[n - (1 + right + offset)];
            let b = &arr[n - (1 + left + offset)];
            if (a < b && !order_greater) || (a > b && order_greater) {
                // Suffix is smaller, period is entire prefix so far.
                right += offset + 1;
                offset = 0;
                period = right - left;
            } else if a == b {
                // Advance through repetition of the current period.
                if offset + 1 == period {
                    right += offset + 1;
                    offset = 0;
                } else {
                    offset += 1;
                }
            } else {
                // Suffix is larger, start over from current location.
                left = right;
                right += 1;
                offset = 0;
                period = 1;
            }
            if period == known_period {
                break;
            }
        }
        debug_assert!(period <= known_period);
        left
    }
}

// TwoWayStrategy allows the algorithm to either skip non-matches as quickly
// as possible, or to work in a mode where it emits Rejects relatively quickly.
trait TwoWayStrategy {
    type Output;
    fn use_early_reject() -> bool;
    fn rejecting(usize, usize) -> Self::Output;
    fn matching(usize, usize) -> Self::Output;
}

/// Skip to match intervals as quickly as possible
enum MatchOnly { }

impl TwoWayStrategy for MatchOnly {
    type Output = Option<(usize, usize)>;

    #[inline]
    fn use_early_reject() -> bool { false }
    #[inline]
    fn rejecting(_a: usize, _b: usize) -> Self::Output { None }
    #[inline]
    fn matching(a: usize, b: usize) -> Self::Output { Some((a, b)) }
}

/// Emit Rejects regularly
enum RejectAndMatch { }

impl TwoWayStrategy for RejectAndMatch {
    type Output = SearchStep;

    #[inline]
    fn use_early_reject() -> bool { true }
    #[inline]
    fn rejecting(a: usize, b: usize) -> Self::Output { SearchStep::Reject(a, b) }
    #[inline]
    fn matching(a: usize, b: usize) -> Self::Output { SearchStep::Match(a, b) }
}
