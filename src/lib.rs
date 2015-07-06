//! This crates provides a library for parsing and expanding handlebars template
//! to use with rust nighly feature build with ```cargo build --features nightly --no-default-features```

#![crate_name="rumblebars"]

#![cfg_attr(feature = "nightly", feature(test))]
#![cfg_attr(feature = "nightly", feature(plugin))]
#![cfg_attr(feature = "nightly", plugin(rustlex))]

#[cfg(feature = "with-syntex")] extern crate rustlex_codegen as rustlex;
#[cfg(feature = "nightly")] #[warn(plugin_as_library)] extern crate rustlex;

#[cfg(feature = "with-syntex")] mod parse { include!(concat!(env!("OUT_DIR"), "/parse.rs")); }
#[cfg(feature = "nightly")] mod parse;


extern crate regex;
extern crate rustc_serialize as serialize;


#[cfg(feature = "nightly")] extern crate test;

#[macro_use]
extern crate lazy_static;

pub use self::parse::parse;
pub use self::parse::ParseError;
pub use self::parse::Template;
pub use self::eval::eval;
pub use self::eval::HBData;
pub use self::eval::HBIter;
pub use self::eval::HBKeysIter;
pub use self::eval::HBValuesIter;
pub use self::eval::HBEvalResult;
pub use self::eval::EvalContext;
pub use self::eval::HelperOptions;
pub use self::eval::HelperOptionsByName;
pub use self::eval::SafeWriting;

mod eval;
mod helpers_builtins;

