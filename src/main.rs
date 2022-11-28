use flux::bnf;

fn main() {
    let example = r#"
    root ::= "hello"+
    "#;
    let parsed = bnf::parse(example).unwrap();
    let test = "hello";
    let thing = test.chars().collect::<Vec<_>>();
    let token = parsed.apply(&thing, 0);
    println!("{:#?}", token);
}
