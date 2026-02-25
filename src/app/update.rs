use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    api::story::get_story_associated_iteration,
    app::{
        App,
        cmd::Cmd,
        model::{LoadingState, ViewType},
        msg::{CreateNoteModalMsg, EpicListMsg, IterationListMsg, Msg},
        pane::{action_menu, create_note_modal, description_modal, epic_list, iteration_list, notes_list, story_list},
    },
    dbg_file,
    error::ErrorInfo,
    keybindings::Key,
};

impl App {
    pub fn update(
        &mut self,
        msg: Msg,
    ) -> Vec<Cmd> {
        dbg_file!("Update: {:?}", msg);

        match msg {
            Msg::Quit => {
                self.exit = true;
                for handle in &self.model.data.async_handles {
                    if !handle.is_finished() {
                        handle.abort();
                    }
                }
                vec![Cmd::None]
            }

            Msg::KeyPressed(key_event) => self.handle_key_input(key_event),

            Msg::StoryList(story_msg) => story_list::update(
                &mut self.model.ui.story_list,
                &self.model.data.stories,
                self.model.data.current_iterations_ref(),
                story_msg,
            ),

            Msg::NotesList(notes_msg) => notes_list::update(
                &mut self.model.ui.notes_list,
                notes_msg,
            ),

            Msg::IterationList(msg) => {
                let current = self.model.data.current_iterations.as_deref().unwrap_or(&[]);
                let all = &self.model.data.iterations;
                iteration_list::update(&mut self.model.ui.iteration_list, current, all, msg)
            }

            Msg::EpicList(msg) => {
                epic_list::update(&mut self.model.ui.epic_list, &self.model.data.epics, msg)
            }

            Msg::StoriesLoaded {
                stories,
                from_cache,
            } => {
                // Only transition to Loaded on fresh API data, not cached
                // This keeps spinner showing during background refresh
                if !from_cache {
                    self.model.ui.loading = LoadingState::Loaded;
                }

                // Select first story if none selected and list is non-empty
                if self.model.ui.story_list.selected_story_id.is_none() && !stories.is_empty() {
                    self.model.ui.story_list.selected_story_id = stories.first().map(|s| s.id);
                }

                if !from_cache
                    && self.model.data.stories.len() == stories.len()
                    && self
                        .model
                        .data
                        .stories
                        .iter()
                        .zip(stories.iter())
                        .all(|(a, b)| a.id == b.id)
                {
                    return vec![Cmd::None];
                }

                self.model.data.stories = stories.clone();

                // Reconcile selection: if selected story no longer exists, select first
                if let Some(selected_id) = self.model.ui.story_list.selected_story_id
                    && !stories.iter().any(|s| s.id == selected_id)
                {
                    self.model.ui.story_list.selected_story_id = stories.first().map(|s| s.id);
                }

                // Reconcile description modal
                if self.model.ui.description_modal.is_showing
                    && let Some(ref modal_story) = self.model.ui.description_modal.story
                {
                    match stories.iter().find(|s| s.id == modal_story.id) {
                        Some(fresh_story) => {
                            // Update modal with fresh data
                            self.model.ui.description_modal.story = Some(fresh_story.clone());
                        }
                        None => {
                            // Story gone — close modal, show error
                            self.model.ui.description_modal.is_showing = false;
                            self.model.ui.description_modal.scroll_view_state = Default::default();
                            self.model.ui.description_modal.story = None;
                            self.model.ui.errors.push(ErrorInfo::new(
                                "Story no longer available".to_string(),
                                "The story was removed or moved out of this iteration".to_string(),
                            ));
                        }
                    }
                }

                // Reconcile action menu
                if self.model.ui.action_menu.is_showing
                    && let Some(target_id) = self.model.ui.action_menu.target_story_id
                    && !stories.iter().any(|s| s.id == target_id)
                {
                    // Story gone — close menu, show error
                    self.model.ui.action_menu.is_showing = false;
                    self.model.ui.action_menu.target_story_id = None;
                    self.model.ui.errors.push(ErrorInfo::new(
                        "Story no longer available".to_string(),
                        "The story was removed or moved out of this iteration".to_string(),
                    ));
                }

                // Reconcile active story
                if let Some(ref active) = self.model.data.active_story
                    && !stories.iter().any(|s| s.id == active.id)
                {
                    // Active story no longer in iteration — clear it
                    self.model.data.active_story = None;
                    self.model.cache.active_story = None;
                    self.model.ui.errors.push(ErrorInfo::new(
                        "Active story cleared".to_string(),
                        "The active story is no longer in the current iteration".to_string(),
                    ));
                }

                self.model.cache.iteration_stories = Some(stories);

                vec![Cmd::WriteCache]
            }

            Msg::EpicsLoaded(mut epics) => {
                epics.sort_by(|a, b| b.created_at.cmp(&a.created_at));

                // Skip re-render if the ID set hasn't changed (same as StoriesLoaded)
                if self.model.data.epics.len() == epics.len()
                    && self
                        .model
                        .data
                        .epics
                        .iter()
                        .zip(epics.iter())
                        .all(|(a, b)| a.id == b.id)
                {
                    return vec![Cmd::None];
                }

                if self.model.ui.epic_list.selected_id.is_none() {
                    self.model.ui.epic_list.selected_id = epics.first().map(|e| e.id);
                }
                self.model.data.epics = epics.clone();
                self.model.cache.epics = epics;
                vec![Cmd::WriteCache]
            }

            Msg::IterationsLoaded(mut iterations) => {
                iterations.sort_by(|a, b| b.start_date.cmp(&a.start_date));
                if self.model.ui.iteration_list.selected_id.is_none() {
                    self.model.ui.iteration_list.selected_id = iterations.first().map(|it| it.id);
                }
                self.model.data.current_iterations = Some(iterations.clone());
                self.model.cache.current_iterations = Some(iterations.clone());
                self.model.ui.loading = LoadingState::FetchingStories;

                dbg_file!(
                    "Got iterations: {:?}",
                    iterations.iter().map(|it| it.id).collect::<Vec<_>>()
                );
                vec![
                    Cmd::WriteCache,
                    Cmd::FetchStories {
                        iteration_ids: iterations.iter().map(|it| it.id).collect(),
                    },
                ]
            }

            Msg::AllIterationsLoaded(mut iterations) => {
                iterations.sort_by(|a, b| b.start_date.cmp(&a.start_date));
                self.model.data.iterations = iterations.clone();
                self.model.cache.iterations = iterations;
                vec![Cmd::WriteCache]
            }

            Msg::SwitchToView(view_type) => {
                self.model.ui.active_view = view_type;
                if view_type == ViewType::Notes {
                    let (daily, stories, iterations, epics, scratch) =
                        notes_list::scan_notes(&self.model.config.notes_dir);
                    self.model.ui.notes_list.daily_notes = daily;
                    self.model.ui.notes_list.story_notes = stories;
                    self.model.ui.notes_list.iteration_notes = iterations;
                    self.model.ui.notes_list.epic_notes = epics;
                    self.model.ui.notes_list.scratch_notes = scratch;
                    if self.model.ui.notes_list.selected_path.is_none() {
                        let nl = &self.model.ui.notes_list;
                        let first = nl.daily_notes.first()
                            .or_else(|| nl.story_notes.first())
                            .or_else(|| nl.iteration_notes.first())
                            .or_else(|| nl.epic_notes.first())
                            .or_else(|| nl.scratch_notes.first())
                            .cloned();
                        self.model.ui.notes_list.selected_path = first;
                    }
                }
                vec![Cmd::None]
            }

            Msg::NoteOpened => {
                if self.model.ui.active_view == ViewType::Notes {
                    let (daily, stories, iterations, epics, scratch) =
                        notes_list::scan_notes(&self.model.config.notes_dir);
                    self.model.ui.notes_list.daily_notes = daily;
                    self.model.ui.notes_list.story_notes = stories;
                    self.model.ui.notes_list.iteration_notes = iterations;
                    self.model.ui.notes_list.epic_notes = epics;
                    self.model.ui.notes_list.scratch_notes = scratch;
                }
                vec![Cmd::None]
            }

            Msg::CacheWritten => vec![Cmd::None],

            Msg::Error(e) => {
                self.model.ui.errors.push(e);
                // Stop loading spinner on error
                self.model.ui.loading = LoadingState::Loaded;
                vec![Cmd::None]
            }

            Msg::ActionMenu(menu_msg) => {
                let story = self
                    .model
                    .ui
                    .action_menu
                    .target_story_id
                    .and_then(|id| self.model.data.stories.iter().find(|s| s.id == id));

                if let Some(hovered_story) = story {
                    action_menu::update(
                        &mut self.model.ui,
                        &self.model.data,
                        menu_msg,
                        hovered_story,
                    )
                } else {
                    // Target story no longer exists, close menu
                    self.model.ui.action_menu.is_showing = false;
                    vec![Cmd::None]
                }
            }

            Msg::ToggleActionMenu => {
                vec![Cmd::ActionMenuVisibility(
                    !self.model.ui.action_menu.is_showing,
                )]
            }

            Msg::ToggleKeybindsPanel => {
                self.model.ui.show_keybinds_panel = !self.model.ui.show_keybinds_panel;
                vec![Cmd::None]
            }

            Msg::DescriptionModal(modal_msg) => {
                description_modal::update(&mut self.model.ui.description_modal, modal_msg)
            }

            Msg::CreateNoteModal(modal_msg) => create_note_modal::update(
                &mut self.model.ui.create_note_modal,
                &self.model.config,
                modal_msg,
            ),
        }
    }

    /// Intercepts keys for search state in Iteration/Epic views.
    ///
    /// Two modes:
    /// - **Active** (`search_active = true`): typing mode. j/k are consumed (not
    ///   navigation), Enter passes through, Esc deactivates but keeps the query.
    /// - **Inactive with query** (`search_active = false`, query non-empty): list is
    ///   filtered but navigation works normally. Esc clears the query entirely.
    fn try_handle_search_key(&mut self, key: KeyEvent) -> Option<Vec<Cmd>> {
        let (search_active, has_query) = match self.model.ui.active_view {
            ViewType::Iterations => (
                self.model.ui.iteration_list.search_active,
                !self.model.ui.iteration_list.search_query.is_empty(),
            ),
            ViewType::Epics => (
                self.model.ui.epic_list.search_active,
                !self.model.ui.epic_list.search_query.is_empty(),
            ),
            _ => return None,
        };

        if search_active {
            let msg = match key.code {
                // Enter still opens the selected item
                KeyCode::Enter => return None,
                // Esc: exit typing mode, keep query so the list stays filtered
                KeyCode::Esc => match self.model.ui.active_view {
                    ViewType::Iterations => Msg::IterationList(IterationListMsg::DeactivateSearch),
                    ViewType::Epics => Msg::EpicList(EpicListMsg::DeactivateSearch),
                    _ => unreachable!(),
                },
                KeyCode::Backspace => match self.model.ui.active_view {
                    ViewType::Iterations => Msg::IterationList(IterationListMsg::SearchBackspace),
                    ViewType::Epics => Msg::EpicList(EpicListMsg::SearchBackspace),
                    _ => unreachable!(),
                },
                KeyCode::Char(c) => match self.model.ui.active_view {
                    ViewType::Iterations => Msg::IterationList(IterationListMsg::SearchInput(c)),
                    ViewType::Epics => Msg::EpicList(EpicListMsg::SearchInput(c)),
                    _ => unreachable!(),
                },
                _ => return Some(vec![Cmd::None]),
            };
            return Some(self.update(msg));
        }

        // Search inactive but query still set: Esc clears it, everything else passes through
        if has_query && key.code == KeyCode::Esc {
            let msg = match self.model.ui.active_view {
                ViewType::Iterations => Msg::IterationList(IterationListMsg::ClearSearch),
                ViewType::Epics => Msg::EpicList(EpicListMsg::ClearSearch),
                _ => unreachable!(),
            };
            return Some(self.update(msg));
        }

        None
    }

    fn handle_key_input(&mut self, key: KeyEvent) -> Vec<Cmd> {
        // Keybinds panel takes highest priority
        if self.model.ui.show_keybinds_panel {
            if matches!(key.code, KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Esc) {
                self.model.ui.show_keybinds_panel = false;
            }
            return vec![Cmd::None];
        }

        // Description modal takes priority (rendered on top of everything)
        if self.model.ui.description_modal.is_showing {
            return if let Some(msg) = description_modal::key_to_msg(key) {
                self.update(Msg::DescriptionModal(msg))
            } else {
                vec![Cmd::None]
            };
        }

        // action menu is rendered as an overlay, so swallows all keybinds when showing
        if self.model.ui.action_menu.is_showing {
            return if let Some(msg) = action_menu::key_to_msg(key) {
                self.update(Msg::ActionMenu(msg))
            } else {
                vec![Cmd::None]
            };
        }

        // Create note modal intercepts all keys when showing
        if self.model.ui.create_note_modal.is_showing {
            return if let Some(modal_msg) = create_note_modal::key_to_msg(key) {
                self.update(Msg::CreateNoteModal(modal_msg))
            } else {
                vec![Cmd::None]
            };
        }

        // Search bar intercepts most keys when active in Iteration/Epic views
        if let Some(cmds) = self.try_handle_search_key(key) {
            return cmds;
        }

        let app_key = Key::from_key_event(key);

        // Global keys
        if let Some(app_key) = app_key {
            match app_key {
                Key::Quit => return self.update(Msg::Quit),
                Key::HelpPanel => return self.update(Msg::ToggleKeybindsPanel),
                Key::ViewNext => {
                    let next = self.model.ui.active_view.next();
                    return self.update(Msg::SwitchToView(next));
                }
                Key::ViewPrev => {
                    let prev = self.model.ui.active_view.prev();
                    return self.update(Msg::SwitchToView(prev));
                }
                Key::DailyNote => {
                    let today = crate::time::today();
                    let path = self.config.notes_dir.join("daily").join(format!("{}.md", today));
                    return vec![Cmd::OpenDailyNote { path }];
                }
                _ => {}
            }
        }

        // Route to active view's key handler
        match self.model.ui.active_view {
            ViewType::Iterations => {
                if key.code == KeyCode::Char('/') {
                    return self.update(Msg::IterationList(IterationListMsg::ActivateSearch));
                }
                if let Some(msg) = iteration_list::key_to_msg(key) {
                    return self.update(Msg::IterationList(msg));
                }
            }
            ViewType::Epics => {
                if key.code == KeyCode::Char('/') {
                    return self.update(Msg::EpicList(EpicListMsg::ActivateSearch));
                }
                if let Some(msg) = epic_list::key_to_msg(key) {
                    return self.update(Msg::EpicList(msg));
                }
            }
            ViewType::Stories => {
                if key.code == KeyCode::Enter {
                    return self.update(Msg::ToggleActionMenu);
                }

                if let Some(app_key) = app_key {
                    match app_key {
                        Key::Description => {
                            let story = self
                                .model
                                .ui
                                .story_list
                                .selected_story_id
                                .and_then(|id| self.model.data.stories.iter().find(|s| s.id == id));

                            if let Some(story) = story {
                                description_modal::open(
                                    &mut self.model.ui.description_modal,
                                    story.clone(),
                                );
                            }
                            return vec![Cmd::None];
                        }
                        Key::IterationNote => {
                            let story = self
                                .model
                                .ui
                                .story_list
                                .selected_story_id
                                .and_then(|id| self.model.data.stories.iter().find(|s| s.id == id));

                            if let Some(story) = story {
                                let iteration = self
                                    .model
                                    .data
                                    .current_iterations_ref()
                                    .and_then(|its| {
                                        get_story_associated_iteration(story.iteration_id, its)
                                    });

                                return match iteration {
                                    Some(it) => vec![Cmd::OpenIterationNote {
                                        iteration_id: it.id,
                                        iteration_name: it.name.clone(),
                                        iteration_app_url: it.app_url.clone(),
                                    }],
                                    None => {
                                        self.model.ui.errors.push(ErrorInfo::new(
                                            "No iteration",
                                            "This story has no associated iteration",
                                        ));
                                        vec![Cmd::None]
                                    }
                                };
                            }
                            return vec![Cmd::None];
                        }
                        _ => {}
                    }
                }

                if let Some(msg) = story_list::key_to_msg(key) {
                    return self.update(Msg::StoryList(msg));
                }
            }
            ViewType::Notes => {
                if key.code == KeyCode::Char('n') {
                    return self.update(Msg::CreateNoteModal(CreateNoteModalMsg::Open));
                }
                if let Some(msg) = notes_list::key_to_msg(key) {
                    return self.update(Msg::NotesList(msg));
                }
            }
            _ => {}
        }

        vec![Cmd::None]
    }
}
