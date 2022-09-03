pub struct ChoiceMatcher {
    name: string,
    min_length: usize,
    children: Vec<MatcherRef>,
}