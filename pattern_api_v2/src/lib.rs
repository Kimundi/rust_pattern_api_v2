#![feature(str_char)]
#![feature(specialization)]
#![feature(associated_type_defaults)]
#![feature(osstring_simple_functions)]
#![feature(decode_utf16)]
#![feature(unicode)]

#[macro_use]
mod macros;

/* TODO & Notes
    - Name: Maybe SequencePattern ?
    - Name: rename OrdSlice/SearchCursors to something like PatternHaystack
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
    - OsStr iteration! Right now each position is valid, which might be
      wrong for unicode sections in it. eg, split("")
    - TODO: Generic replace()

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

pub mod std_integration;

pub use core_traits::*;
