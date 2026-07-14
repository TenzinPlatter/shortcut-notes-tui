//! Read-only MCP server over the notes index (`arc.db`). Speaks stdio, so an
//! MCP client spawns `arc mcp` on demand; it reads the same index the daemon
//! maintains, so there's one source of truth and no cross-process sync.

use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use chrono::NaiveDate;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerInfo};
use rmcp::schemars::{self, JsonSchema};
use rmcp::transport::stdio;
use rmcp::{ErrorData, ServerHandler, ServiceExt, tool, tool_handler, tool_router};
use serde::Deserialize;
use slugify::slugify;
use uuid::Uuid;

use arc_core::Config;
use arc_notes::note::frontmatter::Frontmatter;
use arc_notes::todos::{Todo, TodoSource};

#[derive(Deserialize, JsonSchema)]
struct QueryArgs {
    /// A read-only SQL statement (SELECT/WITH/EXPLAIN/PRAGMA).
    sql: String,
}

#[derive(Deserialize, JsonSchema)]
struct GetNoteArgs {
    /// The note's Obsidian `id` (frontmatter `id`, usually the filename slug).
    id: String,
}

#[derive(Deserialize, JsonSchema)]
struct AppendNoteArgs {
    /// Obsidian `id` of an existing note to append to.
    id: String,
    /// Markdown appended below the note's current content.
    content: String,
}

#[derive(Deserialize, JsonSchema)]
struct CreateNoteArgs {
    /// Human title; also the `# ` heading. The slug becomes the note id/filename.
    title: String,
    /// Markdown body placed under the heading.
    body: String,
    /// Optional frontmatter tags.
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
struct AddTodoArgs {
    /// Todo text.
    text: String,
    /// Optional due date, `YYYY-MM-DD`.
    #[serde(default)]
    date: Option<String>,
}

#[derive(Clone)]
pub struct ArcMcp {
    cache_dir: PathBuf,
    notes_dir: PathBuf,
}

#[tool_router]
impl ArcMcp {
    pub fn new(cache_dir: PathBuf, notes_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            notes_dir,
        }
    }

    #[tool(
        description = "Run a read-only SQL query against the notes index and return rows as a JSON array of objects. Tables: notes(path, id, title, created, type, link_kind['story'|'iteration'|'epic'|null], entity_id, entity_url, entity_name, iteration_url, epic_url); note_tags(path, tag); todos(id, text, date, completed, source['manual'|'note'], file, line, position). Only SELECT/WITH/EXPLAIN/PRAGMA are permitted."
    )]
    async fn query(
        &self,
        Parameters(QueryArgs { sql }): Parameters<QueryArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let cache_dir = self.cache_dir.clone();
        let rows = tokio::task::spawn_blocking(move || -> Result<String> {
            let conn = arc_index::open_readonly(&cache_dir)?;
            arc_index::query(&conn, &sql)
        })
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?
        .map_err(|e| ErrorData::invalid_params(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(rows)]))
    }

    #[tool(
        description = "Return the full Markdown body of a note by its Obsidian id (the `id` column from the notes table)."
    )]
    async fn get_note(
        &self,
        Parameters(GetNoteArgs { id }): Parameters<GetNoteArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let cache_dir = self.cache_dir.clone();
        let notes_dir = self.notes_dir.clone();
        let body = tokio::task::spawn_blocking(move || -> Result<Option<String>> {
            let conn = arc_index::open_readonly(&cache_dir)?;
            arc_index::get_note(&conn, &notes_dir, &id)
        })
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        match body {
            Some(text) => Ok(CallToolResult::success(vec![ContentBlock::text(text)])),
            None => Ok(CallToolResult::error(vec![ContentBlock::text(
                "note not found",
            )])),
        }
    }

    #[tool(
        description = "Append Markdown to an existing note (found by its Obsidian id). Use this to persist context or progress into a note. Non-destructive — content is added below what's already there."
    )]
    async fn append_note(
        &self,
        Parameters(AppendNoteArgs { id, content }): Parameters<AppendNoteArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let (cache_dir, notes_dir) = (self.cache_dir.clone(), self.notes_dir.clone());
        tokio::task::spawn_blocking(move || append_note(&cache_dir, &notes_dir, &id, &content))
            .await
            .map_err(internal)?
            .map_err(invalid)?;
        Ok(CallToolResult::success(vec![ContentBlock::text("appended")]))
    }

    #[tool(
        description = "Create a new general note (no Shortcut link). Returns the new note's id. Errors if a note with the same slug already exists — append_note to add to it instead."
    )]
    async fn create_note(
        &self,
        Parameters(CreateNoteArgs { title, body, tags }): Parameters<CreateNoteArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let (cache_dir, notes_dir) = (self.cache_dir.clone(), self.notes_dir.clone());
        let id = tokio::task::spawn_blocking(move || {
            create_note(&cache_dir, &notes_dir, &title, &body, tags)
        })
        .await
        .map_err(internal)?
        .map_err(invalid)?;
        Ok(CallToolResult::success(vec![ContentBlock::text(id)]))
    }

    #[tool(
        description = "Add a manual todo. Shows up in the arc TUI and is indexed for querying. Optional due date as YYYY-MM-DD."
    )]
    async fn add_todo(
        &self,
        Parameters(AddTodoArgs { text, date }): Parameters<AddTodoArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let date = match date.as_deref() {
            Some(d) => Some(
                NaiveDate::parse_from_str(d, "%Y-%m-%d")
                    .map_err(|e| ErrorData::invalid_params(format!("bad date: {e}"), None))?,
            ),
            None => None,
        };
        let todo = Todo {
            id: Uuid::new_v4(),
            text,
            date,
            completed: false,
            source: TodoSource::Manual,
        };
        let updated = arc_notes::todos::modify_with_lock(&self.cache_dir, move |list| {
            list.push(todo)
        })
        .await
        .map_err(internal)?;

        let cache_dir = self.cache_dir.clone();
        tokio::task::spawn_blocking(move || -> Result<()> {
            let mut conn = arc_index::open(&cache_dir)?;
            arc_index::mirror_todos(&mut conn, &updated)
        })
        .await
        .map_err(internal)?
        .map_err(internal)?;
        Ok(CallToolResult::success(vec![ContentBlock::text("added")]))
    }
}

fn internal(e: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(e.to_string(), None)
}

fn invalid(e: impl std::fmt::Display) -> ErrorData {
    ErrorData::invalid_params(e.to_string(), None)
}

/// Append markdown to an existing note and refresh its index row.
fn append_note(cache_dir: &Path, notes_dir: &Path, id: &str, content: &str) -> Result<()> {
    let conn = arc_index::open(cache_dir)?;
    let rel = arc_index::note_path(&conn, id)?
        .ok_or_else(|| anyhow::anyhow!("no note with id '{id}'"))?;
    let abs = notes_dir.join(&rel);
    let mut body = std::fs::read_to_string(&abs)?;
    if !body.ends_with('\n') {
        body.push('\n');
    }
    body.push('\n');
    body.push_str(content.trim_end());
    body.push('\n');
    std::fs::write(&abs, &body)?;
    arc_index::reindex_note(&conn, Path::new(&rel), &body)?;
    Ok(())
}

/// Create a general note file + index row; returns its id (slug).
fn create_note(
    cache_dir: &Path,
    notes_dir: &Path,
    title: &str,
    body: &str,
    tags: Vec<String>,
) -> Result<String> {
    let slug = slugify!(title);
    if slug.is_empty() {
        bail!("title produced an empty slug");
    }
    let rel = format!("{slug}.md");
    let abs = notes_dir.join(&rel);
    if abs.exists() {
        bail!("note '{slug}' already exists; use append_note");
    }
    let mut content = Frontmatter::general(slug.clone(), tags).to_block()?;
    content.push_str(&format!("\n# {title}\n\n{body}\n"));
    if let Some(p) = abs.parent() {
        std::fs::create_dir_all(p)?;
    }
    std::fs::write(&abs, &content)?;
    let conn = arc_index::open(cache_dir)?;
    arc_index::reindex_note(&conn, Path::new(&rel), &content)?;
    Ok(slug)
}

#[tool_handler]
impl ServerHandler for ArcMcp {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::new(ServerCapabilities::builder().enable_tools().build());
        info.server_info = Implementation::new("arc", env!("CARGO_PKG_VERSION"));
        info.instructions = Some(
            "The arc notes vault as a database. Read: `query` (read-only SQL over notes, \
             tags, todos) and `get_note` (full Markdown by id). Write: `append_note` to add \
             context to a note, `create_note` for a new note, `add_todo` for a todo."
                .into(),
        );
        info
    }
}

/// Serve the MCP protocol over stdio until the client disconnects.
pub async fn run_stdio(config: &Config) -> Result<()> {
    // Ensure the index exists (empty is fine) so read-only opens don't fail
    // before the daemon's first run.
    arc_index::open(&config.cache_dir)?;

    let server = ArcMcp::new(config.cache_dir.clone(), config.notes_dir.clone());
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
