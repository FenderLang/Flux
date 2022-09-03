pub struct ListMatcher {
    name: String,
    min_length: usize,
    children: Vec<Rc<dyn Matcher>>
}