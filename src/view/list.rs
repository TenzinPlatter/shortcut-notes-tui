use ratatui::widgets::ListItem;

pub trait ExpandableListItem {
    fn as_list_item(&self, expanded: bool) -> ListItem<'static>;
}
