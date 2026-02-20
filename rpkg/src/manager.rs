use crate::extract::extract_deb;
use crate::index::PackageIndex;
use crate::resolver::Resolver;
use crate::types::{InstalledPackage, Repository};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub struct PackageManager {
    prefix: PathBuf,
    db_path: PathBuf,
    installed: HashMap<String, InstalledPackage>,
    repo: Repository,
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

    fn save_database(&self) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(&self.installed)?;
        let mut tmp_path = self.db_path.clone();
        tmp_path.set_extension("tmp");
        
        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(json.as_bytes())?;
        file.sync_all()?;
        fs::rename(tmp_path, &self.db_path)?;
        Ok(())
    }

    fn index_path(&self) -> PathBuf {
        self.prefix.join("var/lib/rpkg/Packages.gz")
    }
    pub fn sync(&self) -> anyhow::Result<()> {
        let url = format!("{}/dists/{}/{}/binary-{}/Packages.gz", 
            self.repo.url, self.repo.distribution, 
            self.repo.components[0], self.repo.architecture
        );
        log::info!("Fetching package index from {}", url);
        
        let rsp = ureq::get(&url).call()?;
        let mut reader = rsp.into_body().into_reader();
        let mut file = fs::File::create(self.index_path())?;
        std::io::copy(&mut reader, &mut file)?;
        file.sync_all()?;
        
        log::info!("Package system updated!");
        Ok(())
    }

    pub fn install(&mut self, package_name: &str) -> anyhow::Result<()> {
        let index = PackageIndex::from_cache(&self.index_path())
            .map_err(|e| anyhow::anyhow!("Failed to read package index. Did you run sync? Error: {}", e))?;
        
        let installed_set: HashSet<String> = self.installed.keys().cloned().collect();
        let resolver = Resolver::new(&index, installed_set);
        
        let to_install = resolver.resolve(package_name)?;

        if to_install.is_empty() {
            log::info!("Package '{}' is already installed and up to date.", package_name);
            return Ok(());
        }

        log::info!("Packages to install: {:?}", to_install.iter().map(|p| &p.name).collect::<Vec<_>>());

        for pkg in to_install {
            log::info!("Downloading {}...", pkg.name);
            let url = format!("{}/{}", self.repo.url, pkg.filename);
            let rsp = ureq::get(&url).call()?;
            let reader = rsp.into_body().into_reader();
            
            log::info!("Extracting {}...", pkg.name);
            let installed_files = extract_deb(reader, &self.prefix)?;
            
            log::info!("Registering {}...", pkg.name);
            let installed_pkg = InstalledPackage {
                info: pkg.clone(),
                files: installed_files,
                install_time: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs(),
                explicit: pkg.name == package_name,
                required_by: vec![],
            };

            self.installed.insert(pkg.name.clone(), installed_pkg);
            self.save_database()?;
        }

        log::info!("Successfully installed '{}'", package_name);
        Ok(())
    }

    pub fn remove(&mut self, package_name: &str) -> anyhow::Result<()> {
        if let Some(pkg) = self.installed.remove(package_name) {
            for file_path in &pkg.files {
                let absolute_path = self.prefix.join(file_path);
                if absolute_path.exists() {
                    if absolute_path.is_file() || absolute_path.is_symlink() {
                        let _ = fs::remove_file(&absolute_path);
                    }
                }
            }
            self.save_database()?;
            log::info!("Removed package {}", package_name);
        } else {
            log::warn!("Package {} is not installed.", package_name);
        }
        Ok(())
    }

    pub fn list_installed(&self) -> Vec<&InstalledPackage> {
        self.installed.values().collect()
    }

    pub fn search(&self, query: &str) -> anyhow::Result<Vec<crate::types::PackageInfo>> {
        let index = PackageIndex::from_cache(&self.index_path())
            .map_err(|e| anyhow::anyhow!("Failed to read index: {}", e))?;
        Ok(index.search(query).into_iter().cloned().collect())
    }

    pub fn upgrade(&mut self) -> anyhow::Result<()> {
        log::info!("Upgrading all packages...");
        let index = PackageIndex::from_cache(&self.index_path())
            .map_err(|e| anyhow::anyhow!("Failed to read index: {}", e))?;
        
        let mut to_upgrade = Vec::new();
        for (name, installed) in &self.installed {
            if let Some(latest) = index.get(name) {
                if latest.version != installed.info.version {
                    to_upgrade.push(name.clone());
                }
            }
        }

        if to_upgrade.is_empty() {
            log::info!("Nothing to upgrade.");
            return Ok(());
        }

        for name in to_upgrade {
            self.install(&name)?;
        }
        Ok(())
    }
}
