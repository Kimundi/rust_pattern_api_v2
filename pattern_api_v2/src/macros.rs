
macro_rules! pattern_methods {
    ($t:ty, $pmap:expr, $smap:expr, $slice:ty) => {
        pattern_methods!($t, $pmap, $smap, $slice, |s| s);
    };
    ($t:ty, $pmap:expr, $smap:expr, $slice:ty, $slice_map:expr) => {
        type Searcher = $t;

        #[inline]
        fn into_searcher(self, haystack: $slice) -> $t {
            ($smap)(($pmap)(self).into_searcher($slice_map(haystack)))
        }

        #[inline]
        fn is_contained_in(self, haystack: $slice) -> bool {
            ($pmap)(self).is_contained_in($slice_map(haystack))
        }

        #[inline]
        fn is_prefix_of(self, haystack: $slice) -> bool {
            ($pmap)(self).is_prefix_of($slice_map(haystack))
        }

        #[inline]
        fn is_suffix_of(self, haystack: $slice) -> bool
            where $t: ReverseSearcher<$slice>
        {
            ($pmap)(self).is_suffix_of($slice_map(haystack))
        }
    }
}

macro_rules! searcher_methods {
    (forward, $inner_ident:ident, $inner:expr, $cursor:ty) => {
        #[inline]
        fn haystack(&self) -> ($cursor, $cursor) {
            let $inner_ident = self;
            $inner.haystack()
        }
        #[inline]
        fn next_match(&mut self) -> Option<($cursor, $cursor)> {
            let $inner_ident = self;
            $inner.next_match()
        }
        #[inline]
        fn next_reject(&mut self) -> Option<($cursor, $cursor)> {
            let $inner_ident = self;
            $inner.next_reject()
        }
    };
    (reverse, $inner_ident:ident, $inner:expr, $cursor:ty) => {
        #[inline]
        fn next_match_back(&mut self) -> Option<($cursor, $cursor)> {
            let $inner_ident = self;
            $inner.next_match_back()
        }
        #[inline]
        fn next_reject_back(&mut self) -> Option<($cursor, $cursor)> {
            let $inner_ident = self;
            $inner.next_reject_back()
        }
    }
}
