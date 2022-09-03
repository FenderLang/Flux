use std::rc::Rc;
use super::Matcher;

pub struct RepeatingMatcher<'a> {
    name: String,
    min: usize,
    max: usize,
    child: Rc<&'a dyn Matcher>,
}