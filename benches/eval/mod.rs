use std::default::Default;
use serialize::json::Json;
use test::Bencher;
use rumblebars::eval;

mod big_eval;

#[bench]
fn raw_expansion(b: &mut Bencher) {
  let json: Json = Json::Null;
  let tmpl = r##"raw string"##.parse().ok().unwrap();

  b.iter(|| {
    let mut buf: Vec<u8> = Vec::new();
    eval(&tmpl, &json, &mut buf, &Default::default()).ok();
  })
}

#[bench]
fn simple_expansion(b: &mut Bencher) {
  let json: Json = r##"{"p": "hello"}"##.parse().ok().unwrap();
  let tmpl = r##"{{p}}"##.parse().ok().unwrap();

  b.iter(|| {
    let mut buf: Vec<u8> = Vec::new();
    eval(&tmpl, &json, &mut buf, &Default::default()).ok();
  })
}

#[bench]
fn collection_expansion(b: &mut Bencher) {
  let json: Json = r##"[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]"##.parse().ok().unwrap();
  let tmpl = r##"{{#this}}{{.}}{{/this}}"##.parse().ok().unwrap();

  b.iter(|| {
    let mut buf: Vec<u8> = Vec::new();
    eval(&tmpl, &json, &mut buf, &Default::default()).ok();
  })
}

#[bench]
fn object_collection_expansion(b: &mut Bencher) {
  let json: Json = r##"[{"n": 1}, {"n": 2}, {"n": 3}, {"n": 4}, {"n": 5}, {"n": 6}, {"n": 7}, {"n": 8}, {"n": 9}, {"n": 10}]"##.parse().ok().unwrap();
  let tmpl = r##"{{#this}}{{n}}{{/this}}"##.parse().unwrap_or_else(|e| panic!("{:?}", e));

  b.iter(|| {
    let mut buf: Vec<u8> = Vec::new();
    eval(&tmpl, &json, &mut buf, &Default::default()).ok();
  })
}

#[bench]
fn partial_expansion(b: &mut Bencher) {
  let json: Json = r##"{"u": "𝓾", "v": "𝓿", "w": "𝔀"}"##.parse().ok().unwrap();
  let tmpl = r##"{{> show }}"##.parse().unwrap_or_else(|e| panic!("{:?}", e));
  let mut context: ::rumblebars::EvalContext = Default::default();

  let partial = r##"({{u}},{{v}},{{w}})"##.parse().unwrap_or_else(|e| panic!("{:?}", e));
  context.register_partial("show".to_string(), partial);

  b.iter(|| {
    let mut buf: Vec<u8> = Vec::new();
    eval(&tmpl, &json, &mut buf, &context).ok();
  })
}


#[bench]
fn true_else_expansion(b: &mut Bencher) {
  let json: Json = r##"{"p": "hello"}"##.parse().ok().unwrap();
  let tmpl = r##"{{#p}}t{{else}}f{{/p}}"##.parse().ok().unwrap();

  b.iter(|| {
    let mut buf: Vec<u8> = Vec::new();
    eval(&tmpl, &json, &mut buf, &Default::default()).ok();
  })
}

#[bench]
fn false_else_expansion(b: &mut Bencher) {
  let json: Json = r##"{"p": false}"##.parse().ok().unwrap();
  let tmpl = r##"{{#p}}t{{else}}f{{/p}}"##.parse().ok().unwrap();

  b.iter(|| {
    let mut buf: Vec<u8> = Vec::new();
    eval(&tmpl, &json, &mut buf, &Default::default()).ok();
  })
}

#[bench]
fn each_helper_expansion(b: &mut Bencher) {
  let json: Json = r##"{"u": "𝓾", "v": "𝓿", "w": "𝔀"}"##.parse().ok().unwrap();
  let tmpl = r##"{{#each}}{{@key}}: {{.}}{{^@last}} {{/@last}}{{/each}}"##.parse().ok().unwrap();

  b.iter(|| {
    let mut buf: Vec<u8> = Vec::new();
    eval(&tmpl, &json, &mut buf, &Default::default()).ok();
  })
}

