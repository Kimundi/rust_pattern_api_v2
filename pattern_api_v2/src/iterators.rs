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
            $internal_iterator:ident yielding ($iterty:ty);

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
            pub fn new(h: H, p: P) -> Self {
                $forward_iterator($internal_iterator::new(h, p))
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
            pub fn new(h: H, p: P) -> Self {
                $reverse_iterator($internal_iterator::new(h, p))
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
    fn next(&mut self) -> Option<H> {
        self.0.next_match().map(|(a, b)| unsafe {
            H::range_to_self(self.0.haystack(), a, b)
        })
    }

    #[inline]
    fn next_back(&mut self) -> Option<H>
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
        MatchesInternal yielding (H);
    delegate double ended;
}
