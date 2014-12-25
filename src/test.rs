
#[allow(unused_imports)] // disable warning since only used in tests
use std::str;
#[allow(unused_imports)]
use std::vec::Vec;
#[allow(unused_imports)]
use serialize::json;
#[allow(unused_imports)]
use serialize::json::{Json, decode};
#[allow(unused_imports)]
use std::io::BufWriter;

#[allow(unused_imports)]
use parse;
#[allow(unused_imports)]
use parse_hb_expression;
#[allow(unused_imports)]
use ParseError;
#[allow(unused_imports)]
use Template;
#[allow(unused_imports)]
use HBEntry;
#[allow(unused_imports)]
use get_val_for_key;
#[allow(unused_imports)]
use eval;




#[test]
fn it_works() {
  let t = parse(r##"
    conten content
    {{pouet.so1}}
    {{#pouet2}} do do do {{/pouet2}}
    {{#pouet3}} do do do {{/pouet3 }}
    {{#deep}}
      zero
      {{#deep1}}
        one
        {{#deep2}}
          two
          {{#deep3}}
            bottom 3
            {{at.level.3}}
          {{/deep3}}
        {{/deep2}}
        {{level1}}
      {{/deep1}}
    {{/deep}}
    {{{toto }}}
    {{{toto2 coyote=speed.runner hello=how tip="top"}}}
    {{{toto3.[3].[#jojo] titi="grominet"}}}
    {{t "… param1" well.[that my baby].[1] ~}}
  "##);
  
  assert!((match t { Ok(_) => true, Err(_) => false }))

}

#[test]
fn hb_simple() {
  assert!(match parse_hb_expression("{{i}}") { 
    Ok(_)  => true,
    Err(_) => false,
  })
}

#[test]
fn hb_simple_base() {
  match parse_hb_expression("{{i}}") { 
    Ok(ok)  => assert_eq!(ok.base, vec!["i"]),
    Err(_)  => (),
  }
}

#[test]
fn hb_simple_base_path() {
  match parse_hb_expression("{{i.j}}") { 
    Ok(ok)  => assert_eq!(ok.base, vec!["i", "j"]),
    Err(_)  => (),
  }
}

#[test]
fn hb_simple_base_esc_path() {
  match parse_hb_expression("{{[i]}}") { 
    Ok(ok)  => assert_eq!(ok.base, vec!["i"]),
    Err(_)  => (),
  }
}

#[test]
fn fail_block() {
  assert!(match parse("{{#o}}{{/t}}") { Err((ParseError::UnmatchedBlock, _)) => true, Err(_) => false, Ok(_) => false })
}

#[test]
fn fail_nested_block() {
  assert!(match parse("{{#o}}{{/i}}{{/o}}") { Err((ParseError::UnmatchedBlock, _)) => true, Err(_) => false, Ok(_) => false })
}

#[test]
fn parse_raw() {
  let p = parse("tada").unwrap();
  assert_eq!("tada", match p.content.get(0) {
    Some(&box HBEntry::Raw(ref s)) => s.as_slice(),
    _ => "",
  });
}

#[test]
fn fetch_key_value() {
  let json = json::from_str(r##"{"a": 1}"##).unwrap();
  assert_eq!(match get_val_for_key(&json, &vec![String::from_str("a")]) {
    Some(&Json::U64(a)) => a, 
    _ => 10000
  }, 1);
}

#[test]
fn fetch_key_value_level1() {
  let json = json::from_str(r##"{"a": {"b": 1}}"##).unwrap();
  assert_eq!(1, match get_val_for_key(&json, &vec![String::from_str("a"), String::from_str("b")]) {
    Some(&Json::U64(a)) => a, 
    _ => 10000
  });
}

#[test]
fn fetch_key_value_array_level1() {
  let json = json::from_str(r##"{"a": [1, 2, 3]}"##).unwrap();
  assert_eq!(1, match get_val_for_key(&json, &vec![String::from_str("a"), String::from_str("0")]) {
    Some(&Json::U64(a)) => a, 
    _ => 10000
  });
}

#[test]
fn deep_path_none() {
  let json = json::from_str(r##"{"a": 1}"##).unwrap();
  assert_eq!(None, get_val_for_key(&json, &vec![String::from_str("a"), String::from_str("b")]));
}

#[test]
fn simple_render() {
  let json = json::from_str(r##"{"p": "that poney has something sad in its eye"}"##).unwrap();
  let tmpl = parse(r##"{{p}}"##).unwrap();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf);

  assert_eq!(String::from_utf8(buf).unwrap(), String::from_str("that poney has something sad in its eye"));
}

#[test]
fn simple_render_with_raw() {
  let json = json::from_str(r##"{"p": "that poney has something sad in its eye"}"##).unwrap();
  let tmpl = parse(r##"prelude {{p}} post"##).unwrap();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf);

  assert_eq!(String::from_utf8(buf).unwrap(), String::from_str("prelude that poney has something sad in its eye post"));
}

#[test]
fn simple_render_with_block() {
  let json = json::from_str(r##"{"p": { "k": "that poney has something sad in its eye"}}"##).unwrap();
  let tmpl = parse(r##"prelude {{#p}}{{k}}{{/p}} post"##).unwrap();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf);

  assert_eq!(String::from_utf8(buf).unwrap(), String::from_str("prelude that poney has something sad in its eye post"));
}
