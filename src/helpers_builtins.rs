use serialize::json::ToJson;
use std::collections::HashMap;

use eval::HelperOptions;
use eval::HBData;
use eval::HBEvalResult;
use eval::EvalContext;
use eval::SafeWriting;


pub fn if_helper(_: &[&HBData], options: &HelperOptions, out: &mut SafeWriting, _: &EvalContext) -> HBEvalResult {
  if options.condition {
    options.render_fn(out)
  } else {
    options.inverse(out)
  }
}

pub fn unless_helper(_: &[&HBData], options: &HelperOptions, out: &mut SafeWriting, _: &EvalContext) -> HBEvalResult {
  if options.condition {
    options.inverse(out)
  } else {
    options.render_fn(out)
  }
}

pub fn each_helper(params: &[&HBData], options: &HelperOptions, out: &mut SafeWriting, _: &EvalContext) -> HBEvalResult {
  let use_context = params.first().map(|&c| c).unwrap_or(options.context);

  match use_context.as_array() {
    Some(array) => {
      if array.len() > 0 {
        let mut r = Ok(());
        for (index, item) in array.iter().enumerate() {
          let d_index = index.to_json();
          let first = (index == 0).to_json();
          let last = (index == array.len()-1).to_json();

          let mut each_globs = HashMap::new();

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
    None => if let Some(keys) = use_context.keys() {
      if keys.len() > 0 {
        let mut r = Ok(());
        for &key in keys.iter() {
          if let Some(o) = options.context.get_key(key) {
            let key = key.to_string();

            let mut each_globs = HashMap::new();

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

pub fn lookup_helper(params: &[&HBData], options: &HelperOptions, out: &mut SafeWriting, _: &EvalContext) -> HBEvalResult {
  match params {
    [key] => {
      match options.lookup(key) {
        Some(data) => data.write_value(out),
        None => Ok(())
      }
    },
    [context, key] | [context, key, ..] => {
      match options.lookup_with_context(key, context) {
        Some(data) => data.write_value(out),
        None => Ok(())
      }
    },
    [] => Ok(())
  }
}

pub fn with_helper(params: &[&HBData], options: &HelperOptions, out: &mut SafeWriting, _: &EvalContext) -> HBEvalResult {
  match params {
    [context] if context.as_bool() => options.render_fn_with_context(context, out),
    _ => options.inverse(out),
  }
}

