extern crate rumblebars;
extern crate "rustc-serialize" as serialize;

use serialize::json::Json;
use std::default::Default;

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

  assert_eq!(String::from_utf8(buf).unwrap(), "that poney has something sad in its eye".to_string());
}

#[test]
fn simple_render_with_raw() {
  let json = Json::from_str(r##"{"p": "that poney has something sad in its eye"}"##).ok().unwrap();
  let tmpl = parse(r##"prelude {{p}} post"##).ok().unwrap();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &Default::default()).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "prelude that poney has something sad in its eye post".to_string());
}

#[test]
fn simple_render_with_block() {
  let json = Json::from_str(r##"{"p": { "k": "that poney has something sad in its eye"}}"##).ok().unwrap();
  let tmpl = parse(r##"prelude {{#p}}{{k}}{{/p}} post"##).ok().unwrap();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &Default::default()).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "prelude that poney has something sad in its eye post".to_string());
}

#[test]
fn iteration_with_block() {
  let json = Json::from_str(r##"{"p": [ 1, 2, 3, 4]}"##).ok().unwrap();
  let tmpl = parse(r##"{{#p}}{{.}}{{/p}}"##).ok().unwrap();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &Default::default()).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "1234".to_string());
}

#[test]
fn iteration_with_rich_block() {
  let json = Json::from_str(r##"{"p": [ {"i": 1, "j": "a"}, {"i": 2, "j": "b"}, {"i": 3, "j": "c"}, {"i": 4, "j": "d"}]}"##).ok().unwrap();
  let tmpl = parse(r##"{{#p}}{{i}}({{j}}){{/p}}"##).ok().unwrap();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &Default::default()).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "1(a)2(b)3(c)4(d)".to_string());
}

#[test]
fn parent_key() {
  let json = Json::from_str(r##"{"a": {"b": "bb"}, "c": "ccc"}"##).ok().unwrap();
  let tmpl = parse(r##"{{#a}}{{b}}{{../c}}{{/a}}"##).ok().unwrap();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &Default::default()).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "bbccc".to_string());
}

#[test]
fn iteration_with_block_and_parent_key() {
  let json = Json::from_str(r##"{"p": [ 1,  2,  3,  4], "b": "-"}"##).ok().unwrap();
  let tmpl = parse(r##"{{#p}}{{.}}{{../b}}{{/p}}"##).ok().unwrap();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &Default::default()).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "1-2-3-4-".to_string());
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

  assert_eq!(String::from_utf8(buf).unwrap(), "found this data".to_string()); 
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

  assert_eq!(String::from_utf8(buf).unwrap(), "found this data and yep, was found i yep, was found j yep, was found k ".to_string()); 
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

  assert_eq!(String::from_utf8(buf).unwrap(), "from p eval".to_string()); 

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

  assert_eq!(String::from_utf8(buf).unwrap(), "pouet".to_string()); 

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

  assert_eq!(String::from_utf8(buf).unwrap(), "value : toto".to_string()); 
}

fn cd(_: &[&HBData], options: &HelperOptions, out: &mut Writer, _: &EvalContext) -> HBEvalResult {
  if options.condition {
    options.block_ok(out)
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

  assert_eq!(String::from_utf8(buf).unwrap(), "value : p true z false".to_string()); 
}

#[test]
fn leading_whitespace() {
  let json = Json::from_str(r##"{"p": {}}"##).ok().unwrap();
  let tmpl = parse(r##"{{~#p}}

      Pouet

      {{/p}}"##).ok().unwrap();
  let mut eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  let expected = r##"Pouet

      "##;

  assert_eq!(String::from_utf8(buf).unwrap(), expected.to_string()); 
}


#[test]
fn trailing_whitespace() {
  let json = Json::from_str(r##"{"p": {}}"##).ok().unwrap();
  let tmpl = parse(r##"{{#p~}}

      Pouet

      {{/p}}"##).ok().unwrap();
  let mut eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  let expected = r##"

      Pouet"##;

  assert_eq!(String::from_utf8(buf).unwrap(), expected.to_string()); 
}

#[test]
fn both_whitespace() {
  let json = Json::from_str(r##"{"p": {}}"##).ok().unwrap();
  let tmpl = parse(r##"{{~#p~}}

      Pouet pouet

      {{/p}}"##).ok().unwrap();
  let mut eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  let expected = r##"Pouet pouet"##;

  assert_eq!(String::from_utf8(buf).unwrap(), expected.to_string()); 
}


#[test]
fn nested_whitespace() {
  let json = Json::from_str(r##"{"p": {"u": {}}}"##).ok().unwrap();
  let tmpl = parse(r##"{{~#p~}}

      {{#u~}} Uuuuu {{/u~}}
      ooOOOO
      {{/p}}"##).ok().unwrap();
  let mut eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  let expected = r##" Uuuuu
      ooOOOO"##;

  assert_eq!(String::from_utf8(buf).unwrap(), expected.to_string()); 
}

#[test]
fn autotrim() {
  {
    let json = Json::from_str(r##"{"p": {}}"##).ok().unwrap();
    let tmpl = parse(r##"
      {{#p}}    
        o
  
      {{else}}
      {{/p}}
    "##).ok().unwrap();
  
    let mut eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();
  
    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();
  
    let expected = r##"
        o
  
    "##;
  
    assert_eq!(String::from_utf8(buf).unwrap(), expected.to_string()); 
  }

  {
    let json = Json::from_str(r##"{"p": {"u": {}}}"##).ok().unwrap();
    let tmpl = parse(r##"
      {{#p}}    
        o
        {{#u}}{{/u}}

        {{#u}}
        uU
        {{/u}}
  
      {{else}}
      {{/p}}
    "##).ok().unwrap();
  
    let mut eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();
  
    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();
  
    let expected = r##"
        o
        

        uU
  
    "##;
  
    assert_eq!(String::from_utf8(buf).unwrap(), expected.to_string()); 
  }

  {
    let json = Json::from_str(r##"{"p": {}}"##).ok().unwrap();
    let tmpl = parse(r##"
      {{#p}}i    
        o

  
      {{else}}o
      {{/p}}
    "##).ok().unwrap();
  
    let mut eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();
  
    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();
  
    let expected = r##"
      i    
        o

  
      
    "##;
  
    assert_eq!(String::from_utf8(buf).unwrap(), expected.to_string()); 
  }
}





