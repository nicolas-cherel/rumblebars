
#[allow(unused_imports)] // disable warning since only used in tests
use parse;

#[allow(unused_imports)] // disable warning since only used in tests
use parse_hb_expression;

#[allow(unused_imports)] // disable warning since only used in tests
use ParseError;

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
