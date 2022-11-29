use flux::bnf;

fn main() {
    let example = include_str!("fender.bnf");
    println!("{}", example);
    let parsed = bnf::parse(example).unwrap();
    let test = "0+0";
    let thing = test.chars().collect::<Vec<_>>();
    let token = parsed.apply(&thing, 0);
    println!("{:#?}", token);
}
