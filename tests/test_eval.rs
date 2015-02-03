extern crate rumblebars;
extern crate "rustc-serialize" as serialize;

use serialize::json::Json;
use std::default::Default;
use std::collections::HashMap;

use rumblebars::eval;
use rumblebars::EvalContext;
use rumblebars::Helper;
use rumblebars::HelperOptions;
use rumblebars::HBData;
use rumblebars::HBEvalResult;
use rumblebars::parse;

#[test]
fn simple_render() {
  let json = Json::from_str(r##"{"p": "that poney has something sad in its eye"}"##).ok().unwrap();
  let tmpl = parse(r##"{{p}}"##).ok().unwrap();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &Default::default()).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "that poney has something sad in its eye");
}

#[test]
fn simple_render_with_raw() {
  let json = Json::from_str(r##"{"p": "that poney has something sad in its eye"}"##).ok().unwrap();
  let tmpl = parse(r##"prelude {{p}} post"##).ok().unwrap();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &Default::default()).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "prelude that poney has something sad in its eye post");
}

#[test]
fn simple_render_with_block() {
  let json = Json::from_str(r##"{"p": { "k": "that poney has something sad in its eye"}}"##).ok().unwrap();
  let tmpl = parse(r##"prelude {{#p}}{{k}}{{/p}} post"##).ok().unwrap();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &Default::default()).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "prelude that poney has something sad in its eye post");
}

#[test]
fn iteration_with_block() {
  let json = Json::from_str(r##"{"p": [ 1, 2, 3, 4]}"##).ok().unwrap();
  let tmpl = parse(r##"{{#p}}{{.}}{{/p}}"##).ok().unwrap();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &Default::default()).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "1234");
}

#[test]
fn iteration_with_rich_block() {
  let json = Json::from_str(r##"{"p": [ {"i": 1, "j": "a"}, {"i": 2, "j": "b"}, {"i": 3, "j": "c"}, {"i": 4, "j": "d"}]}"##).ok().unwrap();
  let tmpl = parse(r##"{{#p}}{{i}}({{j}}){{/p}}"##).ok().unwrap();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &Default::default()).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "1(a)2(b)3(c)4(d)");
}

#[test]
fn parent_key() {
  let json = Json::from_str(r##"{"a": {"b": "bb"}, "c": "ccc"}"##).ok().unwrap();
  let tmpl = parse(r##"{{#a}}{{b}}{{../c}}{{/a}}"##).ok().unwrap();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &Default::default()).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "bbccc");
}

#[test]
fn iteration_with_block_and_parent_key() {
  let json = Json::from_str(r##"{"p": [ 1,  2,  3,  4], "b": "-"}"##).ok().unwrap();
  let tmpl = parse(r##"{{#p}}{{.}}{{../b}}{{/p}}"##).ok().unwrap();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &Default::default()).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "1-2-3-4-");
}

#[test]
fn partial() {
  let json = Json::from_str(r##"{"a": "data"}"##).ok().unwrap();
  let tmpl = parse(r##"{{>test}}"##).ok().unwrap();
  let partial = parse(r##"found this {{a}}"##).ok().unwrap();
  let mut eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval_ctxt.register_partial("test".to_string(), partial);

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "found this data");
}

#[test]
fn partial_block() {
  let json = Json::from_str(r##"{"a": "data", "b": ["i", "j", "k"]}"##).ok().unwrap();
  let tmpl = parse(r##"{{>test}} and {{#b}}{{>check}}{{/b}}"##).ok().unwrap();
  let partial = parse(r##"found this {{a}}"##).ok().unwrap();
  let partial_check = parse(r##"yep, was found {{.}} "##).ok().unwrap();
  let mut eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval_ctxt.register_partial("test".to_string(), partial);
  eval_ctxt.register_partial("check".to_string(), partial_check);

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "found this data and yep, was found i yep, was found j yep, was found k ");
}

 #[allow(unused_variables)]
fn p(params: &[&HBData], options: &HelperOptions, out: &mut Writer, hb_context: &EvalContext) -> HBEvalResult {
  write!(out, "from p eval")
}

#[test]
fn helper() {
  let json = Json::from_str(r##""""##).ok().unwrap();
  let tmpl = parse(r##"{{p}}"##).ok().unwrap();
  let mut eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval_ctxt.register_helper("p".to_string(), Helper::new_with_function(p));

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "from p eval");

}

#[allow(unused_variables)]
fn c(params: &[&HBData], options: &HelperOptions, out: &mut Writer, hb_context: &EvalContext) -> HBEvalResult {
  match params {
    [param, ..] => param.write_value(out),
    _ => Ok(()),
  }
}

#[test]
fn helper_context() {
  let json = Json::from_str(r##""""##).ok().unwrap();
  let tmpl = parse(r##"{{c "pouet"}}"##).ok().unwrap();
  let mut eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval_ctxt.register_helper("c".to_string(), Helper::new_with_function(c));

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "pouet");

}

#[allow(unused_variables)]
fn v(params: &[&HBData], options: &HelperOptions, out: &mut Writer, hb_context: &EvalContext) -> HBEvalResult {
  match params {
    [v] => v.write_value(out),
    _ => write!(out, "failed…"),
  }
}

#[test]
fn helper_val() {
  let json = Json::from_str(r##""""##).ok().unwrap();
  let tmpl = parse(r##"value : {{v "toto"}}"##).ok().unwrap();
  let mut eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval_ctxt.register_helper("v".to_string(), Helper::new_with_function(v));

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "value : toto");
}

fn cd(_: &[&HBData], options: &HelperOptions, out: &mut Writer, _: &EvalContext) -> HBEvalResult {
  if options.condition {
    options.render_fn(out)
  } else {
    options.inverse(out)
  }
}

#[test]
fn helper_cond() {
  let json = Json::from_str(r##"{"p": true, "z": false, "r": "rumble"}"##).ok().unwrap();
  let tmpl = parse(r##"value : {{#if p}}p true{{else}}p false{{/if}} {{#if z}}z true{{else}}z false{{/if}}"##).ok().unwrap();
  let mut eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval_ctxt.register_helper("if".to_string(), Helper::new_with_function(cd));

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "value : p true z false");
}

fn globs(_: &[&HBData], options: &HelperOptions, out: &mut Writer, _: &EvalContext) -> HBEvalResult {
  let val = "stored value".to_string();
  let mut vars = HashMap::new();
  vars.insert("@val", &val as &HBData);
  options.render_fn_with_globals(out, &vars)
}

#[test]
fn helper_globals() {
  let json = Json::from_str(r##"{}"##).ok().unwrap();
  let tmpl = parse(r##"value : {{#globs p}}{{@val}}{{/globs}}"##).ok().unwrap();
  let mut eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval_ctxt.register_helper("globs".to_string(), Helper::new_with_function(globs));

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "value : stored value");
}

#[test]
fn leading_whitespace() {
  let json = Json::from_str(r##"{"p": {}}"##).ok().unwrap();
  let tmpl = parse(r##"{{~#p}}

      Pouet

      {{/p}}"##).ok().unwrap();
  let eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  let expected = r##"Pouet

      "##;

  assert_eq!(String::from_utf8(buf).unwrap(), expected);
}


#[test]
fn trailing_whitespace() {
  let json = Json::from_str(r##"{"p": {}}"##).ok().unwrap();
  let tmpl = parse(r##"{{#p~}}

      Pouet

      {{/p}}"##).ok().unwrap();
  let eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  let expected = r##"

      Pouet"##;

  assert_eq!(String::from_utf8(buf).unwrap(), expected);
}

#[test]
fn both_whitespace() {
  let json = Json::from_str(r##"{"p": {}}"##).ok().unwrap();
  let tmpl = parse(r##"{{~#p~}}

      Pouet pouet

      {{/p}}"##).ok().unwrap();
  let eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  let expected = r##"Pouet pouet"##;

  assert_eq!(String::from_utf8(buf).unwrap(), expected);
}


#[test]
fn nested_whitespace() {
  let json = Json::from_str(r##"{"p": {"u": {}}}"##).ok().unwrap();
  let tmpl = parse(r##"{{~#p~}}

      {{#u~}} Uuuuu {{/u~}}
      ooOOOO
      {{/p}}"##).ok().unwrap();
  let eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  let expected = r##" Uuuuu
      ooOOOO"##;

  assert_eq!(String::from_utf8(buf).unwrap(), expected);
}

#[test]
fn autotrim() {
  {
    let json = Json::from_str(r##"{"p": {}}"##).ok().unwrap();
    let tmpl = parse(r##"
      1)
      {{#p}}
        o

      {{else}}
      {{/p}}
    "##).ok().unwrap();

    let eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    let expected = r##"
      1)
        o

    "##;

    assert_eq!(String::from_utf8(buf).unwrap(), expected);
  }

  {
    let json = Json::from_str(r##"{"p": {"u": {}}}"##).ok().unwrap();
    let tmpl = parse(r##"
      2)
      {{#p}}
        o
        {{#u}}{{/u}}

        {{#u}}
        uU
        {{/u}}

      {{else}}
      {{/p}}
    "##).ok().unwrap();

    let eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    let expected = "\n      2)\n        o\n        \n\n        uU\n\n    "; // one line string due to trailing whitespace
    assert_eq!(String::from_utf8(buf).unwrap(), expected);
  }

  {
    let json = Json::from_str(r##"{"p": {}}"##).ok().unwrap();
    let tmpl = parse(r##"
      3)
      {{#p}}i
        o


      {{else}}o
      {{/p}}
    "##).ok().unwrap();

    let eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    let expected = "\n      3)\n      i\n        o\n\n\n      \n    "; // one line string due to trailing whitespace
    assert_eq!(String::from_utf8(buf).unwrap(), expected);
  }
}





