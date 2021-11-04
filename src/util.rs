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