use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph, StatefulWidget, Widget, WidgetRef},
};
use tui_widget_list::{ListBuilder, ListState, ListView};

use crate::{
    app::{
        model::TodosListState,
        pane::todos_list::group_todos_by_section,
    },
    time::today,
    todos::Todo,
};

pub struct TodosListView<'a> {
    todos: &'a [Todo],
    state: &'a TodosListState,
}

impl<'a> TodosListView<'a> {
    pub fn new(todos: &'a [Todo], state: &'a TodosListState) -> Self {
        Self { todos, state }
    }
}

impl WidgetRef for TodosListView<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        if self.todos.is_empty() {
            let block = Block::bordered().border_set(border::THICK);
            let inner = block.inner(area);
            block.render(area, buf);

            let message = "No todos yet. Press 'n' to add one.";
            let style = Style::default().fg(Color::DarkGray);
            let paragraph = Paragraph::new(message)
                .style(style)
                .alignment(Alignment::Center);

            if inner.height > 0 {
                let centered_area =
                    Rect::new(inner.x, inner.y + inner.height / 2, inner.width, 1);
                paragraph.render(centered_area, buf);
            }
            return;
        }

        let today = today();
        let sections = group_todos_by_section(self.todos, today).ordered_sections();

        // Calculate layout constraints for sections:
        // header (1) + bordered list (items*2 + 4 for border+padding) + spacing (1)
        let mut constraints = Vec::new();
        for section in &sections {
            constraints.push(Constraint::Length(1));
            constraints.push(Constraint::Length((section.todos.len() * 2 + 4) as u16));
            constraints.push(Constraint::Length(1));
        }

        if !constraints.is_empty() {
            constraints.pop();
        }
        constraints.push(Constraint::Min(0));

        let section_areas = Layout::vertical(constraints).split(area);

        let mut area_index = 0;
        for section in &sections {
            // Render section header
            let header_area = section_areas[area_index];
            area_index += 1;

            let header_text = section.kind.header();

            let header_style = Style::default().fg(Color::DarkGray);
            let display = format!(" ── {} ──", header_text);
            let title_line = Line::from(display).style(header_style);
            buf.set_line(
                header_area.x,
                header_area.y,
                &title_line,
                header_area.width,
            );

            // Render bordered todos list
            let list_area = section_areas[area_index];
            area_index += 1;

            let list_block = Block::bordered()
                .border_set(border::THICK)
                .padding(Padding::vertical(1));
            let todos_area = list_block.inner(list_area);
            list_block.render(list_area, buf);

            let section_todos: Vec<_> = section.todos.clone();
            let selected_id = self.state.selected_id;

            let builder = ListBuilder::new(move |context| {
                let todo = &section_todos[context.index];
                let is_selected = selected_id.is_some_and(|id| id == todo.id);
                let source_label = match &todo.source {
                    crate::todos::TodoSource::NoteParsed { file, line, .. } => {
                        let stem = file
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("?");
                        Some(format!(" [{}:{}]", stem, line))
                    }
                    crate::todos::TodoSource::Manual => None,
                };
                let widget = TodoItemWidget {
                    text: todo.text.clone(),
                    completed: todo.completed,
                    is_selected,
                    source_label,
                };
                (widget, 2)
            });

            let list = ListView::new(builder, section.todos.len());

            let mut list_state = ListState::default();
            if let Some(selected_id) = self.state.selected_id
                && let Some(pos) = section.todos.iter().position(|t| t.id == selected_id)
            {
                list_state.select(Some(pos));
            }

            StatefulWidget::render(list, todos_area, buf, &mut list_state);

            // Skip spacing area
            area_index += 1;
        }
    }
}

struct TodoItemWidget {
    text: String,
    completed: bool,
    is_selected: bool,
    source_label: Option<String>,
}

impl Widget for TodoItemWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 1 {
            return;
        }

        let checkbox = if self.completed { "☑" } else { "☐" };

        let mut spans = Vec::new();

        let base_style = if self.completed {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };

        let name_style = if self.is_selected && !self.completed {
            base_style.bold()
        } else {
            base_style
        };

        spans.push(Span::styled(format!("{} ", checkbox), base_style));
        spans.push(Span::styled(self.text.clone(), name_style));

        if let Some(label) = &self.source_label {
            spans.push(Span::styled(
                label.clone(),
                Style::default().fg(Color::DarkGray),
            ));
        }

        let content = Line::from(spans);
        buf.set_line(area.x, area.y, &content, area.width);

        // Render divider on second line
        if area.height >= 2 {
            let divider_style = if self.is_selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let divider = Line::from("─".repeat(area.width as usize)).style(divider_style);
            buf.set_line(area.x, area.y + 1, &divider, area.width);
        }
    }
}
