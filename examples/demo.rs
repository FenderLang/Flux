use flux::bnf;

fn main() {
    let example = include_str!("fender.bnf");
    let parsed = bnf::parse(example).unwrap();
    let test = "0+0";
    let thing = test.chars().collect::<Vec<_>>();
    let token = parsed.tokenize(&thing);
    println!("{:#?}", token);
}
