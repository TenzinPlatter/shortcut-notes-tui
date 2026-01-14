mod counter;
pub use counter::CounterBlock;

mod paragraph;
pub use paragraph::ParagraphBlock;

pub trait Selectable {
    fn is_selected(&self) -> bool;
    fn select(&mut self);
    fn unselect(&mut self);
}
