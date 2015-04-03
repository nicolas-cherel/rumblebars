
use serialize::json::Json;
use std::default::Default;
use std::io::Write;
use std::collections::HashMap;

use rumblebars::eval;
use rumblebars::parse;
use rumblebars::EvalContext;
use rumblebars::HelperOptions;
use rumblebars::HBEvalResult;
use rumblebars::SafeWriting;
use rumblebars::HBData;

#[test]
fn if_true() {
  {
    let json = Json::from_str(r##"true"##).ok().unwrap();
    let tmpl = parse(r##"{{#if .}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ok");
  }
  {
    let json = Json::from_str(r##"{}"##).ok().unwrap();
    let tmpl = parse(r##"{{#if .}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ok");
  }
  {
    let json = Json::from_str(r##""any""##).ok().unwrap();
    let tmpl = parse(r##"{{#if .}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ok");
  }
  {
    let json = Json::from_str(r##"[1]"##).ok().unwrap();
    let tmpl = parse(r##"{{#if .}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ok");
  }
  {
    let json = Json::from_str(r##"1"##).ok().unwrap();
    let tmpl = parse(r##"{{#if .}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ok");
  }
  {
    let json = Json::from_str(r##"{"p": true}"##).ok().unwrap();
    let tmpl = parse(r##"{{#if p}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ok");
  }
  {
    let json = Json::from_str(r##"{"p": {"q": true}}"##).ok().unwrap();
    let tmpl = parse(r##"{{#if p.q}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ok");
  }
}

#[test]
fn if_false() {
  {
    let json = Json::from_str(r##"false"##).ok().unwrap();
    let tmpl = parse(r##"{{#if .}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ko");
  }
  {
    let json = Json::from_str(r##"{}"##).ok().unwrap();
    let tmpl = parse(r##"{{#if k}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ko");
  }
  {
    let json = Json::from_str(r##""""##).ok().unwrap();
    let tmpl = parse(r##"{{#if .}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ko");
  }
  {
    let json = Json::from_str(r##"[]"##).ok().unwrap();
    let tmpl = parse(r##"{{#if .}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ko");
  }
  {
    let json = Json::from_str(r##"0"##).ok().unwrap();
    let tmpl = parse(r##"{{#if .}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ko");
  }
  {
    let json = Json::from_str(r##"{"p": false}"##).ok().unwrap();
    let tmpl = parse(r##"{{#if p}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ko");
  }
  {
    let json = Json::from_str(r##"{"p": {"q": false}}"##).ok().unwrap();
    let tmpl = parse(r##"{{#if p.q}}ok{{else}}ko{{/if}}"##).ok().unwrap();
    let eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "ko");
  }
}

#[test]
fn each_index() {
  let json = Json::from_str(r##"["zero", "one", "two", "three"]"##).ok().unwrap();
  let tmpl = parse(r##"{{#each this}}{{@index}}:{{.}} {{/each}}"##).ok().unwrap();
  let eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "0:zero 1:one 2:two 3:three ");
}

#[test]
fn each_first() {
  let json = Json::from_str(r##"["zero", "one", "two", "three"]"##).ok().unwrap();
  let tmpl = parse(r##"{{#each this}}{{#if @first}}{{.}}{{/if}}{{/each}}"##).ok().unwrap();
  let eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "zero");
}

#[test]
fn each_last() {
  let json = Json::from_str(r##"["zero", "one", "two", "three"]"##).ok().unwrap();
  let tmpl = parse(r##"{{#each this}}{{#if @last}}{{.}}{{/if}}{{/each}}"##).ok().unwrap();
  let eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "three");
}

#[test]
fn each_keys() {
  let json = Json::from_str(r##"[{"one": 1}, {"two": 2}, {"three": 3}]"##).ok().unwrap();
  let tmpl = parse(r##"{{#this}}{{#each this}}{{@key}}:{{.}} {{/each}}{{/this}}"##).ok().unwrap();
  let eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "one:1 two:2 three:3 ");
}

#[test]
fn lookup() {
  let json = Json::from_str(r##"{"t": {"j": "../u"}, "u": "u content"}"##).ok().unwrap();
  let tmpl = parse(r##"{{#t}}{{lookup j}}{{/t}}"##).ok().unwrap();
  let eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "u content");
}

#[test]
fn lookup_with_context() {
  let json = Json::from_str(r##"{"t": {"j": "u.v"}, "u": {"v": "v content"}}"##).ok().unwrap();
  let tmpl = parse(r##"{{#t}}path is {{j}} : {{lookup @root j}}{{/t}}"##).ok().unwrap();
  let eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "path is u.v : v content");
}

#[test]
fn with() {
  let json = Json::from_str(r##"{"t": {"j": "result"}}"##).ok().unwrap();
  let tmpl = parse(r##"{{#with t}}{{j}}{{/with}}"##).ok().unwrap();
  let eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "result");
}


 #[allow(unused_variables)]
fn p(params: &[&HBData], options: &HelperOptions, out: &mut SafeWriting, hb_context: &EvalContext) -> HBEvalResult {
  write!(out, "from p eval")
}

#[test]
fn helper() {
  let json = Json::from_str(r##""""##).ok().unwrap();
  let tmpl = parse(r##"{{p}}"##).ok().unwrap();
  let mut eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval_ctxt.register_helper("p".to_string(), p);

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "from p eval");

}

#[allow(unused_variables)]
fn c(params: &[&HBData], options: &HelperOptions, out: &mut SafeWriting, hb_context: &EvalContext) -> HBEvalResult {
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

  eval_ctxt.register_helper("c".to_string(), c);

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "pouet");

}

#[allow(unused_variables)]
fn v(params: &[&HBData], options: &HelperOptions, out: &mut SafeWriting, hb_context: &EvalContext) -> HBEvalResult {
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

  eval_ctxt.register_helper("v".to_string(), v);

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "value : toto");
}

fn cd(_: &[&HBData], options: &HelperOptions, out: &mut SafeWriting, _: &EvalContext) -> HBEvalResult {
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

  eval_ctxt.register_helper("if".to_string(), cd);

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "value : p true z false");
}

fn globs(_: &[&HBData], options: &HelperOptions, out: &mut SafeWriting, _: &EvalContext) -> HBEvalResult {
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

  eval_ctxt.register_helper("globs".to_string(), globs);

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "value : stored value");
}

fn for_root_check(params: &[&HBData], options: &HelperOptions, out: &mut SafeWriting, _: &EvalContext) -> HBEvalResult {
  match params {
    [p] => options.render_fn_with_context(p, out),
    _ => Ok(()),
  }

}

#[test]
fn helper_root_check() {
  let json = Json::from_str(r##"{"i": "i_root", "c": {"b": "pouet"}}"##).ok().unwrap();
  let tmpl = parse(r##"{{@root.i}} {{#for_root_check c.b}}{{.}} {{../../i}} {{@root.i}}{{/for_root_check}}"##).ok().unwrap();

  let mut eval_ctxt: EvalContext = Default::default();
  let mut buf: Vec<u8> = Vec::new();

  eval_ctxt.register_helper("for_root_check".to_string(), for_root_check);

  eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), "i_root pouet i_root i_root");
}

fn example_helper(
  _: &[&HBData], _: &HelperOptions,
  out: &mut SafeWriting, _: &EvalContext
) -> HBEvalResult
{
  let mut buf = Vec::<u8>::new();
  let res = SafeWriting::with_html_safe_writer(&mut buf, &|out| {
    "pouet pouet".to_string().write_value(out)
  });

  if res.is_err() { return res; };

  let mut s = String::from_utf8(buf).ok().unwrap();

  s.insert(5, '∂');

  s.write_value(out)
}


#[test]
fn safe_writing_help() {
  let json = Json::Null;
  let tmpl = parse(r##"{{example_helper}}"##).ok().unwrap();
  let mut eval_ctxt: EvalContext = Default::default();
  let mut buf = Vec::<u8>::new();
  eval_ctxt.register_helper("example_helper".to_string(), example_helper);

  eval(&tmpl, &json, &mut buf, &eval_ctxt).ok();


  assert_eq!(String::from_utf8(buf).ok().unwrap(), "pouet∂ pouet")
}
