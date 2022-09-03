pub struct RepeatingMatcher {
    name: string,
    min: usize,
    max: usize,
    child: Rc<&dyn Matcher>,
}