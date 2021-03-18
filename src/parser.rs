use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_while1};
use nom::character::complete::{alphanumeric1, digit1, newline, none_of};
use nom::combinator::{all_consuming, map, map_opt, opt, recognize};
use nom::error::ParseError;
use nom::multi::{fold_many_m_n, many0, many1, many_till};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::Slice;

use crate::ast::*;
use crate::Span;

mod nom_ext;
use self::nom_ext::*;

#[cfg(test)]
mod tests;

type PResult<'a, T, E> = nom::IResult<Span<'a>, T, E>;

pub fn parse_doc<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> PResult<'a, Document<'a>, E> {
    let (i, header) = opt(parse_doc_header)(i)?;

    let f = terminated(parse_blocks, wsnl);
    let mut f = all_consuming(f);
    let (i, contents) = f(i)?;

    let doc = Document { header, content: contents };
    Ok((i, doc))
}

pub fn parse_doc_header<'a, E: ParseError<Span<'a>>>(
    i: Span<'a>,
) -> PResult<'a, DocumentHeader<'a>, E> {
    let (i, title) = preceded(tag("= "), terminated(take_until("\n"), tag("\n")))(i)?;
    // parse author
    // parse version
    let (i, attributes) = many0(parse_doc_attribute)(i)?;
    let h = DocumentHeader { title, author: None, version: None, attributes };
    Ok((i, h))
}

pub fn parse_doc_attribute<'a, E: ParseError<Span<'a>>>(
    i: Span<'a>,
) -> PResult<'a, DocAttribute<'a>, E> {
    // FIXME: hard/soft wrap attribute values
    let ctor = |((bang1, id), value): ((Option<_>, Span<'a>), Option<_>)| {
        let ends_with_bang = id.text().ends_with('!');
        DocAttribute {
            id: if ends_with_bang { id.slice(..id.len() - 1) } else { id },
            unset: bang1.is_some() || ends_with_bang,
            value: value.into_iter().collect(),
        }
    };
    let id =
        delimited(tag(":"), pair(opt(tag("!")), take_while1(|c| c != '\n' && c != ':')), tag(":"));
    map(terminated(pair(id, opt(preceded(ws1, take_until("\n")))), ws_with_nl), ctor)(i)
}

pub fn parse_attribute<'a, E: ParseError<Span<'a>>>(
    i: Span<'a>,
) -> PResult<'a, (Span<'a>, Option<Span<'a>>), E> {
    let name = recognize(pair(alphanumeric1, many0(alt((alphanumeric1, tag("-"), tag("."))))));
    pair(name, opt(preceded(ws_delimited(tag("=")), recognize(many1(none_of(",]\n"))))))(i)
}

pub fn parse_attribute_list<'a, E: ParseError<Span<'a>>>(
    i: Span<'a>,
) -> PResult<'a, AttributeList<'a>, E> {
    let parse_attributes = |mut i| {
        let mut attr_list = AttributeList::default();
        if let Ok((i2, (key, val))) = parse_attribute::<()>(i) {
            attr_list.insert(key.text(), val.map(|s| s.text()));
            i = i2;
        }
        while let PResult::<_, ()>::Ok((i2, (key, val))) =
            preceded(ws_delimited(tag(",")), parse_attribute)(i)
        {
            i = i2;
            attr_list.insert(key.text(), val.map(|s| s.text()));
        }
        Ok((i, attr_list))
    };
    delimited(tag("["), parse_attributes, tag("]"))(i)
}

pub fn parse_blocks<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> PResult<'a, Blocks<'a>, E> {
    many0(parse_attributed_block)(i)
}

pub fn parse_attributed_block<'a, E: ParseError<Span<'a>>>(
    i: Span<'a>,
) -> PResult<'a, Block<'a>, E> {
    let thematic_break = map(tag("'''"), |_| Context::ThematicBreak);
    let page_break = map(tag(">>>"), |_| Context::PageBreak);
    let fenced = map(delimited(pair(tag("```"), newline), take_until("```"), tag("```")), |span| {
        Context::Listing(span)
    });

    let parse_block = terminated(alt((thematic_break, page_break, fenced)), ws_with_nl);
    preceded(
        many0(ws_with_nl),
        map(
            tuple((opt(parse_attribute_list), parse_block, parse_callouts)),
            |(attr_list, context, callouts)| Block {
                context,
                attributes: attr_list.unwrap_or_default(),
                callouts,
            },
        ),
    )(i)
}

pub fn parse_callouts<'a, E: ParseError<Span<'a>>>(
    i: Span<'a>,
) -> PResult<'a, Vec<Callout<'a>>, E> {
    many0(parse_callout)(i)
}

pub fn parse_callout<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> PResult<'a, Callout<'a>, E> {
    map(
        pair(
            map_opt(delimited(tag("<"), digit1, tag(">")), |span: Span| span.parse::<usize>().ok()),
            delimited(ws, take_until("\n"), newline),
        ),
        |(number, text)| Callout { number, text },
    )(i)
}

pub fn parse_section_title<'a, E: ParseError<Span<'a>>>(
    i: Span<'a>,
) -> PResult<'a, SectionTitle<'a>, E> {
    let parse_level = map(fold_many_m_n(1, 6, tag("="), 0, |acc, _| acc + 1), |level| level - 1);
    let mut parse_level = terminated(parse_level, ws);
    let (i, level) = parse_level(i)?;

    let (i, content) = take_line(i)?;

    Ok((i, SectionTitle { level, content }))
}

pub fn parse_paragraph<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> PResult<'a, Vec<Span<'a>>, E> {
    let (i1, (tags, _)) = many_till(take_line, tag("\n"))(i)?;
    Ok((i1, tags))
}
