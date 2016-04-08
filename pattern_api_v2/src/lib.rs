#![feature(str_char)]
#![feature(specialization)]
#![feature(associated_type_defaults)]
#![feature(osstring_simple_functions)]
#![feature(decode_utf16)]
#![feature(unicode)]
#![feature(stmt_expr_attributes)]
#![feature(inclusive_range_syntax)]
#![feature(range_contains)]

#[macro_use]
mod macros;

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
