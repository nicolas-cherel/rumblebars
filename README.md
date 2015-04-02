# rumblebars — a handlebars template evaluation library
-

Rumblebars passes **all mustaches specs** [[1]](#1) and **272 handlebars tests** [[2]](#2). Template evaluation is rendered to a `io::Writer`, so that you can choose wether if you hold result in memory or not. It also input data angostic, given that your data structure implements the `HBData` trait (Json implementation provided).

 [1] <a name="1"></a> except delimiter changes test suite and one test failing because of a trailing space  
 [2] <a name="2"></a> all tests that does not involves javascript in data and partials, and see the [comments for other cases](https://github.com/nicolas-cherel/rumblebars/blob/master/tests/eval/handlebars.rs#L88-L134)

## HMTL escaping safety

All output is filtered by being written to the `SafeWriting` trait. Helpers, just as regular evaluation do for unescaped content, have to opt out escaped writing by calling `SafeWriting::into_unsafe()` that will return the underlying unfiltered writer.

## Quick start

The api will be enriched with shortcuts, but at the moment use `rumblebars::eval()`

```rust
// we need to specify Json because HBData implement several types that
// complies to FromStr
let json: Json = r##"{"h": "hello"}"##.parse().ok().unwrap_or(Json::Null);
let tmpl = r##"{{h}}"##.parse().ok().unwrap();

let eval_ctxt: EvalContext = Default::default();
let mut buf = Vec::<u8>::new();

rumblebars::eval(&tmpl, &json, &mut buf, &eval_ctxt).ok();

// buf now contains template evaluation result. It would have
// been written to disk if it was a file handle, or
// transmitted through network if it was a socket.

```

## helpers

Helpers are registered to the evaluation context. They are simple bare function (probably Fn() in the future), that have to write their content on the `out: &mut Writer`. If you need to processed content before rendering it to the final `Writer`, just render it to a buffer put into a safe writter.

example:

```rust
fn example_helper(
  params: &[&HBData], options: &HelperOptions,
  out: &mut SafeWriting, hb_context: &EvalContext
) -> HBEvalResult
{
	let mut buf = Vec::<u8>::new();
	let res = SafeWriting::with_html_safe_writer(&mut buf, &|out| {
		"pouet pouet".to_string().write_value(out)
	});

	if !res.ok() { return res; };

   let mut s = String::from_utf8(buf).ok().unwrap();
	s.insert(5, '∂');
	s.write_value(out)
}
```

To use your hepler you just have to register it before evaluating your template:

```rust
let json = Json::Null;
let tmpl = parse(r##"{{example_helper}}"##).ok().unwrap();

let mut eval_ctxt: EvalContext = Default::default();
let mut buf = Vec::<u8>::new();

eval_ctxt.register_helper("example_helper".to_string(), Helper::new_with_function(example_helper));

eval(&tmpl, &json, &mut buf, &eval_ctxt).ok();

```
