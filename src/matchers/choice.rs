use super::MatcherRef;

pub struct ChoiceMatcher {
    name: String,
    min_length: usize,
    children: Vec<MatcherRef>,
}