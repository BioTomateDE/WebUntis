//! TODO:  documentation

#![deny(clippy::correctness)]
#![warn(
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::cargo,
    clippy::nursery,
    clippy::style,
    clippy::pedantic
)]
//
// https://github.com/rust-lang/rust-clippy/issues/16440
#![allow(clippy::multiple_crate_versions)]

pub mod api;
mod json_util;
