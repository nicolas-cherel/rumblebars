use std::io::BufReader;
use std::io::Writer;
use std::slice;

use self::Token::{TokSimpleExp, TokEscapedExp, TokBlockExp, TokBlockEndExp, TokPartialExp, TokRaw};
use self::HBToken::{TokPathStart,TokPathEntry,TokNoWhiteSpace,TokStringParam,TokParamStart, TokParamSep, TokOption};

#[deriving(Show)]
enum Token {
  // base template tokens 
  TokSimpleExp(String),
  TokEscapedExp(String),
  TokPartialExp(String),
  TokBlockExp(String),
  TokBlockEndExp(String),
  TokRaw(String),
}

#[deriving(Show)]
enum HBToken {
  TokPathStart,
  TokPathEntry(String),
  TokNoWhiteSpace,
  TokStringParam(String),
  TokParamStart,
  TokParamSep,
  TokOption(String),
}

rustlex! HandleBarsLexer {
    // expression definitions
    let PASS_THROUGH = .;

    let OPEN  = "{{";
    let CLOSE = [' ''\t']* "}}";
    let EXP = [^'}']*;


    let BLOCK_EXP   = OPEN '#' EXP CLOSE;
    let END_EXP     = OPEN '/' EXP CLOSE;
    let ESC_EXP     = OPEN '{' EXP '}' CLOSE;
    let PARTIAL_EXP = OPEN '>' EXP CLOSE;
    let SIMPLE_EXP  = OPEN EXP CLOSE;


    // then rules
    PASS_THROUGH => |lexer:&mut HandleBarsLexer<R>| Some( TokRaw( lexer.yystr() ) )
    
    SIMPLE_EXP   => |lexer:&mut HandleBarsLexer<R>| Some( TokSimpleExp( lexer.yystr() ) )
    PARTIAL_EXP  => |lexer:&mut HandleBarsLexer<R>| Some( TokPartialExp( lexer.yystr() ) )
    ESC_EXP      => |lexer:&mut HandleBarsLexer<R>| Some( TokEscapedExp( lexer.yystr() ) )
    END_EXP      => |lexer:&mut HandleBarsLexer<R>| Some( TokBlockEndExp( lexer.yystr() ) )
    BLOCK_EXP    => |lexer:&mut HandleBarsLexer<R>| Some( TokBlockExp( lexer.yystr() ) )
    
}


rustlex! HBExpressionLexer {
  token HBToken;
  property in_options:bool = false;
  property in_params:bool = false;

  let START = "{{" ['{''#''/']?;
  let END =  '}'? "}}"; // no escaping triple {{{ check for now
  let NO_WP = '~';

  let STRING_START = '"';
  let STRING_CTNT  = ("\\\"" | [^'"'])*; // either escaped quote or not quote
  let STRING_END   = ['"'];

  let ACCESSOR_SEP_SLASH = "/";
  let DOT_STARTED = ("." | "..");


  let IDENTIFIER = [^'!''"''#''%''&''\'''('')''*''+'',''.''/'';''<''=''>''@''[''\\'']''^''`''{''|''}''~'' ''\t']+;
  let BRACKET_ID_START = '[';
  let BRACKET_ID_END   = ']';
  let BRACKETED_ID     = [^']']+;
  let ACCESSOR_SEP     = ['.''/'];
  let ACCESSOR_END     = [' ''\t']+;

  let THIS             = "this";
  let THIS_ALIAS       = ".";
  let PARENT_ALIAS     = "..";

  let PARAMS_SEP       = [' ''\t']+;

  let OPTION_NAME      = IDENTIFIER "=";

  INITIAL {
    START => |lexer:&mut HBExpressionLexer<R>| { lexer.ACCESSOR(); Some( TokPathStart ) }
    END   => |_:&mut HBExpressionLexer<R>| { None }
  }

  ACCESSOR {
    IDENTIFIER =>       |lexer:&mut HBExpressionLexer<R>| { lexer.PROPERTY_PATH(); Some( TokPathEntry( lexer.yystr() ) ) }
    BRACKET_ID_START => |lexer:&mut HBExpressionLexer<R>| { lexer.ID_ANY(); None }

    STRING_START => |lexer:&mut HBExpressionLexer<R>| { lexer.STRING_PARAM(); None } // for parameters only
    
    THIS         => |lexer:&mut HBExpressionLexer<R>| { lexer.PROPERTY_PATH(); Some( TokPathEntry( ".".to_string()  ) ) }
    THIS_ALIAS   => |lexer:&mut HBExpressionLexer<R>| { lexer.PROPERTY_PATH(); Some( TokPathEntry( ".".to_string()  ) ) }
    PARENT_ALIAS => |lexer:&mut HBExpressionLexer<R>| { lexer.PROPERTY_PATH(); Some( TokPathEntry( "..".to_string() ) ) }
  }

  PROPERTY_PATH {
    ACCESSOR_SEP => |lexer:&mut HBExpressionLexer<R>| { lexer.ACCESSOR(); None }
    ACCESSOR_END => |lexer:&mut HBExpressionLexer<R>| { 
      if lexer.in_options  { lexer.OPTIONS() } else { lexer.PARAMS() };
      if (lexer.in_params) {
        Some( TokParamSep )
      } else {
        lexer.in_params = true;
        Some( TokParamStart )
      }
    }

    // common ending
    NO_WP        => |lexer:&mut HBExpressionLexer<R>| { lexer.FORCE_END(); Some( TokNoWhiteSpace ) }
    END          => |    _:&mut HBExpressionLexer<R>| { None }
  }

  ID_ANY {
    BRACKETED_ID   => |lexer:&mut HBExpressionLexer<R>| { Some( TokPathEntry( lexer.yystr() ) ) }
    BRACKET_ID_END => |lexer:&mut HBExpressionLexer<R>| { lexer.PROPERTY_PATH(); None }
  }

  PARAMS {
    PARAMS_SEP   => |    _:&mut HBExpressionLexer<R>| { Some( TokParamSep ) }
    IDENTIFIER   => |lexer:&mut HBExpressionLexer<R>| { lexer.PROPERTY_PATH(); Some( TokPathEntry( lexer.yystr() ) ) }
    STRING_START => |lexer:&mut HBExpressionLexer<R>| { lexer.STRING_PARAM(); None } 

    // end of parameters
    OPTION_NAME  => |lexer:&mut HBExpressionLexer<R>| {  
      lexer.in_options = true; 
      lexer.OPTION_VALUE(); 
      Some( TokOption( lexer.yystr().as_slice().trim_right_chars('=').to_string() ) ) 
    }

    // common expression ending 
    NO_WP        => |lexer:&mut HBExpressionLexer<R>| { lexer.FORCE_END(); Some( TokNoWhiteSpace ) }
    END          => |    _:&mut HBExpressionLexer<R>| { None }

  }

  STRING_PARAM {
    STRING_CTNT => |lexer:&mut HBExpressionLexer<R>| { Some( TokStringParam( lexer.yystr() ) ) }
    STRING_END  => |lexer:&mut HBExpressionLexer<R>| { if lexer.in_options  { lexer.OPTIONS() } else { lexer.PARAMS() }; None }
  }

  OPTION_VALUE {
    // all of these have conditional ending with in_params
    IDENTIFIER       => |lexer:&mut HBExpressionLexer<R>| { lexer.PROPERTY_PATH(); Some( TokPathEntry( lexer.yystr() ) ) }
    BRACKET_ID_START => |lexer:&mut HBExpressionLexer<R>| { lexer.ID_ANY(); None }
    STRING_START     => |lexer:&mut HBExpressionLexer<R>| { lexer.STRING_PARAM(); None }

    // ok, pure option parsing for now
  }

  OPTIONS {
    OPTION_NAME  => |lexer:&mut HBExpressionLexer<R>| {  lexer.OPTION_VALUE(); Some( TokOption( lexer.yystr().as_slice().trim_right_chars('=').to_string() ) ) }
    PARAMS_SEP    => |_:&mut HBExpressionLexer<R>| { None } 

    // common expression ending 
    NO_WP        => |lexer:&mut HBExpressionLexer<R>| { lexer.FORCE_END(); Some( TokNoWhiteSpace ) }
    END          => |    _:&mut HBExpressionLexer<R>| { None }
  }

  FORCE_END {
    END => |_:&mut HBExpressionLexer<R>| { None }
  }


}

#[deriving(Show)]
enum HBValHolder {
  String(String),
  Path(Vec<String>),
}

#[deriving(Show)]
pub struct HBExpression {
  pub base: Vec<String>,
  pub params: Vec<HBValHolder>,
  pub options: Vec<(String, HBValHolder)>,
  pub escape: bool,
  pub no_white_space: bool,
  pub block: Option<Box<Template>>
}

#[deriving(Show)]
pub enum HBEntry {
  Raw(String),
  Eval(HBExpression),
  Partial(HBExpression),
}

#[deriving(Show, Default)]
pub struct Template {
  content: Vec<Box<HBEntry>>
}

impl Template {
  pub fn iter<'a>(&'a self) -> slice::Iter<'a, Box<HBEntry>> {
    return self.content.iter();
  }
}


#[deriving(Show)]
pub enum ParseError {
  UnkownError, // unknown as ‘still not diagnosed case’, not ’your grandma's TV is set on fire case’
  UnmatchedBlock,
  UnexpectedBlockClose,
}

impl Copy for ParseError {}

fn parse_hb_expression(exp: &str) -> Result<HBExpression, (ParseError, Option<String>)> {
  let mut lexer = HBExpressionLexer::new(BufReader::new(exp.as_bytes()));

  if let Some(tok) = lexer.next()  {
    match tok {
      TokPathStart => {
        let mut path = vec![];
        let mut params = vec![];
        let mut options = vec![];
        let mut no_white_space = false;

        while let Some(tok) = lexer.next() {
          match tok {
            TokPathEntry(path_comp) => { path.push(path_comp) },
            TokParamStart => {
              let mut param_path = vec![];
              while let Some(tok) = lexer.next() {
                match tok {
                  TokPathEntry(path_comp) => { param_path.push(path_comp) },
                  TokStringParam(s) => { params.push(HBValHolder::String(s)) },
                  TokParamSep => {
                    if param_path.len() > 0 {
                      params.push(HBValHolder::Path(param_path));
                      param_path = vec![];
                    }
                  },
                  // options starts here
                  TokOption(opt) => { 
                    let option_name = opt;
                    let mut opt_path = vec![];
                    let mut opt_val  = None;

                    // we have an option, get its value and following options
                    while let Some(tok) = lexer.next() {
                      match tok {
                        TokPathEntry(s) => {
                          opt_path.push(s);
                        },
                        TokStringParam(s) => {
                          opt_val = Some(s);
                          break;
                        },
                        TokNoWhiteSpace => { no_white_space = true },
                        _ => { break }
                      }
                    }

                    options.push((option_name, if let Some(val) = opt_val { HBValHolder::String(val) } else { HBValHolder::Path(opt_path) }));

                  },
                  TokNoWhiteSpace => { no_white_space = true },
                  _ => { break; }
                }
              }
              if param_path.len() > 0 {
                params.push(HBValHolder::Path(param_path));
              }
            },
            TokNoWhiteSpace => { no_white_space = true },
            _ => { break },
          }
        }

        
        return  Ok(HBExpression { base: path, params: params, options: options, no_white_space: no_white_space, escape: false, block: None })
      },
      _ => { return Err((ParseError::UnkownError, None)) }
    }

  } else {
    return Err((ParseError::UnkownError, None))
  }

  
}


pub fn parse(template: &str) -> Result<Template, (ParseError, Option<String>)> {
  let mut lexer = HandleBarsLexer::new(BufReader::new(template.as_bytes()));
  let mut raw = String::new();
  let mut stack = vec![box Template { content: vec![] }];

  for tok in *lexer {
    // first match handle raw content
    match tok {
      TokRaw(ref chr) => raw.push_str(chr.as_slice()),
      TokSimpleExp(_) | TokEscapedExp(_) | TokBlockExp(_) | TokBlockEndExp(_) | TokPartialExp(_) => {
        if ! raw.is_empty() {
          stack.last_mut().unwrap().content.push(box HBEntry::Raw(raw));
          raw = String::new();
        }
      },
    }

    // second match handle handlebars expressions
    match tok {
      TokRaw(_) => (),
      TokSimpleExp(exp) => {
        if let Ok(hb) = parse_hb_expression(exp.as_slice()) {
          stack.last_mut().unwrap().content.push(box HBEntry::Eval(hb))
        }
      },
      TokEscapedExp(exp) => {
        if let Ok(mut hb) = parse_hb_expression(exp.as_slice()) {
          hb.escape = true;
          stack.last_mut().unwrap().content.push(box HBEntry::Eval(hb))
        }
      },
      TokPartialExp(exp) => {
        if let Ok(hb) = parse_hb_expression(exp.as_slice()) {
          stack.last_mut().unwrap().content.push(box HBEntry::Partial(hb))
        }
      },
      TokBlockExp(exp) => {
        if let Ok(hb) = parse_hb_expression(exp.as_slice()) {
          stack.last_mut().unwrap().content.push(box HBEntry::Eval(hb));
          stack.push(box Template { content: vec![] });
        }
      },
      TokBlockEndExp(exp) => {
        if let Ok(hb) = parse_hb_expression(exp.as_slice()) {
          let pop = stack.pop();
          match stack.last_mut().unwrap().content.last_mut() {
            Some(&box HBEntry::Eval(ref mut parent)) => {
              if parent.base == hb.base {
                parent.block = pop;
              } else {
                return Err((ParseError::UnmatchedBlock, Some(format!("‘{}’ does not match ‘{}’", hb.base, parent.base))))
              }
            }
            _ => { return Err((ParseError::UnexpectedBlockClose, Some(format!("‘{}’ does not close any block", hb.base)))) } 
          }
        }
      }
    }

  }

  if ! raw.is_empty() {
    stack.last_mut().unwrap().content.push(box HBEntry::Raw(raw));
  }

  return match stack.remove(0) {
    Some(box t) => Result::Ok(t),
    None        => Result::Err((ParseError::UnkownError, None)),
  };
}

#[cfg(test)]
mod tests {
  use std::io::BufReader;

  use super::{parse, parse_hb_expression, HBEntry, HBExpression, HBValHolder, HBExpressionLexer};

  #[allow(dead_code)]
  fn debug_parse_hb(exp: &str) {
    let mut lexer = HBExpressionLexer::new(BufReader::new(exp.as_bytes()));
    println!("{}", exp);
    for tok in *lexer {
      println!("{}", tok);
    }

  }

  #[test]
  fn hb_simple() {
    assert!(match parse_hb_expression("{{i}}") { 
      Ok(_)  => true,
      Err(_) => false,
    })
  }

  #[test]
  fn hb_simple_base() {
    match parse_hb_expression("{{i}}") { 
      Ok(ok)  => assert_eq!(ok.base, vec!["i"]),
      Err(_)  => (),
    }
  }

  #[test]
  fn hb_simple_base_path() {
    match parse_hb_expression("{{i.j}}") { 
      Ok(ok)  => assert_eq!(ok.base, vec!["i", "j"]),
      Err(_)  => (),
    }
  }

  #[test]
  fn hb_simple_base_esc_path() {
    match parse_hb_expression("{{[i]}}") { 
      Ok(ok)  => assert_eq!(ok.base, vec!["i"]),
      Err(_)  => (),
    }
  }

  #[test]
  fn hb_simple_this_path() {
    match parse_hb_expression("{{.}}") { 
      Ok(ok)  => assert_eq!(ok.base, vec!["."]),
      Err(_)  => (),
    }
  }

  #[test]
  fn hb_this_path() {
    match parse_hb_expression("{{./p}}") { 
      Ok(ok)  => assert_eq!(ok.base, vec![".", "p"]),
      Err(_)  => (),
    }
  }

  #[test]
  fn hb_string_param() {
    match parse_hb_expression(r##"{{p "string"}}"##) { 
      Ok(HBExpression{ref base, ref params, ref options, ref escape, ref no_white_space, ref block})  => {
        assert_eq!(base, &vec!["p"]);
        assert_eq!(match params.get(0).unwrap() { &HBValHolder::String(ref s) => s.clone(), _ => "".to_string()}, "string".to_string());
      },
      Err(_)  => (),
    }
  }

  #[test]
  fn hb_prop_path_param() {
    match parse_hb_expression(r##"{{p some.path}}"##) { 
      Ok(HBExpression{ref base, ref params, ref options, ref escape, ref no_white_space, ref block})  => {
        assert_eq!(base, &vec!["p"]);
        assert_eq!(match params.get(0).unwrap() { &HBValHolder::Path(ref p) => p.clone(), _ => vec![]}, vec!["some", "path"]);
      },
      Err(_)  => (),
    }
  }

  #[test]
  fn hb_2_params() {
    match parse_hb_expression(r##"{{p some path}}"##) { 
      Ok(HBExpression{ref base, ref params, ref options, ref escape, ref no_white_space, ref block})  => {
        assert_eq!(base, &vec!["p"]);
        assert_eq!(match params.get(0).unwrap() { &HBValHolder::Path(ref p) => p.clone(), _ => vec![]}, vec!["some"]);
        assert_eq!(match params.get(1).unwrap() { &HBValHolder::Path(ref p) => p.clone(), _ => vec![]}, vec!["path"]);
      },
      Err(_)  => (),
    }
  }

  #[test]
  fn hb_3_params() {
    match parse_hb_expression(r##"{{p some.path "with_string" yep}}"##) { 
      Ok(HBExpression{ref base, ref params, ref options, ref escape, ref no_white_space, ref block})  => {
        assert_eq!(base, &vec!["p"]);
        assert_eq!(match params.get(0).unwrap() { &HBValHolder::Path(ref p) => p.clone(), _ => vec![]}, vec!["some", "path"]);
        assert_eq!(match params.get(1).unwrap() { &HBValHolder::String(ref s) => s.clone(), _ => "".to_string()}, "with_string".to_string());
        assert_eq!(match params.get(2).unwrap() { &HBValHolder::Path(ref p) => p.clone(), _ => vec![]}, vec!["yep"]);
      },
      Err(_)  => (),
    }
  }

  #[test]
  fn hb_full_feat_param() {
    match parse_hb_expression(r##"{{t "… param1" well.[that my baby].[1] ~}}"##) { 
      Ok(HBExpression{ref base, ref params, ref options, ref escape, ref no_white_space, ref block})  => {
        assert_eq!(base, &vec!["t"]);
        assert_eq!(match params.get(0).unwrap() { &HBValHolder::String(ref s) => s.clone(), _ => "".to_string()}, "… param1".to_string());
        assert_eq!(match params.get(1).unwrap() { &HBValHolder::Path(ref p) => p.clone(), _ => vec![]}, vec!["well", "that my baby", "1"]);
        assert!(*no_white_space);
      },
      Err(_)  => (),
    }
  }

  #[test]
  fn hb_option() {
    match parse_hb_expression(r##"{{t opt=u ~}}"##) { 
      Ok(HBExpression{ref base, ref params, ref options, ref escape, ref no_white_space, ref block})  => {
        assert_eq!(base, &vec!["t"]);
        assert_eq!(("opt".to_string(), vec!["u".to_string()]), match options.get(0).unwrap() { 
          &(ref o, HBValHolder::Path(ref p)) => (o.clone(), p.clone()), 
          _ => ("".to_string(), vec![]),
        });
        assert!(*no_white_space);
      },
      Err(_)  => (),
    }
  }

  #[test]
  fn hb_mutli_options() {
    match parse_hb_expression(r##"{{t opt=u opt2="v" ~}}"##) { 
      Ok(HBExpression{ref base, ref params, ref options, ref escape, ref no_white_space, ref block})  => {
        assert_eq!(base, &vec!["t"]);
        assert_eq!(("opt".to_string(), vec!["u".to_string()]), match options.get(0).unwrap() { 
          &(ref o, HBValHolder::Path(ref p)) => (o.clone(), p.clone()), 
          _ => ("".to_string(), vec![]),
        });
        assert_eq!(("opt2".to_string(), "v".to_string()), match options.get(1).unwrap() { 
          &(ref o, HBValHolder::String(ref s)) => (o.clone(), s.clone()), 
          _ => ("".to_string(), "".to_string()),
        });
        assert!(*no_white_space);
      },
      Err(_)  => (),
    }
  }

  #[test]
  fn hb_param_options() {
    match parse_hb_expression(r##"{{t o.[t}+=] opt="v" ~}}"##) { 
      Ok(HBExpression{ref base, ref params, ref options, ref escape, ref no_white_space, ref block})  => {
        assert_eq!(base, &vec!["t"]);
        assert_eq!(vec!["o", "t}+="], match params.get(0).unwrap() {
          &HBValHolder::Path(ref p) => p.clone(), _ => vec![]
        });
        assert_eq!(("opt".to_string(), "v".to_string()), match options.get(0).unwrap() { 
          &(ref o, HBValHolder::String(ref s)) => (o.clone(), s.clone()), 
          _ => ("".to_string(), "".to_string()),
        });
        assert!(*no_white_space);
      },
      Err(_)  => (),
    }
  }

  #[test]
  fn parse_raw() {
    let p = parse("tada").unwrap();
    assert_eq!("tada", match p.content.get(0) {
      Some(&box HBEntry::Raw(ref s)) => s.as_slice(),
      _ => "",
    });
  }

  #[test]
  fn parse_exp() {
    let p = parse("{{tada}}").unwrap();
    assert_eq!("tada", match p.content.get(0) {
      Some(&box HBEntry::Eval(HBExpression {ref base, ..})) => base.head().unwrap().as_slice(),
      _ => "",
    });
  }

  #[test]
  fn parse_exp_entangled() {
    let p = parse("tidi {{tada}} todo {{tudu}} bar").unwrap();
    assert_eq!("tidi ", match p.content.get(0) {
      Some(&box HBEntry::Raw(ref s)) => s.as_slice(),
      _ => "",
    });
    assert_eq!("tada", match p.content.get(1) {
      Some(&box HBEntry::Eval(HBExpression {ref base, ..})) => base.head().unwrap().as_slice(),
      _ => "",
    });
    assert_eq!(" todo ", match p.content.get(2) {
      Some(&box HBEntry::Raw(ref s)) => s.as_slice(),
      _ => "",
    });
    assert_eq!("tudu", match p.content.get(3) {
      Some(&box HBEntry::Eval(HBExpression {ref base, ..})) => base.head().unwrap().as_slice(),
      _ => "",
    });
    assert_eq!(" bar", match p.content.get(4) {
      Some(&box HBEntry::Raw(ref s)) => s.as_slice(),
      _ => "",
    });
  }
}