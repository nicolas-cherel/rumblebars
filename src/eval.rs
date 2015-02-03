use std::io::{IoError, Writer};
use serialize::json::Json;
use std::collections::HashMap;
use std::num::Float;
use std::vec::Vec;
use std::default::Default;

use parse::Template;
use parse::HBEntry;
use parse::HBExpression;
use parse::HBValHolder;

fn value_for_key_path_in_context<'a>(
  data: &'a HBData,
  key_path: &Vec<String>,
  context_stack: &Vec<&'a HBData>,
  global_data: &HashMap<&str, &'a HBData>,
) ->  Option<&'a (HBData + 'a)>
{
  let mut ctxt = Some(data);
  let mut stack_index = 0;

  for key in key_path.iter() {
    match key.as_slice() {
      "."  => {continue},
      ".." => {
        stack_index += 1;
        ctxt = if stack_index <= context_stack.len() {
          Some(*context_stack.get(context_stack.len() - stack_index).unwrap())
        } else {
          ctxt
        };

        continue;
      },
      _ if key.starts_with("@") => {
        ctxt = match global_data.get(key.as_slice()) {
          Some(&val) => Some(val),
          None => ctxt,
        };

        continue;
      },
      _ => (),
    }

    ctxt = match ctxt {
      Some(c) => c.get_key(key.as_slice()),
      _ => return None, // not found
    }
  }

  return ctxt;
}

pub enum HBNodeType<T> {
  Branch(T),
  Array(T),
  Leaf(T),
  Null,
}

pub type HBEvalResult = Result<(), IoError>;



pub trait HBData {
  fn write_value(&self, out: &mut Writer) -> HBEvalResult;
  fn typed_node(&self) -> HBNodeType<&HBData>;
  fn as_array(&self) -> Option<Vec<&HBData>>;
  fn get_key(&self, key: &str) -> Option<&HBData>;
  fn as_bool(&self) -> bool;
}


impl HBData for Json {

  fn typed_node(&self) -> HBNodeType<&HBData> {
    return match self {
      &Json::Object(_) => HBNodeType::Branch(self as &HBData),
      &Json::Array(_)  => HBNodeType::Array(self as &HBData),
      &Json::Null      => HBNodeType::Null,
      _                => HBNodeType::Leaf(self as &HBData),
    }
  }

  fn write_value(&self, out: &mut Writer) -> HBEvalResult {
    return match self {
      &Json::I64(ref i)     => write!(out, "{}", i),
      &Json::U64(ref u)     => write!(out, "{}", u),
      &Json::F64(ref f)     => write!(out, "{}", f),
      &Json::String(ref s)  => write!(out, "{}", s),
      &Json::Boolean(ref b) => write!(out, "{}", b),
      _  => Ok(()),
    }
  }

  fn as_array(&self) -> Option<Vec<&HBData>> {
    return match self {
      &Json::Array(ref a) => {
        Some(a.iter().map(|e| { e as &HBData }).collect())
      },
      _ => None,
    }

  }

  fn get_key(&self, key: &str) -> Option<&HBData> {
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
        return match self.find(key.as_slice()) {
          Some(json) =>  Some(json as &HBData),
          None => None,
        }
      },
      _ => None,
    }
  }

  fn as_bool(&self) -> bool {
    return match self {
      &Json::I64(ref i)     => *i == 0,
      &Json::U64(ref u)     => *u == 0,
      &Json::F64(ref f)     => (*f == Float::nan() || *f == 0.0),
      &Json::String(ref s)  => s.as_slice() == "",
      &Json::Boolean(ref b) => *b,
      &Json::Null           => false,
      _  => true,
    }
  }

}

impl HBData for String {
  fn write_value(&self, out: &mut Writer) -> HBEvalResult {
    write!(out, "{}", self)
  }

  fn typed_node<'a>(&'a self) -> HBNodeType<&'a HBData> {
    HBNodeType::Leaf(self as &HBData)
  }

  fn as_bool(&self) -> bool { self.as_slice() == "" }

  fn as_array<'a>(&'a self) -> Option<Vec<&'a HBData>> { None }
  fn get_key<'a>(&'a self, _: &str) -> Option<&'a HBData> { None }
}

pub type HBHelperFunction = fn(params: &[&HBData], options: &HelperOptions, out: &mut Writer, hb_context: &EvalContext) -> HBEvalResult;

#[deriving(Copy)]
pub struct Helper {
  helper_func: HBHelperFunction,
}

pub type HelperOptionsByName<'a> = HashMap<&'a String, &'a (HBData + 'a)>;

// alow dead, only used from user defined helpers
#[allow(dead_code)]
pub struct HelperOptions<'a> {
  block: Option<&'a Template>,
  inverse: Option<&'a Template>,
  pub context: &'a (HBData + 'a),
  hb_context: &'a EvalContext,
  pub condition: bool,
  global_data: &'a HashMap<&'a str, &'a (HBData + 'a)>,
  context_stack: &'a Vec<&'a (HBData + 'a)>,
  // options: HelperOptionsByName<'a>,
  options: &'a [(String, HBValHolder)],
}

// alow dead, only used from user defined helpers
#[allow(dead_code)]
impl <'a> HelperOptions<'a> {
  fn render_template(&self, t: &'a Template, data: &'a HBData, out: &mut Writer) -> HBEvalResult {
    let h = HashMap::new();
    eval_with_globals(t, data, out, self.hb_context, &h, self.context_stack)
  }

  pub fn option_by_name(&self, name: &String) -> Option<&'a(HBData + 'a)> {
    match self.options.iter().find(|&&(ref n, ref v)| { n == name }) {
      Some(&(_, HBValHolder::String(ref s))) => Some(s as &HBData),
      Some(&(_, HBValHolder::Path(ref p))) => value_for_key_path_in_context(self.context, p, self.context_stack, self.global_data),
      _ => None,
    }
  }

  pub fn block_ok(&self, out: &mut Writer) -> HBEvalResult{
    match self.block {
      Some(t) => self.render_template(t, self.context, out),
      None    => Ok(()),
    }
  }

  pub fn inverse(&self, out: &mut Writer) -> HBEvalResult{
    match self.inverse {
      Some(t) => self.render_template(t, self.context, out),
      None    => Ok(()),
    }
  }

  pub fn block_ok_with_globals(&self, out: &mut Writer, globals: &HashMap<&str, &HBData>) -> HBEvalResult {
    let mut h = HashMap::new();

    for (k, v) in self.global_data.iter() {
      h.insert(*k, *v);
    }

    for (k, v) in globals.iter() {
      h.insert(*k, *v);
    }

    for k in h.keys() {
      println!("global {}",  k);
    }


    match self.block {
      Some(t) => eval_with_globals(t, self.context, out, self.hb_context, &h, self.context_stack),
      None    => Ok(()),
    }
  }
}

impl Helper {
  pub fn new_with_function(f: HBHelperFunction) -> Helper {
    Helper { helper_func: f }
  }

  fn build_param_vec<'a, 'b>(
    context: &'a HBData,
    params: &'a [HBValHolder],
    ctxt_stack: &'b Vec<&'a HBData>,
    global_data: &HashMap<&str, &'a HBData>
  ) -> Vec<&'a (HBData + 'a)>
  {
    let mut evaluated_params: Vec<&HBData> = vec![];
    for v in params.iter() {
      match v {
        &HBValHolder::String(ref s) => evaluated_params.push(s as &HBData),
        &HBValHolder::Path(ref p) => if let Some(d) = value_for_key_path_in_context(context, p, ctxt_stack, global_data) {
          evaluated_params.push(d)
        },
      }
    };

    evaluated_params
  }

  fn build_options_map<'a, 'b>(
    context:&'a HBData,
    options: &'a [(String, HBValHolder)],
    ctxt_stack: &'b Vec<&'a HBData>,
    global_data: &HashMap<&str, &'a HBData>
  ) -> HelperOptionsByName<'a>
  {
    let mut options_iter = options.iter().map(|&(ref name, ref val)| {
      match val {
        &HBValHolder::String(ref s) => Some((name, s as &HBData)),
        &HBValHolder::Path(ref p) => {
          if let Some(v) = value_for_key_path_in_context(context, p, ctxt_stack, global_data) {
            Some((name, v))
          } else {
            None
          }
        }
      }
    });

    let mut h = HashMap::new();
    for i in options_iter {
      match i {
        Some((n, v)) => h.insert(n, v),
        None => None,
      };
    }

    h
  }

  fn call_for_block<'a, 'b, 'c>(
    &self,
    block: Option<&'a Template>,
    inverse: Option<&'a Template>,
    context: &'a HBData,
    params: &'a [HBValHolder],
    options: &'a [(String, HBValHolder)],
    out: &'b mut Writer,
    hb_context: &'a EvalContext,
    ctxt_stack: &'c Vec<&'a HBData>,
    global_data: &HashMap<&str, &'a HBData>
  ) -> HBEvalResult {

    let condition = match params.as_slice() {
      [ref val, ..] => match val {
        &HBValHolder::String(ref s) => s.as_bool(),
        &HBValHolder::Path(ref p) => if let Some(v) = value_for_key_path_in_context(context, p, ctxt_stack, global_data) {
          v.as_bool()
        } else {
          false
        }
      },
      _ => false
    };

    let helper_options = HelperOptions {
      block: block,
      inverse: inverse,
      context: context,
      hb_context: hb_context,
      condition: condition,
      // options: Helper::build_options_map(options, ctxt_stack, global_data),
      options: options,
      global_data: unsafe { ::std::mem::transmute(global_data) },
      context_stack: unsafe { ::std::mem::transmute(ctxt_stack) },
    };

    (self.helper_func)(Helper::build_param_vec(context, params, ctxt_stack, global_data).as_slice(), &helper_options, out, hb_context)
  }

  fn call_fn<'a, 'b, 'c>(
    &self,
    context: &'a HBData,
    params: &'a [HBValHolder],
    options: &'a [(String, HBValHolder)],
    out: &'b mut Writer,
    hb_context: &'a EvalContext,
    ctxt_stack: &'c Vec<&'a HBData>,
    global_data: &HashMap<&str, &'a HBData>
  ) -> HBEvalResult {
    let helper_options = HelperOptions {
      block: None,
      inverse: None,
      context: context,
      hb_context: hb_context,
      condition: true,
      // options: Helper::build_options_map(options, ctxt_stack, global_data),
      options: options,
      global_data: unsafe { ::std::mem::transmute(global_data) },
      context_stack: unsafe { ::std::mem::transmute(ctxt_stack) },
    };

    (self.helper_func)(Helper::build_param_vec(context, params, ctxt_stack, global_data).as_slice(), &helper_options, out, hb_context)
  }

}


pub struct EvalContext {
  partials: HashMap<String, Template>,
  helpers: HashMap<String, Helper>
}

impl Default for EvalContext {
  fn default() -> EvalContext {
    EvalContext {
      partials: Default::default(),
      helpers: Default::default()
    }
  }
}

impl EvalContext {
  pub fn register_partial(&mut self, name: String, t: Template) {
    self.partials.insert(name, t);
  }

  pub fn partial_with_name(&self, name: &str) -> Option<&Template> {
    return self.partials.get(name);
  }

  pub fn register_helper(&mut self, name: String, h: Helper) {
    self.helpers.insert(name, h);
  }

  pub fn helper_with_name(&self, name: &str) -> Option<&Helper> {
    return self.helpers.get(name);
  }

  pub fn has_helper_with_name(&self, name: &str) -> bool {
    return self.helpers.contains_key(name);
  }
}

pub fn eval(template: &Template, data: &HBData, out: &mut Writer, eval_context: &EvalContext) -> HBEvalResult {
  let log = "info".to_string();
  let mut globals = HashMap::new();
  globals.insert("@root", data);
  globals.insert("@log", &log);

  eval_with_globals(template, data, out, eval_context, &globals, &vec![data])
}

pub fn eval_with_globals<'a: 'b, 'b: 'c, 'c>(template: &'a Template, data: &'a HBData, out: &mut Writer, eval_context: &'a EvalContext, global_data: &HashMap<&str, &'c HBData>, context_stack: &Vec<&'b HBData>) -> HBEvalResult {
  let mut stack:Vec<_> = template.iter().rev().map(|e| {
    (e, data, Vec::new())
  }).collect();

  while stack.len() > 0 {
    if let Some((templ, ctxt, ctxt_stack)) = stack.pop() {
      let w_ok = match templ {
        &box HBEntry::Raw(ref s) => {
          out.write_str(s.as_slice())
        },
        &box HBEntry::Partial(ref exp) => {
          match exp.base.as_slice() {
            [ref single] => {
              match eval_context.partial_with_name(single.as_slice()) {
                Some(t) => {
                  let c_ctxt = if let Some(&HBValHolder::Path(ref p)) = exp.params.get(0) {
                    value_for_key_path_in_context(ctxt, p, &ctxt_stack, global_data).unwrap_or(ctxt)
                  } else {
                    ctxt
                  };

                  for e in t.iter().rev() {
                    let mut c_stack = ctxt_stack.clone();
                    c_stack.push(ctxt);
                    stack.push((e, c_ctxt, c_stack));
                  }
                  Ok(())
                },
                _ => panic!("partial '{}' not found", exp.path())
              }
            }
            [_, ..] => panic!("invalid partial name '{}'", exp.path()),
            [] => panic!("invalid empty string to retrieve partial by name"),
          }
        },

        &box HBEntry::Eval(HBExpression{ref base, ref params, ref options, ref render_options, block: None, else_block: None}) => {
          match base.as_slice() {
            [ref single] if eval_context.has_helper_with_name(single.as_slice()) => {
              let helper = eval_context.helper_with_name(single.as_slice()).unwrap();
              // let helper_params =
              helper.call_fn(ctxt, params.as_slice(), options.as_slice(), out, eval_context, &ctxt_stack, global_data)
            },
            _ => match value_for_key_path_in_context(ctxt, base, &ctxt_stack, global_data) {
              Some(v) => match v.typed_node() {
                HBNodeType::Leaf(_) => v.write_value(out),
                _ => Ok(()),
              },
              None => Ok(()),
            }
          }
        },

        &box HBEntry::Eval(HBExpression{ref base, ref params, ref options, ref render_options, ref block, ref else_block}) => {
          match base.as_slice() {
            [ref single] if eval_context.has_helper_with_name(single.as_slice()) => {
              let helper = eval_context.helper_with_name(single.as_slice()).unwrap();
              let blocks: Vec<_> = [block, else_block].iter().map(|b| {
                match b {
                  &&Some(box ref t) => Some(t),
                  &&None => None,
                }
              }).collect();
              if let [opt_block, opt_else_block] = blocks.as_slice() {
                helper.call_for_block(
                  opt_block,
                  opt_else_block,
                  ctxt,
                  params.as_slice(),
                  options.as_slice(),
                  out,
                  eval_context,
                  &ctxt_stack,
                  global_data
                )
              } else {
                Ok(())
              }
            },
            _ => {
              let c_ctxt = value_for_key_path_in_context(ctxt, base, &ctxt_stack, global_data);

              match (c_ctxt, block) {
                (Some(c), &Some(ref block_found)) => {
                  match c.typed_node() {
                    HBNodeType::Branch(_) => {
                      for e in block_found.iter().rev() {
                        let mut c_stack = ctxt_stack.clone();
                        c_stack.push(ctxt);
                        stack.push((e, c, c_stack));
                      }
                    },
                    HBNodeType::Array(a) => {
                      if let Some(collection) = a.as_array() {
                        for array_i in collection.iter().rev() {
                          for e in block_found.iter().rev() {
                            let mut c_stack = ctxt_stack.clone();
                            c_stack.push(ctxt);
                            stack.push((e, *array_i, c_stack));
                          }
                        }
                      }
                    },
                    _ => (),
                  }
                },
                _ => ()
              }
              Ok(())
            },
          }
        },
      };
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {

  use serialize::json::Json;
  use std::default::Default;
  use std::collections::HashMap;

  use super::value_for_key_path_in_context;
  use super::HBData;
  use super::eval;

  #[test]
  fn fetch_key_value() {
    let json = Json::from_str(r##"{"a": 1}"##).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    let h = HashMap::new();

    value_for_key_path_in_context(&json, &vec!["a".to_string()], &vec![], &h).unwrap().write_value(&mut buf).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "1");
  }

  #[test]
  fn fetch_key_value_level1() {
    let json = Json::from_str(r##"{"a": {"b": 1}}"##).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    let h = HashMap::new();

    value_for_key_path_in_context(&json, &vec!["a".to_string(), "b".to_string()], &vec![], &h).unwrap().write_value(&mut buf).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "1");
  }

  #[test]
  fn fetch_key_value_array_level1() {
    let json = Json::from_str(r##"{"a": [1, 2, 3]}"##).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    let h = HashMap::new();

    value_for_key_path_in_context(&json, &vec!["a".to_string(), "0".to_string()], &vec![], &h).unwrap().write_value(&mut buf).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "1");
  }

  #[test]
  fn resolve_this_in_keypath() {
    let json = Json::from_str(r##""hello""##).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    let h = HashMap::new();

    value_for_key_path_in_context(&json, &vec![".".to_string()], &vec![], &h).unwrap().write_value(&mut buf).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "hello");
  }

  #[test]
  fn resolve_this_subkey_in_keypath() {
    let json = Json::from_str(r##"{"t": "hello"}"##).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    let h = HashMap::new();

    value_for_key_path_in_context(&json, &vec![".".to_string(), "t".to_string()], &vec![], &h).unwrap().write_value(&mut buf).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "hello");
  }

  #[test]
  fn deep_path_none() {
    let json = Json::from_str(r##"{"a": 1}"##).unwrap();
    let h = HashMap::new();

    match value_for_key_path_in_context(&json, &vec!["a".to_string(), "b".to_string()], &vec![], &h) {
      Some(_) => assert!(false),
      None    => assert!(true),
    }
  }

  #[test]
  fn compile_call() {
    let json = Json::from_str(r##"{"a": 1}"##).unwrap();
    let templ = Default::default();
    let mut buf: Vec<u8> = Vec::new();

    eval(&templ, &json, &mut buf, &Default::default()).unwrap();
  }

}


