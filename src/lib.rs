#![crate_name="rumblebars"]

#![feature(phase)]
#[phase(plugin,link)] extern crate rustlex;
#[phase(plugin, link)] extern crate log;

extern crate serialize;

pub use self::parse::parse;
pub use self::parse::ParseError;
pub use self::parse::Template;
pub use self::eval::eval;
pub use self::eval::HBData;

mod parse;
mod eval;

