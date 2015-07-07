//! [![Build Status](https://travis-ci.org/nicolas-cherel/rumblebars.svg?branch=master)](https://travis-ci.org/nicolas-cherel/rumblebars)
//!
//! # rumblebars — a handlebars template expansion library
//!
//! This crates provides a library for parsing and expanding handlebars template
//!
//! to use with rust nighly feature build with ```cargo build --features nightly --no-default-features```

//!
//! Rumblebars passes **all mustaches specs** [[1]](#1) and **272 handlebars tests** [[2]](#2). Template evaluation is rendered to a `io::Writer`, so that you can choose wether if you hold result in memory or not. It also input data angostic, given that your data structure implements the `HBData` trait (Json implementation provided).
//!
//!  [1] <a name="1"></a> except delimiter changes test suite and one test failing because of a trailing space
//!  [2] <a name="2"></a> all tests that does not involves javascript in data and partials, and see the [comments for other cases](https://github.com/nicolas-cherel/rumblebars/blob/master/tests/eval/handlebars.rs#L88-L134)
//!
//! ## HMTL escaping safety
//!
//! All output is filtered by being written to the `SafeWriting` trait. Helpers, just as regular evaluation do for unescaped content, have to opt out escaped writing by calling `SafeWriting::into_unsafe()` that will return the underlying unfiltered writer.
//!
//! ## Quick start
//!
//! # Examples
//!
//! The API shortcuts are provided by object oriented
//! code design.
//!
//! You can render directly into a String :
//!
//! ```
//! extern crate rustc_serialize as serialize;
//! extern crate rumblebars;
//! # fn main() {
//! use serialize::json::Json;
//! use rumblebars::Template;
//!
//! let data = Json::from_str(r##"{"hello": "hi"}"##).unwrap();
//!
//! if let Ok(template) = Template::new("{{hello}}") {
//!   let res = template.eval_to_string(&data).unwrap_or("".to_string());
//!   assert_eq!(&res, "hi");
//! }
//! # else { panic!("should not reach") }
//! # }
//! ```
//!
//! For template parsing, you can also use rust's usual patterns that leverage type inference :
//!
//! ```
//! use rumblebars::Template;
//! let template: Template = "{{hello}}".parse().unwrap(); // thanks to FromStr
//! ```
//!
//! You can just call the `parse()` function too :
//!
//! ```
//! rumblebars::parse("{{hello}}").unwrap();
//! ```
//!
//! Same with `eval()` :
//!
//! ```
//! extern crate rustc_serialize as serialize;
//! extern crate rumblebars;
//! # fn main() {
//! use serialize::json::Json;
//! use rumblebars::{EvalContext};
//!
//! let mut out = Vec::new();
//! rumblebars::eval(&rumblebars::parse("{{hello}}").unwrap(), &Json::Null, &mut out, &EvalContext::new());
//! # }
//! ```
//!
//! ## helpers
//!
//! Helpers are registered to the evaluation context. They are boxed closures (you can hold bare function in them too) that have to write their content on the `out: &mut Writer`. If you need to processed content before rendering it to the final `Writer`, just render it to a buffer put into a safe writter.
//!
//! To use your hepler you just have to register it before evaluating your template:
//!
//! You can control the EvalContext (for custom helpers) and output using `eval()`
//!
//! ```
//! use rumblebars::{Template, HBData, EvalContext};
//!
//! if let Ok(template) = Template::new("{{hello}}") {
//!   let mut context = EvalContext::new();
//!   let mut buf = Vec::new();
//!
//!   context.register_helper("hello".to_string(), Box::new(
//!     |params, options, out, hb_context| {
//!       "hi".write_value(out)
//!   }));
//!
//!   if let Ok(_) = template.eval(&"", &mut buf, &context) {
//!      assert_eq!(String::from_utf8_lossy(&buf), "hi");
//!   }
//! }
//! # else { panic!("should not reach") }
//! ```
//!

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
pub use self::eval::HTMLSafeWriter;

mod eval;
mod helpers_builtins;

