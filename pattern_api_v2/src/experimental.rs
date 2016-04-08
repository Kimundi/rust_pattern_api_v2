use super::*;

// TODO: Look at Quxxys usecase of recursively reusable patterns

pub struct SearcherPattern<H, P>(P::Searcher)
    where H: PatternHaystack,
        P: Pattern<H>;
