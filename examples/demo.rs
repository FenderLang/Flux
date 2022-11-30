use flux::{bnf, lexer::CullStrategy};

fn main() {
    let example = include_str!("fender.bnf");
    let mut lexer = bnf::parse(example).unwrap();
    lexer.add_rule_for_names(vec!["alpha", "alphanum", "sep", "break", "lineBreak"].iter().map(|s| s.to_string()).collect(), CullStrategy::DeleteAll);
    let test = r#"[1+1,2]"#;
    let thing = test.chars().collect::<Vec<_>>();
    let token = lexer.tokenize(&thing);
    println!("{:#?}", token);
}
