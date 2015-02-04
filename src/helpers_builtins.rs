use serialize::json::ToJson;
use std::collections::HashMap;

use eval::HelperOptions;
use eval::HBData;
use eval::HBEvalResult;
use eval::EvalContext;


pub fn if_helper(_: &[&HBData], options: &HelperOptions, out: &mut Writer, _: &EvalContext) -> HBEvalResult {
  if options.condition {
    options.render_fn(out)
  } else {
    options.inverse(out)
  }
}

pub fn unless_helper(_: &[&HBData], options: &HelperOptions, out: &mut Writer, _: &EvalContext) -> HBEvalResult {
  if options.condition {
    options.inverse(out)
  } else {
    options.render_fn(out)
  }
}

pub fn each_helper(_: &[&HBData], options: &HelperOptions, out: &mut Writer, _: &EvalContext) -> HBEvalResult {
  match options.context.as_array() {
    Some(array) => {
      if array.len() > 0 {
        let mut r = Ok(());
        for (index, item) in array.iter().enumerate() {
          let mut each_globs = HashMap::new();
          let d_index = index.to_json();
          let first = (index == 0).to_json();
          let last = (index == array.len()-1).to_json();
          each_globs.insert("@index", &d_index as &HBData);
          each_globs.insert("@first", &first as &HBData);
          each_globs.insert("@last", &last as &HBData);

          r = options.render_fn_with_context_and_globals(*item, out, &each_globs);

          if r.is_err() {
            break;
          }
        }
        r
      } else {
        options.inverse(out)
      }
    },
    None => if let Some(keys) = options.context.keys() {
      if keys.len() > 0 {
        let mut r = Ok(());
        for &key in keys.iter() {
          if let Some(o) = options.context.get_key(key) {
            let mut each_globs = HashMap::new();
            let key = key.to_string();
            each_globs.insert("@key", &key as &HBData);

            r = options.render_fn_with_context_and_globals(o, out, &each_globs);

            if r.is_err() {
              break;
            }
          }
        }
        r
      } else {
        options.inverse(out)
      }
    } else {
      options.render_fn(out)
    },
  }
}

pub fn lookup_helper(params: &[&HBData], options: &HelperOptions, out: &mut Writer, _: &EvalContext) -> HBEvalResult {
  match params {
    [single, ..] => {
      match options.lookup(single) {
        Some(data) => data.write_value(out),
        None => Ok(())
      }
    },
    [] => Ok(())
  }
}

