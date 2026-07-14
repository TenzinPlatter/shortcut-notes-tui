use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// The Shortcut entity a note is linked to. A note is about *at most one*
/// entity — the enum makes "linked to a story AND an epic id" unrepresentable.
/// (A story note still carries its iteration/epic *links* as context.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityLink {
    None,
    Story {
        id: i32,
        url: String,
        name: Option<String>,
        iteration_url: Option<String>,
        epic_url: Option<String>,
    },
    Iteration {
        id: i32,
        url: String,
        name: Option<String>,
    },
    Epic {
        id: i32,
        url: String,
        name: Option<String>,
    },
}

/// Parsed frontmatter of a note. Mirrors the shape already on disk (Obsidian
/// `id`/`aliases`/`tags` plus the Shortcut linkage), so reading is
/// non-destructive and writing stays Obsidian-compatible.
#[derive(Debug, Clone)]
pub struct Frontmatter {
    /// Obsidian slug — the filename stem.
    pub id: String,
    pub created: Option<NaiveDate>,
    pub note_type: Option<String>,
    pub tags: Vec<String>,
    pub aliases: Vec<String>,
    pub link: EntityLink,
}

impl Frontmatter {
    pub fn story(
        slug: String,
        story_id: i32,
        story_name: String,
        story_url: String,
        iteration_url: Option<String>,
    ) -> Self {
        Self {
            id: slug,
            created: Some(arc_core::time::today()),
            note_type: None,
            tags: Vec::new(),
            aliases: Vec::new(),
            link: EntityLink::Story {
                id: story_id,
                url: story_url,
                name: Some(story_name),
                iteration_url,
                epic_url: None,
            },
        }
    }

    pub fn iteration(slug: String, id: i32, name: String, url: String) -> Self {
        Self {
            id: slug,
            created: Some(arc_core::time::today()),
            note_type: None,
            tags: Vec::new(),
            aliases: Vec::new(),
            link: EntityLink::Iteration {
                id,
                url,
                name: Some(name),
            },
        }
    }

    pub fn epic(slug: String, id: i32, name: String, url: String) -> Self {
        Self {
            id: slug,
            created: Some(arc_core::time::today()),
            note_type: None,
            tags: Vec::new(),
            aliases: Vec::new(),
            link: EntityLink::Epic {
                id,
                url,
                name: Some(name),
            },
        }
    }

    /// A plain note with no Shortcut linkage (created via MCP, etc.).
    pub fn general(slug: String, tags: Vec<String>) -> Self {
        Self {
            id: slug,
            created: Some(arc_core::time::today()),
            note_type: None,
            tags,
            aliases: Vec::new(),
            link: EntityLink::None,
        }
    }

    pub fn daily(slug: String) -> Self {
        Self {
            id: slug,
            created: Some(arc_core::time::today()),
            note_type: Some("daily".into()),
            tags: Vec::new(),
            aliases: Vec::new(),
            link: EntityLink::None,
        }
    }

    pub fn scratch(slug: String) -> Self {
        Self {
            id: slug,
            created: Some(arc_core::time::today()),
            note_type: Some("scratch".into()),
            tags: Vec::new(),
            aliases: Vec::new(),
            link: EntityLink::None,
        }
    }

    /// Parse the frontmatter block out of a full note file. Returns `None` when
    /// the file has no leading `---` fence or the YAML doesn't parse.
    pub fn from_note(content: &str) -> Option<Frontmatter> {
        let yaml = extract_yaml(content)?;
        let raw: RawFrontmatter = serde_yaml::from_str(yaml).ok()?;
        Some(raw.into())
    }

    /// The `---\n…\n---\n` block for a fresh note.
    pub fn to_block(&self) -> anyhow::Result<String> {
        let raw = RawFrontmatter::from(self.clone());
        Ok(format!("---\n{}---\n", serde_yaml::to_string(&raw)?))
    }
}

/// Slice the YAML between the leading `---` fence and the next `---` line.
fn extract_yaml(content: &str) -> Option<&str> {
    let rest = content.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

/// Strip a `sc-`/`it-`/`ep-` prefix and parse the numeric id.
fn parse_prefixed_id(s: &str) -> Option<i32> {
    s.rsplit('-').next()?.parse().ok()
}

/// Faithful mirror of on-disk YAML — every field optional, so any real note
/// (including plain Obsidian notes and legacy shapes) deserializes. Used for
/// both read and write; `skip_serializing_if` keeps written blocks tidy.
#[derive(Debug, Default, Deserialize, Serialize)]
struct RawFrontmatter {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    aliases: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created: Option<NaiveDate>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    note_type: Option<String>,

    // Story note
    #[serde(skip_serializing_if = "Option::is_none")]
    story_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    story_link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    story_name: Option<String>,
    /// A story note's link to its iteration/epic.
    #[serde(skip_serializing_if = "Option::is_none")]
    iteration: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    epic: Option<String>,

    // Iteration note
    #[serde(skip_serializing_if = "Option::is_none")]
    iteration_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    iteration_link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    iteration_name: Option<String>,

    // Epic note
    #[serde(skip_serializing_if = "Option::is_none")]
    epic_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    epic_link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    epic_name: Option<String>,
}

impl From<RawFrontmatter> for Frontmatter {
    fn from(r: RawFrontmatter) -> Self {
        // Classify by which id is present. A note links to at most one entity.
        let link = if let Some(id) = r.story_id.as_deref().and_then(parse_prefixed_id) {
            EntityLink::Story {
                id,
                url: r.story_link.unwrap_or_default(),
                name: r.story_name,
                iteration_url: r.iteration,
                epic_url: r.epic,
            }
        } else if let Some(id) = r.iteration_id.as_deref().and_then(parse_prefixed_id) {
            EntityLink::Iteration {
                id,
                url: r.iteration_link.unwrap_or_default(),
                name: r.iteration_name,
            }
        } else if let Some(id) = r.epic_id.as_deref().and_then(parse_prefixed_id) {
            EntityLink::Epic {
                id,
                url: r.epic_link.unwrap_or_default(),
                name: r.epic_name,
            }
        } else {
            EntityLink::None
        };

        Frontmatter {
            id: r.id.unwrap_or_default(),
            created: r.created,
            note_type: r.note_type,
            tags: r.tags,
            aliases: r.aliases,
            link,
        }
    }
}

impl From<Frontmatter> for RawFrontmatter {
    fn from(f: Frontmatter) -> Self {
        let mut raw = RawFrontmatter {
            id: Some(f.id),
            aliases: f.aliases,
            tags: f.tags,
            created: f.created,
            note_type: f.note_type,
            ..Default::default()
        };
        match f.link {
            EntityLink::None => {}
            EntityLink::Story {
                id,
                url,
                name,
                iteration_url,
                epic_url,
            } => {
                raw.story_id = Some(format!("sc-{id}"));
                raw.story_link = Some(url);
                raw.story_name = name;
                raw.iteration = iteration_url;
                raw.epic = epic_url;
            }
            EntityLink::Iteration { id, url, name } => {
                raw.iteration_id = Some(format!("it-{id}"));
                raw.iteration_link = Some(url);
                raw.iteration_name = name;
            }
            EntityLink::Epic { id, url, name } => {
                raw.epic_id = Some(format!("ep-{id}"));
                raw.epic_link = Some(url);
                raw.epic_name = name;
            }
        }
        raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_story_note() {
        let content = "---\nid: my-story\naliases: []\ntags: [a]\ncreated: \"2026-03-31\"\niteration: https://it/1\nstory_id: sc-19559\nstory_link: https://s/19559\n---\n\nbody";
        let fm = Frontmatter::from_note(content).unwrap();
        assert_eq!(fm.id, "my-story");
        assert_eq!(fm.tags, vec!["a"]);
        assert!(matches!(
            fm.link,
            EntityLink::Story { id: 19559, .. }
        ));
    }

    #[test]
    fn reads_bare_created_and_iteration_note() {
        let content =
            "---\niteration_id: it-17842\niteration_link: https://it/17842\ncreated: 2026-02-18\n---\n";
        let fm = Frontmatter::from_note(content).unwrap();
        assert_eq!(fm.created, NaiveDate::from_ymd_opt(2026, 2, 18));
        assert!(matches!(fm.link, EntityLink::Iteration { id: 17842, .. }));
    }

    #[test]
    fn reads_plain_obsidian_note() {
        let content = "---\nid: rockpi\naliases: []\ntags: []\n---\n";
        let fm = Frontmatter::from_note(content).unwrap();
        assert_eq!(fm.id, "rockpi");
        assert_eq!(fm.link, EntityLink::None);
    }

    #[test]
    fn none_without_fence() {
        assert!(Frontmatter::from_note("# just a heading\n").is_none());
    }

    #[test]
    fn story_roundtrips_through_block() {
        let fm = Frontmatter::story(
            "s".into(),
            42,
            "Name".into(),
            "https://s/42".into(),
            Some("https://it/1".into()),
        );
        let block = fm.to_block().unwrap();
        let full = format!("{block}\nbody");
        let back = Frontmatter::from_note(&full).unwrap();
        assert!(matches!(back.link, EntityLink::Story { id: 42, .. }));
        assert!(block.contains("story_id: sc-42"));
    }
}
