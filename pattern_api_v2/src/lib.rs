#![feature(str_char)]

#[macro_use]
mod macros;

// TODO
// Name: Maybe SequencePattern ?
// Replace Iter structs with lib std slice iterators (need ptr access)

pub mod fast_sequence_search;

pub mod core_traits;

pub mod string;
pub mod slice;
pub mod os_string;

pub mod iterators;

pub mod experimental;

pub use core_traits::*;
