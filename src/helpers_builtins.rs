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

  match use_context.typed_node() {
    ::eval::HBNodeType::Array(_) => {
      let values = use_context.values();
      let (len, _) = values.size_hint();

      if len > 0 {
        let mut r = Ok(());
        for (index, item) in values.enumerate() {
          let d_index = index.to_json();
          let first = (index == 0).to_json();
          let last = (index == len-1).to_json();

          let mut each_globs = HashMap::new();

          each_globs.insert("@index", &d_index as &HBData);
          each_globs.insert("@first", &first as &HBData);
          each_globs.insert("@last", &last as &HBData);

          r = options.render_fn_with_context_and_globals(item, out, &each_globs);

          if r.is_err() {
            break;
          }
        }
        r
      } else {
        options.inverse(out)
      }
    },
    ::eval::HBNodeType::Branch(_) => {
      let keys = use_context.keys();
      let (len, _) = keys.size_hint();
      if len > 0 {
        let mut r = Ok(());
        for (index, ref key) in keys.enumerate() {
          if let Some(o) = options.context.get_key(&key) {
            let key = key.to_string();
            let first = (index == 0).to_json();
            let last = (index == len-1).to_json();

            let mut each_globs = HashMap::new();

            each_globs.insert("@key", &key as &HBData);
            each_globs.insert("@first", &first as &HBData);
            each_globs.insert("@last", &last as &HBData);

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
    },
    ::eval::HBNodeType::Leaf(_) | ::eval::HBNodeType::Null => {
      options.render_fn(out)
    },
  }
}

pub fn lookup_helper(params: &[&HBData], options: &HelperOptions, out: &mut SafeWriting, _: &EvalContext) -> HBEvalResult {
  match (params.first(), params.get(1)) {
    (Some(&key), None) => {
      match options.lookup(key) {
        Some(data) => data.write_value(out),
        None => Ok(())
      }
    },
    (Some(&context), Some(&key)) => {
      match options.lookup_with_context(key, context) {
        Some(data) => data.write_value(out),
        None => Ok(())
      }
    },
    (None, _) => Ok(())
  }
}

pub fn with_helper(params: &[&HBData], options: &HelperOptions, out: &mut SafeWriting, _: &EvalContext) -> HBEvalResult {
  match (params.first(), params.len()) {
    (Some(&context), 1) if context.as_bool() => options.render_fn_with_context(context, out),
    _ => options.inverse(out),
  }
}

