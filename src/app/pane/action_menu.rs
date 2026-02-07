use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    style::Style,
    text::Line,
    widgets::{
        Block, BorderType, Clear, HighlightSpacing, List, ListItem, ListState, Padding,
        StatefulWidget, Widget,
    },
};

use crate::{
    api::story::Story,
    app::{
        cmd::Cmd,
        model::{DataState, UiState},
        msg::ActionMenuMsg,
    },
};

pub struct ActionMenuState {
    pub list_state: ListState,
    pub is_showing: bool,
}

impl Default for ActionMenuState {
    fn default() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            list_state,
            is_showing: false,
        }
    }
}

pub struct ActionMenu;

impl ActionMenu {
    pub fn window_dimensions() -> (usize, usize) {
        let longest_label_len = ActionMenuItem::ALL
            .iter()
            .map(|item| item.label().len())
            .max()
            .unwrap_or_default();

        let n_labels = ActionMenuItem::ALL.len();

        (longest_label_len + 6, n_labels + 6)
    }
}

impl StatefulWidget for ActionMenu {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        let items = ActionMenuItem::ALL.iter().map(|item| {
            let line = Line::from(item.label()).centered();
            ListItem::from(line)
        });

        let block = Block::bordered().border_type(BorderType::Rounded).padding(Padding::vertical(2));

        let list = List::new(items)
            .block(block)
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_style(Style::default().reversed());

        let line_len = area.width - 2;
        let line = Line::from("-".repeat(line_len as usize));

        Clear.render(area, buf);
        StatefulWidget::render(list, area, buf, state);

        let top_line_area = Rect::new(area.x + 1, area.y + 1, area.width - 2, 1);
        let bottom_line_area = Rect::new(area.x + 1, area.y + area.height - 2, area.width - 2, 1);

        line.clone().render(top_line_area, buf);
        line.render(bottom_line_area, buf);
    }
}

#[derive(Clone, Copy)]
pub enum ActionMenuItem {
    OpenNote,
    EditContents,
    OpenTmux,
    SetActive,
}

impl ActionMenuItem {
    pub const ALL: &[Self] = &[
        Self::OpenNote,
        Self::EditContents,
        Self::OpenTmux,
        Self::SetActive,
    ];

    pub fn from_idx(idx: usize) -> ActionMenuItem {
        ActionMenuItem::ALL[idx]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::OpenNote => "Open Note",
            Self::EditContents => "Edit Contents",
            Self::OpenTmux => "Open Tmux Session",
            Self::SetActive => "Set as Active Story",
        }
    }
}

pub fn update(
    ui_state: &mut UiState,
    data_state: &DataState,
    msg: ActionMenuMsg,
    story: &Story,
) -> Vec<Cmd> {
    let item_count = ActionMenuItem::ALL.len();
    let state = &mut ui_state.action_menu.list_state;

    match msg {
        ActionMenuMsg::FocusNext => {
            if item_count == 0 {
                return vec![Cmd::None];
            }
            let current = state.selected().unwrap_or(0);
            let next = if current >= item_count - 1 {
                0 // Wrap to start
            } else {
                current + 1
            };
            state.select(Some(next));
            vec![Cmd::None]
        }

        ActionMenuMsg::FocusPrev => {
            if item_count == 0 {
                return vec![Cmd::None];
            }
            let current = state.selected().unwrap_or(0);
            let prev = if current == 0 {
                item_count - 1 // Wrap to end
            } else {
                current - 1
            };
            state.select(Some(prev));
            vec![Cmd::None]
        }

        ActionMenuMsg::Accept => {
            let mut actions = match ActionMenuItem::from_idx(state.selected().unwrap_or(0)) {
                ActionMenuItem::OpenNote => {
                    vec![Cmd::OpenNote {
                        story: story.clone(),
                        iteration: data_state.current_iteration.clone(),
                    }]
                }

                ActionMenuItem::EditContents => {
                    vec![Cmd::EditStoryContent(story.clone())]
                }

                ActionMenuItem::OpenTmux => {
                    vec![Cmd::OpenTmuxSession {
                        story_name: story.name.clone(),
                    }]
                }

                ActionMenuItem::SetActive => {
                    vec![Cmd::SelectStory(Some(story.clone()))]
                }
            };

            actions.push(Cmd::ActionMenuVisibility(false));
            actions
        }

        ActionMenuMsg::Close => {
            vec![Cmd::ActionMenuVisibility(false)]
        }
    }
}

pub fn key_to_msg(key: KeyEvent) -> Option<ActionMenuMsg> {
    // TODO: consolidate with AppKey
    match key.code {
        KeyCode::Enter => Some(ActionMenuMsg::Accept),
        KeyCode::Char('j') | KeyCode::Down => Some(ActionMenuMsg::FocusNext),
        KeyCode::Char('k') | KeyCode::Up => Some(ActionMenuMsg::FocusPrev),
        KeyCode::Esc | KeyCode::Char('q') => Some(ActionMenuMsg::Close),
        _ => None,
    }
}
