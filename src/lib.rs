#![crate_name="rumblebars"]

#![cfg_attr(feature = "nightly", feature(test))]
#![cfg_attr(not(feature = "with-syntex"), feature(plugin))]
#![cfg_attr(not(feature = "with-syntex"), plugin(rustlex))]

#[cfg(feature = "with-syntex")] extern crate rustlex_codegen as rustlex;
#[cfg(not(feature = "with-syntex"))] #[warn(plugin_as_library)] extern crate rustlex;

mod parse {
  #[cfg(not(feature = "with-syntex"))] include!("parse.lex.rs");
  #[cfg(feature = "with-syntex")] include!(concat!(env!("OUT_DIR"), "/parse.rs"));
}


extern crate regex;
extern crate rustc_serialize as serialize;


#[cfg(feature = "nightly")] extern crate test;

#[macro_use]
extern crate lazy_static;


pub use self::parse::parse;
pub use self::parse::ParseError;
pub use self::parse::Template;
pub use self::parse::HBValHolder;
pub use self::eval::eval;
pub use self::eval::HBData;
pub use self::eval::HBEvalResult;
pub use self::eval::EvalContext;
pub use self::eval::Helper;
pub use self::eval::HelperOptions;
pub use self::eval::HelperOptionsByName;
pub use self::eval::SafeWriting;

mod eval;
mod helpers_builtins;

