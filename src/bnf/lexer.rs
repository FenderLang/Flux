use crate::matchers::MatcherRef;

enum CullStrategy {
    DELETE_ALL,
    DELETE_CHILDREN,
    LIFT_CHILDREN
}

struct Lexer {
    root: MatcherRef,
}