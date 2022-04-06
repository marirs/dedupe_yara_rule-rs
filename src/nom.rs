use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, tag_no_case},
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

fn modifier(i: &str) -> IResult<&str, &str>{
    alt((
        tag("nocase"),
        tag("wide"),
        tag("ascii"),
        tag("fullword")
    ))(i)
}

fn string(input: &str) -> IResult<&str, String> {
    let (ii, ss) = delimited(
        char('"'),
        fold_many0(character, String::new, |mut string, c| {
            string.extend(c.chars());
            string
        }),
        char('"'),
    )(input)?;
    let (input, modifiers) = many0(preceded(whitespace0, modifier))(ii)?;
    Ok((input, format!("\"{}\" {}", ss, modifiers.iter().map(|s| s.to_string()).collect::<Vec<String>>().join(" "))))
}

fn hexdecimal_string(input: &str) -> IResult<&str, String> {
    delimited(
        char('{'),
        map(take_until("}"), |t: &str| t.to_string()),
        char('}'),
    )(input)
}

fn regex(input: &str) -> IResult<&str, String> {
    let (ii, ss) = delimited(
        char('/'),
        fold_many0(regex_character, String::new, |mut string, c| {
            string.extend(c.chars());
            string
        }),
        char('/'),
    )(input)?;
    let (input, modifiers) = many0(preceded(whitespace0, modifier))(ii)?;
    Ok((input, format!("/{}/ {}", ss, modifiers.iter().map(|s| s.to_string()).collect::<Vec<String>>().join(" "))))
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

fn boolean(input: &str) -> IResult<&str, bool> {
    alt((
        map(tag_no_case("true"), |_| true),
        map(tag_no_case("false"), |_| false),
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

fn parens(i: &str) -> IResult<&str, crate::YarRuleConditionNode> {
    preceded(whitespace0, delimited(tag("("), delimited(whitespace0, condition, whitespace0), tag(")")))(i)
}

fn bytes_with_offset(i: &str) -> IResult<&str, String>{
    let res = tuple((
        alt((
            tag("int8"),
            tag("int8be"),
            tag("int16"),
            tag("int16be"),
            tag("int32"),
            tag("int32be"),
            tag("uint8"),
            tag("uint8be"),
            tag("uint16"),
            tag("uint16be"),
            tag("uint32"),
            tag("uint32be"),
        )),
        whitespace0,
        tag("("),
        whitespace0,
        alt((
            map(number, |n| format!("{}", n)),
            bytes_with_offset
        )),
        whitespace0,
        tag(")")
    ))(i)?;
    Ok((res.0, format!("{}({})", res.1.0, res.1.4)))
}

fn literal(i: &str) -> IResult<&str, crate::YarRuleConditionNode> {
    preceded(whitespace0,
             alt((
                 map(alt((
                     tag("1"),
                     tag("all"),
                     tag("them")
                 )), |m: &str| crate::YarRuleConditionNode::Reserved(m.to_string())),
                 map(bytes_with_offset, |m| crate::YarRuleConditionNode::BytesWithOffset(m)),
                 map(pair(string_name, tag("*")), |(m, _)| crate::YarRuleConditionNode::StringRef(format!("{}*", m))),
                 map(string_name, |m| crate::YarRuleConditionNode::StringRef(m)),
                 parens,
             )))(i)
}

fn condition(i: &str) -> IResult<&str, crate::YarRuleConditionNode>{
    let (i, l) = literal(i)?;
    fold_many0(
        preceded(whitespace1, pair(alt((tag("and"),
                                        tag("or"),
                                        tag("of"))), preceded(whitespace1, literal))),
        move || l.clone(),
        |acc, (op, val): (&str, crate::YarRuleConditionNode)| {
            match op{
                "and" => crate::YarRuleConditionNode::And(Box::new(acc), Box::new(val)),
                "or" => crate::YarRuleConditionNode::Or(Box::new(acc), Box::new(val)),
                "of" => crate::YarRuleConditionNode::Of(Box::new(acc), Box::new(val)),
                _ => crate::YarRuleConditionNode::None("".to_string())
            }
        },
    )(i)

//    let res = take_until("}")(i)?;
//    Ok((res.0, crate::YarRuleConditionNode::None(res.1.to_string())))
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
                    map(boolean, |b| format!("{}", b)),
                    string,
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
                string_name,
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
        whitespace0,
        tag("rule"),
        whitespace1,
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
        whitespace0,
        tag("}")
    ))(i)?;
    let mut r = crate::YarRule {
        private: if let Some(_) = res.1.1{true} else {false},
        name: res.1.6,
        tags: vec![],
        body: res.1.10,
        refs: std::collections::HashSet::new()
    };
    if let Some((_, _, _, t, tt)) = res.1.7{
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
    ))(i);

    let res = match res{
        Ok(s) => {
            s
        },
        Err(e) => {
            println!("{:#?}", e);
            return Err(e);
        }
    };

    Ok((res.0, crate::YarRuleSet{
        imports: res.1.0,
        includes: res.1.1,
        rules: res.1.2.into_iter().map(|r| (r.name.clone(), r)).collect()
    }))
}
