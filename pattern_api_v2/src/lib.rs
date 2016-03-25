#![feature(str_char)]
#![feature(specialization)]

#[macro_use]
mod macros;

/* TODO & Notes
    - Name: Maybe SequencePattern ?
    - Replace Iter structs with lib std slice iterators (need ptr access)
    - string stuff for [char]
    - osstring:
      - two views:
        - containing unicode sections, only working on them
        - being opaque, containing osstrings
        => probably two sets of methods, one for Pattern<&str> and one for
           Pattern<&OsStr>
           => can probably be Pattern<Self> in all cases?
      => Current solution: can support both, but need an
         unsafe trait to prevent "inverse" slicing operations like split()
         => maybe system to re-map to other type instead
            - example: &OsStr split on &str yielding &OsStr segments
            - though explicit remaping not really needed if normal
              splitting already returns the same values

*/

// TODO: This is mostly stolen from std::str
mod utf8;

pub mod fast_sequence_search;

pub mod core_traits;

pub mod string;
pub mod slice;
pub mod os_string;

pub mod iterators;

pub mod experimental;

pub use core_traits::*;
