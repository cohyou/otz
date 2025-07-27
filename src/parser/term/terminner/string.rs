use combine::{between, many, none_of, token, Parser, Stream};

use crate::term::TermInner;

pub fn string_parser<Input>() -> impl Parser<Input, Output = TermInner>
where
    Input: Stream<Token = char>,
{
    between(token('"'), token('"'), many(none_of("\"".chars()))).map(|s: Vec<_>| {
        let s: String = s.iter().collect();
        TermInner::Str(s.into())
    })
}

#[test]
fn test_parse_string() {
    use combine::EasyParser;

    let input = "\"abc\"";
    let result = string_parser().easy_parse(input);
    dbg!(&result);
    assert!(result.is_ok());
}
