use std::io::BufReader;
use std::io::Writer;
use std::slice;

use self::Token::{TokSimpleExp, TokEscapedExp, TokBlockExp, TokBlockEndExp, TokRaw};
use self::HBToken::{TokPathStart,TokPathSep,TokPathEntry,TokNoWhiteSpace,TokStringParam,TokParamStart, TokParamSep, TokOption};

#[deriving(Show)]
enum Token {
  // base template tokens 
  TokSimpleExp(String),
  TokEscapedExp(String),
  TokBlockExp(String),
  TokBlockEndExp(String),
  TokRaw(String),
}

#[deriving(Show)]
enum HBToken {
  TokPathStart,
  TokPathSep,
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
    let SIMPLE_EXP  = OPEN EXP CLOSE;


    // then rules
    PASS_THROUGH => |lexer:&mut HandleBarsLexer<R>| Some( TokRaw( lexer.yystr() ) )
    
    SIMPLE_EXP   => |lexer:&mut HandleBarsLexer<R>| Some( TokSimpleExp( lexer.yystr() ) )
    ESC_EXP      => |lexer:&mut HandleBarsLexer<R>| Some( TokEscapedExp( lexer.yystr() ) )
    END_EXP      => |lexer:&mut HandleBarsLexer<R>| Some( TokBlockEndExp( lexer.yystr() ) )
    BLOCK_EXP    => |lexer:&mut HandleBarsLexer<R>| Some( TokBlockExp( lexer.yystr() ) )
    
}


rustlex! HBExpressionLexer {
  token HBToken;
  property in_options:bool = false;

  let START = "{{" ['{''#''/']?;
  let END =  '}'? "}}"; // no escaping triple {{{ check for now
  let NO_WP = '~';

  let STRING_START = '"';
  let STRING_CTNT  = ("\\\"" | [^'"'])*; // either escaped quote or not quote
  let STRING_END   = ['"'];

  let IDENTIFIER = [^'!''"''#''%''&''\'''('')''*''+'',''.''/'';''<''=''>''@''[''\\'']''^''`''{''|''}''~'' ''\t']+;
  let BRACKET_ID_START = '[';
  let BRACKET_ID_END   = ']';
  let BRACKETED_ID     = [^']']+;
  let ACCESSOR_SEP     = '.';
  let ACCESSOR_END     = [' ''\t']+;

  let PARAMS_SEP       = [' ''\t']+;

  let OPTION_NAME      = IDENTIFIER "=";

  INITIAL {
    START => |lexer:&mut HBExpressionLexer<R>| { lexer.ACCESSOR(); Some( TokPathStart ) }
    END   => |_:&mut HBExpressionLexer<R>| { None }
  }

  ACCESSOR {
    IDENTIFIER =>       |lexer:&mut HBExpressionLexer<R>| { Some( TokPathEntry( lexer.yystr() ) ) }
    BRACKET_ID_START => |lexer:&mut HBExpressionLexer<R>| { lexer.ID_ANY(); None }
    ACCESSOR_SEP => |    _:&mut HBExpressionLexer<R>| { Some( TokPathSep ) }
    ACCESSOR_END => |lexer:&mut HBExpressionLexer<R>| { 
      if lexer.in_options  { lexer.OPTIONS() } else { lexer.PARAMS() }; 
      Some( TokParamStart ) 
    }

    // common ending
    NO_WP        => |lexer:&mut HBExpressionLexer<R>| { lexer.FORCE_END(); Some( TokNoWhiteSpace ) }
    END          => |    _:&mut HBExpressionLexer<R>| { None }
  }

  ID_ANY {
    BRACKETED_ID   => |lexer:&mut HBExpressionLexer<R>| { Some( TokPathEntry( lexer.yystr() ) ) }
    BRACKET_ID_END => |lexer:&mut HBExpressionLexer<R>| { lexer.ACCESSOR(); None }
  }

  PARAMS {
    PARAMS_SEP   => |    _:&mut HBExpressionLexer<R>| { Some( TokParamSep ) }
    IDENTIFIER   => |lexer:&mut HBExpressionLexer<R>| { lexer.ACCESSOR(); Some( TokPathEntry( lexer.yystr() ) ) }
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
    IDENTIFIER       => |lexer:&mut HBExpressionLexer<R>| { lexer.ACCESSOR(); Some( TokPathEntry( lexer.yystr() ) ) }
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
  Eval(HBExpression)
}

#[deriving(Show)]
pub struct Template {
  content: Vec<Box<HBEntry>>
}

impl Template {
  pub fn iter<'a>(&'a self) -> slice::Iter<'a, Box<HBEntry>> {
    return self.content.iter();
  }
}

#[allow(dead_code)]
fn debug_parse_hb(exp: &str) {
  let mut lexer = HBExpressionLexer::new(BufReader::new(exp.as_bytes()));
  println!("{}", exp);
  for tok in *lexer {
    println!("{}", tok);
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
            TokPathSep => {}
            TokParamStart => {
              let mut param_path = vec![];
              while let Some(tok) = lexer.next() {
                match tok {
                  TokPathEntry(path_comp) => { param_path.push(path_comp) },
                  TokPathSep => {},
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
                        TokPathSep => {},
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
                  _ => { 
                    if param_path.len() > 0 {
                      params.push(HBValHolder::Path(param_path));
                    }
                    break; 
                  }
                }
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
      TokSimpleExp(_) | TokEscapedExp(_) | TokBlockExp(_) | TokBlockEndExp(_) => {
        if ! raw.is_empty() {
          stack.last_mut().unwrap().content.push(box HBEntry::Raw(raw));
          raw = String::new();
        }
      },
    }

    // second match handle handlebars expressions
    match tok {
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

      // irrelevant here (mostly due of use of same enum)
      _ => {}
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
  use super::{parse, parse_hb_expression, HBEntry, HBExpression};

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