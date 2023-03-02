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
    lexer.tokenize("a").unwrap();
    lexer.tokenize("b").unwrap_err();
    lexer.tokenize("u0061").unwrap_err();
}

#[test]
fn similar_token_after_repeating_token() {
    let lexer = bnf::parse(include_str!("bnf/a_after_repeating_ab.bnf")).unwrap();
    lexer.tokenize("aa").unwrap();
    lexer.tokenize("ab").unwrap_err();
    lexer.tokenize("ababaa").unwrap();
    lexer.tokenize("bababb").unwrap_err();
}

#[test]
fn similar_token_after_optional_token() {
    let lexer = bnf::parse(include_str!("bnf/a_after_optional_ab.bnf")).unwrap();
    lexer.tokenize("a").unwrap();
    lexer.tokenize("b").unwrap_err();
    lexer.tokenize("aa").unwrap();
    lexer.tokenize("ab").unwrap_err();
    lexer.tokenize("ba").unwrap();
    lexer.tokenize("aaa").unwrap_err();
}

#[test]
fn not_newline() {
    let lexer = bnf::parse("root ::= [^\\n]").unwrap();
    lexer.tokenize("\n").unwrap_err();
    lexer.tokenize("a").unwrap();
}

#[test]
fn recursion_stop1() {
    let lexer = bnf::parse(include_str!("bnf/recursive_list.bnf")).unwrap();
    lexer.tokenize("a b c").unwrap();
}
#[test]
fn recursion_stop2() {
    let lexer = bnf::parse(include_str!("bnf/numbers.bnf")).unwrap();
    lexer.tokenize("1.23").unwrap();
    lexer.tokenize("-4").unwrap();
    lexer.tokenize("abc").unwrap_err();
}

#[test]
fn paren() {
    let lexer = bnf::parse(include_str!("bnf/parens.bnf")).unwrap();

    lexer.tokenize("").unwrap();
    lexer.tokenize("()()()").unwrap();
    lexer.tokenize("()()(()())").unwrap();
    lexer.tokenize(")(").unwrap_err();
    lexer.tokenize("())(()").unwrap_err();
    lexer.tokenize("(()()(").unwrap_err();
}

#[test]
fn quantifier1() {
    let lexer = bnf::parse("root ::= [0-9]{3,16}").unwrap();
    lexer.tokenize("123456").unwrap();
    lexer.tokenize("12").unwrap_err();
}

#[test]
fn quantifier2() {
    let lexer = bnf::parse("root ::= [0-9]{,16}").unwrap();

    lexer.tokenize("12").unwrap();
    lexer.tokenize("12345678901234567").unwrap_err();
}

#[test]
fn quantifier3() {
    let lexer = bnf::parse("name ::= \"a\"\nroot ::= name{3}").unwrap();
    lexer.tokenize("aaa").unwrap();
    lexer.tokenize("aa").unwrap_err();
}

#[test]
fn case_insensitive() {
    let lexer = bnf::parse("root ::= i\"abc\"").unwrap();
    lexer.tokenize("abc").unwrap();
    lexer.tokenize("aBc").unwrap();
    lexer.tokenize("Abc").unwrap();
    lexer.tokenize("ABC").unwrap();

    let lexer2 = bnf::parse("root ::= \"abc\"").unwrap();
    lexer2.tokenize("abc").unwrap();
    lexer2.tokenize("Abc").unwrap_err();
    lexer2.tokenize("aBc").unwrap_err();
    lexer2.tokenize("aBC").unwrap_err();
}

#[test]
fn leading_space() {
    bnf::parse(" root ::= [a-z]\n    \t other ::= [a-z]").unwrap();
}

#[test]
fn just_eof_test() {
    let lex = bnf::parse(r#"root ::= "a" <eof>"#).unwrap();
    lex.tokenize("a").unwrap();
    lex.tokenize("a ").unwrap_err();
}

#[test]
fn template_test() {
    let lexer = bnf::parse(include_str!("bnf/template.bnf")).unwrap();
    lexer.tokenize("[]").unwrap();
    lexer.tokenize("[1]").unwrap();
    lexer.tokenize("[1, 2]").unwrap();
    lexer.tokenize("[1, 2").unwrap_err();
}

#[test]
fn alt_root_test() {
    let lexer = bnf::parse(include_str!("bnf/numbers.bnf")).unwrap();
    lexer.tokenize("4").unwrap();
    lexer.tokenize_with("root", "4").unwrap();
    lexer.tokenize_with("number", "4").unwrap();
    lexer.tokenize_with("decimal", "4.0").unwrap();
    lexer.tokenize_with("int", "4.0").unwrap_err();
    lexer.tokenize_with("int", "4").unwrap();
    lexer.tokenize_with("digit", "4").unwrap();
    lexer.tokenize_with("digit", "40").unwrap_err();
}