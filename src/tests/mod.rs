use crate::bnf;

// static FENDER_BNF: &'static str = include_str!("bnf/fender.bnf");

#[test]
fn syntax() {
    bnf::parse("root::= [a-z]").unwrap_err();
    bnf::parse("root ::= \"abc").unwrap_err();
    bnf::parse("root ::= \"abc\" | [a-z]").unwrap();
}

#[test]
fn unicode_escape() {
    let lexer = bnf::parse(r#"root ::= "\u0061""#).unwrap();
    lexer.check("a").unwrap();
    lexer.check("b").unwrap_err();
    lexer.check("u0061").unwrap_err();
}

#[test]
fn similar_token_after_repeating_token() {
    let lexer = bnf::parse(include_str!("bnf/a_after_repeating_ab.bnf")).unwrap();
    lexer.check("aa").unwrap();
    lexer.check("ab").unwrap_err();
    lexer.check("ababaa").unwrap();
    lexer.check("bababb").unwrap_err();
}

#[test]
fn similar_token_after_optional_token() {
    let lexer = bnf::parse(include_str!("bnf/a_after_optional_ab.bnf")).unwrap();
    lexer.check("a").unwrap();
    lexer.check("b").unwrap_err();
    lexer.check("aa").unwrap();
    lexer.check("ab").unwrap_err();
    lexer.check("ba").unwrap();
    lexer.check("aaa").unwrap_err();
}

#[test]
fn not_newline() {
    let lexer = bnf::parse("root ::= [^\\n]").unwrap();
    lexer.check("\n").unwrap_err();
    lexer.check("a").unwrap();
}

#[test]
fn recursion_stop1() {
    let lexer = bnf::parse(include_str!("bnf/recursive_list.bnf")).unwrap();
    lexer.check("a b c").unwrap();
}
#[test]
fn recursion_stop2() {
    let lexer = bnf::parse(include_str!("bnf/numbers.bnf")).unwrap();
    lexer.check("1.23").unwrap();
    lexer.check("-4").unwrap();
    lexer.check("abc").unwrap_err();
}

#[test]
fn paren() {
    let lexer = bnf::parse(include_str!("bnf/parens.bnf")).unwrap();

    lexer.check("").unwrap();
    lexer.check("()()()").unwrap();
    lexer.check("()()(()())").unwrap();
    lexer.check(")(").unwrap_err();
    lexer.check("())(()").unwrap_err();
    lexer.check("(()()(").unwrap_err();
}

#[test]
fn quantifier1() {
    let lexer = bnf::parse("root ::= [0-9]{3,16}").unwrap();
    lexer.check("123456").unwrap();
    lexer.check("12").unwrap_err();
}

#[test]
fn quantifier2() {
    let lexer = bnf::parse("root ::= [0-9]{,16}").unwrap();

    lexer.check("12").unwrap();
    lexer.check("12345678901234567").unwrap_err();
}

#[test]
fn quantifier3() {
    let lexer = bnf::parse("name ::= \"a\"\nroot ::= name{3}").unwrap();
    lexer.check("aaa").unwrap();
    lexer.check("aa").unwrap_err();
}

#[test]
fn case_insensitive() {
    let lexer = bnf::parse("root ::= i\"abc\"").unwrap();
    lexer.check("abc").unwrap();
    lexer.check("aBc").unwrap();
    lexer.check("Abc").unwrap();
    lexer.check("ABC").unwrap();

    let lexer2 = bnf::parse("root ::= \"abc\"").unwrap();
    lexer2.check("abc").unwrap();
    lexer2.check("Abc").unwrap_err();
    lexer2.check("aBc").unwrap_err();
    lexer2.check("aBC").unwrap_err();
}

#[test]
fn leading_space() {
    bnf::parse(" root ::= [a-z]\n    \t other ::= [a-z]").unwrap();
}

#[test]
fn just_eof_test() {
    let lex = bnf::parse(r#"root ::= "a" <eof>"#).unwrap();
    lex.check("a").unwrap();
    lex.check("a ").unwrap_err();
}

#[test]
fn template_test() {
    let lexer = bnf::parse(include_str!("bnf/template.bnf")).unwrap();
    lexer.check("[]").unwrap();
    lexer.check("[1]").unwrap();
    lexer.check("[1, 2]").unwrap();
    lexer.check("[1, 2").unwrap_err();
}

#[test]
fn alt_root_test() {
    let lexer = bnf::parse(include_str!("bnf/numbers.bnf")).unwrap();
    lexer.check("4").unwrap();
    lexer.check_with("root", "4").unwrap();
    lexer.check_with("number", "4").unwrap();
    lexer.check_with("decimal", "4.0").unwrap();
    lexer.check_with("int", "4.0").unwrap_err();
    lexer.check_with("int", "4").unwrap();
    lexer.check_with("digit", "4").unwrap();
    lexer.check_with("digit", "40").unwrap_err();
}

#[test]
fn newline_test() {
    let lexer = bnf::parse("root ::= <nl>").unwrap();
    lexer.check("\n").unwrap();
    lexer.check("\r\n").unwrap();
    lexer.check("\r").unwrap();
}
