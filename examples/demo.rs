use flux_bnf::{bnf, lexer::CullStrategy};

fn main() {
    let bnf_input = include_str!("../src/tests/bnf/fender.bnf");
    let test_input = include_str!("test_fender.fndr");

    let mut lexer = bnf::parse(bnf_input).unwrap();
    lexer.add_rule_for_names(vec!["sep", "lineSep"].iter().map(ToString::to_string).collect(), CullStrategy::DeleteAll);

    let res = lexer.tokenize(&test_input);

    let _script = r#"
        struct Empty {}
        struct Person {name: str, age: int}
    "#;

    println!("{:#?}", res);
}
