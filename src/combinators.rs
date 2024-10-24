use crate::parser::Parser;

pub fn map<'a, P, F, A, B>(parser: P, map_fn: F) -> impl Parser<'a, B>
where
    P: Parser<'a, A>,
    F: Fn(A) -> B,
{
    move |input| {
        parser
            .parse(input)
            .map(|(result, next_input)| (map_fn(result), next_input))
    }
}

pub fn and_then<'a, P, F, A, B, NextP>(parser: P, f: F) -> impl Parser<'a, B>
where
    P: Parser<'a, A>,
    NextP: Parser<'a, B>,
    F: Fn(A) -> NextP,
{
    // move |input| match parser.parse(input) {
    //     Ok((result, next_input)) => f(result).parse(next_input),
    //     Err(err) => Err(err),
    // }
    move |input| parser.parse(input).and_then(|(result, next_input)| f(result).parse(next_input))

}


pub fn pred<'a, P, A, F>(parser: P, pred_fn: F) -> impl Parser<'a, A>
where
    P: Parser<'a, A>,
    F: Fn(&A) -> bool,
{
    move |input: &'a str| {
        if let Ok((result, next_input)) = parser.parse(input) {
            if pred_fn(&result) {
                return Ok((result, next_input));
            }
        }

        Err(input)
    }
}

pub fn pair<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, (R1, R2)>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>,
{
    move |input| {
        parser1.parse(input).and_then(|(result1, next_input)| {
            parser2
                .parse(next_input)
                .map(|(result2, final_input)| ((result1, result2), final_input))
        })
    }
}

pub fn left<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R1>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>,
{
    map(pair(parser1, parser2), |(left, _right)| left)
}

pub fn right<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R2>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>,
{
    map(pair(parser1, parser2), |(_left, right)| right)
}

pub fn one_or_more<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
where
    P: Parser<'a, A>,
{
    move |mut input| {
        let mut results = Vec::new();

        if let Ok((first_result, next_input)) = parser.parse(input) {
            input = next_input;
            results.push(first_result);
        } else {
            return Err(input);
        }

        while let Ok((next_result, next_input)) = parser.parse(input) {
            input = next_input;
            results.push(next_result);
        }

        Ok((results, input))
    }
}

pub fn zero_or_more<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
where
    P: Parser<'a, A>,
{
    move |mut input| {
        let mut results = Vec::new();

        while let Ok((next_result, next_input)) = parser.parse(input) {
            input = next_input;
            results.push(next_result);
        }

        Ok((results, input))
    }
}

pub fn either<'a, P1, P2, A>(parser1: P1, parser2: P2) -> impl Parser<'a, A>
where
    P1: Parser<'a, A>,
    P2: Parser<'a, A>,
{
    move |input| match parser1.parse(input) {
        ok @ Ok(_) => ok,
        Err(_) => parser2.parse(input),
    }
}
