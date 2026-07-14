//! Read-only MCP server over the notes index (`arc.db`). Speaks stdio, so an
//! MCP client spawns `arc mcp` on demand; it reads the same index the daemon
//! maintains, so there's one source of truth and no cross-process sync.

use std::path::PathBuf;

use anyhow::Result;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerInfo};
use rmcp::schemars::{self, JsonSchema};
use rmcp::transport::stdio;
use rmcp::{ErrorData, ServerHandler, ServiceExt, tool, tool_handler, tool_router};
use serde::Deserialize;

use arc_core::Config;

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
}

#[tool_handler]
impl ServerHandler for ArcMcp {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::new(ServerCapabilities::builder().enable_tools().build());
        info.server_info = Implementation::new("arc", env!("CARGO_PKG_VERSION"));
        info.instructions = Some(
            "Query the arc notes vault as a database. Use `query` for SQL over notes, \
             tags, and todos; `get_note` to read a note's full Markdown by id."
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
