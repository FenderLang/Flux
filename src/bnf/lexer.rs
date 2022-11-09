use crate::matchers::MatcherRef;

enum CullStrategy {
    /// Delete the token and all of its children
    DELETE_ALL,
    /// Delete the children of the token but not the token itself
    DELETE_CHILDREN,
    /// Delete the token and replace it with its children in its parent
    LIFT_CHILDREN
}

struct Lexer {
    root: MatcherRef,
}