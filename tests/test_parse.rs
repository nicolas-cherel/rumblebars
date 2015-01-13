extern crate rumblebars;

use rumblebars::parse;
use rumblebars::ParseError;

#[test]
fn big_no_err() {
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
  

  assert!((match t { Ok(_) => true, Err((e, mesg)) => { println!("{}", mesg.unwrap_or("".to_string())); false }}))

}

#[test]
fn fail_block() {
  assert!(match parse("{{#o}}{{/t}}") { Err((ParseError::UnmatchedBlock, _)) => true, Err(_) => false, Ok(_) => false })
}

#[test]
fn fail_nested_block() {
  assert!(match parse("{{#o}}{{/i}}{{/o}}") { Err((ParseError::UnmatchedBlock, _)) => true, Err(_) => false, Ok(_) => false })
}