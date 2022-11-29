use flux::bnf;

fn main() {
    let example = r#"
    sep ::= [ \n\t]+
    break ::= sep? ((";" | "\n") sep?)+

    root ::= break? (statement break?)*
    statement ::= assignment | expr | returnStatement
    returnStatement ::= "return" (sep expr)?
    assignment ::= "$"? name sep? "=" sep? expr

    binaryOperator ::= "||" | "&&" | "==" | "<=" | ">=" | [-+*/^<>]
    unaryOperator ::= [-!]

    expr ::= term (sep? binaryOperator sep? term)*
    term ::= (unaryOperator sep?)? value sep? receiverCallChain?
    value ::= (name | literal)

    alpha ::= [a-z] | [A-Z]
    alphanum ::= [a-z] | [A-Z] | [0-9]
    name ::= ("_" | alpha) alphanum*

    literal ::= float | int | string | boolean | function
    int ::= "-"? [0-9]+
    float ::= ("-"? [0-9]+)? "." [0-9]+
    string ::= "\"" (escapeSequence | [^"])* "\""
    escapeSequence ::= "\\" [^]
    boolean ::= "true" | "false"

    invoke ::= invokeArgs | functionBody
    invokeArgs ::= "(" sep? (expr sep? ("," sep? expr)*)? sep? ")"

    functionArg ::= name (sep? ":" sep? name)?
    functionBody ::= "{" break? (statement break)* statement? break? "}"
    functionArgs ::= "(" sep? (functionArg sep? ("," sep? functionArg)*)? sep? ")"
    function ::= functionArgs? sep? functionBody

    receiverCall ::= "." sep? name sep? invoke
    receiverCallChain ::= receiverCall (sep? receiverCall)*
    "#;
    let parsed = bnf::parse(example).unwrap();
    let test = "";
    let thing = test.chars().collect::<Vec<_>>();

    println!("{:#?}", parsed);
}
