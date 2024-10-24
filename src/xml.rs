use crate::parser::{Parser, ParseResult};
use crate::combinators::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<Element>,
}

pub fn any_char(input: &str) -> ParseResult<'_, char> {
    match input.chars().next() {
        Some(next) => Ok((next, &input[next.len_utf8()..])),
        _ => Err(input),
    }
}

pub fn match_literal<'a>(expected: &'static str) -> impl Parser<'a, ()> {
    move |input: &'a str| {
        if let Some(rest) = input.strip_prefix(expected) {
            Ok(((), rest))
        } else {
            Err(input)
        }
    }
}

pub fn whitespace_char<'a>() -> impl Parser<'a, char> {
    pred(any_char, |c| c.is_whitespace())
}

pub fn space1<'a>() -> impl Parser<'a, Vec<char>> {
    one_or_more(whitespace_char())
}

pub fn space0<'a>() -> impl Parser<'a, Vec<char>> {
    zero_or_more(whitespace_char())
}

pub fn quoted_string1<'a>() -> impl Parser<'a, String> {
    right(
        match_literal("\""),
        left(
            zero_or_more(any_char.pred(|c| *c != '"')),
            match_literal("\""),
        ),
    )
    .map(|chars| chars.into_iter().collect())
}

pub fn self_closing_element<'a>() -> impl Parser<'a, Element> {
    whitespace_wrap(left(element_start(), left(space0(), match_literal("/>"))).map(|(name, attributes)| Element {
        name,
        attributes,
        children: vec![],
    }))
}

pub fn match_char<'a>(ch: char) -> impl Parser<'a, ()> {
    move |input: &'a str| match input.chars().next() {
        Some(c) if c == ch => Ok(((), &input[ch.len_utf8()..])),
        _ => Err(input),
    }
}

pub fn identifier(input: &str) -> ParseResult<String> {
    let mut matched = String::new();
    let mut chars = input.chars();

    match chars.next() {
        Some(next) if next.is_alphabetic() => matched.push(next),
        _ => return Err(input),
    }

    for ch in chars {
        if ch.is_alphanumeric() || ch == '-' {
            matched.push(ch);
        } else {
            break;
        }
    }

    let next_index = matched.len();
    Ok((matched, &input[next_index..]))
}

pub fn quoted_string<'a>() -> impl Parser<'a, String> {
    map(
        right(
            match_literal("\""),
            left(
                zero_or_more(pred(any_char, |c| *c != '"')),
                match_literal("\""),
            ),
        ),
        |chars| chars.into_iter().collect(),
    )
}

pub fn attribute_pair<'a>() -> impl Parser<'a, (String, String)> {
    pair(identifier, right(match_literal("="), quoted_string()))
}

pub fn attributes<'a>() -> impl Parser<'a, Vec<(String, String)>> {
    zero_or_more(right(space1(), attribute_pair()))
}

pub fn element_start<'a>() -> impl Parser<'a, (String, Vec<(String, String)>)> {
    right(match_literal("<"), pair(identifier, attributes()))
}

pub fn open_element<'a>() -> impl Parser<'a, Element> {
    whitespace_wrap(left(element_start(), left(space0(), match_literal(">"))).map(|(name, attributes)| Element {
        name,
        attributes,
        children: vec![],
    }))
}

pub fn single_element<'a>() -> impl Parser<'a, Element> {
    whitespace_wrap(either(self_closing_element(), open_element()))
}

pub fn close_element<'a>(expected_name: String) -> impl Parser<'a, String> {
    whitespace_wrap(right(match_literal("</"), left(identifier, match_literal(">")))
        .pred(move |name| name == &expected_name))
}

pub fn parent_element<'a>() -> impl Parser<'a, Element> {
    whitespace_wrap(open_element().and_then(|element| {
        left(
            zero_or_more(single_element()),
            close_element(element.name.clone()),
        )
        .map(move |children| {
            let mut element = element.clone();
            element.children = children;
            element
        })
    }))
}

pub fn whitespace_wrap<'a, P, A>(parser: P) -> impl Parser<'a, A> 
where
    P: Parser<'a, A>
{
    right(space0(), left(parser, space0()))
}

pub fn element<'a>() -> impl Parser<'a, Element> {
    either(single_element(), parent_element())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal() {
        let parse_foo = match_literal("foo");
        assert_eq!(Ok(((), "")), parse_foo.parse("foo"));
        assert_eq!(Ok(((), " bar")), parse_foo.parse("foo bar"));
        assert_eq!(Err("baz"), parse_foo.parse("baz"));
    }

    #[test]
    fn pair_comb() {
        let open_tag = pair(match_literal("<"), identifier);
        assert_eq!(
            Ok((((), "first-element".to_owned()), "/>")),
            open_tag.parse("<first-element/>"),
        );
        assert_eq!(Err("oops"), open_tag.parse("oops"));
        assert_eq!(Err("!oops"), open_tag.parse("<!oops"));
    }

    #[test]
    fn right_comb() {
        let open_tag = right(match_literal("<"), identifier);

        assert_eq!(
            Ok(("first-element".to_owned(), "/>")),
            open_tag.parse("<first-element/>")
        );
        assert_eq!(Err("oops"), open_tag.parse("oops"));
        assert_eq!(Err("!oops"), open_tag.parse("<!oops"));
    }

    #[test]
    fn pred_comb() {
        let parser = pred(any_char, |c| *c == 'o');
        assert_eq!(Ok(('o', "mg")), parser.parse("omg"));
    }

    #[test]
    fn quoted_string_comb() {
        let parser = quoted_string();

        assert_eq!(
            Ok(("Hello World!".to_owned(), "")),
            parser.parse("\"Hello World!\"")
        );
        assert_eq!(Err(""), parser.parse("\"Hello World!"));
        assert_eq!(Err("Hello World!\""), parser.parse("Hello World!\""));
    }

    #[test]
    fn attributes_comb() {
        let parser = attributes();

        assert_eq!(
            Ok((
                vec![
                    ("hello".to_owned(), "world".to_owned()),
                    ("joe".to_owned(), "mama".to_owned())
                ],
                "",
            )),
            parser.parse(" hello=\"world\" joe=\"mama\"")
        );
    }

    #[test]
    fn single_element_comb() {
        let parser = self_closing_element();

        assert_eq!(
            Ok((
                Element {
                    name: "div".to_owned(),
                    attributes: vec![
                        ("id".to_owned(), "container".to_owned()),
                        ("class".to_owned(), "h-full w-full mb-4".to_owned()),
                    ],
                    children: vec![],
                },
                "",
            )),
            parser.parse(r#"<div id="container" class="h-full w-full mb-4" />"#),
        );
    }

    #[test]
    fn open_element_comb() {
        let parser = open_element();

        assert_eq!(
            Ok((
                Element {
                    name: "div".to_owned(),
                    attributes: vec![
                        ("id".to_owned(), "container".to_owned()),
                        ("class".to_owned(), "h-full w-full mb-4".to_owned()),
                    ],
                    children: vec![],
                },
                "",
            )),
            parser.parse(r#"<div id="container" class="h-full w-full mb-4" >"#),
        );
    }

    #[test]
    fn element_comb() {
        let parser = parent_element();

        assert_eq!(
            Ok((
                Element {
                    name: "div".to_owned(),
                    attributes: vec![
                        ("id".to_owned(), "1".to_owned()),
                        ("class".to_owned(), "1".to_owned()),
                    ],
                    children: vec![Element {
                        name: "pre".to_owned(),
                        attributes: vec![("id".to_owned(), "2".to_owned()),],
                        children: vec![],
                    }],
                },
                "",
            )),
            parser.parse(
            r#"<div id="1" class="1" > <pre id="2" /> </div> "#
            ),
        );
    }

    #[test]
    fn xml_parser() {
        let doc = r#"
            <top label="Top">
                <semi-bottom label="Bottom"/>
                <middle>
                    <bottom label="Another bottom"/>
                </middle>
            </top>"#;
        let parsed_doc = Element {
            name: "top".to_string(),
            attributes: vec![("label".to_string(), "Top".to_string())],
            children: vec![
                Element {
                    name: "semi-bottom".to_string(),
                    attributes: vec![("label".to_string(), "Bottom".to_string())],
                    children: vec![],
                },
                Element {
                    name: "middle".to_string(),
                    attributes: vec![],
                    children: vec![Element {
                        name: "bottom".to_string(),
                        attributes: vec![("label".to_string(), "Another bottom".to_string())],
                        children: vec![],
                    }],
                },
            ],
        };
        assert_eq!(Ok((parsed_doc, "")), parent_element().parse(doc));
    }
}

