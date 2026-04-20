use crate::core::types::InstalledPackage;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

mod install;
mod query;
mod remove;
mod upgrade;

use crate::core::types::Repository;

pub struct PackageManager {
    pub(crate) prefix: PathBuf,
    pub(crate) db_path: PathBuf,
    pub(crate) installed: HashMap<String, InstalledPackage>,
    pub(crate) repo: Repository,
}

impl PackageManager {
    pub fn new(prefix: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let prefix = prefix.into();
        let db_path = prefix.join("var/lib/rpkg/db.json");
        let cache_dir = prefix.join("var/cache/rpkg");

        fs::create_dir_all(&prefix.join("var/lib/rpkg"))?;
        fs::create_dir_all(&cache_dir)?;

        let mut pm = Self {
            prefix,
            db_path,
            installed: HashMap::new(),
            repo: Repository::default(),
        };

        pm.load_database()?;
        Ok(pm)
    }

    fn load_database(&mut self) -> anyhow::Result<()> {
        if self.db_path.exists() {
            let data = fs::read_to_string(&self.db_path)?;
            if !data.is_empty() {
                self.installed = serde_json::from_str(&data)?;
            }
        }
        Ok(())
    }

    pub(crate) fn save_database(&self) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(&self.installed)?;
        let mut tmp_path = self.db_path.clone();
        tmp_path.set_extension("tmp");

        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(json.as_bytes())?;
        file.sync_all()?;
        fs::rename(tmp_path, &self.db_path)?;
        Ok(())
    }

    pub(crate) fn index_path(&self) -> PathBuf {
        self.prefix.join("var/lib/rpkg/Packages.gz")
    }
}
