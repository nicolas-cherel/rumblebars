extern crate rumblebars;

use rumblebars::Template;
use rumblebars::ParseError;
use rumblebars::parse;

static BIG: &'static str = r##"
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
  "##;


#[test]
fn big_no_err() {
  let t = BIG.parse::<Template>();

  assert!((match t { Ok(_) => true, Err((_, mesg)) => { println!("{}", mesg.unwrap_or("".to_string())); false }}))
}

#[test]
fn medium() {
  let t = r##"<h1>RumbleBars commits</h1>

<ul>
{{#each}}
  <li>
    <div class="head">
      {{#if commit}}<div>{{commit.message}}</div>{{/if}}
      {{#if commit.author}}
        <div>{{commit.author.name}} — <span>{{sha}}</span></div>
        <div>{{commit.author.date}}</div>
      {{else}}
        -- no author --
      {{/if}}
    </div>
  </li>
{{/each}}
</ul>
"##.parse::<Template>();

  assert!((match t { Ok(_) => true, Err((_, mesg)) => { println!("{}", mesg.unwrap_or("".to_string())); false }}))
}

#[test]
fn fail_block() {
  assert!(match parse("{{#o}}{{/t}}") { Err((ParseError::UnmatchedBlock, _)) => true, Err(_) => false, Ok(_) => false })
}

#[test]
fn fail_nested_block() {
  assert!(match parse("{{#o}}{{/i}}{{/o}}") { Err((ParseError::UnmatchedBlock, _)) => true, Err(_) => false, Ok(_) => false })
}
