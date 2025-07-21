extern crate combine;
use combine::parser::char::{digit, letter};
use combine::stream::position;
use combine::{EasyParser, Parser};

mod context;
mod equation;
mod id;
mod oper;
mod parser;
mod schema;
mod symbol_table;
mod term;
mod theory;
mod r#type;
mod context_table;

const MSG: &'static str = r#"Parse error at line: 1, column: 1
Unexpected `|`
Expected digit or letter
"#;

fn main() {
    // Wrapping a `&str` with `State` provides automatic line and column tracking. If `State`
    // was not used the positions would instead only be pointers into the `&str`
    if let Err(err) = digit().or(letter()).easy_parse(position::Stream::new("6")) {
        assert_eq!(MSG, format!("{}", err));
    }

    use combine::attempt;
    use combine::parser::char::string;
    let mut p = attempt(string("let")).or(string("lex"));
    let result = p.parse("let").map(|x| x.0);
    assert_eq!(result, Ok("let"));

    let result = p.parse("lex").map(|x| x.0);
    assert_eq!(result, Ok("lex"));
    let result = p.parse("aet").map(|x| x.0);
    assert!(result.is_err());
}
