use std::{
    env,
    fs::{self, File, remove_file},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    api::{iteration::Iteration, story::Story},
    dbg_file,
};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Cache {
    pub current_iterations: Option<Vec<Iteration>>,
    pub iterations: Vec<Iteration>,
    pub iteration_stories: Option<Vec<Story>>,
    pub active_story: Option<Story>,
    pub user_id: Option<Uuid>,
    pub cache_dir: PathBuf,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            current_iterations: None,
            iteration_stories: None,
            iterations: Vec::new(),
            user_id: None,
            active_story: None,
            cache_dir: Self::default_cache_dir(),
        }
    }
}

impl Cache {
    pub fn current_iterations_ref(&self) -> Option<Vec<&Iteration>> {
        self.current_iterations.as_ref().map(|v| v.iter().collect())
    }

    fn default_cache_dir() -> PathBuf {
        let mut base = env::home_dir().expect("Couldn't find home dir");
        base.push(".cache");
        base.push("shortcut-notes");
        base
    }

    pub fn get_cache_file(mut cache_dir: PathBuf) -> PathBuf {
        cache_dir.push("cache.json");
        cache_dir
    }

    pub fn read(cache_dir: PathBuf) -> Self {
        dbg_file!("Using {} as cache_dir", cache_dir.display());

        let cache_file = Self::get_cache_file(cache_dir);

        if let Some(parent) = cache_file.parent()
            && !parent.exists()
            && let Err(e) = fs::create_dir_all(parent)
        {
            dbg_file!(
                "Failed to create cache dir parent at: {} with err: {}",
                parent.display(),
                e
            );

            return Self::default();
        }

        let contents = match read_file(&cache_file) {
            Ok(contents) => contents,
            Err(_) => {
                return Self::default();
            }
        };

        match serde_json::from_str::<Cache>(&contents) {
            Ok(cache) => cache,
            Err(_) => {
                let cache_file = Path::new(&cache_file);
                if cache_file.is_file() {
                    let _ = remove_file(cache_file);
                }
                Self::default()
            }
        }
    }

    pub fn write(&self) -> anyhow::Result<()> {
        let cache_file = Self::get_cache_file(self.cache_dir.clone());
        let mut f = File::create(cache_file)?;
        f.write_all(&serde_json::to_string(self)?.into_bytes())?;

        Ok(())
    }
}

fn read_file(file: &PathBuf) -> anyhow::Result<String> {
    let mut f = File::open(file)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    Ok(buf)
}
