use serialize::json::Json;
use std::default::Default;

use rumblebars::eval;
use rumblebars::parse;

fn test_eq_expected(template: &str, json_str: &str, expected: &str) {
  let json = Json::from_str(json_str).ok().unwrap();
  let tmpl = parse(template).ok().unwrap();
  let mut buf: Vec<u8> = Vec::new();

  eval(&tmpl, &json, &mut buf, &Default::default()).unwrap();

  assert_eq!(String::from_utf8(buf).unwrap(), expected);
}

mod simple {
  use super::test_eq_expected;

  #[test]
  fn empty() {
    test_eq_expected("", "{}", "");
  }

  #[test]
  fn none() {
    test_eq_expected("{{none}}", "{}", "");
  }

  #[test]
  fn none_both() {
    test_eq_expected(" {{none}} ", "{}", "  ");
  }

  #[test]
  fn simple_left() {
    test_eq_expected(" {{none}}|", "{}", " |");
  }

  #[test]
  fn simple_right() {
    test_eq_expected("|{{none}} ", "{}", "| ");
  }

  #[test]
  fn trim_right() {
    test_eq_expected("|{{none~}} ", "{}", "|");
  }

  #[test]
  fn trim_left() {
    test_eq_expected(" {{~none}}|", "{}", "|");
  }

  #[test]
  fn trim_both() {
    test_eq_expected(" {{~none~}} ", "{}", "");
  }

  #[test]
  fn trim_left_with_raw() {
    test_eq_expected(" t {{~.}} t ", r##"1"##, " t1 t ");
  }

  #[test]
  fn trim_right_with_raw() {
    test_eq_expected(" t {{.~}} t ", r##"1"##, " t 1t ");
  }

  #[test]
  fn trim_both_with_raw() {
    test_eq_expected(" t {{~none~}} t ", "{}", " tt ");
  }

  #[test]
  fn trim_new_lines() {
    test_eq_expected("\n {{none}} \n", "{}", "\n  \n");
  }

  #[test]
  fn no_trim_seq() {
    test_eq_expected(" |{{none}} {{none}}| ", "{}", " | | ");
  }

  #[test]
  fn trim_seq_first() {
    test_eq_expected(" |{{none~}} {{none}}| ", "{}", " || ");
  }

  #[test]
  fn trim_seq_sec() {
    test_eq_expected(" |{{none}} {{~none}}| ", "{}", " || ");
  }

}

mod blocks {
  use super::test_eq_expected;


  #[test]
  fn empty() {
    test_eq_expected("", "{}", "");
  }

  #[test]
  fn none() {
    test_eq_expected("{{#none}}{{/none}}", "{}", "");
  }

  #[test]
  fn none_both() {
    test_eq_expected(" {{#none}}{{/none}} ", "{}", "  ");
  }

  #[test]
  fn simple_left() {
    test_eq_expected(" {{#none}}{{/none}}|", "{}", " |");
  }

  #[test]
  fn simple_right() {
    test_eq_expected("|{{#none}}{{/none}} ", "{}", "| ");
  }

  #[test]
  fn trim_right() {
    test_eq_expected("|{{#none}}{{/none~}} ", "{}", "|");
  }

  #[test]
  fn trim_left() {
    test_eq_expected(" {{~#none}}{{/none}}|", "{}", "|");
  }

  #[test]
  fn trim_both() {
    test_eq_expected(" {{~#none}}{{/none~}} ", "{}", "");
  }

  #[test]
  fn trim_left_with_raw() {
    test_eq_expected(" t {{~#.}}{{.}}{{/.}} t ", r##"1"##, " t1 t ");
  }

  #[test]
  fn trim_right_with_raw() {
    test_eq_expected(" t {{#.}}{{.}}{{/.~}} t ", r##"1"##, " t 1t ");
  }

  #[test]
  fn trim_both_with_raw() {
    test_eq_expected(" t {{~#none}}{{/none~}} t ", "{}", " tt ");
  }

  #[test]
  fn trim_new_lines() {
    test_eq_expected("\n {{#none}}{{/none}} \n", "{}", "\n  \n");
  }

  #[test]
  fn no_trim_seq() {
    test_eq_expected(" |{{#none}}{{/none}} {{#none}}{{/none}}| ", "{}", " | | ");
  }

  #[test]
  fn trim_seq_first() {
    test_eq_expected(" |{{#none}}{{/none~}} {{#none}}{{/none}}| ", "{}", " || ");
  }

  #[test]
  fn trim_seq_sec() {
    test_eq_expected(" |{{#none}}{{/none}} {{~#none}}{{/none}}| ", "{}", " || ");
  }

}

