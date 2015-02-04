extern crate rumblebars;
extern crate "rustc-serialize" as serialize;

use serialize::json::Json;
use std::default::Default;

use rumblebars::EvalContext;
use rumblebars::parse;
use rumblebars::eval;

#[test]
fn helper_if_true() {
  {
    let json = Json::from_str(r##"true"##).ok().unwrap();
    let tmpl = parse(r##"{{#if .}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let mut eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ok");
  }
  {
    let json = Json::from_str(r##"{}"##).ok().unwrap();
    let tmpl = parse(r##"{{#if .}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let mut eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ok");
  }
  {
    let json = Json::from_str(r##""any""##).ok().unwrap();
    let tmpl = parse(r##"{{#if .}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let mut eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ok");
  }
  {
    let json = Json::from_str(r##"[]"##).ok().unwrap();
    let tmpl = parse(r##"{{#if .}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let mut eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ok");
  }
  {
    let json = Json::from_str(r##"1"##).ok().unwrap();
    let tmpl = parse(r##"{{#if .}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let mut eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ok");
  }
  {
    let json = Json::from_str(r##"{"p": true}"##).ok().unwrap();
    let tmpl = parse(r##"{{#if p}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let mut eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ok");
  }
  {
    let json = Json::from_str(r##"{"p": {"q": true}}"##).ok().unwrap();
    let tmpl = parse(r##"{{#if p.q}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let mut eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ok");
  }
}

#[test]
fn helper_if_false() {
  {
    let json = Json::from_str(r##"false"##).ok().unwrap();
    let tmpl = parse(r##"{{#if .}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let mut eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ko");
  }
  {
    let json = Json::from_str(r##"{}"##).ok().unwrap();
    let tmpl = parse(r##"{{#if k}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let mut eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ko");
  }
  {
    let json = Json::from_str(r##""""##).ok().unwrap();
    let tmpl = parse(r##"{{#if .}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let mut eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ko");
  }
  {
    let json = Json::from_str(r##"[]"##).ok().unwrap();
    let tmpl = parse(r##"{{#if 1}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let mut eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ko");
  }
  {
    let json = Json::from_str(r##"0"##).ok().unwrap();
    let tmpl = parse(r##"{{#if .}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let mut eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ko");
  }
  {
    let json = Json::from_str(r##"{"p": false}"##).ok().unwrap();
    let tmpl = parse(r##"{{#if p}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let mut eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ko");
  }
  {
    let json = Json::from_str(r##"{"p": {"q": false}}"##).ok().unwrap();
    let tmpl = parse(r##"{{#if p.q}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let mut eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ko");
  }
}

#[test]
fn helper_each_index() {
  let json = Json::from_str(r##"["zero", "one", "two", "three"]"##).ok().unwrap();
  let tmpl = parse(r##"{{#each this}}{{@index}}:{{.}} {{/each}}"##).ok().unwrap();
  let mut eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "0:zero 1:one 2:two 3:three ");
}

#[test]
fn helper_each_first() {
  let json = Json::from_str(r##"["zero", "one", "two", "three"]"##).ok().unwrap();
  let tmpl = parse(r##"{{#each this}}{{#if @first}}{{.}}{{/if}}{{/each}}"##).ok().unwrap();
  let mut eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "zero");
}

#[test]
fn helper_each_last() {
  let json = Json::from_str(r##"["zero", "one", "two", "three"]"##).ok().unwrap();
  let tmpl = parse(r##"{{#each this}}{{#if @last}}{{.}}{{/if}}{{/each}}"##).ok().unwrap();
  let mut eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "three");
}

#[test]
fn helper_each_keys() {
  let json = Json::from_str(r##"[{"one": 1}, {"two": 2}, {"three": 3}]"##).ok().unwrap();
  let tmpl = parse(r##"{{#this}}{{#each this}}{{@key}}:{{.}} {{/each}}{{/this}}"##).ok().unwrap();
  let mut eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "one:1 two:2 three:3 ");
}

#[test]
fn helper_lookup() {
  let json = Json::from_str(r##"{"t": {"j": "../u"}, "u": "u content"}"##).ok().unwrap();
  let tmpl = parse(r##"{{#t}}{{lookup j}}{{/t}}"##).ok().unwrap();
  let mut eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "u content");
}
