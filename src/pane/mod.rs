mod counter;
pub use counter::CounterBlock;

mod paragraph;
pub use paragraph::ParagraphPane;

mod epic;
pub use epic::EpicPane;

mod list;
pub use list::ListPane;

pub trait Selectable {
    fn is_selected(&self) -> bool;
    fn select(&mut self);
    fn unselect(&mut self);

    /// Returns the number of selectable children within this selectable item.
    /// Default implementation returns 0 for items without children.
    fn num_selectable_children(&self) -> usize {
        0
    }
}
