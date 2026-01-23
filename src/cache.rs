use std::{
    env,
    fs::{remove_file, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{api::{iteration::Iteration, story::Story}, dbg_file};

#[derive(Deserialize, Serialize)]
pub struct Cache {
    pub current_iteration: Option<Iteration>,
    pub iteration_stories: Option<Vec<Story>>,
    pub user_id: Option<Uuid>,
    cache_dir: PathBuf,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            current_iteration: Default::default(),
            iteration_stories: Default::default(),
            user_id: Default::default(),
            cache_dir: Self::default_cache_dir(),
        }
    }
}

impl Cache {
    fn default_cache_dir() -> PathBuf {
        let mut base = env::home_dir().expect("Couldn't find home dir");
        base.push(".cache");
        base
    }

    fn get_cache_file(cache_dir: &Path) -> PathBuf {
        let mut clone = cache_dir.to_path_buf();
        clone.push("api.cache");
        clone
    }

    pub fn read(cache_dir: Option<String>) -> Self {
        let cache_dir: PathBuf = match cache_dir {
            Some(cache_dir) => cache_dir.into(),
            None => Self::default_cache_dir(),
        };

        dbg_file!("Using {} as cache_dir", cache_dir.display());

        let cache_file = Self::get_cache_file(&cache_dir);

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
        let cache_file = Self::get_cache_file(&self.cache_dir);
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
