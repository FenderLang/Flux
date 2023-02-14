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


