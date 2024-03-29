use flux_bnf::{bnf, lexer::CullStrategy, tokens::Token};
use std::str::FromStr;
use std::{collections::HashMap, error::Error};

type ResultAlias<T> = Result<T, Box<dyn Error>>;

fn main() {
    let json_input = include_str!("finaltest.json");
    let parsed = json_input.parse::<JSONValue>().unwrap();
    println!("{:#?}", parsed);
    drop(parsed);
}

#[derive(Debug)]
enum JSONValue {
    Integer(i64),
    Decimal(f64),
    String(String),
    Boolean(bool),
    List(Vec<JSONValue>),
    Map(HashMap<String, JSONValue>),
    Null,
}

impl FromStr for JSONValue {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lexer = bnf::parse(include_str!("json.bnf")).map_err(|e| format!("{:#}", e))?;
        lexer.add_rule_for_names(vec!["sep", "object"], CullStrategy::LiftChildren);
        lexer.set_unnamed_rule(CullStrategy::LiftChildren);
        lexer.tokenize(s, |t| parse_token(t))?
    }
}

fn parse_token(token: &Token) -> ResultAlias<JSONValue> {
    Ok(match token.get_name().as_deref() {
        Some("integer") => JSONValue::Integer(parse_int(token)?),
        Some("decimal") => JSONValue::Decimal(parse_float(token)?),
        Some("string") => JSONValue::String(parse_string(token)),
        Some("boolean") => JSONValue::Boolean(parse_bool(token)),
        Some("list") => JSONValue::List(parse_list(token)?),
        Some("map") => JSONValue::Map(parse_map(token)?),
        Some("escape") => JSONValue::String(parse_escape_sequence(token)),
        Some("null") => JSONValue::Null,
        _ => unreachable!("Unknown token: {:#?}", token),
    })
}

fn parse_int(token: &Token) -> ResultAlias<i64> {
    Ok(token.get_match().parse()?)
}

fn parse_float(token: &Token) -> ResultAlias<f64> {
    Ok(token.get_match().parse()?)
}

fn parse_string(token: &Token) -> String {
    let token_val = token.get_match();
    token_val[1..token_val.len() - 1].to_string()
}

fn parse_escape_sequence(token: &Token) -> String {
    match &*token.get_match() {
        "\\\"" => "\"".to_string(),
        "\\\\" => "\\".to_string(),
        "\\/" => "/".to_string(),
        "\\b" => "\u{0008}".to_string(),
        "\\f" => "\u{000C}".to_string(),
        "\\n" => "\n".to_string(),
        "\\r" => "\u{000D}".to_string(),
        "\\t" => "\u{0009}".to_string(),
        "\\u" => {
            let hex_string = token.get_match()[2..].to_string();
            let codepoint = u32::from_str_radix(&hex_string, 16).expect("Always Valid Hex");
            let mut string = String::new();
            string.push(std::char::from_u32(codepoint).expect("Always Valid Codepoint"));
            string
        }
        _ => unreachable!("Unknown escape sequence: {:#?}", token),
    }
}

fn parse_bool(token: &Token) -> bool {
    token.get_match().parse().expect("Always Valid Boolean")
}

fn parse_list(token: &Token) -> ResultAlias<Vec<JSONValue>> {
    let mut list = Vec::with_capacity(token.children.len());
    for child in &token.children {
        list.push(parse_token(child)?);
    }
    Ok(list)
}

fn parse_map(token: &Token) -> ResultAlias<HashMap<String, JSONValue>> {
    let mut map = HashMap::with_capacity(token.children.len());
    for child in &token.children {
        let key = parse_string(&child.children[0]);
        let value = parse_token(&child.children[1])?;
        map.insert(key, value);
    }
    Ok(map)
}
