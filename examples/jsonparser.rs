use flux_bnf::{bnf, lexer::CullStrategy, tokens::Token};

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

fn parse_tokens(token: &Token) {
    match token.get_name().as_deref() {
        Some("integer") => parse_int(token),
        Some("float") => parse_float(token),
        Some("string") => parse_string(token), 
        Some("map") => parse_map(token),
        _ => println!("Unknown token"),
    }
}

fn parse_int(token: &Token) {
    token.get_match().parse::<i64>();
}

fn parse_float(token: &Token) {

}

fn parse_string(token: &Token) {

}

fn parse_map(token: &Token) {

}

