use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, tag_no_case},
    character::complete::{char, multispace0, multispace1, none_of, not_line_ending, line_ending, anychar, one_of, digit1},
    combinator::{map, opt, map_res},
    multi::{many0, fold_many0, many1},
    sequence::{delimited, pair, preceded, tuple},
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

fn modifier(i: &str) -> IResult<&str, String>{
    alt((
        map(tag("nocase"), |s: &str| s.to_string()),
        map(tag("wide"), |s: &str| s.to_string()),
        map(tag("ascii"), |s: &str| s.to_string()),
        map(tag("fullword"), |s: &str| s.to_string()),
        map(tuple((
            tag("xor"),
            opt(tuple((
                whitespace0,
                tag("("),
                whitespace0,
                number,
                whitespace0,
                tag("-"),
                whitespace0,
                number,
                whitespace0,
                tag(")")
            )))
        )), |(_, s)| format!("xor{}", if let Some((_, _, _, a, _, _, _, b, _, _)) = s { format!("({}-{})", a, b) }else{"".to_string()}))
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
    Ok((input, format!("\"{}\"{}", ss, if !modifiers.is_empty(){ format!(" {}", modifiers.iter().map(|s| s.to_string()).collect::<Vec<String>>().join(" "))}else{"".to_string()})))
}

fn hexdecimal_string(input: &str) -> IResult<&str, String> {
    let res = delimited(
        char('{'),
        map(take_until("}"), |t: &str| t.to_string()),
        char('}'),
    )(input)?;
    Ok((res.0, format!("{{{}}}", res.1)))
}

fn regex(input: &str) -> IResult<&str, String> {
    let (ii, (ss, sss)) = pair(
        delimited(
            char('/'),
            fold_many0(regex_character, String::new, |mut string, c| {
                string.extend(c.chars());
                string
            }),
            char('/'),
        ),
        many0(one_of("is"))
    )(input)?;
    let (input, modifiers) = many0(preceded(whitespace1, modifier))(ii)?;
    Ok((input, format!("/{}/{} {}", ss, sss.iter().collect::<String>(), modifiers.iter().map(|s| s.to_string()).collect::<Vec<String>>().join(" "))))
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

fn string_count(input: &str) -> IResult<&str, String> {
    let res = pair(
        tag("#"),
        many0(one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_1234567890"))
    )(input)?;
    Ok((res.0, format!("#{}", res.1.1.iter().collect::<String>())))
}

fn string_count2(input: &str) -> IResult<&str, String> {
    let res = tuple((
        tag("@"),
        many0(one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_1234567890")),
        opt(
            tuple((
                tag("["),
                whitespace0,
                condition,
                whitespace0,
                tag("]"),
            ))
        )
    ))(input)?;
    Ok((res.0, format!("@{}{}", res.1.1.iter().collect::<String>(), if let Some((_, _, rr, _, _)) = res.1.2{
        format!("[{}]", rr)
    }else{"".to_string()})))
}

fn import_ref(input: &str) -> IResult<&str, String> {
    let res = tuple((
        many1(one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_1234567890")),
        many1(tuple((
            whitespace0,
            tag("."),
            whitespace0,
            many1(one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_1234567890")),
            opt(delimited(
                tuple((
                    whitespace0,
                    tag("("),
                    whitespace0
                )),
                opt(
                    pair(
                        preceded(
                            whitespace0,
                            condition,
                        ),
                        many0(
                            preceded(
                                tuple((
                                    whitespace0,
                                    tag(","),
                                    whitespace0
                                )),
                                condition
                            ))
                    ),
                ),
                pair(
                    whitespace0,
                    tag(")"),
                )
            )),
            opt(
                delimited(
                    tuple((
                        whitespace0,
                        tag("["),
                        whitespace0,
                    )),
                    condition,
                    pair(
                        whitespace0,
                        tag("]")
                    )
                )
            )
        )))
    ))(input)?;
    Ok((res.0, format!("{}{}", res.1.0.iter().collect::<String>(), res.1.1.iter().map(|(_, _, _, n, pa, ma)|{
        format!(".{}{}{}", n.iter().collect::<String>(), if let Some(Some((ppa, pppa))) = pa{
            format!("({}{}{})", ppa, if pppa.len() >0 {", "} else {""}, pppa.iter().map(|ppp| ppp.to_string()).collect::<Vec<String>>().join(", "))
        }else if let Some(None) = pa{
            "()".to_string()
        }else {
            "".to_string()
        }, if let Some(mma) = ma{
            format!("[{}]", mma)
        }else{
            "".to_string()
        })
    }).collect::<Vec<String>>().join(""))))
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
    Ok((res.0, crate::YarInclude{value: res.1.3[1..res.1.3.len()-1].to_string()}))
}

fn parens(i: &str) -> IResult<&str, crate::YarRuleConditionNode> {
    preceded(whitespace0, delimited(tag("("), delimited(whitespace0, condition, whitespace0), tag(")")))(i)
}

fn bytes_with_offset(i: &str) -> IResult<&str, crate::YarRuleConditionNode>{
    let res = tuple((
        whitespace0,
        alt((
            tag("int8be"),
            tag("int8"),
            tag("int16be"),
            tag("int16"),
            tag("int32be"),
            tag("int32"),
            tag("uint8be"),
            tag("uint8"),
            tag("uint16be"),
            tag("uint16"),
            tag("uint32be"),
            tag("uint32"),
        )),
        whitespace0,
        tag("("),
        whitespace0,
        condition,
        whitespace0,
        tag(")")
    ))(i)?;
    Ok((res.0, crate::YarRuleConditionNode::BytesWithOffset(res.1.1.to_string(), Box::new(res.1.5))))
}

fn size(i: &str) -> IResult<&str, usize> {
    let res = pair(map_res(digit1, |s| usize::from_str_radix(s, 10)),
         alt((tag("KB"),
              tag("MB")
         )))(i)?;
    Ok((res.0, res.1.0 * if res.1.1 == "KB" {1024} else {1024*1024}))
}

fn range(i: &str) -> IResult<&str, crate::YarRuleConditionNode> {
    let res = tuple((
        tag("("),
        whitespace0,
        condition,
        whitespace0,
        tag(".."),
        whitespace0,
        condition,
        whitespace0,
        tag(")")
    ))(i)?;
    Ok((res.0, crate::YarRuleConditionNode::Range(Box::new(res.1.2), Box::new(res.1.6))))
}

fn set(i: &str) -> IResult<&str, crate::YarRuleConditionNode> {
    let res = tuple((
        tag("("),
        whitespace0,
        condition,
        many1(preceded(
            tuple((whitespace0, tag(","), whitespace0)),
            condition
        )),
        whitespace0,
        tag(")")
    ))(i)?;
    let mut aa = vec![];
    for ff in res.1.3{
        aa.push(Box::new(ff));
    }
    Ok((res.0, crate::YarRuleConditionNode::Set(aa)))
}

fn not(i: &str) -> IResult<&str, crate::YarRuleConditionNode> {
    let res = tuple((
        tag("not"),
        whitespace1,
        condition
    ))(i)?;
    Ok((res.0, crate::YarRuleConditionNode::Not(Box::new(res.1.2))))
}

fn for_of(i: &str) -> IResult<&str, crate::YarRuleConditionNode>{
    let res = tuple((
        whitespace0,
        tag("for"),
        whitespace1,
        term2,
        whitespace1,
        tag("of"),
        whitespace1,
        condition,
        whitespace0,
        tag(":"),
        whitespace0,
        tag("("),
        condition,
        whitespace0,
        tag(")")
    ))(i)?;
    Ok((res.0, crate::YarRuleConditionNode::ForOf(Box::new(res.1.3), Box::new(res.1.7), Box::new(res.1.12))))
}

fn for_in(i: &str) -> IResult<&str, crate::YarRuleConditionNode>{
    let res = tuple((
        whitespace0,
        tag("for"),
        whitespace1,
        condition,
        whitespace1,
        name,
        whitespace1,
        tag("in"),
        whitespace1,
        condition,
        whitespace0,
        tag(":"),
        whitespace0,
        tag("("),
        condition,
        whitespace0,
        tag(")")
    ))(i)?;
    Ok((res.0, crate::YarRuleConditionNode::ForIn(Box::new(res.1.3), res.1.5, Box::new(res.1.9), Box::new(res.1.14))))
}

fn literal(i: &str) -> IResult<&str, crate::YarRuleConditionNode> {
    preceded(whitespace0,
             alt((
                 map(alt((
                     tag("all"),
                     tag("filesize"),
                     tag("filepath"),
                     tag("entrypoint"),
                     tag("any"),
                     tag("them")
                 )), |m: &str| crate::YarRuleConditionNode::Reserved(m.to_string())),
                 for_of,
                 for_in,
                 not,
                 map(import_ref, |m| crate::YarRuleConditionNode::ImportRef(m)),
                 bytes_with_offset,
                 map(pair(string_name, tag("*")), |(m, _)| crate::YarRuleConditionNode::StringRefMask(format!("{}*", m))),
                 map(regex, |m| crate::YarRuleConditionNode::Regex(m)),
                 map(string, |m| crate::YarRuleConditionNode::ConstString(m)),
                 map(string_count2, |m| crate::YarRuleConditionNode::StringCount(m)),
                 map(string_count, |m| crate::YarRuleConditionNode::StringCount(m)),
                 map(string_name, |m| crate::YarRuleConditionNode::StringRef(m)),
                 map(name, |m| crate::YarRuleConditionNode::RuleRef(m)),
                 map(size, |m| crate::YarRuleConditionNode::Size(m)),
                 map(number, |m| crate::YarRuleConditionNode::Number(m)),
                 range,
                 set,
                 parens,
             )))(i)
}

fn term2(i: &str) -> IResult<&str, crate::YarRuleConditionNode>{
    let (i, l) = literal(i)?;
    fold_many0(
        preceded(whitespace0, pair(alt((tag("+"),
                                        tag("*"),
                                        tag("/"),
                                        tag("&"),
                                        tag("|"),
                                        tag("%"),
                                        tag(">>"),
                                        tag("<<"),
                                        tag("-"))), preceded(whitespace0, literal))),
        move || l.clone(),
        |acc, (op, val): (&str, crate::YarRuleConditionNode)| {
            match op{
                "+" | "-" | "&" | "|" | "*" | "/" | "%" | ">>" | "<<" => crate::YarRuleConditionNode::Arithm(op.to_string(), Box::new(acc), Box::new(val)),
                _ => crate::YarRuleConditionNode::None("".to_string())
            }
        },
    )(i)
}


fn term(i: &str) -> IResult<&str, crate::YarRuleConditionNode>{
    let (i, l) = term2(i)?;
    fold_many0(
        preceded(whitespace0, pair(alt((tag("of"),
                                        tag("in"),
                                        tag("at"),
                                        tag("=="),
                                        tag("!="),
                                        tag("contains"),
                                        tag("icontains"),
                                        tag("matches"),
                                        tag(">="),
                                        tag("<="),
                                        tag(">"),
                                        tag("<"))), preceded(whitespace0, term2))),
        move || l.clone(),
        |acc, (op, val): (&str, crate::YarRuleConditionNode)| {
            match op{
                "at" => crate::YarRuleConditionNode::At(Box::new(acc), Box::new(val)),
                "of" => crate::YarRuleConditionNode::Of(Box::new(acc), Box::new(val)),
                "in" => crate::YarRuleConditionNode::In(Box::new(acc), Box::new(val)),
                "==" | "!=" | ">" | "<" | "<=" | ">=" | "contains" | "icontains" | "matches" => crate::YarRuleConditionNode::Cmp(op.to_string(), Box::new(acc), Box::new(val)),
                _ => crate::YarRuleConditionNode::None("".to_string())
            }
        },
    )(i)
}

fn condition(i: &str) -> IResult<&str, crate::YarRuleConditionNode>{
    let (i, l) = term(i)?;
    fold_many0(
        preceded(whitespace0, pair(alt((tag("and"),
                                        tag("or"))), preceded(whitespace0, term))),
        move || l.clone(),
        |acc, (op, val): (&str, crate::YarRuleConditionNode)| {
            match op{
                "and" => crate::YarRuleConditionNode::And(Box::new(acc), Box::new(val)),
                "or" => crate::YarRuleConditionNode::Or(Box::new(acc), Box::new(val)),
                _ => crate::YarRuleConditionNode::None("".to_string())
            }
        },
    )(i)
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
                    map(number, |n| format!("0x{:02x}", n)),
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
    ))(i);
    let res = match res{
        Ok(s) => {
            s
        },
        Err(e) => {
//            if let nom::Err::Error(ee) = &e{
//                if ee.input != ""{
//                    println!("{:#?}", e);
//                }
//            }
            return Err(e);
        }
    };
    let mut r = crate::YarRule::new(
        if let Some(_) = res.1.1{true} else {false},
        if let Some(_) = res.1.2{true} else {false},
        res.1.6,
        vec![],
        res.1.10
    );
    if let Some((_, _, _, t, tt)) = res.1.7{
        r.tags.push(t);
        for (_, ttt) in tt{
            r.tags.push(ttt);
        }
    };
    Ok((res.0, r))
}

pub enum Ss{
    Import(crate::YarImport),
    Include(crate::YarInclude),
    Rule(crate::YarRule)
}

pub fn parse_rules(f: String, i: &str) -> IResult<&str, crate::YarRuleSet>{
    let res = many1(preceded(whitespace0, alt((
        map(import, |i| Ss::Import(i)),
        map(include, |i| Ss::Include(i)),
        map(rule, |r| Ss::Rule(r)),
    ))))(i);

    let res = match res{
        Ok(s) => {
            s
        },
        Err(e) => {
//            println!("{:#?}", e);
            return Err(e);
        }
    };

    let mut imports = vec![];
    let mut includes = vec![];
    let mut rules = std::collections::HashMap::new();

    for r in res.1{
        match r{
            Ss::Import(i) => {
                imports.push(i);
            }
            Ss::Include(i) => {
                includes.push(i);
            }
            Ss::Rule(i) =>{
                rules.insert(i.name.clone(), i);
            }
        }
    }

    Ok((res.0, crate::YarRuleSet::new(
        f,
        includes,
        imports,
        rules
    )))
}


#[cfg(test)]
mod tests {
    #[test]
    fn check_condition() {
        println!("{:#?}", super::condition("0"));
        println!("{:#?}", super::condition("uint32be(0)"));
        println!("{:#?}", super::bytes_with_offset("uint32be(0)"));
        println!("{:#?}", super::condition(r##"(
            uint16be(filesize-2) == 0x2722 or  /* Footer 1 */
                ( uint16be(filesize-2) == 0x220a and uint8(filesize-3) == 0x27 )  /* Footer 2 */

        )"##));
    }

    #[test]
    fn check_for_in() {
 //        println!("{:#?}", super::condition("( 0 .. pe.number_of_signatures )"));
 //        println!("{:#?}", super::condition(" pe.signatures [ i ] . issuer contains \"DigiCert SHA2 Assured ID Code Signing CA\" and pe.signatures [ i ] . serial == \"08:68:70:51:50:f1:cf:c1:fc:c3:fc:91:a4:49:49:a6\" "));
 //        println!("{:#?}", super::condition("for any i in ( 0 .. pe.number_of_signatures ) : ( pe.signatures [ i ] . issuer contains \"DigiCert SHA2 Assured ID Code Signing CA\" and pe.signatures [ i ] . serial == \"08:68:70:51:50:f1:cf:c1:fc:c3:fc:91:a4:49:49:a6\" )"));
 //        println!("{:#?}",
 //        super::condition("all of
 //        them and @font > @headers
 //        and @winexec == @font +
        //        ((5 + 44) * 2)"));
//        println!("{:#?}", super::condition("$ in (@keyfrag_esp_5[i]-100..@keyfrag_esp_5[i]+100)"));
//        println!("{:#?}", super::condition("($keyfrag_esp_*)"));
//        println!("{:#?}", super::for_of(r##" for all of ($keyfrag_esp_*): ($ in (@keyfrag_esp_5[i]-100..@keyfrag_esp_5[i]+100))"##));
        println!("{:#?}", super::condition("uint16(0) == 0x5A4D and uint32(uint32(0x3C)) == 0x00004550 and filesize < 300KB and for any i in (0..pe.number_of_sections - 1): (pe.sections[i].name == \".Init\" and pe.sections[i].virtual_size % 1024 == 0)"));

    }

}
