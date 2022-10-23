/// XPath parser
///
/// Grammar rules:
/// ```text
/// xpath:
///     element
///     separator element
///     xpath separator element
/// separator:
///     //
///     /
/// element:
///     ..
///     .
///     @name
///     name [ conditions_or ]
///     name [ index ]
///     * [ conditions_or ]
///     * [ index ]
///     *
///     name
/// conditions_or:
///     conditions_and or conditions_and
///     conditions_and
/// conditions_and:
///     condition and condition
///     condition
/// condition:
///     name operator string
///     @name operator string
///     text() operator string
///     position() operator decimal
///     name
///     @name
///     @*
///     ( condition )
///     ( conditions_and )
///     ( conditions_or )
/// index:
///     decimal
///     last() - decimal
///     last()
/// operator:
///     >=
///     <=
///     >
///     <
///     !=
///     =
/// ```
use std::collections::{HashSet, HashMap};
use nom::{
    IResult,
    bytes::complete::{tag, escaped},
    character::complete::{one_of, none_of, char, anychar, space0, space1, alpha1, alphanumeric1, digit1},
    branch::alt,
    sequence::{pair, tuple, delimited},
    multi::{many0, many0_count},
    combinator::{recognize, opt, map, value},
};

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct XPathSegment {
    pub separator: String,
    pub node: String,
    pub condition: Predictor,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum Predictor {
    And(Box<Predictor>, Box<Predictor>),
    Or(Box<Predictor>, Box<Predictor>),
    Condition(String, Option<String>, Option<String>),
    IndexDecimal(String),
    IndexExpr(String, String),
    None,
}

impl Predictor {
    #[allow(dead_code)]
    pub fn collect(&self) -> (Vec<String>, Vec<String>, Vec<String>) {
        let mut child = HashSet::new();
        let mut attr = HashSet::new();
        let mut func = HashSet::new();
        match self {
            Predictor::And(ref left, ref right) => {
                let (c1, a1, f1) = left.collect();
                child.extend(c1);
                attr.extend(a1);
                func.extend(f1);
                let (c2, a2, f2) = right.collect();
                child.extend(c2);
                attr.extend(a2);
                func.extend(f2);
            },
            Predictor::Or(ref left, ref right) => {
                let (c1, a1, f1) = left.collect();
                child.extend(c1);
                attr.extend(a1);
                func.extend(f1);
                let (c2, a2, f2) = right.collect();
                child.extend(c2);
                attr.extend(a2);
                func.extend(f2);
            },
            Predictor::Condition(ref left, _, _) => {
                if left.starts_with("@") {
                    attr.insert(left.get(1..).unwrap().to_string());
                } else if left.ends_with("()") {
                    func.insert(left.to_string());
                } else {
                    child.insert(left.to_string());
                }
            },
            Predictor::IndexExpr(_, _) => {
                func.insert("last()".to_string());
            },
            _ => {}
        }
        let mut child:Vec<_> = child.into_iter().collect();
        let mut attr:Vec<_> = attr.into_iter().collect();
        let mut func:Vec<_> = func.into_iter().collect();
        child.sort();
        attr.sort();
        func.sort();
        (child, attr, func)
    }
    #[allow(dead_code)]
    pub fn expr(&self, info:&HashMap<String, String>) -> String {
        match self {
            Predictor::And(ref left, ref right) => {
                format!("({}) && ({})", left.expr(info), right.expr(info))
            },
            Predictor::Or(ref left, ref right) => {
                format!("({}) || ({})", left.expr(info), right.expr(info))
            },
            Predictor::Condition(ref left, ref op, ref right) => {
                if info.contains_key(left) {
                    if op.is_none() || right.is_none() {
                        "true".to_string()
                    } else {
                        format!("'{}' {} {}", escape_info(info.get(left).unwrap()).unwrap().1, op.as_ref().unwrap(), right.as_ref().unwrap())
                    }
                } else {
                    "false".to_string()
                }
            },
            Predictor::IndexDecimal(ref left) => {
                debug_assert!(info.contains_key("position()"));
                format!("{} == {}", info.get("position()").unwrap(), left)
            },
            Predictor::IndexExpr(ref left, ref right) => {
                debug_assert!(info.contains_key("position()"));
                debug_assert!(info.contains_key("last()"));
                if right == "" {
                    format!("{} == {}", info.get("position()").unwrap(), info.get(left).unwrap())
                } else {
                    format!("{} == {} - {}", info.get("position()").unwrap(), info.get(left).unwrap(), right)
                }
            },
            _ => {
                "true".to_string()
            }
        }
    }
}

fn escape_info(input:&str) -> IResult<&str, String> {
    map(
        many0(alt((
            value("\\\\".to_string(), char('\\')),
            value("\\'".to_string(), char('\'')),
            map(anychar, |c| c.to_string()),
        ))), |v| v.join("")
    )(input)
}
fn decimal(input:&str) -> IResult<&str, &str> {
    digit1(input)
}

fn name(input:&str) -> IResult<&str, &str> {
    recognize(pair(
            alt((alpha1, tag("_"), tag(":"))),
            many0_count(alt((alphanumeric1, tag("_"), tag(":")))),
    ))(input)
}

fn separator(input:&str) -> IResult<&str, &str> {
    alt((
            tag("//"),
            tag("/"),
    ))(input)
}

fn operator(input:&str) -> IResult<&str, &str> {
    alt((
            tag(">="),
            tag("<="),
            tag(">"),
            tag("<"),
            tag("!="),
            value("==", tag("=")),
    ))(input)
}

fn string(input:&str) -> IResult<&str, &str> {
    recognize(delimited(
            tag("'"),
            many0_count(escaped(none_of("'\\"), '\\', one_of(r#"\'"#))),
            tag("'"),
    ))(input)
}

fn index(input:&str) -> IResult<&str, Predictor> {
    alt((
            map(decimal, |t| Predictor::IndexDecimal(t.to_string())),
            map(tuple((tag("last()"), space0, tag("-"), space0, decimal)), |t| Predictor::IndexExpr(t.0.to_string(), t.4.to_string())),
            map(tag("last()"), |t:&str| Predictor::IndexExpr(t.to_string(), "".to_string())),
    ))(input)
}

fn condition(input:&str) -> IResult<&str, Predictor> {
    alt((
            map(tuple((name, space0, operator, space0, string)), |t| Predictor::Condition(t.0.to_string(), Some(t.2.to_string()), Some(t.4.to_string()))),
            map(tuple((tag("@"), name, space0, operator, space0, string)), |t| Predictor::Condition(format!("@{}", t.1), Some(t.3.to_string()), Some(t.5.to_string()))),
            map(tuple((tag("text()"), space0, operator, space0, string)), |t| Predictor::Condition(t.0.to_string(), Some(t.2.to_string()), Some(t.4.to_string()))),
            map(tuple((tag("position()"), space0, operator, space0, decimal)), |t| Predictor::Condition(t.0.to_string(), Some(t.2.to_string()), Some(t.4.to_string()))),
            map(name, |t| Predictor::Condition(t.to_string(), None, None)),
            map(pair(tag("@"), name), |t| Predictor::Condition(format!("{}{}", t.0, t.1), None, None)),
            map(tag("@*"), |t:&str| Predictor::Condition(t.to_string(), None, None)),
            map(tuple((tag("("), space0, condition, space0, tag(")"))), |t| t.2),
            map(tuple((tag("("), space0, conditions_and, space0, tag(")"))), |t| t.2),
            map(tuple((tag("("), space0, conditions_or, space0, tag(")"))), |t| t.2),
    ))(input)
}

fn conditions_and(input:&str) -> IResult<&str, Predictor> {
    alt((
            map(tuple((condition, space1, tag("and"), space1, condition)), |t| Predictor::And(Box::new(t.0), Box::new(t.4))),
            condition,
    ))(input)
}

fn conditions_or(input:&str) -> IResult<&str, Predictor> {
    alt((
            map(tuple((conditions_and, space1, tag("or"), space1, conditions_and)), |t| Predictor::Or(Box::new(t.0), Box::new(t.4))),
            conditions_and,
    ))(input)
}

fn element(input:&str) -> IResult<&str, XPathSegment> {
    alt((
            map(tag(".."), |t:&str| XPathSegment {
                separator: "".to_string(),
                node: t.to_string(),
                condition: Predictor::None,
            }),
            map(tag("."), |t:&str| XPathSegment {
                separator: "".to_string(),
                node: t.to_string(),
                condition: Predictor::None,
            }),
            map(recognize(pair(tag("@"), name)), |t| XPathSegment {
                separator: "".to_string(),
                node: "*".to_string(),
                condition: Predictor::Condition(t.to_string(), None, None),
            }),
            map(tuple((name, tag("["), space0, conditions_or, space0, tag("]"))), |t| XPathSegment {
                separator: "".to_string(),
                node: t.0.to_string(),
                condition: t.3,
            }),
            map(tuple((name, tag("["), space0, index, space0, tag("]"))), |t| XPathSegment {
                separator: "".to_string(),
                node: t.0.to_string(),
                condition: t.3,
            }),
            map(tuple((tag("*["), space0, conditions_or, space0, tag("]"))), |t| XPathSegment {
                separator: "".to_string(),
                node: "*".to_string(),
                condition: t.2,
            }),
            map(tuple((tag("*["), space0, index, space0, tag("]"))), |t| XPathSegment {
                separator: "".to_string(),
                node: "*".to_string(),
                condition: t.2,
            }),
            map(tag("*"), |t:&str| XPathSegment {
                separator: "".to_string(),
                node: t.to_string(),
                condition: Predictor::None,
            }),
            map(name, |t| XPathSegment {
                separator: "".to_string(),
                node: t.to_string(),
                condition: Predictor::None,
            }),
    ))(input)
}

#[allow(dead_code)]
pub fn xpath(input:&str) -> IResult<&str, Vec<XPathSegment>> {
    let (remaining, initial) = opt(element)(input)?;
    let mut segments = Vec::new();
    if let Some(data) = initial {
        segments.push(data);
    }
    let (remaining, parts) = many0(map(pair(separator, element), |mut t| {
        t.1.separator = t.0.to_string();
        t.1
    }))(remaining)?;
    segments.extend(parts);
    Ok((remaining, segments))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_decimal() {
        assert_eq!(decimal("1234a"), Ok(("a", "1234")));
    }
    #[test]
    fn test_name() {
        assert_eq!(name("h:a12 u"), Ok((" u", "h:a12")));
        assert_eq!(name("_a12[u"), Ok(("[u", "_a12")));
        assert!(name("1a12 u").is_err());
    }
    #[test]
    fn test_separator() {
        assert_eq!(separator("/a"), Ok(("a", "/")));
        assert_eq!(separator("//a"), Ok(("a", "//")));
        assert_eq!(separator("///a"), Ok(("/a", "//")));
    }
    #[test]
    fn test_operator() {
        assert_eq!(operator(">=a"), Ok(("a", ">=")));
        assert_eq!(operator("<=a"), Ok(("a", "<=")));
        assert_eq!(operator(">a"), Ok(("a", ">")));
        assert_eq!(operator("<a"), Ok(("a", "<")));
        assert_eq!(operator("!=a"), Ok(("a", "!=")));
        assert_eq!(operator("=a"), Ok(("a", "==")));
    }
    #[test]
    fn test_string() {
        assert_eq!(string("'ab''"), Ok(("'", "'ab'")));
        assert_eq!(string(r"'ab\'''"), Ok(("'", r"'ab\''")));
    }
    #[test]
    fn test_index() {
        assert_eq!(index("2a"), Ok(("a", Predictor::IndexDecimal("2".to_string()))));
        assert_eq!(index("last()a"), Ok(("a", Predictor::IndexExpr("last()".to_string(), "".to_string()))));
        assert_eq!(index("last()- 2a"), Ok(("a", Predictor::IndexExpr("last()".to_string(), "2".to_string()))));
    }
    #[test]
    fn test_condition() {
        assert_eq!(condition("child_node"), Ok(("", Predictor::Condition("child_node".to_string(), None, None))));
        assert_eq!(condition("child_node= 'aa'"), Ok(("", Predictor::Condition("child_node".to_string(), Some("==".to_string()), Some("'aa'".to_string())))));
        assert_eq!(condition("@*a"), Ok(("a", Predictor::Condition("@*".to_string(), None, None))));
        assert_eq!(condition("@attr"), Ok(("", Predictor::Condition("@attr".to_string(), None, None))));
        assert_eq!(condition("@attr  = 'aa'"), Ok(("", Predictor::Condition("@attr".to_string(), Some("==".to_string()), Some("'aa'".to_string())))));
        assert_eq!(condition("text()!= 'aa'"), Ok(("", Predictor::Condition("text()".to_string(), Some("!=".to_string()), Some("'aa'".to_string())))));
        assert_eq!(condition("position()>= 7a"), Ok(("a", Predictor::Condition("position()".to_string(), Some(">=".to_string()), Some("7".to_string())))));
        assert_eq!(condition("(position()>= 7 )a"), Ok(("a", Predictor::Condition("position()".to_string(), Some(">=".to_string()), Some("7".to_string())))));
    }
    #[test]
    fn test_conditions_or() {
        assert_eq!(conditions_or("@attr  = 'aa'"), Ok(("", Predictor::Condition("@attr".to_string(), Some("==".to_string()), Some("'aa'".to_string())))));
        assert_eq!(conditions_or("text()!= 'aa'"), Ok(("", Predictor::Condition("text()".to_string(), Some("!=".to_string()), Some("'aa'".to_string())))));
        assert_eq!(conditions_or("child_node and @attr)"), Ok((")", Predictor::And(
                Box::new(Predictor::Condition("child_node".to_string(), None, None)),
                Box::new(Predictor::Condition("@attr".to_string(), None, None)),
                ))));
        assert_eq!(conditions_or("text()='aa' or child_node and @attr)"), Ok((")", Predictor::Or(
                Box::new(Predictor::Condition("text()".to_string(), Some("==".to_string()), Some("'aa'".to_string()))),
                Box::new(Predictor::And(
                        Box::new(Predictor::Condition("child_node".to_string(), None, None)),
                        Box::new(Predictor::Condition("@attr".to_string(), None, None)),
                        )),
                ))));
    }
    #[test]
    fn test_xpath() {
        assert_eq!(xpath("@id"), Ok(("", vec![
                    XPathSegment {
                        separator:"".to_string(),
                        node:"*".to_string(),
                        condition:Predictor::Condition("@id".to_string(), None, None)
                    },
        ])));
        assert_eq!(xpath("//NODE[@oid and @attrcatref='abc']"), Ok(("", vec![
                    XPathSegment {
                        separator:"//".to_string(),
                        node:"NODE".to_string(),
                        condition:Predictor::And(
                            Box::new(Predictor::Condition("@oid".to_string(), None, None)),
                            Box::new(Predictor::Condition("@attrcatref".to_string(), Some("==".to_string()), Some("'abc'".to_string()))),
                        )
                    },
        ])));
        assert_eq!(xpath(".//NAME/TUV"), Ok(("", vec![
                    XPathSegment {
                        separator:"".to_string(),
                        node:".".to_string(),
                        condition:Predictor::None
                    },
                    XPathSegment {
                        separator:"//".to_string(),
                        node:"NAME".to_string(),
                        condition:Predictor::None
                    },
                    XPathSegment {
                        separator:"/".to_string(),
                        node:"TUV".to_string(),
                        condition:Predictor::None
                    },
        ])));
    }
    #[test]
    fn test_predictor_expr() {
        let (remaining, segs) = xpath(".//NAME[text()='aa' and (@id='bb' or @gid)]").unwrap();
        assert_eq!(remaining, "");
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[1].condition.collect(), (
                Vec::new(),
                vec!["gid".to_string(), "id".to_string()],
                vec!["text()".to_string(),],
        ));
        let mut info = HashMap::new();
        info.insert("text()".to_string(), "aaa".to_string());
        info.insert("@id".to_string(), "123".to_string());
        assert_eq!(segs[1].condition.expr(&info), "('aaa' == 'aa') && (('123' == 'bb') || (false))")
    }
    #[test]
    fn test_escape_info() {
        assert_eq!(escape_info("ab'c"), Ok(("", "ab\\'c".to_string())));
        assert_eq!(escape_info("ab\\c"), Ok(("", "ab\\\\c".to_string())));
    }
}
