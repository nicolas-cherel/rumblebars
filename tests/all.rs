extern crate rumblebars;
extern crate rustc_serialize as serialize;

mod helpers;
mod parse;

mod eval {
  mod trimming;
  mod handlebars;
  mod mustache;

  use serialize::json::Json;
  use std::default::Default;

  use rumblebars::eval;
  use rumblebars::parse;
  use rumblebars::EvalContext;

  #[test]
  fn from_str() {
    let json: Json = r##"{"p": "hello"}"##.parse().ok().unwrap();
    let tmpl = r##"{{p}}"##.parse().ok().unwrap();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &Default::default()).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "hello");
  }

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

  #[test]
  fn leading_whitespace() {
    let json = Json::from_str(r##"{"p": {}}"##).ok().unwrap();
    let tmpl = parse(r##"{{#p~}}

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

    let expected = r##"Pouet

"##;

    assert_eq!(String::from_utf8(buf).unwrap(), expected);
  }

  #[test]
  fn both_whitespace() {
    let json = Json::from_str(r##"{"p": {}}"##).ok().unwrap();
    let tmpl = parse(r##"{{#p~}}

        Pouet pouet

        {{~/p}}"##).ok().unwrap();
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

    let expected = r##"Uuuuu ooOOOO
"##;

    assert_eq!(String::from_utf8(buf).unwrap(), expected);
  }

  #[test]
  fn autotrim_first() {
    let tmpl = parse("{{#p}}\nv{{/p}}").ok().unwrap();
    let json = Json::from_str(r##"{"p": true}"##).unwrap();
    let eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "v");
  }

  #[test]
  fn autotrim_last() {
    let tmpl = parse("{{#p}}v\n{{/p}}").ok().unwrap();
    let json = Json::from_str(r##"{"p": true}"##).unwrap();
    let eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "v\n");
  }

  #[test]
  fn autotrim_mid() {
    let tmpl = parse("o\n{{#p}}\nv\n{{/p}}\nu").ok().unwrap();
    let json = Json::from_str(r##"{"p": true}"##).unwrap();
    let eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "o\nv\nu");
  }

  #[test]
  fn autotrim_mid_exp() {
    let tmpl = parse("o\n{{#p}}\n{{.}}\n{{/p}}\nu").ok().unwrap();
    let json = Json::from_str(r##"{"p": " "}"##).unwrap();
    let eval_ctxt: EvalContext = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &eval_ctxt).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "o\n \nu");
  }

  #[test]
  fn autotrim_pass1() {
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

      let expected = "\n        1)\n          o\n\n\n      ";

      assert_eq!(String::from_utf8(buf).unwrap(), expected);
    }
  }

  #[test]
  fn autotrim_pass2() {
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

      let expected = "\n        2)\n          o\n          \n\n          uU\n\n      "; // one line string due to trailing whitespace
      assert_eq!(String::from_utf8(buf).unwrap(), expected);
    }
  }

  #[test]
  fn autotrim_pass3() {

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

      let expected = "\n        3)\n        i\n          o\n\n\n              "; // one line string due to trailing whitespace
      assert_eq!(String::from_utf8(buf).unwrap(), expected);
    }

  }


  #[test]
  fn html_escape() {
    let json = Json::from_str(r##"{"unsafe": "<script lang=\"text/javascript\">pawned()</script>"}"##).ok().unwrap();
    let tmpl = parse(r##"{{unsafe}}"##).ok().unwrap();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &Default::default()).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "&lt;script lang=&quot;text/javascript&quot;&gt;pawned()&lt;/script&gt;");
  }

  #[test]
  fn html_noescape() {
    let json = Json::from_str(r##"{"unsafe": "<script lang=\"text/javascript\">pawned()</script>"}"##).ok().unwrap();
    let tmpl = parse(r##"{{{unsafe}}}"##).ok().unwrap();
    let mut buf: Vec<u8> = Vec::new();

    eval(&tmpl, &json, &mut buf, &Default::default()).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "<script lang=\"text/javascript\">pawned()</script>");
  }

}

