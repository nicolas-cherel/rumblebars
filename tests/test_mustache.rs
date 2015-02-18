#![feature(core)]
#![feature(io)]

extern crate rumblebars;
extern crate "rustc-serialize" as serialize;

mod mustache {
  use std::old_path::posix::Path;
  use std::old_path::GenericPath;
  use std::old_io::{fs, FileStat, FileType};
  use std::default::Default;

  use serialize::json::Json;
  use rumblebars::HBData;

  fn test_set(set_name: &str) {
    let specs_path = Path::new(format!("{}/tests/mustache/specs", option_env!("CARGO_MANIFEST_DIR").unwrap_or(".")));
    match fs::stat(&specs_path) {
      Ok(FileStat { size: _, kind: FileType::Directory, .. }) => {

        match fs::File::open(&Path::new(format!("{}/{}.json", specs_path.as_str().unwrap_or(""), set_name))) {
          Ok(ref mut reader) => {
            match reader.read_to_end() {
              Ok(buf_json) => {
                let json_test = Json::from_str(&String::from_utf8(buf_json).ok().unwrap()).ok().unwrap_or_else(|&:| panic!("cannot parse json for tests"));
                let null = Json::Null;
                let mut errors = Vec::new();
                if let Some(tests) = json_test.find("tests") {
                  if let Some(tests_array) = tests.as_array() {
                    for test in tests_array.iter() {
                      let test_name = test.find("name").unwrap_or(&null).as_string().unwrap_or("name n/a");
                      let test_desc = test.find("desc").unwrap_or(&null).as_string().unwrap_or("desc n/a");
                      let template  = test.find("template").unwrap_or(&null).as_string().unwrap_or("template value not found");
                      let data      = test.find("data"    ).unwrap_or(&null);
                      let expected  = test.find("expected").unwrap_or(&null).as_string().unwrap_or("expected value not found");
                      let partials  = test.find("partials").unwrap_or(&null);

                      let tmpl = ::rumblebars::parse(template).ok().unwrap();
                      let mut buf: Vec<u8> = Vec::new();
                      let mut eval_context: ::rumblebars::EvalContext = Default::default();
                      eval_context.compat = true;

                      match partials {
                        &Json::Object(ref o) => {
                          for (key, partial) in o.iter() {
                            eval_context.register_partial(key.clone(), ::rumblebars::parse(partial.as_string().unwrap_or("")).ok().unwrap_or(vec![]));
                          }
                        },
                        _ => (),
                      }



                      ::rumblebars::eval(&tmpl, data, &mut buf, &eval_context).unwrap_or(());

                      let result = String::from_utf8(buf).unwrap_or("<<result has invalid utf8>>".to_string());

                      if result != expected {
                        errors.push((test_name, test_desc, result, expected.to_string()))
                      }
                    }
                  }
                }

                if !errors.is_empty() {
                  panic!(errors.iter().map(
                    |&(ref name, ref desc, ref result, ref expected)| format!("\nFAILED in {}: {} - {}\nresult:\n{:?}\nexpected:\n{:?}\n", set_name, name, desc, result, expected)
                  ).fold(String::new(), |mut s, v| { s.push_str(v.as_slice()); s }))
                }


              },
              _ => panic!("parse error")
            }
          },
          _ => panic!("mustache {} test file not found", set_name),
        }
      },
      _ => panic!("mustache specs not found, make sure to have rumblebars repos submodules initialized properly"),
    }
  }

  macro_rules! mustache_tests_set{
    ($set_name: ident) => (

      #[test]
      fn $set_name() {
        test_set(stringify!($set_name));
      }

    )
  }

  mustache_tests_set!(interpolation);
  mustache_tests_set!(comments);
  mustache_tests_set!(inverted);
  mustache_tests_set!(partials);
  mustache_tests_set!(sections);
}

