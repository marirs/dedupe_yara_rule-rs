use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char, multispace0, multispace1, none_of, not_line_ending, line_ending, anychar, one_of, digit1},
    combinator::{map, opt, recognize, rest, map_res},
    multi::{many0, separated_list1, fold_many0, many1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,

};

fn comments(i: &str) -> IResult<&str, ()>{
    let res  = alt((
        tuple((
            tag("/*"),
            take_until("*/"),
            tag("*/")
        )),
        tuple((
            tag("//"),
            not_line_ending,
            line_ending
        ))
    ))(i)?;
    Ok((res.0, ()))
}

fn whitespace0(i: &str) -> IResult<&str, ()>{
    let res = tuple((
        multispace0,
        many0(pair(comments, multispace0)),
        multispace0,
    ))(i)?;
    Ok((res.0, ()))
}

fn whitespace1(i: &str) -> IResult<&str, ()>{
    let res = alt((
        tuple((
            multispace1,
            many0(pair(comments, multispace0)),
            multispace0,
        )),
        tuple((
            multispace0,
            many0(pair(comments, multispace0)),
            multispace1,
        ))
    ))(i)?;
    Ok((res.0, ()))
}

fn character(input: &str) -> IResult<&str, String> {
    let (input, c) = none_of("\"")(input)?;
    if c == '\\' {
        map(anychar, |c| format!("\\{}", c))(input)
    } else {
        Ok((input, format!("{}", c)))
    }
}

fn regex_character(input: &str) -> IResult<&str, String> {
    let (input, c) = none_of("/")(input)?;
    if c == '\\' {
        map(anychar, |c| format!("\\{}", c))(input)
    } else {
        Ok((input, format!("{}", c)))
    }
}

fn string(input: &str) -> IResult<&str, String> {
    delimited(
        char('"'),
        fold_many0(character, String::new, |mut string, c| {
            string.extend(c.chars());
            string
        }),
        char('"'),
    )(input)
}

fn hexdecimal_string(input: &str) -> IResult<&str, String> {
    delimited(
        char('{'),
        map(take_until("}"), |t: &str| t.to_string()),
        char('}'),
    )(input)
}

fn regex(input: &str) -> IResult<&str, String> {
    delimited(
        char('/'),
        fold_many0(regex_character, String::new, |mut string, c| {
            string.extend(c.chars());
            string
        }),
        char('/'),
    )(input)
}

fn name(input: &str) -> IResult<&str, String> {
    let res = pair(
        one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_"),
        many0(one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_1234567890"))
    )(input)?;
    Ok((res.0, format!("{}{}", res.1.0, res.1.1.iter().collect::<String>())))
}

fn string_name(input: &str) -> IResult<&str, String> {
    let res = pair(
        tag("$"),
        many0(one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_1234567890"))
    )(input)?;
    Ok((res.0, format!("${}", res.1.1.iter().collect::<String>())))
}

fn number(input: &str) -> IResult<&str, i64> {
    alt((
        map_res(pair(tag("0x"), many0(one_of("0123456789abcdefABCDEF"))), |(_, d)| i64::from_str_radix(&d.iter().collect::<String>(), 16)),
        map_res(digit1, |d| i64::from_str_radix(d, 10))
    ))(input)
}

fn import(i: &str) -> IResult<&str, crate::YarImport>{
    let res = tuple((
        whitespace0,
        tag("import"),
        whitespace1,
        string
    ))(i)?;
    Ok((res.0, crate::YarImport{value: res.1.3}))
}

fn include(i: &str) -> IResult<&str, crate::YarInclude>{
    let res = tuple((
        whitespace0,
        tag("include"),
        whitespace1,
        string
    ))(i)?;
    Ok((res.0, crate::YarInclude{value: res.1.3}))
}

fn condition(i: &str) -> IResult<&str, crate::YarRuleConditionNode>{
    let res = take_until("}")(i)?;
    Ok((res.0, crate::YarRuleConditionNode::None(res.1.to_string())))
}

fn body(i: &str) -> IResult<&str, crate::YarRuleBody>{
    let res = tuple((
        opt(tuple((
            whitespace0,
            tag("meta"),
            whitespace0,
            tag(":"),
            many1(tuple((
                whitespace0,
                name,
                whitespace0,
                tag("="),
                whitespace0,
                alt((
                    map(number, |n| format!("{:02x}", n)),
                    string
                ))
            )))
        ))),
        opt(tuple((
            whitespace0,
            tag("strings"),
            whitespace0,
            tag(":"),
            many1(tuple((
                whitespace0,
                name,
                whitespace0,
                tag("="),
                whitespace0,
                alt((
                    hexdecimal_string,
                    string,
                    regex
                ))
            )))
        ))),
        whitespace0,
        tag("condition"),
        whitespace0,
        tag(":"),
        condition
    ))(i)?;
    let mut b = crate::YarRuleBody{
        meta: std::collections::HashMap::new(),
        strings: vec![],
        condition: res.1.6
    };
    if let Some((_, _, _, _, mm)) = res.1.0{
        for (_, n, _, _, _, m) in mm{
            b.meta.insert(n, m);
        }
    }
    if let Some((_, _, _, _, mm)) = res.1.1{
        for (_, n, _, _, _, m) in mm{
            b.strings.push((n, m));
        }
    }
    Ok((res.0, b))
}

pub fn rule(i: &str) -> IResult<&str, crate::YarRule>{
    let res = tuple((
        whitespace0,
        opt(pair(tag("private"), whitespace1)),
        opt(pair(tag("global"), whitespace1)),
        name,
        opt(tuple((
            whitespace0,
            tag(":"),
            whitespace0,
            name,
            many0(pair(whitespace1, name))
        ))),
        whitespace0,
        tag("{"),
        body,
        tag("}")
    ))(i)?;
    let mut r = crate::YarRule {
        private: if let Some(_) = res.1.1{true} else {false},
        name: res.1.3,
        tags: vec![],
        body: res.1.7,
        refs: std::collections::HashSet::new()
    };
    if let Some((_, _, _, t, tt)) = res.1.4{
        r.tags.push(t);
        for (_, ttt) in tt{
            r.tags.push(ttt);
        }
    }
    Ok((res.0, r))
}

pub fn parse_rules(i: &str) -> IResult<&str, crate::YarRuleSet>{
    let res = tuple((
        many0(preceded(whitespace0, import)),
        many0(preceded(whitespace0, include)),
        many1(preceded(whitespace0, rule)),
    ))(i)?;
//    let mut rules = std::collections::HashMap::new();
//    let mut imports = vec![];
//    let mut includes = vec![];

    Ok((res.0, crate::YarRuleSet{
        imports: res.1.0,
        includes: res.1.1,
        rules: res.1.2.into_iter().map(|r| (r.name.clone(), r)).collect()
    }))
}
