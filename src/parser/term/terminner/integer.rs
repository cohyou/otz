use combine::{many1, one_of, Parser, Stream};

use crate::term::TermInner;

pub fn integer_parser<Input>() -> impl Parser<Input, Output = TermInner>
where
    Input: Stream<Token = char>,
{
    many1(one_of("1234567890".chars())).map(|s: Vec<_>| {
        let i: String = s.iter().collect();
        let i: usize = i.parse().expect("error on integer_parser");
        TermInner::Int(i)
    })
}

#[test]
fn test_parse_interger() {
    use combine::EasyParser;

    let input = "12345";
    let result = integer_parser().easy_parse(input);
    dbg!(&result);
    assert!(result.is_ok());
}
