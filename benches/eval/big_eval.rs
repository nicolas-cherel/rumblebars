use std::default::Default;
use serialize::json;
use serialize::json::Json;
use test::Bencher;
use rumblebars::eval;


#[bench]
fn big_eval(b: &mut Bencher) {
  let mut builder = json::Array::new();
  for i in (1..50) {
    let mut i_ = json::Array::new();

    for j in (1..50) {
      i_.push(Json::String(format!("this is iteration {}, {}", i, j)));
    }

    builder.push(Json::Array(i_));
  }

  let json = Json::Array(builder);
  let tmpl = "{{#this}}{{#this}}{{.}}\n{{/this}}{{/this}}".parse().unwrap();

  b.iter(|| {
    let mut buf: Vec<u8> = Vec::new();
    eval(&tmpl, &json, &mut buf, &Default::default()).ok();
  })
}