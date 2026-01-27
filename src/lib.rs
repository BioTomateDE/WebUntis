//! TODO:  documentation

#![deny(unexpected_cfgs)]
//
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
//
// https://github.com/rust-lang/rust-clippy/issues/16440
#![allow(clippy::multiple_crate_versions)]

pub mod api;
mod json_util;
