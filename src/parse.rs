use std::io::BufReader;
use std::slice;
use regex::Regex;
use std::default::Default;


use self::Token::{TokSimpleExp, TokNoEscapeExp, TokBlockExp, TokBlockElseCond, TokBlockEndExp, TokPartialExp, TokRaw};
use self::HBToken::{TokPathEntry,TokNoWhiteSpaceBefore, TokNoWhiteSpaceAfter,TokStringParam,TokParamStart, TokParamSep, TokOption};

enum Token {
  // base template tokens
  TokSimpleExp(String),
  TokNoEscapeExp(String),
  TokPartialExp(String, bool),
  TokBlockExp(String, bool),
  TokBlockElseCond(String, bool),
  TokBlockEndExp(String, bool),
  TokRaw(String),
}

enum HBToken {
  TokPathEntry(String),
  TokNoWhiteSpaceBefore,
  TokNoWhiteSpaceAfter,
  TokStringParam(String),
  TokParamStart,
  TokParamSep,
  TokOption(String),
}

rustlex! HandleBarsLexer {
    // expression definitions
    let PASS_THROUGH = .;
    let NEW_LINE     = (['\n'] | ['\r']['\n']);

    let OPEN  = "{{" '~'?;
    let CLOSE = [' ''\t']* '~'? "}}";
    let EXP = [^'}']*;

    let IGN_WP      = [' ''\t']*;
    let BLOCK_EXP   = OPEN '#' EXP CLOSE;
    let END_EXP     = OPEN '/' EXP CLOSE;
    let NO_ESC_EXP  = OPEN '{' EXP '}' CLOSE;
    let PARTIAL_EXP = OPEN '>' EXP CLOSE;
    let SIMPLE_EXP  = OPEN EXP CLOSE;
    let ELSE_EXP    = OPEN IGN_WP ("else" | '^') IGN_WP CLOSE;

    // autotrim lexing
    let BLOCK_TRIM_EXP    = NEW_LINE IGN_WP BLOCK_EXP   IGN_WP NEW_LINE;
    let END_TRIM_EXP      = NEW_LINE IGN_WP END_EXP     IGN_WP NEW_LINE;
    let PARTIAL_TRIM_EXP  = NEW_LINE IGN_WP PARTIAL_EXP IGN_WP NEW_LINE;
    let ELSE_TRIM_EXP     = NEW_LINE IGN_WP ELSE_EXP    IGN_WP NEW_LINE;

    // then rules
    PASS_THROUGH      => |lexer:&mut HandleBarsLexer<R>| Some( TokRaw( lexer.yystr() ) )

    SIMPLE_EXP        => |lexer:&mut HandleBarsLexer<R>| Some( TokSimpleExp(     lexer.yystr() ) )
    NO_ESC_EXP        => |lexer:&mut HandleBarsLexer<R>| Some( TokNoEscapeExp(   lexer.yystr() ) )
    PARTIAL_EXP       => |lexer:&mut HandleBarsLexer<R>| Some( TokPartialExp(    lexer.yystr(), false ) )
    END_EXP           => |lexer:&mut HandleBarsLexer<R>| Some( TokBlockEndExp(   lexer.yystr(), false ) )
    BLOCK_EXP         => |lexer:&mut HandleBarsLexer<R>| Some( TokBlockExp(      lexer.yystr(), false ) )
    ELSE_EXP          => |lexer:&mut HandleBarsLexer<R>| Some( TokBlockElseCond( lexer.yystr(), false ) )

    PARTIAL_TRIM_EXP  => |lexer:&mut HandleBarsLexer<R>| {
      Some( TokPartialExp( lexer.yystr().trim().to_string(), true ) )
    }
    END_TRIM_EXP      => |lexer:&mut HandleBarsLexer<R>| {
      Some( TokBlockEndExp(lexer.yystr().trim().to_string(), true ) )
    }
    BLOCK_TRIM_EXP    => |lexer:&mut HandleBarsLexer<R>| {
      Some( TokBlockExp(   lexer.yystr().trim().to_string(), true ) )
    }
    ELSE_TRIM_EXP     => |lexer:&mut HandleBarsLexer<R>| {
      Some( TokBlockElseCond( lexer.yystr().trim().to_string(), true ) )
    }

}

rustlex! HBExpressionLexer {
  token HBToken;
  property in_options:bool = false;
  property in_params:bool = false;

  let NO_WP       = '~';
  let START       = "{{" ['{''#''/''>']?;
  let START_NO_WP = "{{" '{'? NO_WP ['#''/''>']?;
  let END         =  '}'? "}}";

  let STRING_START = '"';
  let STRING_CTNT  = ("\\\"" | [^'"'])*; // either escaped quote or not quote
  let STRING_END   = ['"'];

  let ACCESSOR_SEP_SLASH = "/";
  let DOT_STARTED = ("." | "..");


  let IDENTIFIER = '@'? [^'!''"''#''%''&''\'''('')''*''+'',''.''/'';''<''=''>''@''[''\\'']''^''`''{''|''}''~'' ''\t']+;
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
    START       => |lexer:&mut HBExpressionLexer<R>| { lexer.ACCESSOR(); None }
    START_NO_WP => |lexer:&mut HBExpressionLexer<R>| { lexer.ACCESSOR(); Some(TokNoWhiteSpaceBefore) }
    END         => |    _:&mut HBExpressionLexer<R>| { None }
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
      if lexer.in_params {
        Some( TokParamSep )
      } else {
        lexer.in_params = true;
        Some( TokParamStart )
      }
    }

    // common ending
    NO_WP        => |lexer:&mut HBExpressionLexer<R>| { lexer.FORCE_END(); Some( TokNoWhiteSpaceAfter ) }
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

    THIS         => |lexer:&mut HBExpressionLexer<R>| { lexer.PROPERTY_PATH(); Some( TokPathEntry( ".".to_string()  ) ) }
    THIS_ALIAS   => |lexer:&mut HBExpressionLexer<R>| { lexer.PROPERTY_PATH(); Some( TokPathEntry( ".".to_string()  ) ) }
    PARENT_ALIAS => |lexer:&mut HBExpressionLexer<R>| { lexer.PROPERTY_PATH(); Some( TokPathEntry( "..".to_string() ) ) }

    // end of parameters
    OPTION_NAME  => |lexer:&mut HBExpressionLexer<R>| {
      lexer.in_options = true;
      lexer.OPTION_VALUE();
      Some( TokOption( lexer.yystr().as_slice().trim_right_matches('=').to_string() ) )
    }

    // common expression ending
    NO_WP        => |lexer:&mut HBExpressionLexer<R>| { lexer.FORCE_END(); Some( TokNoWhiteSpaceAfter ) }
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
    OPTION_NAME  => |lexer:&mut HBExpressionLexer<R>| {  lexer.OPTION_VALUE(); Some( TokOption( lexer.yystr().as_slice().trim_right_matches('=').to_string() ) ) }
    PARAMS_SEP    => |_:&mut HBExpressionLexer<R>| { None }

    // common expression ending
    NO_WP        => |lexer:&mut HBExpressionLexer<R>| { lexer.FORCE_END(); Some( TokNoWhiteSpaceAfter ) }
    END          => |    _:&mut HBExpressionLexer<R>| { None }
  }

  FORCE_END {
    END => |_:&mut HBExpressionLexer<R>| { None }
  }


}

pub enum HBValHolder {
  String(String),
  Path(Vec<String>),
}

pub struct RenderOptions {
  pub escape: bool,
  pub no_leading_whitespace: bool,
  pub no_trailing_whitespace: bool,
}

pub struct HBExpression {
  pub base: Vec<String>,
  pub params: Vec<HBValHolder>,
  pub options: Vec<(String, HBValHolder)>,
  pub render_options: RenderOptions,
  pub block: Option<Box<Template>>,
  pub else_block: Option<Box<Template>>,
}

impl HBExpression {
  pub fn path(&self) -> String {
    let mut r = String::new();
    self.base.iter().fold(&mut r, |mut a, i| {a.push_str(i.as_slice()); a.push('.'); a});

    r
  }
}

pub enum HBEntry {
  Raw(String),
  Eval(HBExpression),
  Partial(HBExpression),
}

pub type Template = Vec<Box<HBEntry>>;

pub enum ParseError {
  UnkownError, // unknown as ‘still not diagnosed case’, not ’your grandma's TV is set on fire case’
  UnmatchedBlock,
  UnexpectedBlockClose,
}

impl Copy for ParseError {}

fn parse_hb_expression(exp: &str) -> Result<HBExpression, (ParseError, Option<String>)> {
  let mut lexer = HBExpressionLexer::new(BufReader::new(exp.as_bytes()));
  let mut render_options = RenderOptions {escape: false, no_leading_whitespace: false, no_trailing_whitespace: false};

  let mut path = vec![];
  let mut params = vec![];
  let mut options = vec![];


  while let Some(tok) = lexer.next() {
    match tok {
      TokNoWhiteSpaceBefore   => { render_options.no_leading_whitespace = true },
      TokNoWhiteSpaceAfter    => { render_options.no_trailing_whitespace = true },
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
                  TokNoWhiteSpaceAfter => { render_options.no_trailing_whitespace = true },
                  _ => { break }
                }
              }

              options.push((option_name, if let Some(val) = opt_val { HBValHolder::String(val) } else { HBValHolder::Path(opt_path) }));

            },
            TokNoWhiteSpaceAfter => { render_options.no_trailing_whitespace = true },
            _ => { break; }
          }
        }
        if param_path.len() > 0 {
          params.push(HBValHolder::Path(param_path));
        }
      },
      _ => { break },
    }
  }


  return  Ok(HBExpression {
    base: path,
    params: params,
    options: options,
    render_options: render_options,
    block: None,
    else_block: None
  })
}

pub fn parse(template: &str) -> Result<Template, (ParseError, Option<String>)> {
  let mut lexer = HandleBarsLexer::new(BufReader::new(template.as_bytes()));
  let mut raw = String::new();

  // parse stack entry tuple: (template, expect else block)
  let mut stack = vec![(box vec![], false)];

  let mut sink_leading_white_space = false;
  let mut sink_trailing_white_space = false;
  let mut wp_back_track = String::new(); // backtrack empty space for whitespace control in HB expression

  let wp_regex = Regex::new(r"([:blank:]|\n)").unwrap();

  for tok in lexer {
    // first match handle raw content
    match tok {
      TokRaw(ref chr) => {
        match (sink_leading_white_space, wp_regex.is_match(chr.as_slice())) {
          (true, true) => (),
          (_, use_backtrack) => {
            if use_backtrack {
              wp_back_track.push_str(chr.as_slice());
            } else {
              sink_leading_white_space = false;

              if ! wp_back_track.is_empty() {
                raw.push_str(wp_back_track.as_slice());
                wp_back_track = String::new();
              }
              raw.push_str(chr.as_slice());
            }
          }
        }

      },
      TokSimpleExp(_) | TokNoEscapeExp(_) | TokBlockExp(_, _) | TokBlockEndExp(_, _) | TokPartialExp(_, _) | TokBlockElseCond(_, _) => {
        if !sink_trailing_white_space && ! wp_back_track.is_empty() {
          raw.push_str(wp_back_track.as_slice());
        }

        wp_back_track = String::new();

        if ! raw.is_empty() {
          stack.last_mut().unwrap().0.push(box HBEntry::Raw(raw));
          raw = String::new();
        }
        sink_trailing_white_space = false
      },
    }

    // second match handle handlebars expressions
    match tok {
      TokRaw(_) => (),
      TokSimpleExp(exp) => {
        if let Ok(hb) = parse_hb_expression(exp.as_slice()) {
          stack.last_mut().unwrap().0.push(box HBEntry::Eval(hb))
        }
      },
      TokNoEscapeExp(exp) => {
        if let Ok(mut hb) = parse_hb_expression(exp.as_slice()) {
          hb.render_options.escape = true;
          stack.last_mut().unwrap().0.push(box HBEntry::Eval(hb))
        }
      },
      TokPartialExp(exp, trimmed) => {
        if trimmed { raw.push('\n') }
        if let Ok(hb) = parse_hb_expression(exp.as_slice()) {
          stack.last_mut().unwrap().0.push(box HBEntry::Partial(hb))
        }
      },
      TokBlockExp(exp, trimmed) => {
        if trimmed { raw.push('\n') }
        if let Ok(hb) = parse_hb_expression(exp.as_slice()) {
          sink_leading_white_space  = hb.render_options.no_leading_whitespace;
          sink_trailing_white_space = hb.render_options.no_trailing_whitespace;
          stack.last_mut().unwrap().0.push(box HBEntry::Eval(hb));
          stack.push((box vec![], false));
        }
      },
      TokBlockElseCond(exp, trimmed) => {
        if trimmed { raw.push('\n') }
        if let Ok(hb) = parse_hb_expression(exp.as_slice()) {
          sink_leading_white_space  = hb.render_options.no_leading_whitespace;
          sink_trailing_white_space = hb.render_options.no_trailing_whitespace;
          stack.push((box vec![], true));
        }
      },
      TokBlockEndExp(exp, trimmed) => {
        if trimmed { raw.push('\n') }
        if let Ok(hb) = parse_hb_expression(exp.as_slice()) {
          sink_leading_white_space  = hb.render_options.no_leading_whitespace;
          sink_trailing_white_space = hb.render_options.no_trailing_whitespace;
          // inspect stack for else template block
          let has_else = match stack.as_slice() {
            [_, (_, false), ( _, true)] => true,
            _ => false,
          };

          let pop = if has_else {
            (stack.pop(), stack.pop())
          } else {
            (None, stack.pop())
          };

          if let Some(&mut (box ref mut parents, _)) = stack.last_mut() {
            match parents.last_mut() {
              Some(&mut box HBEntry::Eval(ref mut parent)) => {
                if parent.base == hb.base {
                  match pop {
                    (some_else, Some((block, _))) => {
                      parent.block = Some(block);
                      if let Some((else_block, _)) = some_else {
                        parent.else_block = Some(else_block);
                      }
                    },
                    _ => panic!("(some_else, Some((block, _))) pattern should always be matched — parse.rs#parse")
                  }

                } else {
                  return Err((ParseError::UnmatchedBlock, Some(format!("‘{}’ does not match ‘{}’", hb.path(), parent.path()))))
                }
              }
              _ => {
                return Err((ParseError::UnexpectedBlockClose, Some(format!("‘{}’ does not close any block", hb.path()))))
              }
            }
          }
        }
      }
    }

  }

  if ! sink_trailing_white_space && ! wp_back_track.is_empty() {
    raw.push_str(wp_back_track.as_slice());
  }

  if ! raw.is_empty() {
    stack.last_mut().unwrap().0.push(box HBEntry::Raw(raw));
  }

  if stack.len() > 0 {
    Result::Ok(*stack.remove(0).0)
  } else {
    Result::Err((ParseError::UnkownError, None))
  }
}

#[cfg(test)]
mod tests {
  use std::default::Default;
  use super::{parse, parse_hb_expression, HBEntry, HBExpression, HBValHolder};

  // commented out due to the error
  //   error: the trait `core::fmt::String` is not implemented for the type `parse::HBToken`
  // for println!

  // #[allow(dead_code)]
  // fn debug_parse_hb(exp: &str) {
  //   let mut lexer = HBExpressionLexer::new(BufReader::new(exp.as_bytes()));
  //   println!("{}", exp);
  //   for tok in *lexer {
  //     println!("{}", tok);
  //   }

  // }

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

  #[allow(unused_variables)]
  #[test]
  fn hb_string_param() {
    match parse_hb_expression(r##"{{p "string"}}"##) {
      Ok(HBExpression{ref base, ref params, ref options, ref render_options, ref block, ref else_block})  => {
        assert_eq!(base, &vec!["p"]);
        assert_eq!(match params.get(0).unwrap() { &HBValHolder::String(ref s) => s.clone(), _ => "".to_string()}, "string".to_string());
      },
      Err(_)  => (),
    }
  }

  #[allow(unused_variables)]
  #[test]
  fn hb_prop_path_param() {
    match parse_hb_expression(r##"{{p some.path}}"##) {
      Ok(HBExpression{ref base, ref params, ref options, ref render_options, ref block, ref else_block})  => {
        assert_eq!(base, &vec!["p"]);
        assert_eq!(match params.get(0).unwrap() { &HBValHolder::Path(ref p) => p.clone(), _ => vec![]}, vec!["some", "path"]);
      },
      Err(_)  => (),
    }
  }

  #[allow(unused_variables)]
  #[test]
  fn hb_2_params() {
    match parse_hb_expression(r##"{{p some path}}"##) {
      Ok(HBExpression{ref base, ref params, ref options, ref render_options, ref block, ref else_block})  => {
        assert_eq!(base, &vec!["p"]);
        assert_eq!(match params.get(0).unwrap() { &HBValHolder::Path(ref p) => p.clone(), _ => vec![]}, vec!["some"]);
        assert_eq!(match params.get(1).unwrap() { &HBValHolder::Path(ref p) => p.clone(), _ => vec![]}, vec!["path"]);
      },
      Err(_)  => (),
    }
  }

  #[allow(unused_variables)]
  #[test]
  fn hb_3_params() {
    match parse_hb_expression(r##"{{p some.path "with_string" yep}}"##) {
      Ok(HBExpression{ref base, ref params, ref options, ref render_options, ref block, ref else_block})  => {
        assert_eq!(base, &vec!["p"]);
        assert_eq!(match params.get(0).unwrap() { &HBValHolder::Path(ref p) => p.clone(), _ => vec![]}, vec!["some", "path"]);
        assert_eq!(match params.get(1).unwrap() { &HBValHolder::String(ref s) => s.clone(), _ => "".to_string()}, "with_string".to_string());
        assert_eq!(match params.get(2).unwrap() { &HBValHolder::Path(ref p) => p.clone(), _ => vec![]}, vec!["yep"]);
      },
      Err(_)  => (),
    }
  }

  #[allow(unused_variables)]
  #[test]
  fn hb_full_feat_param() {
    match parse_hb_expression(r##"{{t "… param1" well.[that my baby].[1] ~}}"##) {
      Ok(HBExpression{ref base, ref params, ref options, ref render_options, ref block, ref else_block})  => {
        assert_eq!(base, &vec!["t"]);
        assert_eq!(match params.get(0).unwrap() { &HBValHolder::String(ref s) => s.clone(), _ => "".to_string()}, "… param1".to_string());
        assert_eq!(match params.get(1).unwrap() { &HBValHolder::Path(ref p) => p.clone(), _ => vec![]}, vec!["well", "that my baby", "1"]);
        assert!(render_options.no_trailing_whitespace);
      },
      Err(_)  => (),
    }
  }

  #[allow(unused_variables)]
  #[test]
  fn hb_option() {
    match parse_hb_expression(r##"{{t opt=u ~}}"##) {
      Ok(HBExpression{ref base, ref params, ref options, ref render_options, ref block, ref else_block})  => {
        assert_eq!(base, &vec!["t"]);
        assert_eq!(("opt".to_string(), vec!["u".to_string()]), match options.get(0).unwrap() {
          &(ref o, HBValHolder::Path(ref p)) => (o.clone(), p.clone()),
          _ => ("".to_string(), vec![]),
        });
        assert!(render_options.no_trailing_whitespace);
      },
      Err(_)  => (),
    }
  }

  #[allow(unused_variables)]
  #[test]
  fn hb_mutli_options() {
    match parse_hb_expression(r##"{{t opt=u opt2="v" ~}}"##) {
      Ok(HBExpression{ref base, ref params, ref options, ref render_options, ref block, ref else_block})  => {
        assert_eq!(base, &vec!["t"]);
        assert_eq!(("opt".to_string(), vec!["u".to_string()]), match options.get(0).unwrap() {
          &(ref o, HBValHolder::Path(ref p)) => (o.clone(), p.clone()),
          _ => ("".to_string(), vec![]),
        });
        assert_eq!(("opt2".to_string(), "v".to_string()), match options.get(1).unwrap() {
          &(ref o, HBValHolder::String(ref s)) => (o.clone(), s.clone()),
          _ => ("".to_string(), "".to_string()),
        });
        assert!(render_options.no_trailing_whitespace);
      },
      Err(_)  => (),
    }
  }

  #[allow(unused_variables)]
  #[test]
  fn hb_param_options() {
    match parse_hb_expression(r##"{{t o.[t}+=] opt="v" ~}}"##) {
      Ok(HBExpression{ref base, ref params, ref options, ref render_options, ref block, ref else_block})  => {
        assert_eq!(base, &vec!["t"]);
        assert_eq!(vec!["o", "t}+="], match params.get(0).unwrap() {
          &HBValHolder::Path(ref p) => p.clone(), _ => vec![]
        });
        assert_eq!(("opt".to_string(), "v".to_string()), match options.get(0).unwrap() {
          &(ref o, HBValHolder::String(ref s)) => (o.clone(), s.clone()),
          _ => ("".to_string(), "".to_string()),
        });
        assert!(render_options.no_trailing_whitespace);
      },
      Err(_)  => (),
    }
  }

  #[test]
  fn parse_raw() {
    let p = parse("tada").unwrap_or(Default::default());
    assert_eq!("tada", match p.get(0) {
      Some(&box HBEntry::Raw(ref s)) => s.as_slice(),
      _ => "",
    });
  }

  #[test]
  fn parse_exp() {
    let p = parse("{{tada}}").unwrap_or(Default::default());
    assert_eq!("tada", match p.get(0) {
      Some(&box HBEntry::Eval(HBExpression {ref base, ..})) => base.iter().next().unwrap().as_slice(),
      _ => "",
    });
  }

  #[allow(unused_variables)]
  #[test]
  fn parse_else_block() {
    let p = parse("{{#tada}}i{{else}}o{{/tada}}").unwrap_or(Default::default());;
    assert_eq!(true, match p.get(0) {
      Some(&box HBEntry::Eval(HBExpression {ref base, ref params, ref options, ref render_options, ref block, ref else_block})) => {
        match (block, else_block) { (&Some(_), &Some(_)) => true, _ => false }
      },
      _ => false,
    });
  }


  #[test]
  fn parse_exp_entangled() {
    let p = parse("tidi {{tada}} todo {{tudu}} bar").unwrap_or(Default::default());;
    assert_eq!("tidi ", match p.get(0) {
      Some(&box HBEntry::Raw(ref s)) => s.as_slice(),
      _ => "",
    });
    assert_eq!("tada", match p.get(1) {
      Some(&box HBEntry::Eval(HBExpression {ref base, ..})) => base.iter().next().unwrap().as_slice(),
      _ => "",
    });
    assert_eq!(" todo ", match p.get(2) {
      Some(&box HBEntry::Raw(ref s)) => s.as_slice(),
      _ => "",
    });
    assert_eq!("tudu", match p.get(3) {
      Some(&box HBEntry::Eval(HBExpression {ref base, ..})) => base.iter().next().unwrap().as_slice(),
      _ => "",
    });
    assert_eq!(" bar", match p.get(4) {
      Some(&box HBEntry::Raw(ref s)) => s.as_slice(),
      _ => "",
    });
  }
}
