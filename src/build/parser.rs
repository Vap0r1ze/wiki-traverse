use std::ops::RangeFrom;

use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag},
    character::complete::{char, digit1, one_of},
    combinator::{map, opt, recognize, value, verify},
    error::Error,
    multi::{fold_many0, many1},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

fn uint(input: &str) -> IResult<&str, &str> {
    recognize(many1(digit1))(input)
}
fn int(input: &str) -> IResult<&str, &str> {
    recognize(tuple((opt(char('-')), many1(digit1))))(input)
}
fn float(input: &str) -> IResult<&str, &str> {
    recognize(tuple((uint, char('.'), uint)))(input)
}
fn null(input: &str) -> IResult<&str, &str> {
    tag("NULL")(input)
}
fn string(input: &str) -> IResult<&str, &str> {
    recognize(delimited(
        char('\''),
        opt(escaped(is_not("\\'"), '\\', one_of("'\\\""))),
        char('\''),
    ))(input)
}

pub fn rows<I: Clone + nom::Slice<RangeFrom<usize>> + nom::InputIter, O, F>(
    mut row_parser: F,
) -> impl FnMut(I) -> IResult<I, Vec<O>, Error<I>>
where
    F: nom::Parser<I, O, Error<I>>,
    <I as nom::InputIter>::Item: nom::AsChar,
{
    move |input| {
        let mut input = input;
        let mut rows = Vec::new();
        loop {
            let (i, row) = row_parser.parse(input.clone())?;
            rows.push(row);
            if let Ok((i, _)) = char::<I, Error<I>>(',')(i.clone()) {
                input = i;
            } else {
                (input, _) = char(';')(i)?;
                return Ok((input, rows));
            }
        }
    }
}

fn escaped_char(input: &str) -> IResult<&str, char, Error<&str>> {
    preceded(
        char('\\'),
        // `alt` tries each parser in sequence, returning the result of
        // the first successful match
        alt((
            value('\'', char('\'')),
            value('\\', char('\\')),
            value('"', char('"')),
        )),
    )(input)
}

fn str_literal(input: &str) -> IResult<&str, &str, Error<&str>> {
    verify(is_not("'\\"), |s: &str| !s.is_empty())(input)
}

enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
}
fn str_fragment(input: &str) -> IResult<&str, StringFragment<'_>, Error<&str>> {
    alt((
        map(str_literal, StringFragment::Literal),
        map(escaped_char, StringFragment::EscapedChar),
    ))(input)
}

pub fn string_value<'a>(input: &'a str) -> String {
    let build_string = fold_many0(str_fragment, String::new, |mut string, fragment| {
        match fragment {
            StringFragment::Literal(s) => string.push_str(s),
            StringFragment::EscapedChar(c) => string.push(c),
        }
        string
    });
    let result: IResult<&str, String, Error<&'a str>> =
        delimited(char('\''), build_string, char('\''))(input);
    result.unwrap().1
}

macro_rules! tuple_parser {
    ($name:ident, ($( $p:expr ),*)) => {
        pub fn $name(input: &str) -> IResult<&str, ($(${ignore(p)} &str),*)> {
            delimited(tag("("), tuple((tuple_parser!(#($($p),*), ()))), tag(")"))(input)
        }
    };
    (#($p1:expr, $( $p:expr ),+), ($( $i:expr ),*)) => {
        tuple_parser!(#($($p),*), ($($i,)* tuple_parser!(# $p1)))
    };
    (# $p:expr) => {
        terminated($p, char(','))
    };
    (#($p:expr), ($( $i:expr ),*)) => {
        ($($i,)* $p)
    };
}

// tuple_parser!(
//     page_row,
//     (
//         uint,
//         uint,
//         string,
//         uint,
//         uint,
//         float,
//         string,
//         alt((string, null)),
//         uint,
//         uint,
//         alt((string, null)),
//         alt((string, null))
//     )
// );
tuple_parser!(
    page_row,
    (
        uint,
        uint,
        string,
        string,
        uint,
        uint,
        float,
        string,
        alt((string, null)),
        uint,
        uint,
        alt((string, null)),
        alt((string, null))
    )
);
tuple_parser!(pagelink_row, (uint, uint, string, uint));
tuple_parser!(redirect_row, (uint, int, string, string, string));
