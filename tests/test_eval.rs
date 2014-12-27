extern crate rumblebars;
extern crate serialize; 

use serialize::json;

use rumblebars::eval;
use rumblebars::parse;

#[test]
fn simple_render() {
  let json = json::from_str(r##"{"p": "that poney has something sad in its eye"}"##).unwrap();
  let tmpl = parse(r##"{{p}}"##).unwrap();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "that poney has something sad in its eye".to_string());
}

#[test]
fn simple_render_with_raw() {
  let json = json::from_str(r##"{"p": "that poney has something sad in its eye"}"##).unwrap();
  let tmpl = parse(r##"prelude {{p}} post"##).unwrap();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "prelude that poney has something sad in its eye post".to_string());
}

#[test]
fn simple_render_with_block() {
  let json = json::from_str(r##"{"p": { "k": "that poney has something sad in its eye"}}"##).unwrap();
  let tmpl = parse(r##"prelude {{#p}}{{k}}{{/p}} post"##).unwrap();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "prelude that poney has something sad in its eye post".to_string());
}

#[test]
fn iteration_with_block() {
  let json = json::from_str(r##"{"p": [{"i": 1}, {"i": 2}, {"i": 3}, {"i": 4}]}"##).unwrap();
  let tmpl = parse(r##"{{#p}}{{i}}{{/p}}"##).unwrap();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "1234".to_string());
}