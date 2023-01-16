use flux_bnf::{
    bnf,
    lexer::{CullStrategy},
};

fn main() {
    let test_json = r#"[14, true, {"serpents": ["snake", "anaconda", "python 2"]}]"#;
    let bnf_input = include_str!("../src/tests/bnf/json.bnf");

    let mut lexer = bnf::parse(bnf_input).unwrap();
    lexer.add_rule_for_names(vec!["sep".to_string()], CullStrategy::DeleteAll);
    lexer.add_rule_for_names(vec!["object".to_string()], CullStrategy::LiftChildren);
    let res = lexer.tokenize(&test_json);

    println!("{:#?}", res);
}
