use flux_bnf::{bnf, lexer::CullStrategy};

fn main() {
    let example = include_str!("../src/tests/bnf/json.bnf");
    let mut lexer = bnf::parse(example).unwrap();
    lexer.add_rule_for_names(vec!["alpha", "alphanum", "sep", "break", "lineBreak", "lineSep", "comment"].iter().map(|s| s.to_string()).collect(), CullStrategy::DeleteAll);
    lexer.add_rule_for_names(vec!["object".to_owned()], CullStrategy::LiftChildren);
    let test = r#"[14, true, {"serpents": ["snake", "anaconda", "python 2"]}]"#;
    let thing = test.chars().collect::<Vec<_>>();
    let token = lexer.tokenize(&thing);
    println!("{:#?}", token.unwrap());
}
