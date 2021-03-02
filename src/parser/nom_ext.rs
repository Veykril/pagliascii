use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_while, take_while1};
use nom::combinator::{map, peek, recognize, rest_len, verify};
use nom::error::ParseError;
use nom::sequence::{delimited, terminated};
use nom::Parser;

use crate::parser::PResult;
use crate::span::Span;

pub fn take_until1<'a, E>(tag: &'static str) -> impl FnMut(Span<'a>) -> PResult<'a, Span<'a>, E>
where
    E: ParseError<Span<'a>>,
{
    verify(take_until(tag), |s: &Span<'a>| s.len() > 1)
}

pub fn ws<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> PResult<'a, Span<'a>, E> {
    take_while(|c: char| c != '\n' && c.is_whitespace())(i)
}

pub fn ws1<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> PResult<'a, Span<'a>, E> {
    take_while1(|c: char| c != '\n' && c.is_whitespace())(i)
}

pub fn wsnl<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> PResult<'a, Span<'a>, E> {
    take_while(char::is_whitespace)(i)
}

pub fn ws_delimited<'a, E: ParseError<Span<'a>>, O, P: Parser<Span<'a>, O, E>>(
    parser: P,
) -> impl FnMut(Span<'a>) -> PResult<'a, O, E> {
    delimited(ws, parser, ws)
}

pub fn spacer<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> PResult<'a, Span<'a>, E> {
    take_while1(is_spacer)(i)
}

pub fn rspaced<'a, F, O, E>(f: F) -> impl FnMut(Span<'a>) -> PResult<'a, O, E>
where
    E: ParseError<Span<'a>>,
    F: Fn(Span<'a>) -> PResult<'a, O, E>,
{
    let eof = recognize(verify(rest_len, |&i| i == 0));
    let is_end = map(peek(eof), |_| ());
    let after = alt((is_end, map(peek(spacer), |_| ())));
    terminated(f, after)
}

pub fn is_spacer(char: char) -> bool {
    match char {
        '\\' => false,
        _ => !char.is_alphanumeric(),
    }
}

pub fn take_line<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> PResult<'a, Span<'a>, E> {
    terminated(take_until("\n"), tag("\n"))(i)
}
pub fn ws_with_nl<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> PResult<'a, Span<'a>, E> {
    dbg!(i);
    terminated(ws, tag("\n"))(i)
}
