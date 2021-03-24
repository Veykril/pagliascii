use std::fmt::{self, Write as _};

use expect_test::{expect, Expect};
use nom::error::VerboseError;
use nom::IResult;

use crate::Span;

fn assert_debug_eq_nom<T: fmt::Debug>(
    input: Span,
    expect: Expect,
    res: IResult<Span, T, VerboseError<Span>>,
) {
    match res {
        Ok((_, res)) => expect.assert_debug_eq(&res),
        Err(e) => match e {
            nom::Err::Incomplete(e) => panic!("{:?}", e),
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                let mut buf = String::new();
                writeln!(buf, "\n").unwrap();
                for (err_pos, err) in e.errors {
                    let line = err_pos.location_line() as usize;
                    let col = err_pos.get_utf8_column();

                    let line_text = input.fragment().lines().nth(line - 1).unwrap_or_default();
                    writeln!(buf, "{:>4} | {}", line, line_text).unwrap();
                    writeln!(buf, "{}^", " ".repeat(4 + 3 + col - 1)).unwrap();
                    writeln!(buf, "Parsing error: {:?}", err).unwrap();
                }
                panic!("{}", buf);
            }
        },
    }
}

fn check_parse<'a, T: fmt::Debug>(
    parser: impl FnOnce(Span<'a>) -> nom::IResult<Span<'_>, T, VerboseError<Span<'_>>>,
    input: &'a str,
    expected: Expect,
) {
    let input = Span::new(input);
    assert_debug_eq_nom(input, expected, parser(input));
}

#[test]
fn parse_attr() {
    check_parse(
        super::parse_attribute,
        "foobar",
        expect![[r#"
            (
                "foobar",
                None,
            )
        "#]],
    );
    check_parse(
        super::parse_attribute,
        "foobar,foobar",
        expect![[r#"
            (
                "foobar",
                None,
            )
        "#]],
    );
    check_parse(
        super::parse_attribute,
        "foobar=14\n",
        expect![[r#"
            (
                "foobar",
                Some(
                    "14",
                ),
            )
        "#]],
    );
    check_parse(
        super::parse_attribute,
        "foobar = 14",
        expect![[r#"
            (
                "foobar",
                Some(
                    "14",
                ),
            )
        "#]],
    );
    check_parse(
        super::parse_attribute,
        "foobar = \"14\"abc",
        // FIXME
        expect![[r#"
            (
                "foobar",
                Some(
                    "\"14\"abc",
                ),
            )
        "#]],
    );
}

#[test]
fn parse_attr_list() {
    check_parse(
        super::parse_attribute_list,
        "[foobar]",
        expect![[r#"
            {
                "foobar": None,
            }
        "#]],
    );
    check_parse(
        super::parse_attribute_list,
        "[foobar,baz]",
        expect![[r#"
            {
                "foobar": None,
                "baz": None,
            }
        "#]],
    );
    check_parse(
        super::parse_attribute_list,
        "[foobar = foo ,baz , qux]",
        expect![[r#"
            {
                "foobar": Some(
                    "foo ",
                ),
                "baz": None,
                "qux": None,
            }
        "#]],
    );
}

#[test]
fn parse_doc_attribute() {
    check_parse(
        super::parse_doc_attribute,
        ":foo:\n",
        expect![[r#"
            DocAttribute {
                id: "foo",
                unset: false,
                value: [],
            }
        "#]],
    );
    check_parse(
        super::parse_doc_attribute,
        ":foo: bar baz qux\n",
        expect![[r#"
            DocAttribute {
                id: "foo",
                unset: false,
                value: [
                    "bar baz qux",
                ],
            }
        "#]],
    );
    check_parse(
        super::parse_doc_attribute,
        ":foo: bar baz qux     \n",
        expect![[r#"
            DocAttribute {
                id: "foo",
                unset: false,
                value: [
                    "bar baz qux     ",
                ],
            }
        "#]],
    );
    check_parse(
        super::parse_doc_attribute,
        ":!foo: bar\n",
        expect![[r#"
            DocAttribute {
                id: "foo",
                unset: true,
                value: [
                    "bar",
                ],
            }
        "#]],
    );
    check_parse(
        super::parse_doc_attribute,
        ":foo!:\n",
        expect![[r#"
            DocAttribute {
                id: "foo",
                unset: true,
                value: [],
            }
        "#]],
    );
    check_parse(
        super::parse_doc_attribute,
        ":!foo!:\n",
        expect![[r#"
            DocAttribute {
                id: "foo",
                unset: true,
                value: [],
            }
        "#]],
    );
}

#[test]
fn parse_doc_header() {
    check_parse(
        super::parse_doc_header,
        r"= Headline
:header_attr: attr
:header_attr:

:doc_attr:
",
        expect![[r#"
            DocumentHeader {
                title: "Headline",
                author: None,
                version: None,
                attributes: [
                    DocAttribute {
                        id: "header_attr",
                        unset: false,
                        value: [
                            "attr",
                        ],
                    },
                    DocAttribute {
                        id: "header_attr",
                        unset: false,
                        value: [],
                    },
                ],
            }
        "#]],
    );
}

#[test]
fn parse_callouts() {
    check_parse(
        super::parse_callouts,
        r"<0> foo
<1> foo
<2>bar
<1337>baz
",
        expect![[r#"
            [
                Callout {
                    number: 0,
                    text: "foo",
                },
                Callout {
                    number: 1,
                    text: "foo",
                },
                Callout {
                    number: 2,
                    text: "bar",
                },
                Callout {
                    number: 1337,
                    text: "baz",
                },
            ]
        "#]],
    );
}
#[test]
fn parse_single_block_no_attributes() {
    check_parse(
        super::parse_attributed_block,
        r">>>",
        expect![[r#"
            Block {
                context: PageBreak,
                attributes: {},
                callouts: [],
            }
        "#]],
    );
    check_parse(
        super::parse_attributed_block,
        r"'''",
        expect![[r#"
            Block {
                context: ThematicBreak,
                attributes: {},
                callouts: [],
            }
        "#]],
    );
    check_parse(
        super::parse_attributed_block,
        r"```
This is a listing block
with multiple lines
```",
        expect![[r#"
            Block {
                context: Listing(
                    "This is a listing block\nwith multiple lines\n",
                ),
                attributes: {},
                callouts: [],
            }
        "#]],
    );
    check_parse(
        super::parse_attributed_block,
        r"image::foo.png[width=240]",
        expect![[r#"
            Block {
                context: BlockMacro(
                    Macro {
                        name: "image",
                        target: "foo.png",
                        attribute_list: {
                            "width": Some(
                                "240",
                            ),
                        },
                    },
                ),
                attributes: {},
                callouts: [],
            }
        "#]],
    );
}
