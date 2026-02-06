use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer, layout::Rect, style::Style, text::Line, widgets::{Block, Clear, HighlightSpacing, List, ListItem, ListState, StatefulWidget, Widget}
};

use crate::app::{
    cmd::Cmd,
    model::{DataState, UiState},
    msg::ActionMenuMsg,
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

impl StatefulWidget for ActionMenu {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        let items = ActionMenuItem::ALL
            .iter()
            .map(|item| {
                let line = Line::from(item.label()).centered();
                ListItem::from(line)
            });

        let block = Block::bordered();

        let list = List::new(items)
            .block(block)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_style(Style::default().reversed());

        Clear.render(area, buf);
        StatefulWidget::render(list, area, buf, state);
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

pub fn update(ui_state: &mut UiState, data_state: &DataState, msg: ActionMenuMsg) -> Vec<Cmd> {
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

        ActionMenuMsg::Accept => match ActionMenuItem::from_idx(state.selected().unwrap_or(0)) {
            ActionMenuItem::OpenNote => {
                if let Some(story) = &ui_state.story_list.active_story {
                    vec![Cmd::OpenNote {
                        story: story.clone(),
                        iteration: data_state.current_iteration.clone(),
                    }]
                } else {
                    vec![Cmd::None]
                }
            }

            ActionMenuItem::EditContents => {
                if let Some(story) = &ui_state.story_list.active_story {
                    vec![Cmd::EditStoryContent(story.clone())]
                } else {
                    vec![Cmd::None]
                }
            }

            ActionMenuItem::OpenTmux => {
                if let Some(story) = &ui_state.story_list.active_story {
                    vec![Cmd::OpenTmuxSession {
                        story_name: story.name.clone(),
                    }]
                } else {
                    vec![Cmd::None]
                }
            }

            ActionMenuItem::SetActive => {
                vec![Cmd::SelectStory(ui_state.story_list.active_story.clone())]
            }
        },
    }
}

pub fn key_to_msg(key: KeyEvent) -> Option<ActionMenuMsg> {
    // TODO: consolidate with AppKey
    match key.code {
        KeyCode::Enter => Some(ActionMenuMsg::Accept),
        KeyCode::Char('j') => Some(ActionMenuMsg::FocusNext),
        KeyCode::Char('k') => Some(ActionMenuMsg::FocusPrev),
        _ => None,
    }
}
