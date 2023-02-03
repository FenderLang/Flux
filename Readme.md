# Flux

A lexer generator for any context-free syntax in 100% rust with no dependencies.

Created for [Fender-lang](https://github.com/FenderLang/).

# Lexer

## What is a Lexer
- A Lexer is part of an interpreter that takes sequence of characters and turns them into tokens.

## How to use the Lexer

First define your BNF input
```rust
let bnf_input = include_str!("path/to/example.bnf");
```

To parse the input you should use `flux_bnf::bnf::parse`
```rust
let mut lexer = flux_bnf::bnf::parse(bnf_input).unwrap();     
```

## How to set Lexer Rules
Now that you have `Lexer` we can use that to now set some rules

Before being able to set some rules you will need write a BNF describing the syntax which you can find [here](#bnf)

Here is an example of what our BNF looks like
```
root ::= NumberList
sep ::= " "
number ::= [0-9]+
NumberList ::= number (sep number)*
```

Token input using BNF
`1 2 3`

Token output
```
Some(
    Token {
        name: Some(
            "NumberList",
        ),
        match: "1 2 3",
        range: 0..5,
        children: [
            Token {
                name: Some(
                    "number",
                ),
                match: "1",
                range: 0..1,
            },
            Token {
                name: Some(
                    "number",
                ),
                match: "2",
                range: 2..3,
            },
            Token {
                name: Some(
                    "number",
                ),
                match: "3",
                range: 4..5,
            },
        ],
        ..
    },
)
```
To apply the rules to the `lexer` you simpily do
```rust
lexer.add_rule_for_names(vec!["sep"], CullStrategy::DeleteAll);
```

This is what the tokenized tree would look like before applying the rules
```
Some(
    Token {
        name: Some(
            "NumberList",
        ),
        match: "1 2 3",
        range: 0..5,
        children: [
            Token {
                name: Some(
                    "number",
                ),
                match: "1",
                range: 0..1,
            },
            Token {
                name: Some(
                    "sep",
                ),
                match: " ",
                range: 1..2,
            },
            Token {
                name: Some(
                    "number",
                ),
                match: "2",
                range: 2..3,
            },
            Token {
                name: Some(
                    "sep",
                ),
                match: " ",
                range: 3..4,
            },
            Token {
                name: Some(
                    "number",
                ),
                match: "3",
                range: 4..5,
            },
        ],
        ..
    },
)
```
This is what the tree would look like after the rules were applied. You can see that the seps were taken out of the tree making it easier to read. 
```
Some(
    Token {
        name: Some(
            "NumberList",
        ),
        match: "1 2 3",
        range: 0..5,
        children: [
            Token {
                name: Some(
                    "number",
                ),
                match: "1",
                range: 0..1,
            },
            Token {
                name: Some(
                    "number",
                ),
                match: "2",
                range: 2..3,
            },
            Token {
                name: Some(
                    "number",
                ),
                match: "3",
                range: 4..5,
            },
        ],
        ..
    },
)
```
`CullStrategy` is what we want to do with those name when they are in the list. By default the lexer will insert `CullStrategy::LiftChildren` if no argument is found. 

`CullStrategy::None` - Leaves the tokens alone

`CullStrategy::DeleteAll` - Deletes the token and all of its children

`CullStrategy::DeleteChildren` - Deletes the children of the token but not the token itself

`CullStrategy::LiftChildren` - Deletes the token and replaces it with its children in its parent

`CullStrategy::LiftAtMost(usize)` - Delete the token and replace it with its children only if it has N or less children

# Tokens

To tokenize your input you simpily just do
```rust
lexer.tokenize(test_input).unwrap()
```

However `FluxError` has some options for debug printing to make it much nicer

`{:?}` - Standard Debug

`{:#?}` - Pretty Debug

`{:#}` - User-friendly Debug

`{:+#}` - User-friendly with more details

Pretty Debug Output
```
FluxError {
    description: "expected",
    location: 0,
    match_ref: Some(
        "number",
    ),
}
```
User-friendly with more 
```
FluxError expected `number` at line 1 col 0
 1 2 3
^
```


# BNF

Flux BNF is a syntax that's somewhat like regex, and will have many familiar elements if you know regex.

Every BNF file defines "rules", which are named expressions specifying syntax. Every rule is a name followed by a definition.

```
digit ::= [0-9]
```

This defines a rule called `digit` which matches any single digit character.

```
number ::= [0-9]+
```

This defines a rule called `number` which matches one or more digits repeatedly.

```
boolean ::= "true" | "false"
```

This defines a rule called `boolean` which matches either the literal text `true` or the literal text `false`.

Strings can be prefixed with `i` to make them case-insensitive, like `i"text here"`.

Every BNF file must include a rule called `root`, which will always be expected to match the entire input.

```
root ::= numberList
number ::= [0-9]+
numberList ::= number ("," number)*
```

Here we define that the input is a `numberList`, defined as a number followed by zero or more commas followed by further numbers.
It will match the input `1`, `1,2`, `9,1023854,16765`, but not an empty input, since the first number is non-optional.

Following a matcher with `?` will make it optional, though:

```
root ::= numberList
sep ::= " "+
number ::= [0-9]+
numberList ::= number sep? ("," sep? number)*
```

Here we use `sep` optionally to allow the input to include spaces between numbers, so it will now match things like `1, 2, 3`.

This pattern is very common in BNF, and is called the "delimited list pattern". You can generify it using template rules:

```
delimitedList<elem, delim, whitespace> ::= elem whitespace? (delim whitespace? elem)*

root ::= numberList
sep ::= " "+
number ::= [0-9]+
numberList ::= delimitedList<number, ",", sep>
```

# Matcher types
`[abc]` - Character set matcher, matches any one character in the set

`[a-z]` - Character range matcher, matches any character within the range specified

`[^abc]`, `[^a-z]` - Inverted character set/range, matches any character NOT in the set

`"hello"` - String matcher, matches the literal contents of the quotes (supports escape sequences)

`i"hello"` - Case insensitive string matcher

`<eof>` - Matches the end of the input

`"(" [0-9] ")"` - List matcher, any matchers separated by spaces will be applied in sequence

`"hello" | "hey" | "hi"` - Choice matcher, tries all matchers one-by-one until one succeeds

`!"hello"` - Inverted matcher, `!` asserts that the following matcher cannot be matched at that location

`number+` - Repeating matcher, `+` applies the matcher it's on from 1-unlimited times

`number*` - Repeating matcher, `*` applies the matcher it's on from 0-unlimited times

`number?` - Optional matcher, `?` attempts to match but will proceed even if it fails

`number{3,5}` - Repeating bounds, matches 3-5 times
