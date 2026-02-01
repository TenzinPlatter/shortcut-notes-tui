use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::StatefulWidget,
};

/// Trait for items that can be rendered in a CustomList
pub trait ListRow {
    /// Render this item into the given area
    /// `is_cursor` - true if the cursor is on this item
    fn render(&self, area: Rect, buf: &mut Buffer, is_cursor: bool);

    /// Height in lines (excluding divider)
    fn height(&self) -> u16;
}

/// State for the list (tracks cursor position and scroll offset)
#[derive(Default, Clone)]
pub struct ListState {
    pub selected: Option<usize>,
    offset: usize,
}

impl ListState {
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
    }

    pub fn next(&mut self, len: usize) {
        let i = match self.selected {
            Some(i) => (i + 1).min(len.saturating_sub(1)),
            None => 0,
        };
        self.selected = Some(i);
    }

    pub fn previous(&mut self) {
        let i = match self.selected {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.selected = Some(i);
    }
}

/// A list widget that supports custom item rendering and dividers
pub struct CustomList<'a, T> {
    items: &'a [T],
    divider: Option<Line<'a>>,
    highlight_style: Style,
}

impl<'a, T: ListRow> CustomList<'a, T> {
    pub fn new(items: &'a [T]) -> Self {
        Self {
            items,
            divider: None,
            highlight_style: Style::default(),
        }
    }

    pub fn divider(mut self, divider: Line<'a>) -> Self {
        self.divider = Some(divider);
        self
    }

    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }
}

impl<T: ListRow> StatefulWidget for CustomList<'_, T> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if area.height == 0 || area.width == 0 || self.items.is_empty() {
            return;
        }

        let divider_height = if self.divider.is_some() { 1 } else { 0 };

        // Bounds checking: ensure selected index is valid
        if let Some(selected) = state.selected {
            if selected >= self.items.len() {
                state.selected = Some(self.items.len().saturating_sub(1));
            }
        }

        // Calculate cumulative heights to determine scroll offset
        if let Some(selected) = state.selected {
            let mut cumulative_heights: Vec<u16> = Vec::with_capacity(self.items.len());
            let mut total: u16 = 0;

            for (i, item) in self.items.iter().enumerate() {
                cumulative_heights.push(total);
                total += item.height();
                if i < self.items.len() - 1 {
                    total += divider_height;
                }
            }

            // Calculate the start and end positions of the selected item
            let selected_start = cumulative_heights[selected];
            let selected_end = selected_start + self.items[selected].height();

            // Calculate the visible range based on current offset
            let mut visible_start: u16 = 0;
            for i in 0..state.offset {
                if i < self.items.len() {
                    visible_start += self.items[i].height();
                    if i < self.items.len() - 1 {
                        visible_start += divider_height;
                    }
                }
            }
            let visible_end = visible_start + area.height;

            // Adjust offset to keep selected item visible
            if selected_end > visible_end {
                // Selected item is below visible area, scroll down
                while state.offset < self.items.len() {
                    let mut new_visible_start: u16 = 0;
                    for i in 0..state.offset + 1 {
                        if i < self.items.len() {
                            new_visible_start += self.items[i].height();
                            if i < self.items.len() - 1 {
                                new_visible_start += divider_height;
                            }
                        }
                    }
                    let new_visible_end = new_visible_start + area.height;

                    if selected_end <= new_visible_end {
                        state.offset += 1;
                        break;
                    }
                    state.offset += 1;
                }
            } else if selected_start < visible_start {
                // Selected item is above visible area, scroll up
                state.offset = selected;
            }
        }

        // Render items starting from offset
        let mut y = area.y;

        for (i, item) in self.items.iter().enumerate().skip(state.offset) {
            let item_height = item.height();

            // Check if we have room for this item
            if y + item_height > area.y + area.height {
                break;
            }

            let is_cursor = state.selected == Some(i);

            // Create item area
            let item_area = Rect {
                x: area.x,
                y,
                width: area.width,
                height: item_height,
            };

            // Apply highlight style if cursor
            if is_cursor {
                // Fill background with highlight style
                for row in y..y + item_height {
                    for col in area.x..area.x + area.width {
                        buf[(col, row)].set_style(self.highlight_style);
                    }
                }
            }

            // Let item render itself
            item.render(item_area, buf, is_cursor);

            y += item_height;

            // Render divider if not last item and there's room
            if let Some(ref divider) = self.divider {
                if i < self.items.len() - 1 && y < area.y + area.height {
                    // Render divider line
                    buf.set_line(area.x, y, divider, area.width);
                    y += 1;
                }
            }
        }
    }
}
