// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Author: Alexander Seifarth
use nom;

/// Return always an Ok with either a Some(O) or a None depending on whether the parser succeeds
/// or not
pub fn option<I, O, E: nom::error::ParseError<I>, F>( mut parser: F ) -> impl FnMut(I) -> nom::IResult<I, Option<O>, E>
    where F: FnMut(I) -> nom::IResult<I, O, E>, I: Clone
{
    move |input: I| {
        match parser(input.clone()) {
            Ok((rem, value)) => Ok((rem, Some(value))),
            Err(_) => Ok((input, None))
        }
    }
}

pub fn keyword(kwrd: &str) -> impl FnMut(&str) -> nom::IResult<&str, &str>
{
    use nom::combinator::verify;
    use nom::character::complete::alpha1;

    let kwdr_str = kwrd.to_string();
    move |i: &str| {
        let k = kwdr_str.clone();
        verify(alpha1,move |s| { s == k })(i)
    }
}

#[cfg(test)]
mod test {
    use crate::util::keyword;

    #[test]
    fn test_keyword() {
        assert_eq!(keyword("identifier")("identifier _ax"), Ok((" _ax", "identifier")));
        assert!(keyword("module")("moduleA").is_err());
    }
}