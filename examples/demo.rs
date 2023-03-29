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

    lexer.set_unnamed_rule(CullStrategy::LiftChildren);
    lexer.add_rule_for_names(
        vec!["sep", "lineSep", "lineBreak", "newLine", "alpha"],
        CullStrategy::DeleteAll,
    );

    lexer.add_rule_for_names(
        vec![
            "pow",
            "add",
            "mul",
            "range",
            "cmp",
            "and",
            "or",
            "term",
            "expr",
            "value",
            "tailOperation",
        ],
        CullStrategy::LiftAtMost(1),
    );

    let res = lexer.tokenize(test_input);

    let root_token = match res {
        Ok(token) => token,
        Err(e) => {
            println!("{:+#}", e);
            return;
        }
    };

    println!("{:#?}", root_token);
}
