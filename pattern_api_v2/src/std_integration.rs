use ::{Pattern, SearchCursors, Searcher, ReverseSearcher, DoubleEndedSearcher};
use iterators::{Split, RSplit};
use iterators::{SplitTerminator, RSplitTerminator};
use iterators::{SplitN, RSplitN};
use iterators::{Matches, RMatches};
use iterators::{MatchIndices, RMatchIndices};
use ::InverseMatchesAreValid;

pub trait IteratorConstructors: SearchCursors {
    #[inline]
    fn contains<P: Pattern<Self>>(self, pat: P) -> bool {
        pat.is_contained_in(self)
    }

    #[inline]
    fn split<P: Pattern<Self>>(self, pat: P) -> Split<Self, P>
        where Self: InverseMatchesAreValid
    {
        Split::new(self, pat)
    }

    #[inline]
    fn rsplit<P: Pattern<Self>>(self, pat: P) -> RSplit<Self, P>
        where P::Searcher: ReverseSearcher<Self>,
              Self: InverseMatchesAreValid
    {
        RSplit::new(self, pat)
    }

    #[inline]
    fn splitn<P: Pattern<Self>>(self, count: usize, pat: P) -> SplitN<Self, P>
        where Self: InverseMatchesAreValid
    {
        SplitN::new(self, pat, count)
    }

    #[inline]
    fn rsplitn<P: Pattern<Self>>(self, count: usize, pat: P) -> RSplitN<Self, P>
        where P::Searcher: ReverseSearcher<Self>,
              Self: InverseMatchesAreValid
    {
        RSplitN::new(self, pat, count)
    }

    #[inline]
    fn split_terminator<P: Pattern<Self>>(self, pat: P) -> SplitTerminator<Self, P>
        where Self: InverseMatchesAreValid
    {
        SplitTerminator::new(self, pat)
    }

    #[inline]
    fn rsplit_terminator<P: Pattern<Self>>(self, pat: P) -> RSplitTerminator<Self, P>
        where P::Searcher: ReverseSearcher<Self>,
              Self: InverseMatchesAreValid
    {
        RSplitTerminator::new(self, pat)
    }

    #[inline]
    fn matches<P: Pattern<Self>>(self, pat: P) -> Matches<Self, P> {
        Matches::new(self, pat)
    }

    #[inline]
    fn rmatches<P: Pattern<Self>>(self, pat: P) -> RMatches<Self, P>
        where P::Searcher: ReverseSearcher<Self>
    {
        RMatches::new(self, pat)
    }

    #[inline]
    fn match_indices<P: Pattern<Self>>(self, pat: P) -> MatchIndices<Self, P> {
        MatchIndices::new(self, pat)
    }

    #[inline]
    fn rmatch_indices<P: Pattern<Self>>(self, pat: P) -> RMatchIndices<Self, P>
        where P::Searcher: ReverseSearcher<Self>
    {
        RMatchIndices::new(self, pat)
    }

    #[inline]
    fn starts_with<P: Pattern<Self>>(self, pat: P) -> bool {
        pat.is_prefix_of(self)
    }

    #[inline]
    fn ends_with<P: Pattern<Self>>(self, pat: P) -> bool
        where P::Searcher: ReverseSearcher<Self>
    {
        pat.is_suffix_of(self)
    }

    #[inline]
    fn trim_matches<P: Pattern<Self>>(self, pat: P) -> Self::MatchType
        where P::Searcher: DoubleEndedSearcher<Self>
    {
        let mut matcher = pat.into_searcher(self);
        let mut i = Self::cursor_at_front(matcher.haystack());
        let mut j = Self::cursor_at_front(matcher.haystack());
        if let Some((a, b)) = matcher.next_reject() {
            i = a;
            j = b; // Remember earliest known match, correct it below if
                   // last match is different
        }
        if let Some((_, b)) = matcher.next_reject_back() {
            j = b;
        }
        unsafe {
            // Searcher is known to return valid indices
            Self::range_to_self(matcher.haystack(), i, j)
        }
    }

    #[inline]
    fn trim_left_matches<P: Pattern<Self>>(self, pat: P) -> Self::MatchType {
        let mut matcher = pat.into_searcher(self);
        let mut i = Self::cursor_at_back(matcher.haystack());
        let j = Self::cursor_at_back(matcher.haystack());
        if let Some((a, _)) = matcher.next_reject() {
            i = a;
        }
        unsafe {
            // Searcher is known to return valid indices
            Self::range_to_self(matcher.haystack(), i, j)
        }
    }

    #[inline]
    fn trim_right_matches<P: Pattern<Self>>(self, pat: P) -> Self::MatchType
        where P::Searcher: ReverseSearcher<Self>
    {
        let mut matcher = pat.into_searcher(self);
        let i = Self::cursor_at_front(matcher.haystack());
        let mut j = Self::cursor_at_front(matcher.haystack());
        if let Some((_, b)) = matcher.next_reject_back() {
            j = b;
        }
        unsafe {
            // Searcher is known to return valid indices
            Self::range_to_self(matcher.haystack(), i, j)
        }
    }

    fn find<P: Pattern<Self>>(self, pat: P) -> Option<usize> {
        let mut searcher = pat.into_searcher(self);
        let h = searcher.haystack();
        searcher.next_match().map(|(i, _)| Self::offset_from_front(h, i))
    }

    fn rfind<P: Pattern<Self>>(self, pat: P) -> Option<usize>
        where P::Searcher: ReverseSearcher<Self>
    {
        let mut searcher = pat.into_searcher(self);
        let h = searcher.haystack();
        searcher.next_match_back().map(|(i, _)| Self::offset_from_front(h, i))
    }
}

impl<T: SearchCursors> IteratorConstructors for T {}
