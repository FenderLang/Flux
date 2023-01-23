use flux_bnf::{bnf, lexer::CullStrategy};

fn main() {
    let bnf_input = include_str!("../src/tests/bnf/fender.bnf");
    let test_input = include_str!("test_fender.fndr");

    let mut lexer = match bnf::parse(bnf_input){
        Ok(v) => v,
        Err(e) => {  let line = "-".repeat(40);

            println!(    "Full Monty                     |  {{}} \n{line}\n{}\n{line}\n", e);
            println!("\n\ndefault but with -             |  {{:-}} \n{line}\n{:-}\n{line}\n", e);
            println!("\n\nuser friendly                  |  {{:#}} \n{line}\n{:#}\n{line}\n", e);
            println!("\n\nuser friendly with plus sign   |  {{:+#}} \n{line}\n{:+#}\n{line}\n", e);
            return;
        },
    };

    lexer.add_rule_for_names(
        vec!["sep", "lineSep", "alpha", "alphanum", "break", "newLine"]
            .iter()
            .map(ToString::to_string)
            .collect(),
        CullStrategy::DeleteAll,
    );

    let res = lexer.tokenize(test_input);

    match res {
        Ok(token) => println!("{:#?}", token),
        Err(e) => {
            let line = "-".repeat(40);

            println!(    "Full Monty                     |  {{}} \n{line}\n{}\n{line}\n", e);
            println!("\n\ndefault but with -             |  {{:-}} \n{line}\n{:-}\n{line}\n", e);
            println!("\n\nuser friendly                  |  {{:#}} \n{line}\n{:#}\n{line}\n", e);
            println!("\n\nuser friendly with plus sign   |  {{:+#}} \n{line}\n{:+#}\n{line}\n", e);


        },
    }
}
