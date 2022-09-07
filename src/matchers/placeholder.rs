use super::Matcher;


pub struct PlaceholderMatcher {

    name: String,

}

impl PlaceholderMatcher {
    pub fn new(name: String) -> PlaceholderMatcher {
        PlaceholderMatcher {name}
    }
}

impl Matcher for PlaceholderMatcher {
    fn apply(&self, source: std::rc::Rc<Vec<char>>, pos: usize) -> crate::error::Result<crate::tokens::Token> {
        unreachable!()
    }

    fn min_length(&self) -> usize {
        unreachable!()
    }

    fn get_name(&self) -> Option<&str> {
        Some(&self.name)
    }

    fn set_name(&mut self, new_name: String) {
        unreachable!()
    }

    fn is_placeholder(&self) -> bool {
        true
    }
}