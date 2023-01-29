use flux_bnf::{bnf, lexer::CullStrategy};

fn main() {
    let bnf_input = include_str!("../src/tests/bnf/fender.bnf");
    let test_input = include_str!("test_fender.fndr");

    let mut lexer = match bnf::parse(bnf_input) {
        Ok(v) => v,
        Err(e) => {
            println!("Full error:\n{}", e);
            println!("user friendly:\n{:+#}", e);
            return;
        }
    };

    lexer.add_rule_for_names(
        vec!["sep"]
            .iter()
            .map(ToString::to_string)
            .collect(),
        CullStrategy::DeleteAll,
    );

    let res = lexer.tokenize(test_input);

    let root_token = match res {
        Ok(token) => token,
        Err(e) => {
            println!("Full error:\n{}", e);
            println!("user friendly:\n{:+#}", e);
            return;
        }
    };

    // println!("{:#?}", root_token);

    root_token.children_named("args")
        .for_each(|t| println!("{:?}  {}", t.get_name(), t.get_match()));
    println!("{:#?}", root_token.first());
}
