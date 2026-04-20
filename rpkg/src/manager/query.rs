use crate::core::index::PackageIndex;
use crate::core::types::InstalledPackage;
use crate::manager::PackageManager;

impl PackageManager {
    pub fn list_installed(&self) -> Vec<&InstalledPackage> {
        self.installed.values().collect()
    }

    pub fn search(&self, query: &str) -> anyhow::Result<Vec<crate::core::types::PackageInfo>> {
        let index = PackageIndex::from_cache(&self.index_path())
            .map_err(|e| anyhow::anyhow!("Failed to read index: {}", e))?;
        Ok(index.search(query).into_iter().cloned().collect())
    }
}
