use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{
    fs::{OpenOptions, create_dir_all, read_to_string},
    process::Command,
};

use anyhow::{Context, Result};
use slugify::slugify;
use tokio::sync::mpsc::UnboundedSender;

use crate::app::model::Model;
use crate::error::ErrorInfo;
use crate::tmux::{session_attach, session_create, session_exists};
use crate::{
    api::{ApiClient, story::Story},
    app::msg::Msg,
    config::Config,
    dbg_file,
    note::Note,
};

#[derive(Debug, Clone)]
pub enum Cmd {
    None,
    OpenNote {
        story_id: i32,
        story_name: String,
        story_app_url: String,
        iteration_app_url: Option<String>,
    },
    WriteCache,
    FetchStories {
        iteration_ids: Vec<i32>,
    },
    EditStoryContent {
        story_id: i32,
        description: String,
    },
    FetchEpics,
    SelectStory(Option<Story>),
    ActionMenuVisibility(bool),
    CreateGitWorktree {
        branch_name: String,
    },
    OpenTmuxSession {
        story_name: String,
    },
    Batch(Vec<Cmd>),
    OpenInBrowser {
        app_url: String,
    },
    OpenIterationNote {
        iteration_id: i32,
        iteration_name: String,
        iteration_app_url: String,
    },
    OpenEpicNote {
        epic_id: i32,
        epic_name: String,
        epic_app_url: String,
    },
    OpenDailyNote {
        path: PathBuf,
    },
    OpenScratchNote {
        path: PathBuf,
        name: String,
    },
}

pub async fn execute(
    cmd: Cmd,
    sender: UnboundedSender<Msg>,
    model: &mut Model,
    api_client: &ApiClient,
) -> Result<()> {
    match cmd {
        Cmd::None => Ok(()),

        Cmd::WriteCache => {
            model.cache.write().await?;
            sender.send(Msg::CacheWritten).ok();
            Ok(())
        }

        Cmd::FetchStories { iteration_ids } => {
            let sender = sender.clone();
            let api_client = api_client.clone();

            let handle = tokio::spawn(async move {
                match api_client.get_owned_iteration_stories(iteration_ids).await {
                    Ok(stories) => {
                        sender
                            .send(Msg::StoriesLoaded {
                                stories,
                                from_cache: false,
                            })
                            .ok();
                    }
                    Err(e) => {
                        let info = ErrorInfo::new(
                            "Failed to get stories for current iteration".to_string(),
                            e.to_string(),
                        );

                        sender.send(Msg::Error(info)).ok();
                    }
                }
            });

            model.data.async_handles.push(handle);

            Ok(())
        }

        Cmd::FetchEpics => {
            let sender = sender.clone();
            let api_client = api_client.clone();

            let handle = tokio::spawn(async move {
                match api_client.get_all_epics_slim(false).await {
                    Ok(epics) => {
                        sender.send(Msg::EpicsLoaded(epics)).ok();
                    }
                    Err(e) => {
                        let info = ErrorInfo::new(
                            "Failed to fetch epics".to_string(),
                            e.to_string(),
                        );
                        sender.send(Msg::Error(info)).ok();
                    }
                }
            });

            model.data.async_handles.push(handle);
            Ok(())
        }

        Cmd::SelectStory(story) => {
            if let Some(active_story) = &model.data.active_story
                && let Some(story) = &story
                && active_story.id == story.id
            {
                model.cache.active_story = None;
                model.data.active_story = None;
            } else {
                model.data.active_story = story.clone();
                model.cache.active_story = story;
            }

            Ok(())
        }

        Cmd::Batch(commands) => {
            for cmd in commands {
                Box::pin(execute(cmd, sender.clone(), model, api_client)).await?;
            }

            Ok(())
        }

        Cmd::OpenTmuxSession { story_name } => {
            let session_name = Story::tmux_session_name(&story_name);
            open_tmux_session(&session_name).await?;
            Ok(())
        }

        Cmd::ActionMenuVisibility(enabled) => {
            model.ui.action_menu.is_showing = enabled;
            if enabled {
                // Capture the currently selected story ID
                model.ui.action_menu.target_story_id = model.ui.story_list.selected_story_id;
            } else {
                model.ui.action_menu.list_state.select(Some(0));
                model.ui.action_menu.target_story_id = None;
            }

            Ok(())
        }

        Cmd::OpenInBrowser { app_url } => {
            open::that(&app_url).with_context(|| format!("Failed to open {} in browser", app_url))
        }

        // TUI-suspending commands are handled in main_loop, not here
        Cmd::OpenNote { .. }
        | Cmd::OpenIterationNote { .. }
        | Cmd::OpenEpicNote { .. }
        | Cmd::EditStoryContent { .. }
        | Cmd::CreateGitWorktree { .. }
        | Cmd::OpenDailyNote { .. }
        | Cmd::OpenScratchNote { .. } => {
            unreachable!("TUI-suspending commands should be handled in main_loop")
        }
    }
}

pub fn open_in_editor(config: &Config, path: &Path) -> anyhow::Result<()> {
    if path.is_dir() {
        anyhow::bail!("Note path: {} is not a file", path.display());
    }

    if let Some(p) = path.parent() {
        create_dir_all(p)?;
    }

    if !path.is_file() {
        File::create(path)?;
    }

    dbg_file!("Opening in editor: {}", path.display());

    let res = Command::new(&config.editor).arg(path).status()?;

    if !res.success() {
        anyhow::bail!("Failed to open {} in editor", path.display());
    }

    Ok(())
}

pub fn open_note_in_editor(
    story_id: i32,
    story_name: String,
    story_app_url: String,
    iteration_app_url: Option<String>,
    config: &Config,
) -> anyhow::Result<()> {
    let note = Note::new(
        &config.notes_dir,
        story_id,
        story_name,
        story_app_url,
        iteration_app_url,
    );

    open_in_editor(config, &note.path)?;

    Ok(())
}

pub fn open_iteration_note_in_editor(
    iteration_id: i32,
    iteration_name: String,
    iteration_app_url: String,
    config: &Config,
) -> anyhow::Result<()> {
    let slug = slugify!(&iteration_name);
    let mut path = config.notes_dir.clone();
    path.push("iterations");
    path.push(format!("{}.md", slug));

    if path.is_dir() {
        anyhow::bail!("Note path: {} is not a file", path.display());
    }
    if let Some(p) = path.parent() {
        create_dir_all(p)?;
    }

    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(&path)?;
    let buf = read_to_string(&path)?;
    if buf.is_empty() {
        let today = crate::time::today();
        let frontmatter = format!(
            "---\niteration_id: it-{}\niteration_link: {}\niteration_name: {}\ncreated: {}\n---\n",
            iteration_id, iteration_app_url, iteration_name, today
        );
        f.write_all(frontmatter.as_bytes())?;
    }

    Command::new(&config.editor).arg(&path).status()?;
    Ok(())
}

pub fn open_epic_note_in_editor(
    epic_id: i32,
    epic_name: String,
    epic_app_url: String,
    config: &Config,
) -> anyhow::Result<()> {
    let slug = slugify!(&epic_name);
    let mut path = config.notes_dir.clone();
    path.push("epics");
    path.push(format!("{}.md", slug));

    if path.is_dir() {
        anyhow::bail!("Note path: {} is not a file", path.display());
    }
    if let Some(p) = path.parent() {
        create_dir_all(p)?;
    }

    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(&path)?;
    let buf = read_to_string(&path)?;
    if buf.is_empty() {
        let today = crate::time::today();
        let frontmatter = format!(
            "---\nepic_id: ep-{}\nepic_link: {}\nepic_name: {}\ncreated: {}\n---\n",
            epic_id, epic_app_url, epic_name, today
        );
        f.write_all(frontmatter.as_bytes())?;
    }

    Command::new(&config.editor).arg(&path).status()?;
    Ok(())
}

pub fn open_daily_note_with_frontmatter(config: &Config, path: &Path) -> anyhow::Result<()> {
    if path.is_dir() {
        anyhow::bail!("Note path: {} is not a file", path.display());
    }

    if let Some(p) = path.parent() {
        create_dir_all(p)?;
    }

    // Write frontmatter if file is new or empty
    let needs_frontmatter = if path.is_file() {
        read_to_string(path)?.is_empty()
    } else {
        true
    };

    if needs_frontmatter {
        let today = crate::time::today();
        let frontmatter = format!("---\ncreated: {}\ntype: daily\n---\n", today);
        let mut f = File::create(path)?;
        f.write_all(frontmatter.as_bytes())?;
    }

    dbg_file!("Opening daily note in editor: {}", path.display());

    let res = Command::new(&config.editor).arg(path).status()?;
    if !res.success() {
        anyhow::bail!("Failed to open {} in editor", path.display());
    }

    Ok(())
}

pub fn open_scratch_note_in_editor(name: &str, path: &Path, config: &Config) -> anyhow::Result<()> {
    if path.is_dir() {
        anyhow::bail!("Note path: {} is not a file", path.display());
    }

    if let Some(p) = path.parent() {
        create_dir_all(p)?;
    }

    let needs_frontmatter = if path.is_file() {
        read_to_string(path)?.is_empty()
    } else {
        true
    };

    if needs_frontmatter {
        let today = crate::time::today();
        let frontmatter = format!("---\nname: {}\ncreated: {}\ntype: scratch\n---\n", name, today);
        let mut f = File::create(path)?;
        f.write_all(frontmatter.as_bytes())?;
    }

    open_in_editor(config, path)
}

pub async fn open_tmux_session(name: &str) -> anyhow::Result<()> {
    if !session_exists(name).await? {
        session_create(name).await?;
    }
    session_attach(name).await?;
    Ok(())
}
