//! # ql-quotes
//!
//! Market quotes and observable values for quantlib-rs.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

/// `Quote` trait and concrete implementations.
pub mod quote;

pub use quote::{
    CompositeQuote, DerivedQuote, ForwardValueQuote, ImpliedStdDevQuote, LastFixingQuote,
    NegativeQuote, Quote, SimpleQuote,
};
