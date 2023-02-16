use flux_bnf::{bnf, lexer::CullStrategy};

fn main() {
    let bnf_input = include_str!("../src/tests/bnf/json.bnf");
    let test_input = include_str!("test.json");

    let mut lexer = match bnf::parse(bnf_input) {
        Ok(v) => v,
        Err(e) => {
            println!("Full error:\n{}", e);
            println!("user friendly:\n{:+#}", e);
            return;
        }
    };

    lexer.add_rule_for_names(
        vec!["sep".to_string(), "null".to_string()],
        CullStrategy::LiftChildren,
    );

    // lexer.add_rule_for_names(vec!["pow", "add", "mul", "range", "cmp", "and", "or"], CullStrategy::LiftAtMost(1));

    let res = lexer.tokenize(test_input);

    let root_token = match res {
        Ok(token) => token,
        Err(e) => {
            println!("Pretty error:\n{:#?}", e);
            println!("user friendly:\n{:+#}", e);
            return;
        }
    };

    // println!("{:#?}", root_token);

    root_token.children_named("args")
        .for_each(|t| println!("{:?}  {}", t.get_name(), t.get_match()));
    println!("{:#?}", root_token.first());
}

