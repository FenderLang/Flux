use flux_bnf::{bnf, lexer::{CullStrategy}};

fn main() {
    let bnf_input = include_str!("json.bnf");
    let json_input = include_str!("json.json");

    let mut lexer = match bnf::parse(bnf_input) {
        Ok(v) => v,
        Err(e) => { 
            println!("Error parsing BNF:\n{:#?}", e);
            return;
        }
    };

    lexer.add_rule_for_names(
        vec!["sep".to_string()], 
        CullStrategy::LiftChildren
    );

    let result = lexer.tokenize(json_input);

    let root_token = match result {
        Ok(token) => token,
        Err(e) => { 
            println!("Error parsing JSON:\n{:#?}", e);
            return;
        }
    };
}

pub enum JSONValues {
    Integers(i64),
    Decimal(f64),
    String(String),
    Boolean(bool),
    List(Vec<JSONValues>),
    Map(HashMap<(String, JSONValues)>),
    Null,
}

fn parse_json(input: &[char]) -> Result(JSONValues, usize) {
    return match input[0] {
        't' => Ok(JSONValues::Boolean(true), 4),
        'f' => Ok(JSONValues::Boolean(false), 5),
        'n' => Ok((JSONValues::Null, 4)),
        '[' => parse_list(input),
        '{' => parse_map(input),
        '"' => parse_string(input),
        '0'..='9' | '.' => parse_number(input),
        _ => bnf::error("Invalid JSON value")
    }
}

fn parse_string(char: &[char]) -> Result(JSONValues, usize) {

}

fn parse_list(char: &[char]) -> Result(JSONValues, usize)  {

}

fn parse_map(char: &[char]) -> Result(JSONValues, usize)  {

}

// Parses a number, either an integer or a decimal
fn parse_number(char: &[char]) -> Result(JSONValues, usize)  {

}

