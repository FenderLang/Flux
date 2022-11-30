use flux::{bnf, lexer::CullStrategy};

fn main() {
    let example = include_str!("fender.bnf");
    let mut lexer = bnf::parse(example).unwrap();
    lexer.add_rule_for_names(vec!["alpha", "alphanum", "sep", "break", "lineBreak", "lineSep"].iter().map(|s| s.to_string()).collect(), CullStrategy::DeleteAll);
    let test = r#"
    $x = (a) {
        $y = (b) {
            return "hello world"
        }
        y(a)
        "bad things"
    }

    $createor = {
        $y = {
            pass 7
        }
        y
    }
    $other_get_num = {
        return 7
    }

    $b = x(0)
    $get_num = creator()
    getr_num() == other_get_num()
    b() # Errors, since b is trying to return from x, which is no longer the function being invoked
    "#;
    let thing = test.chars().collect::<Vec<_>>();
    let token = lexer.tokenize(&thing);
    println!("{:#?}", token);
}
