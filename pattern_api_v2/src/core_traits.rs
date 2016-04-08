pub trait Pattern<H: PatternHaystack>: Sized {
    type Searcher: Searcher<H>;
    fn into_searcher(self, haystack: H) -> Self::Searcher;

    fn is_prefix_of(self, haystack: H) -> bool {
        let mut searcher = self.into_searcher(haystack);

        // looking for first reject under the assumption that
        // a search will have more misses than matches
        searcher.next_reject()
                .map(|t| t.0 != H::cursor_at_front(searcher.haystack()))
                .unwrap_or(false)
    }

    fn is_suffix_of(self, haystack: H) -> bool
        where Self::Searcher: ReverseSearcher<H>
    {
        let mut searcher = self.into_searcher(haystack);

        // looking for first reject under the assumption that
        // a search will have more misses than matches
        searcher.next_reject_back()
                .map(|t| t.1 != H::cursor_at_back(searcher.haystack()))
                .unwrap_or(false)
    }

    fn is_contained_in(self, haystack: H) -> bool {
        self.into_searcher(haystack).next_match().is_some()
    }
}

// Defined associated types and functions
// for dealing with positions in a slice-like type
// with pointer-like cursors
// Logically, Haystack <= Cursor <= Back
pub trait PatternHaystack: Sized {
    // For storing the bounds of the haystack.
    // Usually a combination of Memory address in form of a raw pointer or usize
    type Haystack: Copy;

    // Begin or End of a Match.
    // Two of these can be used to define a range of elements
    // as found by a Searcher.
    // Can be absolute, or relative to Haystack.
    // Usually a Memory address in form of a raw pointer or usize
    type Cursor: Copy + Eq + Ord;

    type MatchType;

    fn into_haystack(self) -> Self::Haystack;

    fn offset_from_front(hs: Self::Haystack, begin: Self::Cursor) -> usize;
    fn cursor_at_front(hs: Self::Haystack) -> Self::Cursor;
    fn cursor_at_back(hs: Self::Haystack) -> Self::Cursor;

    unsafe fn range_to_self(hs: Self::Haystack,
                            start: Self::Cursor,
                            end: Self::Cursor) -> Self::MatchType;

    fn haystack_len(hs: Self::Haystack) -> usize {
        let haystack = hs;
        let back = Self::cursor_at_back(haystack);
        Self::offset_from_front(haystack, back)
    }

    fn cursor_diff(haystack: Self::Haystack,
                   start: Self::Cursor,
                   end: Self::Cursor) -> usize {
        Self::offset_from_front(haystack, end)
        - Self::offset_from_front(haystack, start)
    }

    fn match_type_len(mt: &Self::MatchType) -> usize;
}

pub unsafe trait Searcher<H: PatternHaystack> {
    fn haystack(&self) -> H::Haystack;

    fn next_match(&mut self) -> Option<(H::Cursor, H::Cursor)>;
    fn next_reject(&mut self) -> Option<(H::Cursor, H::Cursor)>;
}

pub unsafe trait ReverseSearcher<H: PatternHaystack>: Searcher<H> {
    fn next_match_back(&mut self) -> Option<(H::Cursor, H::Cursor)>;
    fn next_reject_back(&mut self) -> Option<(H::Cursor, H::Cursor)>;
}

pub trait DoubleEndedSearcher<H: PatternHaystack>: ReverseSearcher<H> {}

pub unsafe trait InverseMatchesAreValid {}
