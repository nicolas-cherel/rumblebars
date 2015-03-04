use std::old_io::{IoError, Writer};
use serialize::json::Json;
use std::collections::HashMap;
use std::slice::SliceExt;
use std::num::Float;
use std::vec::Vec;
use std::default::Default;
use regex::Regex;

use parse::Template;
use parse::HBEntry;
use parse::HBExpression;
use parse::HBValHolder;

fn value_for_key_path_in_context<'a>(
  data: &'a HBData,
  key_path: &Vec<String>,
  context_stack: &Vec<&'a HBData>,
  global_data: &HashMap<&str, &'a HBData>,
  compat: bool,
) ->  Option<&'a (HBData + 'a)>
{
  let mut ctxt = Some(data);
  let mut stack_index = 0;
  let mut first_key = true;

  for key in key_path.iter().map(|k| &k[..]) {
    match key {
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
        match global_data.get(key) {
          Some(&val) => {
            ctxt = Some(val);
            continue;
          },
          None => (),
        };
      },
      _ => (),
    }

    ctxt = match ctxt {
      Some(c) => {
        match (compat, first_key, c.get_key(key)) {
          (true, true, None) => {
            let mut found = None;
            for o in context_stack.iter().rev() {
              match o.get_key(key) {
                v @ Some(_) => {
                  found = v;
                  break;
                },
                None => (),
              }
            }
            found
          },
          (_, _, v) => v,
        }
      },
      _ => return None, // not found
    };

    first_key = false;
  }

  return ctxt;
}

pub enum HBNodeType<T> {
  Branch(T),
  Array(T),
  Leaf(T),
  Null,
}

struct IndentWriter<'a> {
  w: &'a mut SafeWriting<'a>,
  indent: Option<String>,
}

impl <'a> IndentWriter<'a> {
  fn with_indent(s: Option<String>, out: &mut SafeWriting, funkt: &Fn(&mut SafeWriting) -> Result<(), IoError>) -> Result<(), IoError> {
    let mut indenter = IndentWriter {w: unsafe { ::std::mem::transmute(out) }, indent: s};
    let mut safe = SafeWriting::Unsafe(&mut indenter);
    funkt(&mut safe)
  }
}

impl <'a> Writer for IndentWriter<'a> {
  fn write_all(&mut self, buf: &[u8]) -> Result<(), IoError> {
    match self.indent {
      None => self.w.write_all(buf),
      Some(ref indent_str) => {
        let nl = "\n".as_bytes()[0];

        let mut r = Ok(());
        for c in buf.iter() {
          r = match c {
            &chr if chr == nl => {
              r = self.w.write_u8(chr);
              if r.is_ok() {
                self.w.write_str(&indent_str)
              } else {
                r
              }
            },
            &chr => {
              self.w.write_u8(chr)
            }
          };

          if r.is_err() {
            break;
          }
        }

        r
      },
    }
  }
}

pub enum SafeWriting<'a> {
  Safe(&'a mut (SafeWriter +'a)),
  Unsafe(&'a mut (Writer +'a)),
}

impl <'a> SafeWriting<'a> {
  fn into_unsafe(&mut self) -> SafeWriting {
    match self {
      &mut SafeWriting::Safe(ref mut w) => SafeWriting::Unsafe(w.writer()),
      &mut SafeWriting::Unsafe(ref mut w) => SafeWriting::Unsafe(w),
    }
  }
}

impl <'a> Writer for SafeWriting<'a> {
  fn write_all(&mut self, buf: &[u8]) -> Result<(), IoError> {
    match self {
      &mut SafeWriting::Safe(ref mut w)  => w.write_all(buf),
      &mut SafeWriting::Unsafe(ref mut w) => w.write_all(buf),
    }
  }
}

pub trait SafeWriter: Writer {
  fn writer(&mut self) -> &mut Writer;
}


pub struct HTMLSafeWriter<'a> {
  w: &'a mut (Writer + 'a)
}

impl <'a> HTMLSafeWriter<'a> {
  fn new(writer: &'a mut (Writer + 'a)) -> HTMLSafeWriter {
    HTMLSafeWriter {
      w: writer
    }
  }
}

impl <'a> Writer for HTMLSafeWriter<'a> {
  fn write_all(&mut self, buf: &[u8]) -> Result<(), IoError> {
    let mut r = Ok(());
    if let Ok(_str) = ::std::str::from_utf8(buf) {
      let writer = self.writer();
      for c in _str.chars() {
        r = match c {
          '<'  => writer.write_str("&lt;"),
          '>'  => writer.write_str("&gt;"),
          '&'  => writer.write_str("&amp;"),
          '"'  => writer.write_str("&quot;"),
          '\'' => writer.write_str("&#x27;"),
          '`'  => writer.write_str("&#x60;"),
          '\\' => writer.write_str("\\"),

          chr => {
            writer.write_char(chr)
          }
        };

        if r.is_err() {
          break;
        }
      }
    }

    r
  }
}

impl <'a> SafeWriter for HTMLSafeWriter<'a> {
  fn writer(&mut self) -> &mut Writer {
    self.w
  }
}

pub type HBEvalResult = Result<(), IoError>;

pub trait HBData  {
  fn write_value(&self, out: &mut SafeWriting) -> HBEvalResult;
  fn typed_node(&self) -> HBNodeType<&HBData>;
  fn as_array(&self) -> Option<Vec<&HBData>>;
  fn get_key(&self, key: &str) -> Option<&HBData>;
  fn as_bool(&self) -> bool;
  fn keys(&self) -> Option<Vec<&str>>;
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

  fn write_value(&self, out: &mut SafeWriting) -> HBEvalResult {
    return match self {
      &Json::I64(ref i)     => write!(out, "{}", i),
      &Json::U64(ref u)     => write!(out, "{}", u),
      &Json::F64(ref f)     => write!(out, "{}", f),
      &Json::String(ref s)  => write!(out, "{}", s),
      &Json::Boolean(ref b) => write!(out, "{}", b),
      &Json::Array(ref a)   => {
        let mut err:HBEvalResult = Ok(());

        for (index, e) in a.iter().enumerate() {
          err = err.and(e.write_value(out));

          if index < (a.len() - 1) && err.is_ok() {
            err = err.and(write!(out, ","))
          };

          if err.is_err() {
            break;
          }
        }
        err
      }
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
        if let Ok(num_key) = (&key).parse() {
          match a.get(num_key) {
            Some(v) => Some(v as &HBData),
            None => None,
          }
        } else {
          None
        }
      },
      &Json::Object(_) => {
        return match self.find(&key) {
          Some(json) =>  Some(json as &HBData),
          None => None,
        }
      },
      _ => None,
    }
  }

  fn as_bool(&self) -> bool {
    return match self {
      &Json::I64(ref i)     => *i != 0,
      &Json::U64(ref u)     => *u != 0,
      &Json::F64(ref f)     => *f != 0.0 && ! f.is_nan(),
      &Json::String(ref s)  => &s[..] != "",
      &Json::Boolean(ref b) => *b,
      &Json::Null           => false,
      &Json::Array(ref a)   => !a.is_empty(),
      &Json::Object(_)      => true,
    }
  }

  fn keys(&self) -> Option<Vec<&str>> {
    match self {
      &Json::Object(ref o) => {
        let mut keys:Vec<_> = o.keys().map(|k| &k[..]).collect();
        keys.sort_by(|&:a, b| a.cmp(b));
        Some(keys)
      },
      _ => None,
    }
  }


}

impl HBData for String {
  fn write_value(&self, out: &mut SafeWriting) -> HBEvalResult {
    write!(out, "{}", self)
  }

  fn typed_node<'a>(&'a self) -> HBNodeType<&'a HBData> {
    HBNodeType::Leaf(self as &HBData)
  }

  fn as_bool(&self) -> bool { &self[..] == "" }

  fn as_array<'a>(&'a self) -> Option<Vec<&'a HBData>> { None }
  fn get_key<'a>(&'a self, _: &str) -> Option<&'a HBData> { None }
  fn keys(&self) -> Option<Vec<&str>> { None }
}

struct FallbackToOptions<'a> {
  data: &'a (HBData + 'a),
  options: HashMap<&'a str, &'a (HBData+'a)>,
}

impl <'a> HBData for FallbackToOptions<'a> {
  fn write_value(&self, out: &mut SafeWriting) -> HBEvalResult {
    self.data.write_value(out)
  }

  fn typed_node(&self) -> HBNodeType<&HBData> {
    self.data.typed_node()
  }

  fn as_array(&self) -> Option<Vec<&HBData>> {
    self.data.as_array()
  }

  fn get_key(&self, key: &str) -> Option<&HBData> {
    match self.data.get_key(key) {
      v @ Some(_) => v,
      None => {
        self.options.get(key).map(|&v| v)
      }
    }
  }

  fn as_bool(&self) -> bool {
    self.data.as_bool()
  }

  fn keys(&self) -> Option<Vec<&str>> {
    let mut res = vec![];
    res.append(&mut self.data.keys().iter().map(|e| e.iter().map(|&e| e)).flat_map(|e| e).collect::<Vec<_>>());
    res.append(&mut self.options.keys().map(|&e| e).collect::<Vec<_>>());
    Some(res)
  }
}

pub type HBHelperFunction = fn(params: &[&HBData], options: &HelperOptions, out: &mut SafeWriting, hb_context: &EvalContext) -> HBEvalResult;

#[derive(Copy)]
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

  // rough handlebars path parsing, not solid AT ALL, but should do the job
  fn parse_path(path: &str) -> Vec<String> {
    let path_reg = Regex::new(r##"(\.\.|\.)/?|(@?[^!"#%&\\'()*+,./;<=>\[\]^`{|}~ \t]+)[./]?|\[([^\]]+)\][./]?"##).ok().unwrap();
    let mut r = Vec::new();
    for captures1 in path_reg.captures_iter(path) {
      match (captures1.at(1), captures1.at(2), captures1.at(3)) {
        (Some(dot), None, None) => {
          r.push(dot.to_string())
        },
        (None, Some(id), None) => {
          r.push(id.to_string())
        },
        (None, None, Some(id)) => {
          r.push(id.to_string())
        },
        _ => (),
      }
    }
    r
  }

  fn render_template(&self, template: Option<&'a Template>, data: &'a HBData, out: &mut SafeWriting) -> HBEvalResult {
    match template {
      Some(t) => eval_with_globals(t, data, out, self.hb_context, self.global_data, self.context_stack, None),
      None => Ok(()),
    }

  }

  pub fn option_by_name(&self, name: &String) -> Option<&'a(HBData + 'a)> {
    match self.options.iter().find(|&&(ref n, _)| { n == name }) {
      Some(&(_, HBValHolder::String(ref s))) => Some(s as &HBData),
      Some(&(_, HBValHolder::Path(ref p))) => value_for_key_path_in_context(self.context, p, self.context_stack, self.global_data, self.hb_context.compat),
      _ => None,
    }
  }

  pub fn lookup(&self, key: &HBData) -> Option<&'a (HBData + 'a)> {
    self.lookup_with_context(key, self.context)
  }

  pub fn lookup_with_context(&self, key: &HBData, context: &HBData) -> Option<&'a (HBData + 'a)>  {
    let mut buf:Vec<u8> = vec![];
    let key_write_ok = {
      let mut html_safe = HTMLSafeWriter::new(&mut buf);
      let mut s_writer = SafeWriting::Safe(&mut html_safe);
      key.write_value(&mut s_writer)
    };

    if key_write_ok.is_ok() {
      if let Ok(str_key) = String::from_utf8(buf) {
        let key_path = HelperOptions::parse_path(&str_key);
        value_for_key_path_in_context(unsafe { ::std::mem::transmute(context) }, &key_path, self.context_stack, self.global_data, self.hb_context.compat)
      } else {
        None
      }
    } else {
      None
    }
  }

  pub fn render_fn(&self, out: &mut SafeWriting) -> HBEvalResult{
      self.render_template(self.block, self.context, out)
  }

  pub fn render_fn_with_context(&self, data: &HBData, out: &mut SafeWriting) -> HBEvalResult{
      self.render_template(self.block, unsafe { ::std::mem::transmute(data) }, out)
  }

  pub fn inverse(&self, out: &mut SafeWriting) -> HBEvalResult{
      self.render_template(self.inverse, self.context, out)
  }

  pub fn inverse_with_context(&self, data: &'a HBData, out: &mut SafeWriting) -> HBEvalResult{
      self.render_template(self.inverse, data, out)
  }

  pub fn render_fn_with_context_and_globals(&self, data: &HBData, out: &mut SafeWriting, globals: &HashMap<&str, &HBData>) -> HBEvalResult {
    let mut h = HashMap::new();

    for (k, v) in self.global_data.iter() {
      h.insert(*k, *v);
    }

    for (k, v) in globals.iter() {
      h.insert(*k, *v);
    }

    match self.block {
      Some(t) => eval_with_globals(t, unsafe {::std::mem::transmute(data)}, out, self.hb_context, &h, self.context_stack, None),
      None    => Ok(()),
    }
  }

  pub fn render_fn_with_globals(&self, out: &mut SafeWriting, globals: &HashMap<&str, &HBData>) -> HBEvalResult {
    self.render_fn_with_context_and_globals(self.context, out, globals)
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
    global_data: &HashMap<&str, &'a HBData>,
    hb_context: &'a EvalContext,
  ) -> Vec<&'a (HBData + 'a)>
  {
    params.iter().map(|v| {
      match v {
        &HBValHolder::String(ref s) => s as &HBData,
        &HBValHolder::Path(ref p) => value_for_key_path_in_context(context, p, ctxt_stack, global_data, false).unwrap_or(&hb_context.falsy),
        &HBValHolder::Literal(ref d, ref s) => value_for_key_path_in_context(context, &vec![s.clone()], ctxt_stack, global_data, false)
          .unwrap_or(d as &HBData)
      }
    }).collect::<Vec<_>>()
  }

  fn call_for_block<'a, 'b, 'c>(
    &self,
    block: Option<&'a Template>,
    inverse: Option<&'a Template>,
    inverse_condition: bool,
    context: &'a HBData,
    params: &'a [HBValHolder],
    options: &'a [(String, HBValHolder)],
    out: &'b mut SafeWriting,
    hb_context: &'a EvalContext,
    ctxt_stack: &'c Vec<&'a HBData>,
    global_data: &HashMap<&str, &'a HBData>
  ) -> HBEvalResult {

    let condition = match &params[..] {
      [ref val, ..] => match val {
        &HBValHolder::String(ref s) => s.as_bool(),
        &HBValHolder::Path(ref p) => if let Some(v) = value_for_key_path_in_context(context, p, ctxt_stack, global_data, hb_context.compat) {
          v.as_bool()
        } else {
          false
        },
        &HBValHolder::Literal(ref d, ref s) => value_for_key_path_in_context(context, &vec![s.clone()], ctxt_stack, global_data, false)
          .unwrap_or(d as &HBData).as_bool()
      },
      _ => false
    };

    let helper_options = HelperOptions {
      block: block,
      inverse: inverse,
      context: context,
      hb_context: hb_context,
      condition: (!inverse_condition && condition) || (inverse_condition && !condition),
      // options: Helper::build_options_map(options, ctxt_stack, global_data),
      options: options,
      global_data: unsafe { ::std::mem::transmute(global_data) },
      context_stack: unsafe { ::std::mem::transmute(ctxt_stack) },
    };

    (self.helper_func)(&Helper::build_param_vec(context, params, ctxt_stack, global_data, hb_context), &helper_options, out, hb_context)
  }

  fn call_fn<'a, 'b, 'c>(
    &self,
    context: &'a HBData,
    params: &'a [HBValHolder],
    options: &'a [(String, HBValHolder)],
    out: &'b mut SafeWriting,
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

    (self.helper_func)(&Helper::build_param_vec(context, params, ctxt_stack, global_data, hb_context), &helper_options, out, hb_context)
  }

}


pub struct EvalContext {
  partials: HashMap<String, Template>,
  helpers: HashMap<String, Helper>,
  pub compat: bool,
  falsy: Json,
}

impl Default for EvalContext {
  fn default() -> EvalContext {
    let mut helpers = HashMap::new();

    helpers.insert("each".to_string(),   Helper::new_with_function(::helpers_builtins::each_helper));
    helpers.insert("if".to_string(),     Helper::new_with_function(::helpers_builtins::if_helper));
    helpers.insert("unless".to_string(), Helper::new_with_function(::helpers_builtins::unless_helper));
    helpers.insert("lookup".to_string(), Helper::new_with_function(::helpers_builtins::lookup_helper));
    helpers.insert("with".to_string(),   Helper::new_with_function(::helpers_builtins::with_helper));

    EvalContext {
      partials: Default::default(),
      helpers: helpers,
      compat: false,
      falsy: Json::Null,
    }
  }
}

impl EvalContext {
  pub fn partials(&self) -> &HashMap<String, Template> {
    return &self.partials;
  }

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
  globals.insert("@level", &log);

  let mut html_safe = HTMLSafeWriter::new(out);
  let mut safe_writer = SafeWriting::Safe(&mut html_safe);

  eval_with_globals(template, data, &mut safe_writer, eval_context, &globals, &vec![data], None)
}

pub fn eval_with_globals<'a: 'b, 'b: 'c, 'c>(template: &'a Template, data: &'a HBData, out: &mut SafeWriting, eval_context: &'a EvalContext, global_data: &HashMap<&str, &'c HBData>, context_stack: &Vec<&'b HBData>, indent: Option<String>) -> HBEvalResult {

  // evaluation is done by iterating through each HBEntry to evaluate
  //  - raw copy,
  //  - simple expression evaluation (render value execute helper call)
  //  - partial evalutation, stacking each entry from registered partial)
  //  - block evaluation, stacking each entry of block with parameterized context, with basic flow control (each, if)

  // given the above, we start by stacking each entries of template root level
  // each entry comes along with :
  //  - a ref to their associated context
  //  - a context stack, to have access of context of parent blocks (copied for each entry)
  //  - an indentation level (for partials, copied for each entry)
  let mut stack:Vec<_> = template.iter().rev().map(|e| {
    (e, data, context_stack.iter().map(|s| *s).collect::<Vec<_>>(), indent.clone())
  }).collect();

  let mut partial_options_contexts = vec![];

  while stack.len() > 0 {
    let w_ok = if let Some((ref templ, ref ctxt, ref ctxt_stack, ref indent)) = stack.pop() {
      match ***templ {
        HBEntry::Raw(ref s) => {
          IndentWriter::with_indent(indent.clone(), &mut out.into_unsafe(), &|w| {
            w.write_str(&s)
          })
        },
        HBEntry::Partial(ref exp) => {
          match &exp.base[..] {
            [ref single] => {
              match eval_context.partial_with_name(&single) {
                Some(t) => {
                  let c_ctxt = if let Some(&HBValHolder::Path(ref p)) = exp.params.get(0) {
                    value_for_key_path_in_context(*ctxt, p, &ctxt_stack, global_data, eval_context.compat).unwrap_or(*ctxt)
                  } else {
                    *ctxt
                  };

                  let with_options_fallback = if exp.options.len() > 0 {
                    let mut options_contexts: HashMap<&str, &HBData> = HashMap::new();
                    for o in exp.options.iter() {
                      match o {
                        &(ref name, HBValHolder::String(ref s)) => {
                          options_contexts.insert(&name, s as &HBData);
                        },
                        &(ref name, HBValHolder::Path(ref p)) => {
                          options_contexts.insert(&name, value_for_key_path_in_context(*ctxt, p, &ctxt_stack, global_data, eval_context.compat).unwrap_or(&eval_context.falsy));
                        },
                        &(ref name, HBValHolder::Literal(ref j, _)) => {
                          options_contexts.insert(&name, j as &HBData);
                        },
                      }
                    }

                    // store data into a collections with enough lifetime, with need of a known safe transmute
                    partial_options_contexts.push(FallbackToOptions { data: c_ctxt, options: options_contexts });
                    unsafe { ::std::mem::transmute(partial_options_contexts.last().map(|f| f as &HBData)) }
                  } else {
                    None
                  };

                  // calculate indentation content
                  let may_indent = match (indent, &exp.render_options.indent) {
                    (&None, ref i @ &Some(_)) | (ref i @ &Some(_), &None) => (*i).clone(),
                    (&Some(ref i), &Some(ref j)) => Some(format!("{}{}", i, j)),
                    (&None, &None) => None,
                  };

                  for e in t.iter().rev() {
                    let mut c_stack = ctxt_stack.clone();
                    c_stack.push(*ctxt);
                    stack.push((e, with_options_fallback.unwrap_or(c_ctxt), c_stack, may_indent.clone()));
                  }

                  Ok(())
                },
                _ => Ok(())
              }
            }
            [_, ..] => panic!("invalid partial name '{}'", exp.path()),
            [] => panic!("invalid empty string to retrieve partial by name"),
          }
        },

        HBEntry::Eval(HBExpression{ref base, ref params, ref options, ref render_options, block: None, else_block: None}) => {
          match &base[..] {
            [ref single] if eval_context.has_helper_with_name(&single) => {
              let helper = eval_context.helper_with_name(&single).unwrap();
              if render_options.escape {
                IndentWriter::with_indent(indent.clone(), out, &|w| {
                  helper.call_fn(*ctxt, &params, &options, w, eval_context, &ctxt_stack, global_data)
                })
              } else {
                IndentWriter::with_indent(indent.clone(), &mut out.into_unsafe(), &|w| {
                  helper.call_fn(*ctxt, &params, &options, w, eval_context, &ctxt_stack, global_data)
                })
              }
            },
            _ => match value_for_key_path_in_context(*ctxt, base, &ctxt_stack, global_data, eval_context.compat) {
              Some(v) => match v.typed_node() {
                HBNodeType::Leaf(_) | HBNodeType::Array(_)=> {
                  if render_options.escape {
                    IndentWriter::with_indent(indent.clone(), out, &|w| {
                      v.write_value(w)
                    })
                  } else {
                    IndentWriter::with_indent(indent.clone(), &mut out.into_unsafe(), &|w| {
                      v.write_value(w)
                    })
                  }
                },
                _ => Ok(()),
              },
              None => Ok(()),
            }
          }
        },

        HBEntry::Eval(HBExpression{ref base, ref params, ref options, ref render_options, ref block, ref else_block}) => {
          render_options.escape; // only suppress unused warning
          match &base[..] {
            [ref single] if eval_context.has_helper_with_name(&single) => {
              let helper = eval_context.helper_with_name(&single).unwrap();

              // collect options of deref'd blocks
              let blocks: Vec<_> = [block, else_block].iter().map(|b| {
                match b {
                  &&Some(ref t) => Some(&**t),
                  &&None => None,
                }
              }).collect();

              if let [opt_block, opt_else_block] = &blocks[..] {
                helper.call_for_block(
                  opt_block,
                  opt_else_block,
                  render_options.inverse,
                  *ctxt,
                  &params,
                  &options,
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
              let c_ctxt = value_for_key_path_in_context(*ctxt, base, &ctxt_stack, global_data, eval_context.compat);

              match (c_ctxt.unwrap_or(&eval_context.falsy), block) {
                (c, &Some(ref block_found)) => {
                  match c.typed_node() {
                    HBNodeType::Branch(_) | HBNodeType::Leaf(_) | HBNodeType::Null => {
                      if c.as_bool() && !render_options.inverse || !c.as_bool() && render_options.inverse {
                        for e in block_found.iter().rev() {
                          let mut c_stack = ctxt_stack.clone();
                          c_stack.push(*ctxt);
                          stack.push((e, c, c_stack, indent.clone()));
                        }
                      } else if let &Some(ref inv_block) = else_block {
                        for e in inv_block.iter().rev() {
                          stack.push((e, *ctxt, ctxt_stack.clone(), indent.clone()));
                        }
                      }
                    },
                    HBNodeType::Array(_) => {
                      let inverse = render_options.inverse;
                      let collection: Vec<&HBData> = match c.as_array() {
                        Some(ref a) if inverse &&  a.is_empty() => vec![&eval_context.falsy],
                        Some(ref a) if inverse && !a.is_empty() => vec![],

                        Some(a) => if !inverse { a } else { vec![] },

                        None if inverse =>  vec![&eval_context.falsy],
                        None => vec![],
                      };

                      if ! collection.is_empty() {
                        for array_i in collection.iter().rev() {
                          for e in block_found.iter().rev() {
                            let mut c_stack = ctxt_stack.clone();
                            c_stack.push(*ctxt);
                            stack.push((e, *array_i, c_stack, indent.clone()));
                          }
                        }
                      } else if let &Some(ref inv_block) = else_block {
                        for e in inv_block.iter().rev() {
                          stack.push((e, *ctxt, ctxt_stack.clone(), indent.clone()));
                        }
                      }

                    },
                  }
                },
                _ => ()
              }
              Ok(())
            },
          }
        },
      }
    } else {
      Ok(())
    };

    if w_ok.is_err() { return w_ok };
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
  use super::HelperOptions;
  use super::HTMLSafeWriter;
  use super::SafeWriting;

  #[test]
  fn basic_keypath_matching() {

    assert_eq!(HelperOptions::parse_path("pouet"), vec!["pouet"]);
    assert_eq!(HelperOptions::parse_path("."), vec!["."]);
    assert_eq!(HelperOptions::parse_path("./"), vec!["."]);
    assert_eq!(HelperOptions::parse_path("[pouet]"), vec!["pouet"]);
    assert_eq!(HelperOptions::parse_path("[&!'i%%oeiod'].j"), vec!["&!'i%%oeiod'", "j"]);
    assert_eq!(HelperOptions::parse_path("j.[∂ßé©Ç]"), vec!["j", "∂ßé©Ç"]);
    assert_eq!(HelperOptions::parse_path("t.f"), vec!["t", "f"]);
    assert_eq!(HelperOptions::parse_path("t/f"), vec!["t", "f"]);
    assert_eq!(HelperOptions::parse_path("./prop"), vec![".", "prop"]);
    assert_eq!(HelperOptions::parse_path("../sibling"), vec!["..", "sibling"]);
    assert_eq!(HelperOptions::parse_path("@test"), vec!["@test"]);
    assert_eq!(HelperOptions::parse_path("../@ok"), vec!["..", "@ok"]);
    assert_eq!(HelperOptions::parse_path("t.f.@g.h.i"), vec!["t", "f", "@g", "h", "i"]);
    assert_eq!(HelperOptions::parse_path("../@g/h/i"), vec!["..", "@g", "h", "i"]);

  }

  #[test]
  fn fetch_key_value() {
    let json = Json::from_str(r##"{"a": 1}"##).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    let h = HashMap::new();

    {
      let mut safe_writer = HTMLSafeWriter::new(&mut buf);
      let mut html_safe = SafeWriting::Safe(&mut safe_writer);
      value_for_key_path_in_context(&json, &vec!["a".to_string()], &vec![], &h, false).unwrap().write_value(&mut html_safe).unwrap();
    }


    assert_eq!(String::from_utf8(buf).unwrap(), "1");
  }

  #[test]
  fn fetch_key_value_level1() {
    let json = Json::from_str(r##"{"a": {"b": 1}}"##).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    let h = HashMap::new();

    {
      let mut safe_writer = HTMLSafeWriter::new(&mut buf);
      let mut html_safe = SafeWriting::Safe(&mut safe_writer);
      value_for_key_path_in_context(&json, &vec!["a".to_string(), "b".to_string()], &vec![], &h, false).unwrap().write_value(&mut html_safe).unwrap();
    }


    assert_eq!(String::from_utf8(buf).unwrap(), "1");
  }

  #[test]
  fn fetch_key_value_array_level1() {
    let json = Json::from_str(r##"{"a": [1, 2, 3]}"##).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    let h = HashMap::new();

    {
      let mut safe_writer = HTMLSafeWriter::new(&mut buf);
      let mut html_safe = SafeWriting::Safe(&mut safe_writer);
      value_for_key_path_in_context(&json, &vec!["a".to_string(), "0".to_string()], &vec![], &h, false).unwrap().write_value(&mut html_safe).unwrap();
    }


    assert_eq!(String::from_utf8(buf).unwrap(), "1");
  }

  #[test]
  fn resolve_this_in_keypath() {
    let json = Json::from_str(r##""hello""##).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    let h = HashMap::new();

    {
      let mut safe_writer = HTMLSafeWriter::new(&mut buf);
      let mut html_safe = SafeWriting::Safe(&mut safe_writer);
      value_for_key_path_in_context(&json, &vec![".".to_string()], &vec![], &h, false).unwrap().write_value(&mut html_safe).unwrap();
    }


    assert_eq!(String::from_utf8(buf).unwrap(), "hello");
  }

  #[test]
  fn resolve_this_subkey_in_keypath() {
    let json = Json::from_str(r##"{"t": "hello"}"##).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    let h = HashMap::new();

    {
      let mut safe_writer = HTMLSafeWriter::new(&mut buf);
      let mut html_safe = SafeWriting::Safe(&mut safe_writer);
      value_for_key_path_in_context(&json, &vec![".".to_string(), "t".to_string()], &vec![], &h, false).unwrap().write_value(&mut html_safe).unwrap();
    }


    assert_eq!(String::from_utf8(buf).unwrap(), "hello");
  }

  #[test]
  fn deep_path_none() {
    let json = Json::from_str(r##"{"a": 1}"##).unwrap();
    let h = HashMap::new();

    match value_for_key_path_in_context(&json, &vec!["a".to_string(), "b".to_string()], &vec![], &h, false) {
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


