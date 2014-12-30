use std::io::{IoError, Writer};
use serialize::json::Json;

use parse::Template;
use parse::HBEntry;
use parse::HBExpression;

fn get_val_for_key<'a>(data: &'a HBData, key_path: &Vec<String>, context_stack: &Vec<&'a HBData>) ->  Option<&'a (HBData + 'a)> {
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

    ctxt = match ctxt {
      Some(c) => c.get_key(key.as_slice()),
      _ => None, // keys only match against arrays and objects
    }
  }

  return ctxt;
}

pub enum HBNodeType<Sized? T> {
  Branch(T),
  Array(T),
  Leaf(T),
  Null,
}

pub trait HBData {
  fn value_for_key_path<'a>(&'a self, path: &Vec<String>, stack: &Vec<&'a HBData>) -> Option<&'a HBData>;
  fn write_value(&self, out: &mut Writer) -> Result<(), IoError>;
  fn typed_node<'a>(&'a self) -> HBNodeType<&'a HBData>;
  fn as_array<'a>(&'a self) -> Option<Vec<&'a HBData>>;
  fn get_key<'a>(&'a self, key: &str) -> Option<&'a HBData>;
}


impl HBData for Json {
  fn value_for_key_path<'a>(&'a self, path: &Vec<String>, stack: &Vec<&'a HBData>) -> Option<&'a HBData> {
    return match get_val_for_key(self, path, stack) {
      Some(v) => Some(v),
      _ => None,
    };
  }

  fn typed_node<'a>(&'a self) -> HBNodeType<&'a HBData> {
    return match self {
      &Json::Object(_) => HBNodeType::Branch(self as &HBData),
      &Json::Array(_)  => HBNodeType::Array(self as &HBData),
      &Json::Null      => HBNodeType::Null,
      _                => HBNodeType::Leaf(self as &HBData),
    }
  }

  fn write_value(&self, out: &mut Writer) -> Result<(), IoError> {
    return match self {
      &Json::I64(ref i)     => write!(out, "{}", i),
      &Json::U64(ref u)     => write!(out, "{}", u),
      &Json::F64(ref f)     => write!(out, "{}", f),
      &Json::String(ref s)  => write!(out, "{}", s),
      &Json::Boolean(ref b) => write!(out, "{}", b),
      _  => Ok(()),
    }
  }

  fn as_array<'a>(&'a self) -> Option<Vec<&'a HBData>> {
    return match self {
      &Json::Array(ref a) => {
        Some(FromIterator::from_iter(a.iter().map(|e| { e as &HBData })))
      },
      _ => None,
    }
    
  }

  fn get_key<'a>(&'a self, key: &str) -> Option<&'a HBData> {
    return match self {
      &Json::Array(ref a) => {
        if let Some(num_key) = key.as_slice().parse() {
          match a.get(num_key) {
            Some(v) => Some(v as &HBData),
            None => None,
          }
        } else {
          None
        }
      },
      &Json::Object(_) => {
        return Some(self.index(&key.as_slice()) as &HBData);  
      },
      _ => None,
    }
  }
}


pub fn eval(template: &Template, data: &HBData, out: &mut Writer) -> Result<(), IoError> {
  let mut stack:Vec<_> = FromIterator::from_iter(template.iter().map(|e| {
    (e, data, Vec::new())
  }));

  while let Some((templ, ctxt, ctxt_stack)) = stack.remove(0) {
    let w_ok = match templ {
      &box HBEntry::Raw(ref s) => { 
        out.write_str(s.as_slice())
      },
      &box HBEntry::Eval(HBExpression{ref base, ref params, ref options, ref escape, ref no_white_space, block: None}) => {
        match ctxt.value_for_key_path(base, &ctxt_stack) {
          Some(v) => match v.typed_node() {
            HBNodeType::Leaf(_) => v.write_value(out),
            // // should use a serializer here
            // &Json::I64(ref i)     => write!(out, "{}", i),
            // &Json::U64(ref u)     => write!(out, "{}", u),
            // &Json::F64(ref f)     => write!(out, "{}", f),
            // &Json::String(ref s)  => write!(out, "{}", s),
            // &Json::Boolean(ref b) => write!(out, "{}", b),
            _ => Ok(()),
          },
          None => Ok(()),
        }
      },

      &box HBEntry::Eval(HBExpression{ref base, ref params, ref options, ref escape, ref no_white_space, ref block}) => {
        let c_ctxt = ctxt.value_for_key_path(base, &ctxt_stack);

        match (c_ctxt, block) {
          (Some(c), &Some(ref block_found)) => {
            match c.typed_node() {
              HBNodeType::Branch(_) => {
              // &Json::Object(_) => {
                for e in block_found.iter().rev() {
                  let mut c_stack = ctxt_stack.clone();
                  c_stack.push(ctxt);
                  stack.insert(0, (e, c, c_stack));
                }                
              },
              HBNodeType::Array(a) => {
              // &Json::Array(ref a) => {
                if let Some(collection) = a.as_array() {
                  for array_i in collection.iter().rev() {
                    for e in block_found.iter().rev() {
                      let mut c_stack = ctxt_stack.clone();
                      c_stack.push(ctxt);
                      stack.insert(0, (e, *array_i, c_stack));
                    }   
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
  use std::default::Default;

  use super::HBData;
  use super::eval;

  #[test]
  fn fetch_key_value() {
    let json = json::from_str(r##"{"a": 1}"##).unwrap();
    let mut buf: Vec<u8> = Vec::new();

    json.value_for_key_path(&vec!["a".to_string()], &vec![]).unwrap().write_value(&mut buf).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "1".to_string());
  }

  #[test]
  fn fetch_key_value_level1() {
    let json = json::from_str(r##"{"a": {"b": 1}}"##).unwrap();
    let mut buf: Vec<u8> = Vec::new();

    json.value_for_key_path(&vec!["a".to_string(), "b".to_string()], &vec![]).unwrap().write_value(&mut buf).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "1".to_string());
  }

  #[test]
  fn fetch_key_value_array_level1() {
    let json = json::from_str(r##"{"a": [1, 2, 3]}"##).unwrap();
    let mut buf: Vec<u8> = Vec::new();

    json.value_for_key_path(&vec!["a".to_string(), "0".to_string()], &vec![]).unwrap().write_value(&mut buf).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "1".to_string());
  }

  #[test]
  fn resolve_this_in_keypath() {
    let json = json::from_str(r##""hello""##).unwrap();
    let mut buf: Vec<u8> = Vec::new();

    json.value_for_key_path(&vec![".".to_string()], &vec![]).unwrap().write_value(&mut buf).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "hello".to_string());
  }

  #[test]
  fn resolve_this_subkey_in_keypath() {
    let json = json::from_str(r##"{"t": "hello"}"##).unwrap();
    let mut buf: Vec<u8> = Vec::new();

    json.value_for_key_path(&vec![".".to_string(), "t".to_string()], &vec![]).unwrap().write_value(&mut buf).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "hello".to_string());
  }

  #[test]
  fn deep_path_none() {
    let json = json::from_str(r##"{"a": 1}"##).unwrap();

    match json.value_for_key_path(&vec!["a".to_string(), "b".to_string()], &vec![]) {
      Some(_) => assert!(false),
      None    => assert!(true),
    }
  }

  #[test]
  fn compile_call() {
    let json = json::from_str(r##"{"a": 1}"##).unwrap();
    let templ = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&templ, &json, &mut buf).unwrap();
  }

}


