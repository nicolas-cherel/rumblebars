#![crate_name="rumblebars"]
#![feature(plugin)]
#![feature(std_misc)]
#![feature(collections)]
#![plugin(regex_macros)]
#![plugin(rustlex)]
#![feature(test)]

#![feature(str_char)]

// for rustlex
#![feature(box_syntax)]

extern crate regex;
extern crate rustlex;
extern crate "rustc-serialize" as serialize;

extern crate test;


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

mod parse;
mod eval;
mod helpers_builtins;

