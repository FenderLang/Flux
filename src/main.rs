use flux::bnf;

fn main() {
    let example = r#"root ::= "hello"+"#;
    let parsed = bnf::parse(example);
    println!("{:#?}", parsed);
}
