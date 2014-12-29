use std::io::{IoError, Writer};
use serialize::json::Json;

use parse::Template;
use parse::HBEntry;
use parse::HBExpression;

fn get_val_for_key<'a>(data: &'a Json, key_path: &Vec<String>, context_stack: &Vec<&'a Json>) ->  Option<&'a Json> {
  let mut ctxt = Some(data);
  let mut stack_index = 0;
  
  for key in key_path.iter() {
    match key.as_slice() {
      "."  => {continue},
      ".." => {
        stack_index += 1;
        assert!(stack_index <= context_stack.len());
        ctxt = Some(*context_stack.get(context_stack.len() - stack_index).unwrap());
        continue;
      }
      _ => (),
    }

    let some_num_key = key.as_slice().parse();
    ctxt = match ctxt {
      Some(&Json::Array(ref a)) => {
        if let Some(num_key) = some_num_key {
          a.get(num_key)
        } else {
          None
        }
      },
      Some(&Json::Object(ref o)) => {
        o.get(key)
      },
      _ => None, // keys only match against arrays and objects
    }
  }

  return ctxt;
}

pub fn eval(template: &Template, data: &Json, out: &mut Writer) -> Result<(), IoError> {
  let mut stack:Vec<_> = FromIterator::from_iter(template.iter().map(|e| {
    (e, data, Vec::new())
  }));

  while let Some((templ, ctxt, ctxt_stack)) = stack.remove(0) {
    let w_ok = match templ {
      &box HBEntry::Raw(ref s) => { 
        out.write_str(s.as_slice())
      },
      &box HBEntry::Eval(HBExpression{ref base, ref params, ref options, ref escape, ref no_white_space, block: None}) => {
        match get_val_for_key(ctxt, base, &ctxt_stack) {
          Some(v) => match v {
            // should use a serializer here
            &Json::I64(ref i)     => write!(out, "{}", i),
            &Json::U64(ref u)     => write!(out, "{}", u),
            &Json::F64(ref f)     => write!(out, "{}", f),
            &Json::String(ref s)  => write!(out, "{}", s),
            &Json::Boolean(ref b) => write!(out, "{}", b),
            _ => Ok(()),
          },
          None => Ok(()),
        }
      },

      &box HBEntry::Eval(HBExpression{ref base, ref params, ref options, ref escape, ref no_white_space, ref block}) => {
        let c_ctxt = get_val_for_key(ctxt, base, &ctxt_stack);

        match (c_ctxt, block) {
          (Some(c), &Some(ref t)) => {
            match c {
              &Json::Object(_) => {
                for e in t.iter().rev() {
                  let mut c_stack = ctxt_stack.clone();
                  c_stack.push(ctxt);
                  stack.insert(0, (e, c, c_stack));
                }                
              },
              &Json::Array(ref a) => {
                for i in a.iter().rev() {
                  for e in t.iter().rev() {
                    let mut c_stack = ctxt_stack.clone();
                    c_stack.push(ctxt);
                    stack.insert(0, (e, i, c_stack));
                  }   
                }
              }
              _ => (),
            }
            Ok(())
          },
          _ => Ok(()),
        }
      },
    };

    if let Err(no_ok) = w_ok {
      return Err(no_ok);
    }
  }
  return Ok(());
}


#[cfg(test)]
mod tests {

  use serialize::json;
  use serialize::json::Json;

  use super::get_val_for_key;

  #[test]
  fn fetch_key_value() {
    let json = json::from_str(r##"{"a": 1}"##).unwrap();
    assert_eq!(match get_val_for_key(&json, &vec!["a".to_string()], &vec![]) {
      Some(&Json::U64(a)) => a, 
      _ => 10000
    }, 1);
  }

  #[test]
  fn fetch_key_value_level1() {
    let json = json::from_str(r##"{"a": {"b": 1}}"##).unwrap();
    assert_eq!(1, match get_val_for_key(&json, &vec!["a".to_string(), "b".to_string()], &vec![]) {
      Some(&Json::U64(a)) => a, 
      _ => 10000
    });
  }

  #[test]
  fn fetch_key_value_array_level1() {
    let json = json::from_str(r##"{"a": [1, 2, 3]}"##).unwrap();
    assert_eq!(1, match get_val_for_key(&json, &vec!["a".to_string(), "0".to_string()], &vec![]) {
      Some(&Json::U64(a)) => a, 
      _ => 10000
    });
  }

  #[test]
  fn resolve_this_in_keypath() {
    let json = json::from_str(r##""hello""##).unwrap();
    assert_eq!("hello", match get_val_for_key(&json, &vec![".".to_string()], &vec![]) {
      Some(&Json::String(ref v)) => v.clone(),
      _ => "".to_string(),
    })
  }

  #[test]
  fn resolve_this_subkey_in_keypath() {
    let json = json::from_str(r##"{"t": "hello"}"##).unwrap();
    assert_eq!("hello", match get_val_for_key(&json, &vec![".".to_string(), "t".to_string()], &vec![]) {
      Some(&Json::String(ref v)) => v.clone(),
      _ => "".to_string(),
    })
  }

  #[test]
  fn deep_path_none() {
    let json = json::from_str(r##"{"a": 1}"##).unwrap();
    assert_eq!(None, get_val_for_key(&json, &vec!["a".to_string(), "b".to_string()], &vec![]));
  }

}


