use crate::core::index::PackageIndex;
use crate::manager::PackageManager;
use colored::Colorize;

impl PackageManager {
    pub fn sync(&self) -> anyhow::Result<()> {
        let url = format!(
            "{}/dists/{}/{}/binary-{}/Packages.gz",
            self.repo.url, self.repo.distribution, self.repo.components[0], self.repo.architecture
        );
        log::info!("Fetching package index from {}", url);

        let rsp = ureq::get(&url).call()?;
        let mut reader = rsp.into_body().into_reader();
        let mut file = std::fs::File::create(self.index_path())?;
        std::io::copy(&mut reader, &mut file)?;
        file.sync_all()?;

        log::info!("Package system updated!");
        Ok(())
    }

    pub fn upgrade(&mut self) -> anyhow::Result<()> {
        println!("{}", ":: Starting full system upgrade...".blue().bold());
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
            println!(" there is nothing to do");
            return Ok(());
        }

        self.install(&to_upgrade, true)?;
        Ok(())
    }
}
