use flux::bnf;

fn main() {
    let example = "root ::= \"hello\"";
    let parsed = bnf::parse(example).unwrap();
    println!("{:?}", parsed);
}
