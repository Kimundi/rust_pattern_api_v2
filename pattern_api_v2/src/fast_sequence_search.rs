use super::*;

use std::cmp;
use std::usize;

pub trait OrdSlice: SearchCursors {
    type NeedleElement: Ord;
    type FastSkipOptimization: FastSkipOptimization<Self::NeedleElement>;

    fn next_valid_pos(hs: &Self::Haystack, pos: usize) -> Option<usize>;
    fn next_valid_pos_back(hs: &Self::Haystack, pos: usize) -> Option<usize>;

    fn haystack_as_slice(hs: &Self::Haystack) -> &[Self::NeedleElement];

    fn pos_is_valid(hs: &Self::Haystack, pos: usize) -> bool;

    unsafe fn cursor_at_offset(hs: Self::Haystack, offset: usize) -> Self::Cursor;

    fn starts_with(hs: &Self::Haystack, needle: &[Self::NeedleElement]) -> bool {
        let haystack = Self::haystack_as_slice(hs);
        haystack.len() >= needle.len()
            && haystack[..needle.len()] == *needle
    }
    fn ends_with(hs: &Self::Haystack, needle: &[Self::NeedleElement]) -> bool {
        let haystack = Self::haystack_as_slice(hs);
        haystack.len() >= needle.len()
            && haystack[haystack.len() - needle.len()..] == *needle
    }
}

// Only temporary used to make the old code work without major changes
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum SearchStep {
    Match(usize, usize),
    Reject(usize, usize),
    Done
}

// H,        H::Needle
//
// &[T],       &[T]
// &mut [T],   &[T]
//
// Iter<T>,    Iter<T>
// MutIter<T>, Iter<T>
//
// Pattern<&'a [T]> for &'b [T]
// Pattern<&'a mut [T]> for &'b [T]

pub struct OrdSlicePattern<'b, H: OrdSlice>(pub &'b [H::NeedleElement])
    where H::NeedleElement: 'b;

/// Non-allocating substring search.
///
/// Will handle the pattern `""` as returning empty matches at each character
/// boundary.
impl<'b, H: OrdSlice> OrdSlicePattern<'b, H> {
    #[inline]
    pub fn into_searcher(self, haystack: H) -> OrdSeqSearcher<'b, H> {
        OrdSeqSearcher::new(haystack, self.0)
    }

    /// Checks whether the pattern matches at the front of the haystack
    #[inline]
    pub fn is_prefix_of(self, haystack: H) -> bool {
        let hs = haystack.into_haystack();
        H::starts_with(&hs, &self.0)
    }

    /// Checks whether the pattern matches at the back of the haystack
    #[inline]
    pub fn is_suffix_of(self, haystack: H) -> bool {
        let hs = haystack.into_haystack();
        H::ends_with(&hs, &self.0)
    }

    #[inline]
    pub fn is_contained_in(self, haystack: H) -> bool {
        self.into_searcher(haystack).next_match().is_some()
    }
}

#[derive(Copy, Clone)]
pub struct Iter<H: SearchCursors> {
    haystack: H::Haystack,
    start: H::Cursor,
    end: H::Cursor,
    _marker: ::std::marker::PhantomData<H>,
}

impl<H: SearchCursors> Iter<H> {
    #[inline]
    fn new(haystack: H::Haystack) -> Self {
        let start = H::cursor_at_front(haystack);
        let end = H::cursor_at_back(haystack);
        Iter {
            haystack: haystack,
            start: start,
            end: end,
            _marker: ::std::marker::PhantomData,
        }
    }

    fn haystack_len(&self) -> usize {
        H::haystack_len(self.haystack)
    }
}

/////////////////////////////////////////////////////////////////////////////
// Two Way substring searcher
/////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
/// Associated type for `<&str as Pattern<&'a str>>::Searcher`.
pub struct OrdSeqSearcher<'b, H: OrdSlice>
    where H::NeedleElement: 'b
{
    iter: Iter<H>,
    needle: &'b [H::NeedleElement],

    searcher: OrdSeqSearcherImpl<H::NeedleElement, H::FastSkipOptimization>,
}

#[derive(Clone, Debug)]
enum OrdSeqSearcherImpl<T, O> {
    Empty(EmptyN),
    TwoWay(TwoWaySearcher<T, O>),
}

#[derive(Clone, Debug)]
struct EmptyN {
    position: usize,
    end: usize,
    is_match_fw: bool,
    is_match_bw: bool,
}

impl<'b, H: OrdSlice> OrdSeqSearcher<'b, H> {
    fn new(haystack: H, needle: &'b [H::NeedleElement]) -> OrdSeqSearcher<H> {
        let hs = haystack.into_haystack();

        OrdSeqSearcher {
            iter: Iter::new(hs),
            needle: needle,
            searcher: if needle.is_empty() {
                let hs_len = H::haystack_len(hs);
                OrdSeqSearcherImpl::Empty(EmptyN {
                    position: 0,
                    end: hs_len,
                    is_match_fw: true,
                    is_match_bw: true,
                })
            } else {
                let hs_len = H::haystack_len(hs);
                OrdSeqSearcherImpl::TwoWay(
                    TwoWaySearcher::new(needle, hs_len)
                )
            }
        }
    }
}

impl<'b, H: OrdSlice> OrdSeqSearcher<'b, H> {
    #[inline]
    fn next(&mut self) -> SearchStep {
        match self.searcher {
            OrdSeqSearcherImpl::Empty(ref mut searcher) => {
                // empty needle rejects every char and matches every empty string between them
                let is_match = searcher.is_match_fw;
                searcher.is_match_fw = !searcher.is_match_fw;
                let pos = searcher.position;
                match H::next_valid_pos(&self.iter.haystack, pos) {
                    _ if is_match => SearchStep::Match(pos, pos),
                    None => SearchStep::Done,
                    Some(new_pos) => {
                        searcher.position = new_pos;
                        SearchStep::Reject(pos, searcher.position)
                    }
                }
            }
            OrdSeqSearcherImpl::TwoWay(ref mut searcher) => {
                // TwoWaySearcher produces valid *Match* indices that split at char boundaries
                // as long as it does correct matching and that haystack and needle are
                // valid UTF-8
                // *Rejects* from the algorithm can fall on any indices, but we will walk them
                // manually to the next character boundary, so that they are utf-8 safe.
                if searcher.position == self.iter.haystack_len() {
                    return SearchStep::Done;
                }
                let is_long = searcher.memory == usize::MAX;
                match searcher.next::<RejectAndMatch>(H::haystack_as_slice(
                                                          &self.iter.haystack),
                                                      self.needle,
                                                      is_long)
                {
                    SearchStep::Reject(a, mut b) => {
                        // skip to next char boundary
                        while !H::pos_is_valid(&self.iter.haystack, b) {
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
            OrdSeqSearcherImpl::Empty(ref mut searcher) => {
                let is_match = searcher.is_match_bw;
                searcher.is_match_bw = !searcher.is_match_bw;
                let end = searcher.end;
                match H::next_valid_pos_back(&self.iter.haystack, end) {
                    _ if is_match => SearchStep::Match(end, end),
                    None => SearchStep::Done,
                    Some(next_end) => {
                        searcher.end = next_end;
                        SearchStep::Reject(searcher.end, end)
                    }
                }
            }
            OrdSeqSearcherImpl::TwoWay(ref mut searcher) => {
                if searcher.end == 0 {
                    return SearchStep::Done;
                }
                let is_long = searcher.memory == usize::MAX;
                match searcher.next_back::<RejectAndMatch>(H::haystack_as_slice(
                                                               &self.iter.haystack),
                                                           self.needle,
                                                           is_long)
                {
                    SearchStep::Reject(mut a, b) => {
                        // skip to next char boundary
                        while !H::pos_is_valid(&self.iter.haystack, a) {
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

unsafe impl<'b, H: OrdSlice> Searcher<H> for OrdSeqSearcher<'b, H>
    where H: SearchCursors
{
    fn haystack(&self) -> H::Haystack {
        self.iter.haystack
    }

    #[inline(always)]
    fn next_match(&mut self) -> Option<(H::Cursor, H::Cursor)> {
        (|| match self.searcher {
            OrdSeqSearcherImpl::Empty(..) => {
                loop {
                    match self.next() {
                        SearchStep::Match(a, b) => return Some((a, b)),
                        SearchStep::Done => return None,
                        SearchStep::Reject(..) => { }
                    }
                }
            }
            OrdSeqSearcherImpl::TwoWay(ref mut searcher) => {
                let is_long = searcher.memory == usize::MAX;
                // write out `true` and `false` cases to encourage the compiler
                // to specialize the two cases separately.
                if is_long {
                    searcher.next::<MatchOnly>(H::haystack_as_slice(
                                                   &self.iter.haystack),
                                               self.needle,
                                               true)
                } else {
                    searcher.next::<MatchOnly>(H::haystack_as_slice(
                                                   &self.iter.haystack),
                                               self.needle,
                                               false)
                }
            }
        })().map(|(a, b)| unsafe {
            let hs = self.haystack();
            (H::cursor_at_offset(hs, a), H::cursor_at_offset(hs, b))
        })
    }

    #[inline(always)]
    fn next_reject(&mut self) -> Option<(H::Cursor, H::Cursor)> {
        (|| loop {
            match self.next() {
                SearchStep::Reject(a, b) => return Some((a, b)),
                SearchStep::Done => return None,
                _ => (),
            }
        })().map(|(a, b)| unsafe {
            let hs = self.haystack();
            (H::cursor_at_offset(hs, a), H::cursor_at_offset(hs, b))
        })
    }
}

unsafe impl<'b, H: OrdSlice> ReverseSearcher<H> for OrdSeqSearcher<'b, H>
    where H: SearchCursors
{
    #[inline]
    fn next_match_back(&mut self) -> Option<(H::Cursor, H::Cursor)> {
        (|| match self.searcher {
            OrdSeqSearcherImpl::Empty(..) => {
                loop {
                    match self.next_back() {
                        SearchStep::Match(a, b) => return Some((a, b)),
                        SearchStep::Done => return None,
                        SearchStep::Reject(..) => { }
                    }
                }
            }
            OrdSeqSearcherImpl::TwoWay(ref mut searcher) => {
                let is_long = searcher.memory == usize::MAX;
                // write out `true` and `false`, like `next_match`
                if is_long {
                    searcher.next_back::<MatchOnly>(H::haystack_as_slice(
                                                        &self.iter.haystack),
                                                    self.needle,
                                                    true)
                } else {
                    searcher.next_back::<MatchOnly>(H::haystack_as_slice(
                                                        &self.iter.haystack),
                                                    self.needle,
                                                    false)
                }
            }
        })().map(|(a, b)| unsafe {
            let hs = self.haystack();
            (H::cursor_at_offset(hs, a), H::cursor_at_offset(hs, b))
        })
    }

    #[inline(always)]
    fn next_reject_back(&mut self) -> Option<(H::Cursor, H::Cursor)> {
        (|| loop {
            match self.next_back() {
                SearchStep::Reject(a, b) => return Some((a, b)),
                SearchStep::Done => return None,
                _ => (),
            }
        })().map(|(a, b)| unsafe {
            let hs = self.haystack();
            (H::cursor_at_offset(hs, a), H::cursor_at_offset(hs, b))
        })
    }

}

pub trait FastSkipOptimization<T> {
    fn new(needle: &[T]) -> Self;
    fn contains(&self, byte: &T) -> bool;
}

/// `ByteOptimization` is a 64-bit "fingerprint" where each set bit `j` corresponds
/// to a (byte & 63) == j present in the needle.
pub struct ByteOptimization(u64);
impl FastSkipOptimization<u8> for ByteOptimization {
    fn new(needle: &[u8]) -> Self {
        ByteOptimization(needle.iter().fold(0, |a, &b| (1 << (b & 0x3f)) | a))
    }

    fn contains(&self, &byte: &u8) -> bool {
        (self.0 >> ((byte & 0x3f) as usize)) & 1 != 0
    }
}

pub struct NoOptimization;
impl<T> FastSkipOptimization<T> for NoOptimization {
    fn new(_: &[T]) -> Self {
        NoOptimization
    }

    fn contains(&self, _: &T) -> bool {
        true
    }
}

/// The internal state of the two-way substring search algorithm.
#[derive(Clone, Debug)]
struct TwoWaySearcher<T, O> {
    // constants
    /// critical factorization index
    crit_pos: usize,
    /// critical factorization index for reversed needle
    crit_pos_back: usize,
    period: usize,

    /// `fast_skip_state` is an extension (not part of the two way algorithm);
    /// it can be used to skip over multiple rejected elements at once
    fast_skip_state: O,

    // variables
    position: usize,
    end: usize,
    /// index into needle before which we have already matched
    memory: usize,
    /// index into needle after which we have already matched
    memory_back: usize,

    _marker: ::std::marker::PhantomData<(T, O)>,
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
impl<T: Ord, O: FastSkipOptimization<T>> TwoWaySearcher<T, O> {
    fn new(needle: &[T], end: usize) -> TwoWaySearcher<T, O> {
        let (crit_pos_false, period_false)
            = TwoWaySearcher::<T, O>::maximal_suffix(needle, false);
        let (crit_pos_true, period_true)
            = TwoWaySearcher::<T, O>::maximal_suffix(needle, true);

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
                TwoWaySearcher::<T, O>::reverse_maximal_suffix(needle, period, false),
                TwoWaySearcher::<T, O>::reverse_maximal_suffix(needle, period, true));

            TwoWaySearcher {
                crit_pos: crit_pos,
                crit_pos_back: crit_pos_back,
                period: period,

                fast_skip_state: O::new(&needle[..period]),

                position: 0,
                end: end,
                memory: 0,
                memory_back: needle.len(),

                _marker: ::std::marker::PhantomData,
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

                fast_skip_state: O::new(needle),

                position: 0,
                end: end,
                memory: usize::MAX, // Dummy value to signify that the period is long
                memory_back: usize::MAX,

                _marker: ::std::marker::PhantomData,
            }
        }
    }

    // One of the main ideas of Two-Way is that we factorize the needle into
    // two halves, (u, v), and begin trying to find v in the haystack by scanning
    // left to right. If v matches, we try to match u by scanning right to left.
    // How far we can jump when we encounter a mismatch is all based on the fact
    // that (u, v) is a critical factorization for the needle.
    #[inline(always)]
    fn next<S>(&mut self, haystack: &[T], needle: &[T], long_period: bool)
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
            let tail_byte = match haystack.get(self.position + needle_last) {
                Some(b) => b,
                None => {
                    self.position = haystack.len();
                    return S::rejecting(old_pos, self.position);
                }
            };

            if S::use_early_reject() && old_pos != self.position {
                return S::rejecting(old_pos, self.position);
            }

            // Quickly skip by large portions unrelated to our substring
            if !self.fast_skip_state.contains(tail_byte) {
                self.position += needle.len();
                if !long_period {
                    self.memory = 0;
                }
                continue 'search;
            }

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
    fn next_back<S>(&mut self, haystack: &[T], needle: &[T], long_period: bool)
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
            let front_byte = match haystack.get(self.end.wrapping_sub(needle.len())) {
                Some(b) => b,
                None => {
                    self.end = 0;
                    return S::rejecting(0, old_end);
                }
            };

            if S::use_early_reject() && old_end != self.end {
                return S::rejecting(self.end, old_end);
            }

            // Quickly skip by large portions unrelated to our substring
            if !self.fast_skip_state.contains(front_byte) {
                self.end -= needle.len();
                if !long_period {
                    self.memory_back = needle.len();
                }
                continue 'search;
            }

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
    fn maximal_suffix(arr: &[T], order_greater: bool) -> (usize, usize) {
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
    fn reverse_maximal_suffix(arr: &[T], known_period: usize,
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
