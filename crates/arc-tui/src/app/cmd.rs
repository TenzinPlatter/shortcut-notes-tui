use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{
    fs::{OpenOptions, create_dir_all, read_to_string},
    process::Command,
};

use slugify::slugify;

use crate::{api::story::Story, note::Note, tmux, zellij};
use arc_core::{Config, Mux, dbg_file};

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
    WriteTodos,
    SyncNoteCheckbox {
        file: PathBuf,
        text: String,
        complete: bool,
    },
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

    if let Some(p) = note.path.parent() {
        create_dir_all(p)?;
    }

    let needs_frontmatter = if note.path.is_file() {
        read_to_string(&note.path)?.is_empty()
    } else {
        true
    };

    if needs_frontmatter {
        let frontmatter_string = format!("---\n{}---", note.frontmatter.to_yaml_string()?);
        std::fs::write(&note.path, frontmatter_string)?;
    }

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
        let today = arc_core::time::today();
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
        let today = arc_core::time::today();
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
        let today = arc_core::time::today();
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
        let today = arc_core::time::today();
        let frontmatter = format!(
            "---\nname: {}\ncreated: {}\ntype: scratch\n---\n",
            name, today
        );
        let mut f = File::create(path)?;
        f.write_all(frontmatter.as_bytes())?;
    }

    open_in_editor(config, path)
}

pub async fn open_mux_session(name: &str, mux: &Mux) -> anyhow::Result<()> {
    match mux {
        Mux::Tmux => {
            if !tmux::session_exists(name).await? {
                tmux::session_create(name).await?;
            }
            tmux::session_attach(name).await?;
        }
        Mux::Zellij => {
            if !zellij::session_exists(name).await? {
                zellij::session_create(name).await?;
            }
            zellij::session_attach(name).await?;
        }
    }
    Ok(())
}

pub fn open_mux_session_sync(name: &str, mux: &Mux) -> anyhow::Result<()> {
    use std::process::Command;

    match mux {
        Mux::Tmux => {
            let out = Command::new("tmux")
                .args(["list-sessions", "-F", "#{session_name}"])
                .output()?;
            let exists = String::from_utf8_lossy(&out.stdout)
                .lines()
                .any(|l| l.trim() == name);

            if !exists {
                Command::new("tmux")
                    .args(["new-session", "-d", "-s", name])
                    .status()?;
            }

            let cmd = if tmux::attatched_to_session() {
                "switch-client"
            } else {
                "attach-session"
            };
            Command::new("tmux").args([cmd, "-t", name]).status()?;
        }
        Mux::Zellij => {
            let out = Command::new("zellij").arg("list-sessions").output();
            let exists = match out {
                Ok(out) if out.status.success() => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    stdout
                        .lines()
                        .any(|l| l.split_whitespace().next() == Some(name))
                }
                _ => false,
            };

            if !exists {
                Command::new("zellij")
                    .args(["attach", "--create-background", name])
                    .status()?;
            }

            Command::new("zellij").args(["attach", name]).status()?;
        }
    }
    Ok(())
}
