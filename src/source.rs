use nom::branch::alt;
use nom::bytes::complete::{tag, take_till, take_till1};
use nom::combinator::{map, recognize};
use nom::sequence::{delimited, pair, preceded};
use nom::IResult;

use crate::attributes::AttributeMap;

#[derive(Clone, Copy, Debug, Default)]
pub struct TextRange {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct LineIndex(usize);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileId(usize);

#[derive(Debug)]
struct CondDirective {
    targets: String,
    /// Whether this directive caused us to start skipping
    skipping: bool,
}

#[derive(Debug)]
struct Include {
    source: String,
    processed: usize,
}

impl Include {
    fn next_line(&mut self) -> Option<&str> {
        let (_, line) = <IResult<_, _, ()>>::ok(recognize(take_till(|c| c == '\n'))(
            self.source.get(self.processed..)?,
        ))?;
        let l = line.len();
        self.processed += l + 1;
        if let Some(b'\r') = line.as_bytes().last() {
            Some(&line[0..l - 1])
        } else {
            Some(line)
        }
    }
}

pub struct DocumentSource {
    amalgamated: String,
}

impl DocumentSource {
    pub fn new<S, E, CB>(source: S, include_cb: CB) -> Result<Self, PreprocessError<E>>
    where
        S: Into<String>,
        CB: FnMut(&AttributeMap, &str) -> Result<String, E>,
    {
        let mut pp = Preprocessor::new(source.into(), include_cb, Default::default());
        pp.amalgamate()?;
        Ok(DocumentSource { amalgamated: pp.amalgamated })
    }
}

#[derive(Debug)]
pub enum PreprocessError<IE> {
    MaxIncludeDepthReached,
    IncludeError(IE),
}

impl<IE> From<IE> for PreprocessError<IE> {
    fn from(ie: IE) -> Self {
        PreprocessError::IncludeError(ie)
    }
}

struct Preprocessor<E, CB>
where
    CB: FnMut(&AttributeMap, &str) -> Result<String, E>,
{
    amalgamated: String,
    include_stack: Vec<Include>,
    conditional_stack: Vec<CondDirective>,
    skipping: bool,
    include_cb: CB,
    max_include_depth: usize,
    attribute_map: AttributeMap,
}

impl<'cb, E, CB> Preprocessor<E, CB>
where
    CB: for<'a> FnMut(&AttributeMap, &'a str) -> Result<String, E> + 'cb,
{
    pub fn new(source: impl Into<String>, include_cb: CB, attributes: AttributeMap) -> Self {
        let source = source.into();
        Preprocessor {
            skipping: false,
            amalgamated: String::with_capacity(source.len()),
            conditional_stack: vec![],
            include_stack: vec![Include { source, processed: 0 }],
            include_cb,
            max_include_depth: 64,
            attribute_map: attributes,
        }
    }

    pub fn amalgamate(&mut self) -> Result<(), PreprocessError<E>> {
        loop {
            let n_includes = self.include_stack.len();
            let line = match self.include_stack.last_mut() {
                Some(include) => match include.next_line() {
                    Some(line) => line,
                    None => {
                        self.include_stack.pop();
                        continue;
                    }
                },
                None => break,
            };

            if let Some(doc_attrib) = Self::parse_doc_attrib(line) {
            } else if let Some(directive) = Self::parse_pp_directive(line) {
                match directive {
                    PreprocessorDirective::EndIf { targets: _ } => {
                        // FIXME: check that targets match
                        if let Some(directive) = self.conditional_stack.pop() {
                            if directive.skipping {
                                self.skipping = false;
                            }
                        }
                    }
                    PreprocessorDirective::Include { target, attributes: _ } if !self.skipping => {
                        if n_includes >= self.max_include_depth {
                            return Err(PreprocessError::MaxIncludeDepthReached);
                        }
                        let source = (self.include_cb)(&self.attribute_map, target)?;
                        if matches!(
                            self.include_stack.last(),
                                Some(&Include { ref source, processed }) if processed >= source.len()
                        ) {
                            self.include_stack.pop();
                        }
                        self.include_stack.push(Include { processed: 0, source });
                    }
                    // would be nice to unify the following arm pairs
                    PreprocessorDirective::IfDef { targets, inline: Some(line) } => {
                        if !self.skipping
                            && Self::check_targets_active(targets, &self.attribute_map)
                        {
                            Self::push_line(&mut self.amalgamated, line);
                        }
                    }
                    PreprocessorDirective::IfDef { targets, inline: None } => {
                        let skipping = !Self::check_targets_active(targets, &self.attribute_map);
                        self.conditional_stack.push(CondDirective {
                            targets: targets.to_owned(),
                            skipping: !self.skipping & skipping,
                        });
                        self.skipping |= skipping;
                    }
                    PreprocessorDirective::IfNotDef { targets, inline: Some(line) } => {
                        if !self.skipping
                            && !Self::check_targets_active(targets, &self.attribute_map)
                        {
                            Self::push_line(&mut self.amalgamated, line);
                        }
                    }
                    PreprocessorDirective::IfNotDef { targets, inline: None } => {
                        let skipping = Self::check_targets_active(targets, &self.attribute_map);
                        self.conditional_stack.push(CondDirective {
                            targets: targets.to_owned(),
                            skipping: !self.skipping & skipping,
                        });
                        self.skipping |= skipping;
                    }
                    // PreprocessorDirective::IfEval { attribute } => {} FIXME implement
                    _ => {}
                }
            } else if !self.skipping {
                Self::push_line(&mut self.amalgamated, line);
            }
        }
        self.amalgamated.pop();
        Ok(())
    }

    #[inline]
    fn push_line(amalgamated: &mut String, line: &str) {
        amalgamated.reserve(line.len() + 1);
        amalgamated.push_str(line);
        amalgamated.push('\n');
    }

    fn parse_doc_attrib(line: &str) -> Option<(Box<str>, Option<Box<str>>)> {
        // FIXME
        None
    }

    fn check_targets_active(targets: &str, attributes: &AttributeMap) -> bool {
        if let Some(&c) = targets.as_bytes().iter().find(|&&c| c == b'+' || c == b',') {
            (if c == b'+' { std::str::Split::all } else { std::str::Split::any })(
                &mut targets.split(c as char),
                |target| attributes.contains(target),
            )
        } else {
            attributes.contains(targets)
        }
    }

    #[allow(clippy::clippy::toplevel_ref_arg)]
    fn parse_pp_directive(line: &str) -> Option<PreprocessorDirective<'_>> {
        if line.starts_with('[') {
            return None;
        }

        let ref path = |t| preceded(tag(t), take_till1(|c| c == '['));
        let ref path_opt = |t| preceded(tag(t), take_till(|c| c == '['));
        let res = alt((
            map(pair(path("include::"), Self::attr_list), |(target, attributes)| {
                PreprocessorDirective::Include { target, attributes }
            }),
            map(pair(path("ifdef::"), Self::attr_list_ifdef), |(targets, inline)| {
                PreprocessorDirective::IfDef { targets, inline }
            }),
            map(pair(path("ifndef::"), Self::attr_list_ifdef), |(targets, inline)| {
                PreprocessorDirective::IfNotDef { targets, inline }
            }),
            map(pair(path_opt("endif::"), Self::attr_list), |(targets, _)| {
                PreprocessorDirective::EndIf { targets }
            }),
            map(pair(path_opt("ifeval::"), Self::attr_list), |(_, attribute)| {
                PreprocessorDirective::IfEval { attribute }
            }),
        ))(line);
        res.ok().map(|(_, pp)| pp)
    }

    fn attr_list(s: &str) -> IResult<&str, &str, ()> {
        delimited(tag("["), take_till(|c| c == ']'), tag("]"))(s)
    }

    fn attr_list_ifdef(s: &str) -> IResult<&str, Option<&str>, ()> {
        map(
            Self::attr_list,
            |attr_list: &str| if attr_list.is_empty() { None } else { Some(attr_list) },
        )(s)
    }
}

#[derive(Debug)]
enum PreprocessorDirective<'a> {
    Include { target: &'a str, attributes: &'a str },
    IfDef { targets: &'a str, inline: Option<&'a str> },
    IfNotDef { targets: &'a str, inline: Option<&'a str> },
    IfEval { attribute: &'a str },
    EndIf { targets: &'a str },
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::convert::Infallible;
    use std::iter::{self, FromIterator};

    use expect_test::{expect, Expect};

    use super::*;

    fn no_include_cb(_: &AttributeMap, _: &str) -> Result<String, Infallible> {
        panic!("test used include directive which wasn't expected")
    }

    fn check<E: std::fmt::Debug>(
        fixture: &str,
        cb: impl FnMut(&AttributeMap, &str) -> Result<String, E>,
        expect: Expect,
    ) {
        check_with_attributes(fixture, cb, AttributeMap::default(), expect);
    }

    fn check_with_attributes<E: std::fmt::Debug>(
        fixture: &str,
        cb: impl FnMut(&AttributeMap, &str) -> Result<String, E>,
        attributes: AttributeMap,
        expect: Expect,
    ) {
        let mut pp = Preprocessor::new(fixture, cb, attributes);
        pp.amalgamate().unwrap();
        expect.assert_eq(&pp.amalgamated);
    }

    #[test]
    pub fn test_simple() {
        check(
            r#"Demo
================

:some:
:random:
:attributes:
"#,
            no_include_cb,
            expect![[r#"
                Demo
                ================

                :some:
                :random:
                :attributes:
            "#]],
        );
    }

    #[test]
    pub fn test_include() {
        let mut files = HashMap::new();
        files.insert("foo.adoc", ":neeeeerd:");
        files.insert("bar.adoc", ":neeeeeeeeeerd:\n\n");
        files.insert("unsafe-secrets.rs", "unsafe {\n    *std::ptr::null()\n}\n");
        files.insert("empty", "");
        let fixture = r#"Asciidoctor Demo
================
include::foo.adoc[]

include::bar.adoc[]

include::unsafe-secrets.rs[]

include::empty[]
"#;
        check(
            fixture,
            |_: &_, path: &_| -> Result<_, ()> { Ok(files[path].into()) },
            expect![[r#"
                Asciidoctor Demo
                ================
                :neeeeerd:

                :neeeeeeeeeerd:



                unsafe {
                    *std::ptr::null()
                }


            "#]],
        );
    }

    #[test]
    pub fn test_recursive_include() {
        let mut files = HashMap::new();
        files.insert("foo.adoc", "include::bar.adoc[]");
        files.insert("bar.adoc", "bar\ninclude::baz.adoc[]\nbar\n");
        files.insert("baz.adoc", "baz");
        let fixture = r"include::foo.adoc[]";
        check(
            fixture,
            |_: &_, path: &_| -> Result<_, ()> { Ok(files[path].into()) },
            expect![[r#"
                bar
                baz
                bar
            "#]],
        );
    }

    #[test]
    pub fn test_ifdef_inline() {
        let fixture = r"ifdef::foo[This is an inline ifdef]";
        check_with_attributes(
            fixture,
            no_include_cb,
            AttributeMap::from_iter(iter::once(("foo", ""))),
            expect![[r#"This is an inline ifdef"#]],
        );
        check(fixture, no_include_cb, expect![[r#""#]]);
    }

    #[test]
    pub fn test_ifndef_inline() {
        let fixture = r"ifndef::foo[This is an inline ifndef]";
        check_with_attributes(
            fixture,
            no_include_cb,
            AttributeMap::from_iter(iter::once(("foo", ""))),
            expect![[r#""#]],
        );
        check(fixture, no_include_cb, expect![[r#"This is an inline ifndef"#]]);
    }

    #[test]
    pub fn test_ifdef() {
        let fixture = r#"flip the table
ifdef::flip[]
(╯°□°）╯︵ ┻━┻
endif::[]
flip the table
"#;
        check_with_attributes(
            fixture,
            no_include_cb,
            AttributeMap::from_iter(iter::once(("flip", ""))),
            expect![[r#"
                flip the table
                (╯°□°）╯︵ ┻━┻
                flip the table
            "#]],
        );
        check(
            fixture,
            no_include_cb,
            expect![[r#"
                flip the table
                flip the table
            "#]],
        );
    }

    #[test]
    pub fn test_ifndef() {
        let fixture = r#"unflip the table
ifndef::unflip[]
┬─┬ ノ( ゜-゜ノ)
endif::[]
unflip the table
"#;
        check_with_attributes(
            fixture,
            no_include_cb,
            AttributeMap::from_iter(iter::once(("unflip", ""))),
            expect![[r#"
                unflip the table
                unflip the table
            "#]],
        );
        check(
            fixture,
            no_include_cb,
            expect![[r#"
                unflip the table
                ┬─┬ ノ( ゜-゜ノ)
                unflip the table
            "#]],
        );
    }

    #[test]
    pub fn test_ifdef_and() {
        let fixture = r#"Flip Flappers is a
ifdef::flip+flap[]
nice
endif::[]
show
"#;
        check_with_attributes(
            fixture,
            no_include_cb,
            AttributeMap::from_iter(iter::once(("flip", ""))),
            expect![[r#"
                Flip Flappers is a
                show
            "#]],
        );
        check_with_attributes(
            fixture,
            no_include_cb,
            AttributeMap::from_iter(vec![("flip", ""), ("flap", "")]),
            expect![[r#"
                Flip Flappers is a
                nice
                show
            "#]],
        );
    }

    #[test]
    pub fn test_ifdef_or() {
        let fixture = r#"Wonder
ifdef::flip,flap[]
Egg
endif::[]
Priority
"#;
        check(
            fixture,
            no_include_cb,
            expect![[r#"
                Wonder
                Priority
            "#]],
        );
        check_with_attributes(
            fixture,
            no_include_cb,
            AttributeMap::from_iter(iter::once(("flip", ""))),
            expect![[r#"
                Wonder
                Egg
                Priority
            "#]],
        );
        check_with_attributes(
            fixture,
            no_include_cb,
            AttributeMap::from_iter(vec![("flip", ""), ("flap", "")]),
            expect![[r#"
                Wonder
                Egg
                Priority
            "#]],
        );
    }

    #[test]
    pub fn test_ifdef_nested() {
        let fixture = r#"ifdef::flip[]
Flip
ifdef::flap[]
Flap
endif::[]
Flop
endif::[]
"#;
        check(fixture, no_include_cb, expect![[r#""#]]);
        check_with_attributes(
            fixture,
            no_include_cb,
            AttributeMap::from_iter(iter::once(("flip", ""))),
            expect![[r#"
                Flip
                Flop
            "#]],
        );
        check_with_attributes(
            fixture,
            no_include_cb,
            AttributeMap::from_iter(iter::once(("flap", ""))),
            expect![[r#""#]],
        );
        check_with_attributes(
            fixture,
            no_include_cb,
            AttributeMap::from_iter(vec![("flip", ""), ("flap", "")]),
            expect![[r#"
                Flip
                Flap
                Flop
            "#]],
        );
    }
}
