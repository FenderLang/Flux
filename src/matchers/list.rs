use super::MatcherRef;

pub struct ListMatcher {
    name: String,
    min_length: usize,
    children: Vec<MatcherRef>
}