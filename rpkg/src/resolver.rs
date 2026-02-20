use crate::index::PackageIndex;
use crate::types::PackageInfo;
use std::collections::HashSet;

pub struct Resolver<'a> {
    index: &'a PackageIndex,
    installed: HashSet<String>,
}

impl<'a> Resolver<'a> {
    pub fn new(index: &'a PackageIndex, installed: HashSet<String>) -> Self {
        Self { index, installed }
    }

    pub fn resolve(&self, target_package: &str) -> anyhow::Result<Vec<PackageInfo>> {
        let mut to_install = Vec::new();
        let mut visited = HashSet::new();
        let mut in_stack = HashSet::new();

        self.resolve_recursive(target_package, &mut to_install, &mut visited, &mut in_stack)?;

        Ok(to_install)
    }

    fn resolve_recursive(
        &self,
        package_name: &str,
        result: &mut Vec<PackageInfo>,
        visited: &mut HashSet<String>,
        in_stack: &mut HashSet<String>,
    ) -> anyhow::Result<()> {
        if visited.contains(package_name) || self.installed.contains(package_name) {
            return Ok(());
        }
        if in_stack.contains(package_name) {
            log::warn!("Circular dependency detected involving: {}", package_name);
            return Ok(());
        }
        let pkg = match self.index.get(package_name) {
            Some(p) => p,
            None => {
                let provider = self.index.iter().find(|p| p.provides.contains(&package_name.to_string()));
                match provider {
                    Some(p) => p,
                    None => anyhow::bail!("Package not found in index: {}", package_name),
                }
            }
        };

        in_stack.insert(pkg.name.clone());

        for dep in &pkg.depends {
            self.resolve_recursive(&dep.name, result, visited, in_stack)?;
        }

        in_stack.remove(&pkg.name);
        visited.insert(pkg.name.clone());
        result.push(pkg.clone());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
}
