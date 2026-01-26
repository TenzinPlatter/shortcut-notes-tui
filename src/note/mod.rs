use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use chrono::{Datelike, Utc};

use crate::{
    api::{iteration::Iteration, story::Story},
    note::frontmatter::Frontmatter,
};

pub mod frontmatter;

pub struct Note {
    pub frontmatter: Frontmatter,
    pub path: PathBuf,
}

impl Note {
    pub fn new<P: AsRef<Path>>(
        notes_dir: &P,
        story: &Story,
        current_iteration: Option<&Iteration>,
    ) -> Self {
        let now = Utc::now();

        let frontmatter = Frontmatter::new(story, current_iteration);
        let mut path = PathBuf::from(notes_dir.as_ref());
        path.push(format!("{}", now.year()));
        path.push(format!("{}", now.month()));
        path.push(format!("{}.md", &frontmatter.slug_id));

        Self { frontmatter, path }
    }

    pub fn write_frontmatter(&self, file: &mut File) -> anyhow::Result<()> {
        if !file_is_empty(file)? {
            anyhow::bail!("Tried to write frontmatter to non empty file")
        }

        let frontmatter_string = format!("---\n{}---", self.frontmatter.to_yaml_string()?);
        file.write_all(frontmatter_string.as_bytes())?;

        Ok(())
    }
}

fn file_is_empty(file: &mut File) -> anyhow::Result<bool> {
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;

    Ok(buf.is_empty())
}
