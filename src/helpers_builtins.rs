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

pub fn each_helper(_: &[&HBData], options: &HelperOptions, out: &mut Writer, _: &EvalContext) -> HBEvalResult {
  match options.context.as_array() {
    Some(array) => {
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
    },
    None => if let Some(keys) = options.context.keys() {
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
      options.render_fn(out)
    },
  }
}
