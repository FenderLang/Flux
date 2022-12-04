use crate::bnf;

// static FENDER_BNF: &'static str = include_str!("bnf/fender.bnf");

#[test]
fn syntax() {
    bnf::parse("root::= [a-z]").unwrap_err();
    bnf::parse("root ::= \"abc").unwrap_err();
    bnf::parse("root ::= \"abc\" | [a-z]").unwrap();
}

#[test]
#[ignore = "not yet implemented"]
fn self_reference_test1() {
    bnf::parse("root ::= root").unwrap_err();
}

#[test]
#[ignore = "not yet implemented"]
fn self_reference_test2() {
    bnf::parse("root ::= test\ntest ::= test").unwrap_err();
}

#[test]
fn unicode_escape() {
    let lexer = bnf::parse(r#"root ::= "\u0061""#).unwrap();
    lexer.tokenize(&"a".chars().collect::<Vec<_>>()).unwrap();
    lexer
        .tokenize(&"b".chars().collect::<Vec<_>>())
        .unwrap_err();
    lexer
        .tokenize(&"u0061".chars().collect::<Vec<_>>())
        .unwrap_err();
}

#[test]
#[ignore = "not yet implemented"]
fn similar_token_after_repeating_token() {
    let lexer = bnf::parse(include_str!("bnf/a_after_repeating_ab.bnf")).unwrap();
    lexer.tokenize(&"aa".chars().collect::<Vec<_>>()).unwrap();
    lexer
        .tokenize(&"ab".chars().collect::<Vec<_>>())
        .unwrap_err();
    lexer
        .tokenize(&"ababaa".chars().collect::<Vec<_>>())
        .unwrap();
    lexer
        .tokenize(&"bababb".chars().collect::<Vec<_>>())
        .unwrap_err();
}

#[test]
#[ignore = "not yet implemented"]
fn similar_token_after_optional_token() {
    let lexer = bnf::parse(include_str!("bnf/a_after_optional_ab.bnf")).unwrap();
    lexer.tokenize(&"a".chars().collect::<Vec<_>>()).unwrap();
    lexer
        .tokenize(&"b".chars().collect::<Vec<_>>())
        .unwrap_err();
    lexer.tokenize(&"aa".chars().collect::<Vec<_>>()).unwrap();
    lexer
        .tokenize(&"ab".chars().collect::<Vec<_>>())
        .unwrap_err();
    lexer.tokenize(&"ba".chars().collect::<Vec<_>>()).unwrap();
    lexer
        .tokenize(&"aaa".chars().collect::<Vec<_>>())
        .unwrap_err();
}

#[test]
fn not_newline() {
    let lexer = bnf::parse("root ::= [^\\n]").unwrap();
    lexer
        .tokenize(&"\n".chars().collect::<Vec<_>>())
        .unwrap_err();
    lexer.tokenize(&"a".chars().collect::<Vec<_>>()).unwrap();
}

#[test]
fn recursion_stop1() {
    let lexer = bnf::parse(include_str!("bnf/recursive_list.bnf")).unwrap();
    lexer
        .tokenize(&"a b c".chars().collect::<Vec<_>>())
        .unwrap();
}
#[test]
fn recursion_stop2() {
    let lexer = bnf::parse(include_str!("bnf/numbers.bnf")).unwrap();
    lexer.tokenize(&"1.23".chars().collect::<Vec<_>>()).unwrap();
    lexer.tokenize(&"-4".chars().collect::<Vec<_>>()).unwrap();
    lexer
        .tokenize(&"abc".chars().collect::<Vec<_>>())
        .unwrap_err();
}

#[test]
fn paren() {
    let lexer = bnf::parse(include_str!("bnf/parens.bnf")).unwrap();

    lexer.tokenize(&"".chars().collect::<Vec<_>>()).unwrap();
    lexer
        .tokenize(&"()()()".chars().collect::<Vec<_>>())
        .unwrap();
    lexer
        .tokenize(&"()()(()())".chars().collect::<Vec<_>>())
        .unwrap();
    lexer
        .tokenize(&")(".chars().collect::<Vec<_>>())
        .unwrap_err();
    lexer
        .tokenize(&"())(()".chars().collect::<Vec<_>>())
        .unwrap_err();
    lexer
        .tokenize(&"(()()(".chars().collect::<Vec<_>>())
        .unwrap_err();
}

#[test]
fn quantifier1() {
    let lexer = bnf::parse("root ::= [0-9]{3,16}").unwrap();
    lexer
        .tokenize(&"123456".chars().collect::<Vec<_>>())
        .unwrap();
    lexer
        .tokenize(&"12".chars().collect::<Vec<_>>())
        .unwrap_err();
}

#[test]
fn quantifier2() {
    let lexer = bnf::parse("root ::= [0-9]{,16}").unwrap();

    lexer.tokenize(&"12".chars().collect::<Vec<_>>()).unwrap();
    lexer
        .tokenize(&"12345678901234567".chars().collect::<Vec<_>>())
        .unwrap_err();
}

#[test]
fn quantifier3() {
    let lexer = bnf::parse("name ::= \"a\"\nroot ::= name{3}").unwrap();
    lexer.tokenize(&"aaa".chars().collect::<Vec<_>>()).unwrap();
    lexer
        .tokenize(&"aa".chars().collect::<Vec<_>>())
        .unwrap_err();
}

#[test]
fn case_insensitive() {
    let lexer = bnf::parse("root ::= i\"abc\"").unwrap();
    lexer.tokenize(&"abc".chars().collect::<Vec<_>>()).unwrap();
    lexer.tokenize(&"aBc".chars().collect::<Vec<_>>()).unwrap();
    lexer.tokenize(&"Abc".chars().collect::<Vec<_>>()).unwrap();
    lexer.tokenize(&"ABC".chars().collect::<Vec<_>>()).unwrap();

    let lexer2 = bnf::parse("root ::= \"abc\"").unwrap();
    lexer2.tokenize(&"abc".chars().collect::<Vec<_>>()).unwrap();
    lexer2
        .tokenize(&"Abc".chars().collect::<Vec<_>>())
        .unwrap_err();
    lexer2
        .tokenize(&"aBc".chars().collect::<Vec<_>>())
        .unwrap_err();
    lexer2
        .tokenize(&"aBC".chars().collect::<Vec<_>>())
        .unwrap_err();
}


#[test]
fn leading_space() {
    bnf::parse(" root ::= [a-z]\n    \t other ::= [a-z]").unwrap();
}