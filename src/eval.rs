use std::io;
use std::io::Write;
use serialize::json::Json;
use std::collections::HashMap;
use std::vec::Vec;
use std::default::Default;
use regex::Regex;

use parse::Template;
use parse::Entries;
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

/// enum to qualify data node type,
/// Branch for key/values assoc
/// Array for collection
/// Leaf for scalar-like data
/// Null Special Leaf that holds nothing
pub enum HBNodeType<T> {
  Branch(T),
  Array(T),
  Leaf(T),
  Null,
}

struct IndentWriter<'a> {
  w: &'a mut (io::Write+'a),
  indent: Option<String>,
}

impl <'a> IndentWriter<'a> {
  fn with_indent(s: Option<String>, out: &mut SafeWriting, funkt: &Fn(&mut SafeWriting) -> io::Result<()>) -> io::Result<()> {
    let mut indenter = IndentWriter {w: out, indent: s};
    let mut safe = SafeWriting::Unsafe(&mut indenter);
    funkt(&mut safe)
  }
}


impl <'a> io::Write for IndentWriter<'a> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    match self.indent {
      None => self.w.write(buf),
      Some(ref indent_str) => {
        let mut r = Ok(0);
        let ref mut writer = self.w;
        let as_utf8 = unsafe { ::std::str::from_utf8_unchecked(buf) };
        let mut chars = as_utf8.char_indices();
        let mut i = chars.next().unwrap_or((0, ' ')).0; // init with first char
        while i < as_utf8.len() {
          match chars.next().or(Some((as_utf8.len(), ' '))) {
            Some((next, _)) => {
              r = match &as_utf8[i..next] {
                "\n"  => {
                  writer.write("\n".as_bytes()).and_then(|_| {
                    writer.write(&indent_str.as_bytes())
                  })
                },
                chr => {
                  writer.write(chr.as_bytes())
                }
              };

              i = next;

              if r.is_err() {
                break;
              }
            },
            None => break,
          }
        }

        r.and(Ok(buf.len()))
      },
    }
  }

  fn flush(&mut self) -> io::Result<()> {
    self.w.flush()
  }
}

/// Enum that holds either a writer or a wrapped writer, this
/// is needed to opt-out filtering easily, so rumblebar uses `SafeWriting`
/// to make use of `into_safe()` mostly transparent and stateless, while
/// enforcing safe writing through the api.
///
/// Typically, the wrapped writer will filter unsafe string/characters.
///
/// # Examples
///
/// Simple html escaping and unsafe support :
///
/// ```
/// use std::io::Write;
/// use rumblebars::SafeWriting;
/// let mut buf = Vec::new();
///
/// SafeWriting::with_html_safe_writer(&mut buf, &|out| {
///   out.write_all("<>".as_bytes());
///   out.into_unsafe().write_all("<>".as_bytes())
/// });
///
/// assert_eq!(String::from_utf8(buf).unwrap(), "&lt;&gt;<>")
/// ```
/// Double escaping will work as expected :
///
/// ```
/// use std::io::Write;
/// use rumblebars::HTMLSafeWriter;
/// use rumblebars::SafeWriting;
/// let mut buf = Vec::new();
/// {
///   let mut esc1 = HTMLSafeWriter::new(&mut buf);
///   SafeWriting::with_html_safe_writer(&mut esc1, &|out| {
///     out.write_all("<>".as_bytes())
///   });
/// }
///
/// assert_eq!(String::from_utf8(buf).unwrap(), "&amp;lt;&amp;gt;")
/// ```
pub enum SafeWriting<'a> {
  Safe(&'a mut (SafeWriter +'a)),
  Unsafe(&'a mut (io::Write +'a)),
}

impl <'a> SafeWriting<'a> {
  /// disclose the unsafe writer without touching self
  pub fn into_unsafe(&mut self) -> SafeWriting {
    match self {
      &mut SafeWriting::Safe(ref mut w) => SafeWriting::Unsafe(w.writer()),
      &mut SafeWriting::Unsafe(ref mut w) => SafeWriting::Unsafe(w),
    }
  }

  /// quick way to get a SafeWriter that escapes html
  pub fn with_html_safe_writer(out: &mut io::Write, safe: &Fn(&mut SafeWriting) -> HBEvalResult) -> HBEvalResult {
    let mut html_safe = HTMLSafeWriter::new(out);
    safe(&mut SafeWriting::Safe(&mut html_safe))
  }
}

impl <'a> io::Write for SafeWriting<'a> {
  fn write(&mut self, buf: &[u8]) -> io::Result<(usize)> {
    match self {
      &mut SafeWriting::Safe(ref mut w)  => w.write(buf),
      &mut SafeWriting::Unsafe(ref mut w) => w.write(buf),
    }
  }

  fn flush(&mut self) -> io::Result<()> {
    match self {
      &mut SafeWriting::Safe(ref mut w)  => w.flush(),
      &mut SafeWriting::Unsafe(ref mut w) => w.flush(),
    }
  }
}

pub trait SafeWriter: io::Write {
  fn writer(&mut self) -> &mut io::Write;
}

/// This writer implementation wraps a Writer and
/// will escape all unsafe html characters
pub struct HTMLSafeWriter<'a> {
  w: &'a mut (io::Write + 'a)
}

impl <'a> HTMLSafeWriter<'a> {
  pub fn new(writer: &'a mut (io::Write + 'a)) -> HTMLSafeWriter {
    HTMLSafeWriter {
      w: writer
    }
  }
}

impl <'a> io::Write for HTMLSafeWriter<'a> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    let mut r = Ok(0);
    let writer = self.writer();

    let as_utf8 = unsafe { ::std::str::from_utf8_unchecked(buf) };
    let mut chars = as_utf8.char_indices();
    let mut i = chars.next().unwrap_or((0, ' ')).0; // init with first char
    while i < as_utf8.len() {
      match chars.next().or(Some((as_utf8.len(), ' '))) {
        Some((next, _)) => {

          r = match &as_utf8[i..next] {
            "<"  => writer.write("&lt;".as_bytes()),
            ">"  => writer.write("&gt;".as_bytes()),
            "&"  => writer.write("&amp;".as_bytes()),
            "\"" => writer.write("&quot;".as_bytes()),
            "\'" => writer.write("&#x27;".as_bytes()),
            "`"  => writer.write("&#x60;".as_bytes()),
            "\\" => writer.write("\\".as_bytes()),
            chr  => writer.write(chr.as_bytes()),
          };

          i = next;

          if r.is_err() {
            break;
          }
        },
        None => break,
      }
    }

    r.and(Ok(buf.len()))
  }

  fn flush(&mut self) -> io::Result<()> {
    self.writer().flush()
  }
}

impl <'a> SafeWriter for HTMLSafeWriter<'a> {
  fn writer(&mut self) -> &mut io::Write {
    self.w
  }
}

pub type HBEvalResult = io::Result<()>;
pub type HBKeysIter<'a> = Box<Iterator<Item = &'a str> + 'a>;
pub type HBValuesIter<'a> = Box<Iterator<Item = &'a (HBData + 'a)> + 'a>;
pub type HBIter<'a> = Box<Iterator<Item = (&'a str, &'a (HBData + 'a))> + 'a>;

pub trait HBData  {
  fn write_value(&self, out: &mut SafeWriting) -> HBEvalResult;

  fn typed_node(&self) -> HBNodeType<&HBData>;
  fn get_key(&self, key: &str) -> Option<&HBData>;
  fn as_bool(&self) -> bool;

  fn keys<'a>(&'a self)   -> HBKeysIter<'a>;
  fn values<'a>(&'a self) -> HBValuesIter<'a>;

  fn iter<'a>(&'a self)   -> HBIter<'a>;
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

  fn values<'a>(&'a self) -> HBValuesIter<'a> {
    return match self {
      &Json::Array(ref a)   => Box::new(a.iter().map(|v| v as &'a HBData)) as HBValuesIter<'a>,
      &Json::Object(_)      => Box::new(self.iter().map(|(_, v)| v as &'a HBData))  as HBValuesIter<'a>,
      _                     => Box::new(None.into_iter()),
    }
  }

  fn keys<'a>(&'a self) -> HBKeysIter<'a> {
    self.as_object().map(|o| Box::new(o.keys().map(|s| &s[..])) as HBKeysIter<'a> ).unwrap_or(Box::new(None.into_iter()))
  }

  fn iter<'a>(&'a self) -> HBIter<'a> {
    self.as_object().map(|o|
      Box::new(o.into_iter().map(|(s, j)| (&s[..], j as &HBData))) as HBIter<'a>
    ).unwrap_or(Box::new(None.into_iter()))
  }
}

impl HBData for String {
  fn write_value(&self, out: &mut SafeWriting) -> HBEvalResult {
    write!(out, "{}", self)
  }

  fn typed_node<'a>(&'a self) -> HBNodeType<&'a HBData> {
    HBNodeType::Leaf(self as &HBData)
  }

  fn as_bool(&self) -> bool { &self[..] != "" }

  fn get_key<'a>(&'a self, _: &str) -> Option<&'a HBData> { None }
  fn keys<'a>(&'a self) -> HBKeysIter<'a> { Box::new(None.into_iter()) }
  fn values<'a>(&'a self) -> HBValuesIter<'a> { Box::new(None.into_iter()) }
  fn iter<'a>(&'a self) -> HBIter<'a> { Box::new(None.into_iter()) }
}

impl<'b> HBData for &'b str {
  fn write_value(&self, out: &mut SafeWriting) -> HBEvalResult {
    write!(out, "{}", self)
  }

  fn typed_node<'a>(&'a self) -> HBNodeType<&'a HBData> {
    HBNodeType::Leaf(self as &HBData)
  }

  fn as_bool(&self) -> bool { *self != "" }

  fn get_key<'a>(&'a self, _: &str) -> Option<&'a HBData> { None }
  fn keys<'a>(&'a self) -> HBKeysIter<'a> { Box::new(None.into_iter()) }
  fn values<'a>(&'a self) -> HBValuesIter<'a> { Box::new(None.into_iter()) }
  fn iter<'a>(&'a self) -> HBIter<'a> { Box::new(None.into_iter()) }
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

  fn keys<'b>(&'b self) -> HBKeysIter<'b> {
    Box::new(self.data.keys().chain(self.options.keys().map(|&s| s))) as HBKeysIter<'b>
  }

  fn values<'b>(&'b self) -> HBValuesIter<'b> {
    Box::new(self.iter().map(|(_, v)| v as &'b HBData))
  }

  fn iter<'b>(&'b self) -> HBIter<'b> {
    Box::new(self.data.iter().chain(self.options.iter().map(|(&s, &v)| (s, v)))) as HBIter<'b>
  }
}

pub type HelperFunction = Box<Fn(&[&HBData], &HelperOptions, &mut SafeWriting, &EvalContext) -> HBEvalResult>;

pub struct Helper {
  helper_func: HelperFunction,
}

pub type HelperOptionsByName<'a> = HashMap<&'a String, &'a (HBData + 'a)>;


/// Provides rendering and lookup features to registered helpers
#[allow(dead_code)] // alow dead, only used from user defined helpers
pub struct HelperOptions<'a> {
  /// current expansion data context
  pub context: &'a (HBData + 'a),
  /// if a parameter was given to helper, holds the result of its truthy/falsy evaluation
  pub condition: bool,
  block: Option<&'a Entries>,
  inverse: Option<&'a Entries>,
  hb_context: &'a EvalContext,
  global_data: &'a HashMap<&'a str, &'a (HBData + 'a)>,
  context_stack: &'a Vec<&'a (HBData + 'a)>,
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

  fn render_template(&self, template: Option<&'a Entries>, data: &'a HBData, out: &mut SafeWriting) -> HBEvalResult {
    match template {
      Some(t) => eval_with_globals(t, data, out, self.hb_context, self.global_data, self.context_stack, None),
      None => Ok(()),
    }

  }

  /// data passed as options (eg `{{helper option="one" option=name.val}}`)
  pub fn option_by_name(&self, name: &String) -> Option<&'a(HBData + 'a)> {
    match self.options.iter().find(|&&(ref n, _)| { n == name }) {
      Some(&(_, HBValHolder::String(ref s))) => Some(s as &HBData),
      Some(&(_, HBValHolder::Path(ref p))) => value_for_key_path_in_context(self.context, p, self.context_stack, self.global_data, self.hb_context.compat),
      _ => None,
    }
  }

  /// fetch data at given path
  ///
  /// # Examples
  ///
  /// ```
  /// use rumblebars::EvalContext;
  ///
  /// let mut context = EvalContext::new();
  ///
  /// context.register_helper("fetch_hello".to_string(), Box::new(
  ///   |params, options, out, hb_context| {
  ///     // if context is {"hello": {"content": "hi"}}, returns Some("hi")
  ///     options.lookup(&"hello.content");
  ///     Ok(())
  /// }));
  /// ```

  pub fn lookup(&self, key: &HBData) -> Option<&'a (HBData + 'a)> {
    self.lookup_with_context(key, self.context)
  }


  /// same as lookup, but with custom context instead of current one (this)
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

  /// for a helper with a block, such as `{{if}}{{/if}}`, renders block content
  pub fn render_fn(&self, out: &mut SafeWriting) -> HBEvalResult{
      self.render_template(self.block, self.context, out)
  }

  /// same as render_fn, but with custom context
  pub fn render_fn_with_context(&self, data: &HBData, out: &mut SafeWriting) -> HBEvalResult{
      self.render_template(self.block, unsafe { ::std::mem::transmute(data) }, out)
  }

  /// for a helpers with blocks, such as `{{if}}{{else}}{{/if}}`, renders the `{{else}}` block content
  pub fn inverse(&self, out: &mut SafeWriting) -> HBEvalResult{
      self.render_template(self.inverse, self.context, out)
  }

  /// same as inverse, but with custom context
  pub fn inverse_with_context(&self, data: &'a HBData, out: &mut SafeWriting) -> HBEvalResult{
      self.render_template(self.inverse, data, out)
  }

  /// allow block rendering with custom context and custom globals (data available with @key)
  ///
  /// # Examples
  ///
  /// ```
  /// use rumblebars::EvalContext;
  /// use rumblebars::HBData;
  /// use ::std::default::Default;
  ///
  /// let mut context: EvalContext = Default::default();
  /// let data = "Some Data".to_string();
  /// context.register_helper("fetch_hello".to_string(), Box::new(
  ///   move |params, options, out, hb_context| {
  ///     let mut globals = ::std::collections::HashMap::<&str, &HBData>::new();
  ///     globals.insert("@some_key", &data);
  ///     // if block contains {{@some_key}}, it'll expand to 'Some Data'
  ///     options.render_fn_with_context_and_globals(options.context, out, &globals)
  /// }));
  /// ```

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
  pub fn new_with_function(f: HelperFunction) -> Helper {
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
    block: Option<&'a Entries>,
    inverse: Option<&'a Entries>,
    inverse_condition: bool,
    context: &'a HBData,
    params: &'a [HBValHolder],
    options: &'a [(String, HBValHolder)],
    out: &'b mut SafeWriting,
    hb_context: &'a EvalContext,
    ctxt_stack: &'c Vec<&'a HBData>,
    global_data: &HashMap<&str, &'a HBData>
  ) -> HBEvalResult {

    let condition = match params.first() {
      Some(val) => match val {
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

/// Stores partials, helpers and configuration flags
pub struct EvalContext {
  partials: HashMap<String, Template>,
  helpers: HashMap<String, Helper>,
  /// mustache compatibility flag, to enable recursive lookups
  pub compat: bool,
  falsy: Json,
}

impl Default for EvalContext {
  fn default() -> EvalContext {
    let mut helpers = HashMap::new();

    helpers.insert("each".to_string(),   Helper::new_with_function(Box::new(::helpers_builtins::each_helper)));
    helpers.insert("if".to_string(),     Helper::new_with_function(Box::new(::helpers_builtins::if_helper)));
    helpers.insert("unless".to_string(), Helper::new_with_function(Box::new(::helpers_builtins::unless_helper)));
    helpers.insert("lookup".to_string(), Helper::new_with_function(Box::new(::helpers_builtins::lookup_helper)));
    helpers.insert("with".to_string(),   Helper::new_with_function(Box::new(::helpers_builtins::with_helper)));

    EvalContext {
      partials: Default::default(),
      helpers: helpers,
      compat: false,
      falsy: Json::Null,
    }
  }
}

/// Holds the partials and helpers available for the template expansion.
///
/// # Examples
///
/// Add partial to make it available on expansion :
///
/// ```
/// # extern crate rustc_serialize as serialize;
/// # extern crate rumblebars;
/// # fn main() {
/// use rumblebars::Template;
/// use rumblebars::EvalContext;
/// # use serialize::json::Json;
///
/// let mut context = EvalContext::new();
/// # let mut say_hi = String::new();
/// # let sample: Json = r##"{"hello": "hi"}"##.parse().unwrap();
/// # let mut buf = say_hi.into_bytes();
///
/// context.register_partial("greet".to_string(), "say {{hello}}".parse().unwrap());
/// #
/// # Template::new("{{>greet}}").unwrap().eval(&sample, &mut buf, &context).unwrap();
/// #
/// # say_hi = String::from_utf8(buf).unwrap();
/// #
/// # assert_eq!(say_hi, "say hi")
/// # }
/// ```
///
/// Helpers uses the same logic as partials, you register it into the evaluation
/// context, and it'll be available on all expansion.
///
/// ```
/// use rumblebars::EvalContext;
/// use rumblebars::HBData; // for write_value
///
/// let mut context = EvalContext::new();
///
/// context.register_helper("hello".to_string(), Box::new(
///   |params, options, out, hb_context| {
///     "hi".write_value(out)
/// }));
/// ```
///
impl EvalContext {
  /// just return a default EvalContext without having to declare use of std:default
  pub fn new() -> EvalContext {
    Default::default()
  }

  /// Map of registered partials
  pub fn partials(&self) -> &HashMap<String, Template> {
    return &self.partials;
  }

  /// adds a partial in the evaluation context
  pub fn register_partial(&mut self, name: String, t: Template) {
    self.partials.insert(name, t);
  }

  /// fetch a registered partial by name
  pub fn partial_with_name(&self, name: &str) -> Option<&Template> {
    return self.partials.get(name);
  }


  /// adds a helper to the evaluation context
  pub fn register_helper(&mut self, name: String, h: HelperFunction) {
    self.helpers.insert(name, Helper::new_with_function(h));
  }

  /// fetch a registered helper by name
  pub fn helper_with_name(&self, name: &str) -> Option<&Helper> {
    return self.helpers.get(name);
  }

  /// true if a helper with given name is registered
  pub fn has_helper_with_name(&self, name: &str) -> bool {
    return self.helpers.contains_key(name);
  }
}

/// Global function for template evaluation. See [`Template`](struct.Template.html) docs.
pub fn eval(template: &Template, data: &HBData, out: &mut io::Write, eval_context: &EvalContext) -> HBEvalResult {
  let log = "info".to_string();
  let mut globals = HashMap::new();
  globals.insert("@root", data);
  globals.insert("@level", &log);

  let mut html_safe = HTMLSafeWriter::new(out);
  let mut safe_writer = SafeWriting::Safe(&mut html_safe);

  eval_with_globals(&template.entries, data, &mut safe_writer, eval_context, &globals, &vec![data], None)
}

struct RenderEntry<'a> {
  entry: &'a Box<HBEntry>,
  data: &'a (HBData+'a),
  stack: Vec<&'a (HBData+'a)>,
  indent: Option<String>,
}

impl<'a> RenderEntry<'a> {
  fn new(
    entry: &'a Box<HBEntry>,
    data: &'a (HBData+'a),
    stack: Vec<&'a (HBData+'a)>, indent: Option<String>
  ) -> RenderEntry<'a> {
    RenderEntry {
      entry: entry,
      data: data,
      stack: stack,
      indent: indent,
    }
  }
}

struct IterationControl<'a> {
  entries: &'a Entries,
  data_iter: ::std::rc::Rc<::std::cell::RefCell<HBValuesIter<'a>>>,
  stack: Vec<&'a (HBData+'a)>,
  indent: Option<String>,
}

enum StackEntry<'a> {
  CleanUpPartialContext(usize),
  ContextIterControlPoint(IterationControl<'a>),
  FlowEntry(RenderEntry<'a>)
}


pub fn eval_with_globals<'a: 'b, 'b: 'c, 'c>(entries: &'a Entries, data: &'a HBData, out: &mut SafeWriting, eval_context: &'a EvalContext, global_data: &HashMap<&str, &'c HBData>, context_stack: &Vec<&'b HBData>, indent: Option<String>) -> HBEvalResult {
  use self::StackEntry::{FlowEntry, ContextIterControlPoint, CleanUpPartialContext};
  // evaluation is done by iterating through each HBEntry to evaluate
  //  - raw copy,
  //  - simple expression evaluation (render value execute helper call)
  //  - partial evalutation, stacking each entry from registered partial)
  //  - block evaluation, stacking each entry of block with parameterized
  //    context, with basic flow control (each, if)

  // given the above, we start by stacking each entries of template root level
  // each entry comes along with :
  //  - a ref to their associated context
  //  - a context stack, to have access of context of parent blocks (copied for each entry)
  //  - an indentation level (for partials, copied for each entry)
  let mut stack = entries.iter().rev().map(|e| {
    FlowEntry(RenderEntry::new(
      e, data,
      context_stack.iter().map(|s| *s).collect::<Vec<_>>(),
      indent.clone()
    ))
  }).collect::<Vec<_>>();

  // used for storage of partials optional keys, computed at evaluation
  // this can leak, but probably not in relevant cases.
  let mut partial_options_current_index = 0;
  let mut partial_options_contexts = HashMap::<usize, FallbackToOptions>::new();

  while stack.len() > 0 {
    let w_ok = if let Some(stack_entry) = stack.pop() {
      match stack_entry {
        CleanUpPartialContext(ref index) => { partial_options_contexts.remove(index); Ok(()) },
        ContextIterControlPoint(ref control) => if let Some(next) = control.data_iter.borrow_mut().next() {
          stack.push(ContextIterControlPoint(IterationControl {
            entries: control.entries,
            data_iter: control.data_iter.clone(),
            stack: control.stack.clone(),
            indent: control.indent.clone(),
          }));
          for e in control.entries.iter().rev() {
            stack.push(FlowEntry(RenderEntry::new(
              &e, next,
              control.stack.clone(), control.indent.clone()
            )));
          }
          Ok(())
        } else {
          Ok(())
        },
        FlowEntry(flow_entry) => {
          let ctxt = flow_entry.data;

          match **flow_entry.entry {
            HBEntry::Raw(ref s) => {
              IndentWriter::with_indent(flow_entry.indent.clone(), &mut out.into_unsafe(), &|w| {
                w.write_all(&s.as_bytes())
              })
            },
            HBEntry::Partial(ref exp) => {
              match exp.base.first() {
                Some(ref single) if exp.base.len() == 1 => {
                  match eval_context.partial_with_name(&single) {
                    Some(ref t) => {
                      let c_ctxt = if let Some(&HBValHolder::Path(ref p)) = exp.params.get(0) {
                        value_for_key_path_in_context(ctxt, p, &flow_entry.stack, global_data, eval_context.compat).unwrap_or(ctxt)
                      } else {
                        ctxt
                      };

                      let with_options_fallback = if exp.options.len() > 0 {
                        let mut options_contexts: HashMap<&str, &HBData> = HashMap::new();
                        for o in exp.options.iter() {
                          match o {
                            &(ref name, HBValHolder::String(ref s)) => {
                              options_contexts.insert(&name, s as &HBData);
                            },
                            &(ref name, HBValHolder::Path(ref p)) => {
                              options_contexts.insert(&name, value_for_key_path_in_context(ctxt, p, &flow_entry.stack, global_data, eval_context.compat).unwrap_or(&eval_context.falsy));
                            },
                            &(ref name, HBValHolder::Literal(ref j, _)) => {
                              options_contexts.insert(&name, j as &HBData);
                            },
                          }
                        }

                        // store data into a collections with enough lifetime, transmute is safe
                        // because, while partial_options_contexts is mangled with data
                        // with a greater lifetime, it'll never be accessed
                        // outside it's current scope
                        partial_options_contexts.insert(
                          partial_options_current_index,
                          FallbackToOptions { data: c_ctxt, options: options_contexts }
                        );
                        unsafe { ::std::mem::transmute(
                          partial_options_contexts.get(&partial_options_current_index).map(|f| f as &HBData).unwrap()
                        ) }
                      } else {
                        c_ctxt
                      };

                      // calculate indentation content
                      let may_indent = match (&flow_entry.indent, &exp.render_options.indent) {
                        (&None, & ref i @ Some(_)) | (& ref i @ Some(_), &None) => i.clone(),
                        (&Some(ref i), &Some(ref j)) => Some(format!("{}{}", i, j)),
                        (&None, &None) => None,
                      };

                      stack.push(CleanUpPartialContext(partial_options_current_index));
                      partial_options_current_index += 1;

                      for ref e in t.entries.iter().rev() {
                        stack.push(FlowEntry(RenderEntry::new(
                          e, with_options_fallback,
                          flow_entry.stack.clone(), may_indent.clone()
                        )))
                      }

                      Ok(())
                    },
                    _ => Ok(())
                  }
                }
                Some(_) => panic!("invalid partial name '{}'", exp.path()),
                None => panic!("invalid empty string to retrieve partial by name"),
              }
            },

            HBEntry::Eval(HBExpression{ref base, ref params, ref options, ref render_options, block: None, else_block: None}) => {
              match (base.first(), base.len()) {
                (Some(ref single), 1) if eval_context.has_helper_with_name(&single) => {
                  let helper = eval_context.helper_with_name(&single).unwrap();
                  if render_options.escape {
                    IndentWriter::with_indent(flow_entry.indent.clone(), out, &|w| {
                      helper.call_fn(ctxt, &params, &options, w, eval_context, &flow_entry.stack, global_data)
                    })
                  } else {
                    IndentWriter::with_indent(flow_entry.indent.clone(), &mut out.into_unsafe(), &|w| {
                      helper.call_fn(ctxt, &params, &options, w, eval_context, &flow_entry.stack, global_data)
                    })
                  }
                },
                _ => match value_for_key_path_in_context(ctxt, base, &flow_entry.stack, global_data, eval_context.compat) {
                  Some(v) => match v.typed_node() {
                    HBNodeType::Leaf(_) | HBNodeType::Array(_)=> {
                      if render_options.escape {
                        IndentWriter::with_indent(flow_entry.indent.clone(), out, &|w| {
                          v.write_value(w)
                        })
                      } else {
                        IndentWriter::with_indent(flow_entry.indent.clone(), &mut out.into_unsafe(), &|w| {
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
              match (base.first(), base.len()) {
                (Some(ref single), 1) if eval_context.has_helper_with_name(&single) => {
                  let helper = eval_context.helper_with_name(&single).unwrap();

                  // collect options of deref'd blocks
                  let blocks: Vec<_> = [block, else_block].iter().map(|b| {
                    match b {
                      &&Some(ref t) => Some(&**t),
                      &&None => None,
                    }
                  }).collect();

                  if let (Some(&opt_block), Some(&opt_else_block), 2) = (blocks.first(), blocks.get(1), blocks.len()) {
                    helper.call_for_block(
                      opt_block,
                      opt_else_block,
                      render_options.inverse,
                      ctxt,
                      &params,
                      &options,
                      out,
                      eval_context,
                      &flow_entry.stack,
                      global_data
                    )
                  } else {
                    Ok(())
                  }
                },
                _ => {
                  let c_ctxt = value_for_key_path_in_context(ctxt, base, &flow_entry.stack, global_data, eval_context.compat);

                  match (c_ctxt.unwrap_or(&eval_context.falsy), block) {
                    (c, &Some(ref block_found)) => {
                      match c.typed_node() {
                        HBNodeType::Branch(_) | HBNodeType::Leaf(_) | HBNodeType::Null => {
                          if c.as_bool() && !render_options.inverse || !c.as_bool() && render_options.inverse {
                            for e in block_found.iter().rev() {
                              let mut c_stack = flow_entry.stack.clone();
                              c_stack.push(ctxt);
                              stack.push(FlowEntry(RenderEntry::new(
                                e, c,
                                c_stack, flow_entry.indent.clone()
                              )))
                            }
                          } else if let &Some(ref inv_block) = else_block {
                            for e in inv_block.iter().rev() {
                              stack.push(FlowEntry(RenderEntry::new(
                                e, ctxt,
                                flow_entry.stack.clone(), flow_entry.indent.clone()
                              )))
                            }
                          }
                        },
                        HBNodeType::Array(_) => {
                          let inverse = render_options.inverse;
                          let (len, _) = c.values().size_hint();

                          let collection_iter: HBValuesIter = match (0 >= len, inverse) {
                            (true,  true)  => Box::new(Some(&eval_context.falsy as &HBData).into_iter()),
                            (false, true)  => Box::new(None.into_iter()),
                            (_, false) => c.values(),
                          };

                          let (c_len, _) = collection_iter.size_hint();

                          let iter_cell = ::std::rc::Rc::new(::std::cell::RefCell::new(collection_iter));

                          if c_len > 0 {
                            let mut c_stack = flow_entry.stack.clone();
                            c_stack.push(ctxt);

                            stack.push(ContextIterControlPoint(IterationControl {
                              entries: block_found,
                              data_iter: iter_cell.clone(),
                              stack: c_stack,
                              indent: flow_entry.indent.clone()
                            }));
                          } else if let &Some(ref inv_block) = else_block {
                            for e in inv_block.iter().rev() {
                              stack.push(FlowEntry(RenderEntry::new(
                                e, ctxt,
                                flow_entry.stack.clone(), flow_entry.indent.clone()
                              )))
                            }
                          }

                        },
                      }
                    },
                    _ => (),
                  }

                  Ok(())
                },
              }
            }
          }
        }
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


