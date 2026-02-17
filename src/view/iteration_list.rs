use crate::api::iteration::Iteration;

pub struct IterationListView<'a> {
    pub iterations: &'a [Iteration],
}

impl<'a> IterationListView<'a> {
    pub fn new(iterations: &'a [Iteration]) -> IterationListView<'a> {
        IterationListView {
            iterations,
        }
    }
}
