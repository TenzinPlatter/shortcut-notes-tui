//! SQLite index of the notes vault — a queryable projection the daemon keeps
//! fresh. `todos.json` remains the authoritative todo store; this mirrors it
//! alongside note metadata + Shortcut links so the MCP server can answer SQL.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rusqlite::OpenFlags;
pub use rusqlite::Connection;

use arc_notes::note::frontmatter::{EntityLink, Frontmatter};
use arc_notes::todos::{Todo, TodoSource};

const DB_FILE: &str = "arc.db";

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS notes (
    path          TEXT PRIMARY KEY,
    id            TEXT,
    title         TEXT,
    created       TEXT,
    type          TEXT,
    link_kind     TEXT,
    entity_id     INTEGER,
    entity_url    TEXT,
    entity_name   TEXT,
    iteration_url TEXT,
    epic_url      TEXT
);
CREATE TABLE IF NOT EXISTS note_tags (
    path TEXT NOT NULL,
    tag  TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS todos (
    id        TEXT PRIMARY KEY,
    text      TEXT,
    date      TEXT,
    completed INTEGER,
    source    TEXT,
    file      TEXT,
    line      INTEGER,
    position  INTEGER
);
CREATE INDEX IF NOT EXISTS idx_note_tags_path ON note_tags(path);
CREATE INDEX IF NOT EXISTS idx_notes_entity ON notes(link_kind, entity_id);
";

fn db_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join(DB_FILE)
}

/// Open (creating + migrating) the writer connection. WAL so a concurrent
/// read-only reader (the MCP server) never blocks the daemon's writes.
pub fn open(cache_dir: &Path) -> Result<Connection> {
    std::fs::create_dir_all(cache_dir).ok();
    let conn = Connection::open(db_path(cache_dir))
        .with_context(|| format!("open index at {}", db_path(cache_dir).display()))?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.execute_batch(SCHEMA)?;
    Ok(conn)
}

/// Open a read-only connection for querying. Errors if the index doesn't exist
/// yet (the daemon creates it on first run).
pub fn open_readonly(cache_dir: &Path) -> Result<Connection> {
    let conn = Connection::open_with_flags(db_path(cache_dir), OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("open index (ro) at {}", db_path(cache_dir).display()))?;
    Ok(conn)
}

/// Upsert one note's frontmatter + tags. `rel` is the notes-dir-relative path.
pub fn reindex_note(conn: &Connection, rel: &Path, content: &str) -> Result<()> {
    let path = rel.to_string_lossy().to_string();
    let fm = Frontmatter::from_note(content);
    let title = note_title(content, fm.as_ref());

    let (id, created, note_type, link) = match &fm {
        Some(f) => (
            Some(f.id.clone()),
            f.created.map(|d| d.to_string()),
            f.note_type.clone(),
            &f.link,
        ),
        None => (None, None, None, &EntityLink::None),
    };

    let (kind, eid, eurl, ename, it_url, ep_url) = match link {
        EntityLink::None => (None, None, None, None, None, None),
        EntityLink::Story {
            id,
            url,
            name,
            iteration_url,
            epic_url,
        } => (
            Some("story"),
            Some(*id),
            Some(url.clone()),
            name.clone(),
            iteration_url.clone(),
            epic_url.clone(),
        ),
        EntityLink::Iteration { id, url, name } => (
            Some("iteration"),
            Some(*id),
            Some(url.clone()),
            name.clone(),
            None,
            None,
        ),
        EntityLink::Epic { id, url, name } => (
            Some("epic"),
            Some(*id),
            Some(url.clone()),
            name.clone(),
            None,
            None,
        ),
    };

    conn.execute(
        "INSERT OR REPLACE INTO notes
         (path, id, title, created, type, link_kind, entity_id, entity_url, entity_name, iteration_url, epic_url)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![path, id, title, created, note_type, kind, eid, eurl, ename, it_url, ep_url],
    )?;
    conn.execute("DELETE FROM note_tags WHERE path = ?1", [&path])?;
    if let Some(f) = &fm {
        for tag in &f.tags {
            conn.execute(
                "INSERT INTO note_tags (path, tag) VALUES (?1, ?2)",
                rusqlite::params![path, tag],
            )?;
        }
    }
    Ok(())
}

/// Drop a note (and its tags) from the index.
pub fn remove_note(conn: &Connection, rel: &Path) -> Result<()> {
    let path = rel.to_string_lossy().to_string();
    conn.execute("DELETE FROM notes WHERE path = ?1", [&path])?;
    conn.execute("DELETE FROM note_tags WHERE path = ?1", [&path])?;
    Ok(())
}

/// Replace the whole todos mirror. `todos.json` stays authoritative; this is a
/// read-only projection for querying, refreshed whenever the daemon has the
/// full list in hand.
pub fn mirror_todos(conn: &mut Connection, todos: &[Todo]) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM todos", [])?;
    for (pos, t) in todos.iter().enumerate() {
        let (source, file, line) = match &t.source {
            TodoSource::Manual => ("manual", None, None),
            TodoSource::NoteParsed { file, line, .. } => {
                ("note", Some(file.to_string_lossy().to_string()), Some(*line))
            }
        };
        tx.execute(
            "INSERT INTO todos (id, text, date, completed, source, file, line, position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                t.id.to_string(),
                t.text,
                t.date.map(|d| d.to_string()),
                t.completed as i64,
                source,
                file,
                line,
                pos as i64,
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Run a read-only SELECT and return the rows as a JSON array of objects.
/// Rejects anything that isn't a read — belt to the read-only connection's
/// braces so a caller can't mutate even if handed a writer connection.
pub fn query(conn: &Connection, sql: &str) -> Result<String> {
    if !is_read_only(sql) {
        anyhow::bail!("only read-only queries (SELECT/WITH/EXPLAIN/PRAGMA) are allowed");
    }
    let mut stmt = conn.prepare(sql)?;
    let cols: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
    let mut rows = stmt.query([])?;
    let mut out: Vec<serde_json::Value> = Vec::new();
    while let Some(row) = rows.next()? {
        let mut obj = serde_json::Map::new();
        for (i, name) in cols.iter().enumerate() {
            obj.insert(name.clone(), value_to_json(row.get_ref(i)?));
        }
        out.push(serde_json::Value::Object(obj));
    }
    Ok(serde_json::to_string(&out)?)
}

/// Full markdown body of a note by its Obsidian `id` (frontmatter id / slug).
pub fn get_note(conn: &Connection, notes_dir: &Path, id: &str) -> Result<Option<String>> {
    let rel: Option<String> = conn
        .query_row("SELECT path FROM notes WHERE id = ?1 LIMIT 1", [id], |r| {
            r.get(0)
        })
        .ok();
    let Some(rel) = rel else { return Ok(None) };
    let abs = notes_dir.join(rel);
    Ok(std::fs::read_to_string(abs).ok())
}

fn is_read_only(sql: &str) -> bool {
    let head = sql.split_whitespace().next().unwrap_or("");
    matches!(
        head.to_ascii_uppercase().as_str(),
        "SELECT" | "WITH" | "EXPLAIN" | "PRAGMA"
    )
}

fn value_to_json(v: rusqlite::types::ValueRef) -> serde_json::Value {
    use rusqlite::types::ValueRef;
    match v {
        ValueRef::Null => serde_json::Value::Null,
        ValueRef::Integer(i) => serde_json::Value::from(i),
        ValueRef::Real(f) => serde_json::Value::from(f),
        ValueRef::Text(t) => serde_json::Value::from(String::from_utf8_lossy(t).into_owned()),
        ValueRef::Blob(b) => serde_json::Value::from(format!("<{} bytes>", b.len())),
    }
}

/// First `# ` heading, else the frontmatter id, else the file stem later.
fn note_title(content: &str, fm: Option<&Frontmatter>) -> Option<String> {
    for line in content.lines() {
        if let Some(h) = line.strip_prefix("# ") {
            return Some(h.trim().to_string());
        }
    }
    fm.map(|f| f.id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn indexes_and_queries_a_story_note() {
        let tmp = tempfile::tempdir().unwrap();
        let conn = open(tmp.path()).unwrap();
        let content = "---\nid: my-story\ntags: [backend]\ncreated: \"2026-03-31\"\nstory_id: sc-19559\nstory_link: https://s/19559\n---\n\n# Ship the thing\n\nbody";
        reindex_note(&conn, Path::new("stories/my-story.md"), content).unwrap();

        let rows = query(
            &conn,
            "SELECT id, title, entity_id, link_kind FROM notes WHERE link_kind = 'story'",
        )
        .unwrap();
        assert!(rows.contains("my-story"));
        assert!(rows.contains("Ship the thing"));
        assert!(rows.contains("19559"));

        let tags = query(&conn, "SELECT tag FROM note_tags").unwrap();
        assert!(tags.contains("backend"));
    }

    #[test]
    fn reindex_replaces_not_duplicates() {
        let tmp = tempfile::tempdir().unwrap();
        let conn = open(tmp.path()).unwrap();
        let p = Path::new("a.md");
        reindex_note(&conn, p, "---\nid: a\n---\n").unwrap();
        reindex_note(&conn, p, "---\nid: a\n---\n").unwrap();
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM notes", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn mirrors_and_gets_note_and_rejects_writes() {
        let tmp = tempfile::tempdir().unwrap();
        let notes = tmp.path().join("notes");
        std::fs::create_dir_all(notes.join("stories")).unwrap();
        std::fs::write(notes.join("stories/a.md"), "---\nid: a\n---\n# Hi\n").unwrap();

        let mut conn = open(tmp.path()).unwrap();
        reindex_note(&conn, Path::new("stories/a.md"), "---\nid: a\n---\n# Hi\n").unwrap();

        let todos = vec![Todo::new_manual(
            "do it".into(),
            NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        )];
        mirror_todos(&mut conn, &todos).unwrap();
        let got = query(&conn, "SELECT text FROM todos").unwrap();
        assert!(got.contains("do it"));

        let body = get_note(&conn, &notes, "a").unwrap().unwrap();
        assert!(body.contains("# Hi"));

        assert!(query(&conn, "DELETE FROM todos").is_err());
    }
}
