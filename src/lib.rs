pub type Result<T> = std::result::Result<(String, T), String>;

pub fn letter_a(input: String) -> Result<()> {
    match input.chars().next() {
        Some('a') => Ok((input['a'.len_utf8()..].to_owned(), ())),
        _ => Err(input),
    }
}

pub fn match_literal(expected: &'static str) -> impl Fn(String) -> Result<()> {
    move |input| {
        if input.starts_with(expected) {
            Ok((input[expected.len()..].to_owned(), ()))
        } else {
            Err(input)
        }
    }
}

pub fn identifier(input: String) -> Result<String> {
    let mut matched = String::new();
    let mut chars = input.chars();

    match chars.next() {
        Some(next) if next.is_alphabetic() => matched.push(next),
        _ => return Err(input),
    }

    while let Some(next) = chars.next() {
        if next.is_alphanumeric() || next == '-' {
            matched.push(next);
        } else {
            break;
        }
    }

    let next_index = matched.len();

    Ok((input[next_index..].to_owned(), matched))
}

fn pair<P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Fn(String) -> Result<(R1, R2)>
where
    P1: Fn(String) -> Result<R1>,
    P2: Fn(String) -> Result<R2>,
{
    // move |input| match parser1(input) {
    //     Ok((next_input, result1)) => match parser2(next_input) {
    //         Ok((final_input, result2)) => Ok((final_input, (result1, result2))),
    //         Err(err) => Err(err),
    //     },
    //     Err(err) => Err(err),
    // }

    move |input| {
        parser1(input).and_then(|(next_input, result1)| {
            parser2(next_input).map(|(final_input, result2)| (final_input, (result1, result2)))
        })
    }
}

fn map<P, F, A, B>(parser: P, map_fn: F) -> impl Fn(String) -> Result<B>
where
    P: Fn(String) -> Result<A>,
    F: Fn(A) -> B,
{
    // move |input| match parser(input) {
    //     Ok((next_input, result)) -> Ok((next_input, map_fn(result))),
    //     Err(err) => Err(err),
    // }

    move |input| parser(input).map(|(next_input, result)| (next_input, map_fn(result)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal() {
        let parse_foo = match_literal("foo");
        assert_eq!(Ok(("".to_owned(), ())), parse_foo("foo".to_owned()));
        assert_eq!(Ok((" bar".to_owned(), ())), parse_foo("foo bar".to_owned()));
        assert_eq!(Err("baz".to_owned()), parse_foo("baz".to_owned()));
    }

    #[test]
    fn pair_test() {
        let open_tag = pair(match_literal("<"), identifier);
        assert_eq!(
            Ok(("/>".to_owned(), ((), "first-element".to_string()))),
            open_tag("<first-element/>".to_owned())
        );
        assert_eq!(Err("oops".to_owned()), open_tag("oops".to_owned()));
        assert_eq!(Err("!oops".to_owned()), open_tag("<!oops".to_owned()));
    }
}
