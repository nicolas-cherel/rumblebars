use test::Bencher;
use rumblebars::Template;


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

#[bench]
fn parse_mid_size_template(b: &mut Bencher) {
  b.iter(|| {
    BIG.parse::<Template>().ok();
  })
}

#[bench]
fn parse_small_template(b: &mut Bencher) {
  b.iter(|| {
    "{{p}}".parse::<Template>().ok();
  })
}

#[bench]
fn parse_small_else_template(b: &mut Bencher) {
  b.iter(|| {
    "{{#p}}{{^}}{{/p}}".parse::<Template>().ok();
  })
}
