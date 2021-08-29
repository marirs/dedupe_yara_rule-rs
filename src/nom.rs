use super::YarObj;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char, multispace0, multispace1, none_of},
    combinator::{map, opt, recognize, rest},
    multi::{many0, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

fn simple_block(i: &str) -> IResult<&str, &str> {
    recognize(terminated(
        many0(preceded(
            many0(none_of("{}\\")),
            alt((tag("\\{"), tag("\\}"), tag("\\"))),
        )),
        many0(none_of("{}\\")),
    ))(i)
}

fn curly_brace_block(i: &str) -> IResult<&str, &str> {
    alt((
        recognize(delimited(char('{'), simple_block, char('}'))),
        recognize(delimited(
            pair(char('{'), simple_block),
            separated_list1(curly_brace_block, simple_block),
            pair(simple_block, char('}')),
        )),
    ))(i)
}

fn parse_tuple(i: &str) -> IResult<&str, YarObj> {
    map(
        tuple((
            preceded(
                pair(tag("rule"), multispace1),
                recognize(many0(none_of(":{"))),
            ),
            opt(preceded(
                delimited(multispace0, char(':'), multispace0),
                recognize(many0(none_of("{"))),
            )),
            preceded(multispace0, curly_brace_block),
        )),
        |(name, tag, block)| YarObj {
            name: name.into(),
            tag: tag.map(|x| format!(" : {}", x)).unwrap_or_default(),
            block: block.into(),
        },
    )(i)
}

pub fn parse_vec(i: &str) -> IResult<&str, Vec<YarObj>> {
    many0(preceded(alt((take_until("rule"), rest)), parse_tuple))(i)
}
