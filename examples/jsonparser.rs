use std::{error::Error, collections::HashMap};
use flux_bnf::{bnf, lexer::CullStrategy, tokens::Token};

type ResultAlias<T> = Result<T, Box<dyn Error>>;

fn main() {
    let bnf_input = include_str!("json.bnf");
    let json_input = include_str!("test.json");

    let mut lexer = match bnf::parse(bnf_input) {
        Ok(v) => v,
        Err(e) => { 
            println!("Error parsing BNF:\n{:#?}", e);
            return;
        }
    };

    lexer.add_rule_for_names(
        vec!["sep"], 
        CullStrategy::DeleteAll
    );

    let result = lexer.tokenize(json_input);

    let root_token = match result {
        Ok(token) => token,
        Err(e) => { 
            println!("Error parsing JSON:\n{:#?}", e);
            return;
        }
    };

    parse_token(&root_token);


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

fn parse_token(token: &Token) -> ResultAlias<JSONValue> {
    Ok(match token.get_name().as_deref() {
        Some("integer") => JSONValue::Integer(parse_int(token)?),
        Some("float") => JSONValue::Decimal(parse_float(token)?),
        Some("string") => JSONValue::String(parse_string(token)), 
        Some("bool") => JSONValue::Boolean(parse_bool(token)),
        Some("list") => JSONValue::List(parse_list(token)?),
        Some("map") => JSONValue::Map(parse_map(token)?),
        Some("null") => JSONValue::Null,
        _ => unreachable!("Unknown token"),
    })
}

fn parse_int(token: &Token) -> ResultAlias<i64> {
    Ok(token.get_match().parse()?)
}

fn parse_float(token: &Token) -> ResultAlias<f64> {
    Ok(token.get_match().parse()?)
}

fn parse_string(token: &Token) -> String {
   token.get_match()
}

fn parse_bool(token: &Token) -> bool {
    token.get_match().parse().expect("Always Valid Boolean")
}

fn parse_list(token: &Token) -> ResultAlias<Vec<JSONValue>>{
    let mut list = Vec::new();
    for child in &token.children {
        list.push(parse_token(child)?);
    }
    Ok(list)
}

fn parse_map(token: &Token) -> ResultAlias<HashMap<String, JSONValue>> {
    let mut map = HashMap::new();
    for child in &token.children {
        let key = child.children[0].get_match();
        let value = parse_token(&child.children[1])?;
        map.insert(key, value);
    }
    Ok(map)
}

