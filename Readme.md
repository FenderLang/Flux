# Flux

A lexer generator for any context-free syntax in 100% rust with no dependencies.

Created for [Fender-lang](https://github.com/FenderLang/).

# Lexer

## What is a Lexer
- A Lexer is part of an interpreter that takes sequence of characters and turns them into tokens.

## How to use the Lexer

First define your BNF input
```rust
let bnf_input = include_str!("../src/tests/bnf/fender.bnf");
```

Then you need to setup a matcher to be able to parse the bnf
```rust
    let mut lexer = match bnf::parse(bnf_input){
        Ok(v) => v,
        Err(e) => { 
            //Error Messages
            return;
        },
    };
```

## How to set Lexer Rules
Now that you have `lexer` we can use that to now set some rules
```rust
 lexer.add_rule_for_names(vec!["pow", "add", "mul", "range", "cmp", "and", "or"], CullStrategy::LiftNChildren(1));
```

# Tokens
To tokenize your input we need to first define the result like so
```rust
let res = lexer.tokenize(test_input);
```

Then using res we can match it like this to get our tokenized BNF
```rust
match res {
        Ok(token) => println!("{:#?}", token),
        Err(e) => {
        //Error Messages
        },
    }
```


# BNF

Flux BNF is a syntax that's somewhat like regex, and will have many familiar elements if you know regex.

Every BNF file defines "rules", which are named expressions specifying syntax. Every rule is a name followed by a definition.

```rust
digit ::= [0-9]
```

This defines a rule called `digit` which matches any single digit character.

```rust
number ::= [0-9]+
```

This defines a rule called `number` which matches one or more digits repeatedly.

```rust
boolean ::= "true" | "false"
```

This defines a rule called `boolean` which matches either the literal text `true` or the literal text `false`.

Strings can be prefixed with `i` to make them case-insensitive, like `i"text here"`.

Every BNF file must include a rule called `root`, which will always be expected to match the entire input.

```rust
root ::= numberList
number ::= [0-9]+
numberList ::= number ("," number)*
```

Here we define that the input is a `numberList`, defined as a number followed by zero or more commas followed by further numbers.
It will match the input `1`, `1,2`, `9,1023854,16765`, but not an empty input, since the first number is non-optional.

Following a matcher with `?` will make it optional, though:

```rust
root ::= numberList
sep ::= " "+
number ::= [0-9]+
numberList ::= number sep? ("," sep? number)*
```

Here we use `sep` optionally to allow the input to include spaces between numbers, so it will now match things like `1, 2, 3`.

This pattern is very common in BNF, and is called the "delimited list pattern". You can generify it using template rules:

```rust
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
