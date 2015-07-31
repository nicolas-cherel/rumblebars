extern crate rumblebars;
extern crate rustc_serialize as serialize;

#[cfg(feature = "stream_test")] extern crate rand;
#[cfg(feature = "stream_test")] extern crate time;

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

/// stream testing, run with :
/// `cargo test stream::test_stream --release  --features stream_test -- --nocapture`

#[cfg(feature = "stream_test")]
mod stream {
  use std::io::Write;
  use std::cell::RefCell;

  use rand;
  use serialize::json;
  use serialize::json::Json;

  use rumblebars::Template;
  use rumblebars::EvalContext;
  use rumblebars::preludes::hbdata::*;

  struct RandIter<'a, RNG> where RNG: rand::Rng {
    rng: RNG,
    total: usize,
    cell: &'a RefCell<Json>,
  }

  impl<'a, RNG> Iterator for RandIter<'a, RNG> where RNG: rand::Rng {
    type Item = &'a HBData;

    fn next(&mut self) -> Option<Self::Item> {
      if self.total <= 0 { return None }

      self.total -= 1;

      let size = ((self.rng.gen::<u16>() >> 5) + 15) as usize;
      let key_size = ((self.rng.gen::<u8>() >> 3) + 6) as usize;

      {
        let mut json_borrow = self.cell.borrow_mut();
        let mut json = json_borrow.as_object_mut().unwrap();
        json.clear();
        for _ in (0..self.rng.gen::<u8>()) {
          let key = self.rng.gen_ascii_chars().take(key_size).collect();
          let value = self.rng.gen_ascii_chars().take(size).collect();
          json.insert(key, Json::String(value));
        }
      }

      // borrow returns a Ref on the stack, so its lifetime is to short
      // but referenced value actually has 'a lifetime.
      Some(unsafe { ::std::mem::transmute(&*self.cell.borrow() as &HBData) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
      (self.total, Some(self.total))
    }
  }

  struct Rand {
    cell: RefCell<Json>,
    size_limit: usize,
  }


  impl HBData for Rand {
    fn write_value(&self, out: &mut SafeWriting) -> HBEvalResult {
      write!(out, "{}", "self")
    }

    fn typed_node<'a>(&'a self) -> HBNodeType<&'a HBData> {
      HBNodeType::Array(self as &HBData)
    }

    fn as_bool(&self) -> bool { true }

    fn get_key<'a>(&'a self, _: &str) -> Option<&'a HBData> { None }
    fn keys<'a>(&'a self) -> HBKeysIter<'a> { Box::new(None.into_iter()) }

    fn values<'a>(&'a self) -> HBValuesIter<'a> { Box::new(RandIter {
      cell: &self.cell, total: self.size_limit, rng: rand::weak_rng()
    } ) }

    fn iter<'a>(&'a self) -> HBIter<'a> { Box::new(None.into_iter()) }
  }

  struct SenderWriter(::std::sync::mpsc::Sender<Vec<u8>>);

  impl ::std::io::Write for SenderWriter {
    fn write(&mut self, buf: &[u8]) -> ::std::io::Result<usize> {
      use std::io::{Error, ErrorKind};
      try!(self.0.send(buf.to_vec()).map_err(|e| Error::new(ErrorKind::Other, e) ));
      Ok(buf.len())
    }

    fn flush(&mut self) -> ::std::io::Result<()> {
      Ok(())
    }
  }

  #[test]
  fn test_stream() {
    use std::sync::mpsc::channel;
    use std::thread;


    let (tx, rx) = channel::<Vec<u8>>();

    let h = thread::spawn(move|| {
        let mut tick = ::time::PreciseTime::now();
        let mut total_data = 0;
        let mut acc = 0;
        for data in rx.iter() {
          total_data += data.len();
          acc += data.len();

          let duration = tick.to(::time::PreciseTime::now());
          if duration.num_seconds() > 0 {
            println!("{:?}MiB/s", (acc as i64 / duration.num_milliseconds()) as f64 / 1000f64 );
            acc = 0;
            tick = ::time::PreciseTime::now();
          }
        }
        println!("total received : {:?}", total_data);
    });

    let rand_collection = Rand { cell: RefCell::new(Json::Object(json::Object::new())), size_limit: 2000 };

    if let Ok(t) = Template::new("{{#this}}{{#each this}}{{@key}} {{.}}{{/each}}{{/this}}") {
      println!("total {:?}", ::time::Duration::span(|| {
        t.eval(&rand_collection, &mut ::std::io::BufWriter::new(SenderWriter(tx)), &EvalContext::new()).ok();
        // t.eval(&rand_collection, &mut ::std::io::sink(), &EvalContext::new());
        // t.eval(&rand_collection, &mut ::std::io::stdout(), &EvalContext::new());
      }));

    }

    h.join().ok();

  }
}

