use super::*;

use std::fmt;

/// This macro generates two public iterator structs
/// wrapping a private internal one that makes use of the `Pattern` API.
///
/// For all patterns `P: Pattern<H>` the following items will be
/// generated (generics omitted):
///
/// struct $forward_iterator($internal_iterator);
/// struct $reverse_iterator($internal_iterator);
///
/// impl Iterator for $forward_iterator
/// { /* internal ends up calling Searcher::next_match() */ }
///
/// impl DoubleEndedIterator for $forward_iterator
///       where P::Searcher: DoubleEndedSearcher
/// { /* internal ends up calling Searcher::next_match_back() */ }
///
/// impl Iterator for $reverse_iterator
///       where P::Searcher: ReverseSearcher
/// { /* internal ends up calling Searcher::next_match_back() */ }
///
/// impl DoubleEndedIterator for $reverse_iterator
///       where P::Searcher: DoubleEndedSearcher
/// { /* internal ends up calling Searcher::next_match() */ }
///
/// The internal one is defined outside the macro, and has almost the same
/// semantic as a DoubleEndedIterator by delegating to `pattern::Searcher` and
/// `pattern::ReverseSearcher` for both forward and reverse iteration.
///
/// "Almost", because a `Searcher` and a `ReverseSearcher` for a given
/// `Pattern` might not return the same elements, so actually implementing
/// `DoubleEndedIterator` for it would be incorrect.
/// (See the docs in `str::pattern` for more details)
///
/// However, the internal struct still represents a single ended iterator from
/// either end, and depending on pattern is also a valid double ended iterator,
/// so the two wrapper structs implement `Iterator`
/// and `DoubleEndedIterator` depending on the concrete pattern type, leading
/// to the complex impls seen above.
macro_rules! generate_pattern_iterators {
    {
        // Forward iterator
        forward:
            $(#[$forward_iterator_attribute:meta])*
            struct $forward_iterator:ident;

        // Reverse iterator
        reverse:
            $(#[$reverse_iterator_attribute:meta])*
            struct $reverse_iterator:ident;

        // Stability of all generated items
        stability:
            $(#[$common_stability_attribute:meta])*

        // Internal almost-iterator that is being delegated to
        internal:
            $internal_iterator:ident($($iterarg_ident:ident: $iterarg_ty:ty),*) yielding ($iterty:ty);

        // Kind of delgation - either single ended or double ended
        delegate $($t:tt)*
    } => {
        $(#[$forward_iterator_attribute])*
        $(#[$common_stability_attribute])*
        pub struct $forward_iterator<H, P>($internal_iterator<H, P>)
            where P: Pattern<H>,
                  H: SearchCursors;

        $(#[$common_stability_attribute])*
        impl<H, P> $forward_iterator<H, P>
            where P: Pattern<H>,
                  H: SearchCursors,
        {
            #[inline]
            pub fn new(h: H, p: P $(,$iterarg_ident: $iterarg_ty)*) -> Self {
                $forward_iterator($internal_iterator::new(h, p $(,$iterarg_ident)*))
            }
        }

        $(#[$common_stability_attribute])*
        impl<H, P> fmt::Debug for $forward_iterator<H, P>
            where P: Pattern<H>,
                  H: SearchCursors,
                  P::Searcher: fmt::Debug,
        {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.debug_tuple(stringify!($forward_iterator))
                    .field(&self.0)
                    .finish()
            }
        }

        $(#[$common_stability_attribute])*
        impl<H, P> Iterator for $forward_iterator<H, P>
            where P: Pattern<H>,
                  H: SearchCursors,
        {
            type Item = $iterty;

            #[inline]
            fn next(&mut self) -> Option<$iterty> {
                self.0.next()
            }
        }

        $(#[$common_stability_attribute])*
        impl<H, P> Clone for $forward_iterator<H, P>
            where P: Pattern<H>,
                  H: SearchCursors,
                  P::Searcher: Clone
        {
            fn clone(&self) -> Self {
                $forward_iterator(self.0.clone())
            }
        }

        $(#[$reverse_iterator_attribute])*
        $(#[$common_stability_attribute])*
        pub struct $reverse_iterator<H, P>($internal_iterator<H, P>)
            where P: Pattern<H>,
                  H: SearchCursors;

        $(#[$common_stability_attribute])*
        impl<H, P> $reverse_iterator<H, P>
            where P: Pattern<H>,
                  H: SearchCursors,
        {
            #[inline]
            pub fn new(h: H, p: P $(,$iterarg_ident: $iterarg_ty)*) -> Self {
                $reverse_iterator($internal_iterator::new(h, p $(,$iterarg_ident)*))
            }
        }

        $(#[$common_stability_attribute])*
        impl<H, P> fmt::Debug for $reverse_iterator<H, P>
            where P: Pattern<H>,
                  H: SearchCursors,
                  P::Searcher: fmt::Debug
        {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.debug_tuple(stringify!($reverse_iterator))
                    .field(&self.0)
                    .finish()
            }
        }

        $(#[$common_stability_attribute])*
        impl<H, P> Iterator for $reverse_iterator<H, P>
            where P: Pattern<H>,
                  H: SearchCursors,
                  P::Searcher: ReverseSearcher<H>
        {
            type Item = $iterty;

            #[inline]
            fn next(&mut self) -> Option<$iterty> {
                self.0.next_back()
            }
        }

        $(#[$common_stability_attribute])*
        impl<H, P> Clone for $reverse_iterator<H, P>
            where P: Pattern<H>,
                  H: SearchCursors,
                  P::Searcher: Clone
        {
            fn clone(&self) -> Self {
                $reverse_iterator(self.0.clone())
            }
        }

        generate_pattern_iterators!($($t)* with $(#[$common_stability_attribute])*,
                                                $forward_iterator,
                                                $reverse_iterator, $iterty);
    };
    {
        double ended; with $(#[$common_stability_attribute:meta])*,
                           $forward_iterator:ident,
                           $reverse_iterator:ident, $iterty:ty
    } => {
        $(#[$common_stability_attribute])*
        impl<H, P> DoubleEndedIterator for $forward_iterator<H, P>
            where P: Pattern<H>,
                  H: SearchCursors,
                  P::Searcher: DoubleEndedSearcher<H>
        {
            #[inline]
            fn next_back(&mut self) -> Option<$iterty> {
                self.0.next_back()
            }
        }

        $(#[$common_stability_attribute])*
        impl<H, P: Pattern<H>> DoubleEndedIterator for $reverse_iterator<H, P>
            where P: Pattern<H>,
                  H: SearchCursors,
                  P::Searcher: DoubleEndedSearcher<H>
        {
            #[inline]
            fn next_back(&mut self) -> Option<$iterty> {
                self.0.next()
            }
        }
    };
    {
        single ended; with $(#[$common_stability_attribute:meta])*,
                           $forward_iterator:ident,
                           $reverse_iterator:ident, $iterty:ty
    } => {}
}

/// This macro generates a Clone impl for string pattern API
/// wrapper types of the form X<H, P>
macro_rules! derive_pattern_clone {
    (clone $t:ident with |$s:ident| $e:expr) => {
        impl<H, P: Pattern<H>> Clone for $t<H, P>
            where P::Searcher: Clone,
                  H: SearchCursors,
        {
            fn clone(&self) -> Self {
                let $s = self;
                $e
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// .matches()
///////////////////////////////////////////////////////////////////////////////

derive_pattern_clone!{
    clone MatchesInternal
    with |s| MatchesInternal(s.0.clone())
}

struct MatchesInternal<H, P>(P::Searcher)
    where P: Pattern<H>,
          H: SearchCursors;

impl<H, P> fmt::Debug for MatchesInternal<H, P>
    where P: Pattern<H>,
          P::Searcher: fmt::Debug,
          H: SearchCursors,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("MatchesInternal")
            .field(&self.0)
            .finish()
    }
}

impl<H, P> MatchesInternal<H, P>
    where P: Pattern<H>,
          H: SearchCursors,
{
    #[inline]
    fn new(h: H, p: P) -> Self {
        MatchesInternal(p.into_searcher(h))
    }

    #[inline]
    fn next(&mut self) -> Option<H::MatchType> {
        self.0.next_match().map(|(a, b)| unsafe {
            H::range_to_self(self.0.haystack(), a, b)
        })
    }

    #[inline]
    fn next_back(&mut self) -> Option<H::MatchType>
        where P::Searcher: ReverseSearcher<H>
    {
        self.0.next_match_back().map(|(a, b)| unsafe {
            H::range_to_self(self.0.haystack(), a, b)
        })
    }
}

generate_pattern_iterators! {
    forward:
        /// Created with the method [`matches()`].
        ///
        /// [`matches()`]: ../../std/primitive.str.html#method.matches
        struct Matches;
    reverse:
        /// Created with the method [`rmatches()`].
        ///
        /// [`rmatches()`]: ../../std/primitive.str.html#method.rmatches
        struct RMatches;
    stability:
        //#[stable(feature = "str_matches", since = "1.2.0")]
    internal:
        MatchesInternal() yielding (H::MatchType);
    delegate double ended;
}

///////////////////////////////////////////////////////////////////////////////
// .match_indices()
///////////////////////////////////////////////////////////////////////////////

derive_pattern_clone!{
    clone MatchIndicesInternal
    with |s| MatchIndicesInternal(s.0.clone())
}

struct MatchIndicesInternal<H, P>(P::Searcher)
    where P: Pattern<H>,
          H: SearchCursors;

impl<H, P: Pattern<H>> fmt::Debug for MatchIndicesInternal<H, P>
    where P::Searcher: fmt::Debug,
          P: Pattern<H>,
          H: SearchCursors
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("MatchIndicesInternal")
            .field(&self.0)
            .finish()
    }
}

impl<H, P> MatchIndicesInternal<H, P>
    where P: Pattern<H>,
          H: SearchCursors,
{
    #[inline]
    fn new(h: H, p: P) -> Self {
        MatchIndicesInternal(p.into_searcher(h))
    }

    #[inline]
    fn next(&mut self) -> Option<(usize, H::MatchType)> {
        self.0.next_match().map(|(a, b)| unsafe {
            (H::offset_from_front(self.0.haystack(), a),
             H::range_to_self(self.0.haystack(), a, b))
        })
    }

    #[inline]
    fn next_back(&mut self) -> Option<(usize, H::MatchType)>
        where P::Searcher: ReverseSearcher<H>
    {
        self.0.next_match_back().map(|(a, b)| unsafe {
            (H::offset_from_front(self.0.haystack(), a),
             H::range_to_self(self.0.haystack(), a, b))
        })
    }
}

generate_pattern_iterators! {
    forward:
        /// Created with the method [`match_indices()`].
        ///
        /// [`match_indices()`]: ../../std/primitive.str.html#method.match_indices
        struct MatchIndices;
    reverse:
        /// Created with the method [`rmatch_indices()`].
        ///
        /// [`rmatch_indices()`]: ../../std/primitive.str.html#method.rmatch_indices
        struct RMatchIndices;
    stability:
        //#[stable(feature = "str_match_indices", since = "1.5.0")]
    internal:
        MatchIndicesInternal() yielding ((usize, H::MatchType));
    delegate double ended;
}

///////////////////////////////////////////////////////////////////////////////
// .split()
///////////////////////////////////////////////////////////////////////////////

derive_pattern_clone!{
    clone SplitInternal
    with |s| SplitInternal { matcher: s.matcher.clone(), ..*s }
}

struct SplitInternal<H, P>
    where P: Pattern<H>,
          H: SearchCursors,
{
    start: H::Cursor,
    end: H::Cursor,
    matcher: P::Searcher,
    allow_trailing_empty: bool,
    finished: bool,
}

impl<H, P> fmt::Debug for SplitInternal<H, P>
    where P::Searcher: fmt::Debug,
          P: Pattern<H>,
          H: SearchCursors,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SplitInternal")
            /*
            TODO: How to handle this?
            .field("start", &self.start)
            .field("end", &self.end)
            .field("matcher", &self.matcher)
            */
            .field("allow_trailing_empty", &self.allow_trailing_empty)
            .field("finished", &self.finished)
            .finish()
    }
}

impl<H, P> SplitInternal<H, P>
    where P: Pattern<H>,
          H: SearchCursors,
{
    #[inline]
    fn new(h: H, p: P) -> Self {
        let matcher = p.into_searcher(h);
        let start = H::cursor_at_front(matcher.haystack());
        let end = H::cursor_at_back(matcher.haystack());

        SplitInternal {
            start: start,
            end: end,
            matcher: matcher,
            allow_trailing_empty: true,
            finished: false,
        }
    }

    #[inline]
    fn get_end(&mut self) -> Option<H::MatchType> {
        let h = self.matcher.haystack();
        let diff = |start: H::Cursor, end: H::Cursor| {
            H::cursor_diff(h, start, end)
        };
        if !self.finished && (self.allow_trailing_empty || diff(self.start, self.end) > 0) {
            self.finished = true;
            unsafe {
                let string = H::range_to_self(self.matcher.haystack(),
                                              self.start,
                                              self.end);
                Some(string)
            }
        } else {
            None
        }
    }

    #[inline]
    fn next(&mut self) -> Option<H::MatchType> {
        if self.finished { return None }

        match self.matcher.next_match() {
            Some((a, b)) => unsafe {
                let elt = H::range_to_self(self.matcher.haystack(),
                                           self.start,
                                           a);
                self.start = b;
                Some(elt)
            },
            None => self.get_end(),
        }
    }

    #[inline]
    fn next_back(&mut self) -> Option<H::MatchType>
        where P::Searcher: ReverseSearcher<H>
    {
        if self.finished { return None }

        if !self.allow_trailing_empty {
            self.allow_trailing_empty = true;
            match self.next_back() {
                Some(elt) => {
                    if H::match_type_len(&elt) > 0 {
                        return Some(elt)
                    } else if self.finished {
                        return None
                    }
                }
                _ => if self.finished { return None }
            }
        }

        match self.matcher.next_match_back() {
            Some((a, b)) => unsafe {
                let elt = H::range_to_self(self.matcher.haystack(),
                                           b,
                                           self.end);
                self.end = a;
                Some(elt)
            },
            None => unsafe {
                self.finished = true;
                Some(H::range_to_self(self.matcher.haystack(),
                                      self.start,
                                      self.end))
            },
        }
    }
}

generate_pattern_iterators! {
    forward:
        /// Created with the method [`split()`].
        ///
        /// [`split()`]: ../../std/primitive.str.html#method.split
        struct Split;
    reverse:
        /// Created with the method [`rsplit()`].
        ///
        /// [`rsplit()`]: ../../std/primitive.str.html#method.rsplit
        struct RSplit;
    stability:
        //#[stable(feature = "rust1", since = "1.0.0")]
    internal:
        SplitInternal() yielding (H::MatchType);
    delegate double ended;
}

///////////////////////////////////////////////////////////////////////////////
// .split_terminator()
///////////////////////////////////////////////////////////////////////////////

derive_pattern_clone!{
    clone SplitTerminatorInternal
    with |s| SplitTerminatorInternal(s.0.clone())
}

struct SplitTerminatorInternal<H, P>(SplitInternal<H, P>)
    where P: Pattern<H>,
          H: SearchCursors;

impl<H, P> fmt::Debug for SplitTerminatorInternal<H, P>
    where P::Searcher: fmt::Debug,
          P: Pattern<H>,
          H: SearchCursors,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<H, P> SplitTerminatorInternal<H, P>
    where P: Pattern<H>,
          H: SearchCursors,
{
    #[inline]
    fn new(h: H, p: P) -> Self {
        let mut s = SplitInternal::new(h, p);
        s.allow_trailing_empty = false;
        SplitTerminatorInternal(s)
    }

    #[inline]
    fn next(&mut self) -> Option<H::MatchType> {
        self.0.next()
    }

    #[inline]
    fn next_back(&mut self) -> Option<H::MatchType>
        where P::Searcher: ReverseSearcher<H>
    {
        self.0.next_back()
    }
}

generate_pattern_iterators! {
    forward:
        /// Created with the method [`split_terminator()`].
        ///
        /// [`split_terminator()`]: ../../std/primitive.str.html#method.split_terminator
        struct SplitTerminator;
    reverse:
        /// Created with the method [`rsplit_terminator()`].
        ///
        /// [`rsplit_terminator()`]: ../../std/primitive.str.html#method.rsplit_terminator
        struct RSplitTerminator;
    stability:
        //#[stable(feature = "rust1", since = "1.0.0")]
    internal:
        SplitTerminatorInternal() yielding (H::MatchType);
    delegate double ended;
}

///////////////////////////////////////////////////////////////////////////////
// .splitn()
///////////////////////////////////////////////////////////////////////////////

derive_pattern_clone!{
    clone SplitNInternal
    with |s| SplitNInternal { iter: s.iter.clone(), ..*s }
}

struct SplitNInternal<H, P>
    where P: Pattern<H>,
          H: SearchCursors,
{
    iter: SplitInternal<H, P>,
    /// The number of splits remaining
    count: usize,
}

impl<H, P> fmt::Debug for SplitNInternal<H, P>
    where P: Pattern<H>,
          H: SearchCursors,
          P::Searcher: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SplitNInternal")
            .field("iter", &self.iter)
            .field("count", &self.count)
            .finish()
    }
}

impl<H, P> SplitNInternal<H, P>
    where P: Pattern<H>,
          H: SearchCursors,
{
    #[inline]
    fn new(h: H, p: P, count: usize) -> Self {
        SplitNInternal {
            iter: SplitInternal::new(h, p),
            count: count
        }
    }

    #[inline]
    fn next(&mut self) -> Option<H::MatchType> {
        match self.count {
            0 => None,
            1 => { self.count = 0; self.iter.get_end() }
            _ => { self.count -= 1; self.iter.next() }
        }
    }

    #[inline]
    fn next_back(&mut self) -> Option<H::MatchType>
        where P::Searcher: ReverseSearcher<H>
    {
        match self.count {
            0 => None,
            1 => { self.count = 0; self.iter.get_end() }
            _ => { self.count -= 1; self.iter.next_back() }
        }
    }
}

generate_pattern_iterators! {
    forward:
        /// Created with the method [`splitn()`].
        ///
        /// [`splitn()`]: ../../std/primitive.str.html#method.splitn
        struct SplitN;
    reverse:
        /// Created with the method [`rsplitn()`].
        ///
        /// [`rsplitn()`]: ../../std/primitive.str.html#method.rsplitn
        struct RSplitN;
    stability:
        //#[stable(feature = "rust1", since = "1.0.0")]
    internal:
        SplitNInternal(count: usize) yielding (H::MatchType);
    delegate single ended;
}
