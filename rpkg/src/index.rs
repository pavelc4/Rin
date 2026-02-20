use crate::types::{Dependency, PackageInfo, VersionConstraint, VersionOp};
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

pub struct PackageIndex {
    packages: HashMap<String, PackageInfo>,
}

impl PackageIndex {
    pub fn from_url(url: &str) -> anyhow::Result<Self> {
        log::info!("Fetching {}", url);
        let response = ureq::get(url).call()?;
        let decoder = GzDecoder::new(response.into_body().into_reader());
        Self::parse(BufReader::new(decoder))
    }

    pub fn from_cache(path: &std::path::Path) -> anyhow::Result<Self> {
        let file = std::fs::File::open(path)?;
        let decoder = GzDecoder::new(file);
        Self::parse(BufReader::new(decoder))
    }

    fn parse<R: Read>(reader: BufReader<R>) -> anyhow::Result<Self> {
        let mut packages = HashMap::new();
        let mut current: HashMap<String, String> = HashMap::new();
        let mut current_key: Option<String> = None;

        for line in reader.lines() {
            let line = line?;

            if line.is_empty() {
                if let Some(pkg) = Self::build_package(&current) {
                    packages.insert(pkg.name.clone(), pkg);
                }
                current.clear();
                current_key = None;
            } else if line.starts_with(' ') || line.starts_with('\t') {
                if let Some(key) = &current_key {
                    if let Some(value) = current.get_mut(key) {
                        value.push('\n');
                        value.push_str(line.trim());
                    }
                }
            } else if let Some((key, value)) = line.split_once(": ") {
                current_key = Some(key.to_string());
                current.insert(key.to_string(), value.to_string());
            }
        }

        if !current.is_empty() {
            if let Some(pkg) = Self::build_package(&current) {
                packages.insert(pkg.name.clone(), pkg);
            }
        }

        log::debug!("Parsed {} packages from index", packages.len());
        Ok(Self { packages })
    }

    fn build_package(fields: &HashMap<String, String>) -> Option<PackageInfo> {
        Some(PackageInfo {
            name: fields.get("Package")?.clone(),
            version: fields.get("Version")?.clone(),
            architecture: fields.get("Architecture").cloned().unwrap_or_default(),
            filename: fields.get("Filename")?.clone(),
            size: fields.get("Size")?.parse().ok()?,
            installed_size: fields
                .get("Installed-Size")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            sha256: fields.get("SHA256").cloned().unwrap_or_default(),
            depends: Self::parse_depends(fields.get("Depends")),
            provides: Self::parse_simple_list(fields.get("Provides")),
            conflicts: Self::parse_simple_list(fields.get("Conflicts")),
            description: fields.get("Description").cloned().unwrap_or_default(),
            homepage: fields.get("Homepage").cloned(),
            maintainer: fields.get("Maintainer").cloned(),
        })
    }

    fn parse_depends(deps: Option<&String>) -> Vec<Dependency> {
        deps.map(|s| {
            s.split(", ")
                .filter_map(|d| {
                    let d = d.split(" | ").next()?;
                    let d = d.trim();

                    if let Some((name, ver_part)) = d.split_once(" (") {
                        let ver_part = ver_part.trim_end_matches(')');
                        let (op, version) = if let Some(v) = ver_part.strip_prefix(">=") {
                            (VersionOp::Ge, v.trim())
                        } else if let Some(v) = ver_part.strip_prefix("<=") {
                            (VersionOp::Le, v.trim())
                        } else if let Some(v) = ver_part.strip_prefix(">>") {
                            (VersionOp::Gt, v.trim())
                        } else if let Some(v) = ver_part.strip_prefix("<<") {
                            (VersionOp::Lt, v.trim())
                        } else if let Some(v) = ver_part.strip_prefix("=") {
                            (VersionOp::Eq, v.trim())
                        } else {
                            return Some(Dependency {
                                name: name.to_string(),
                                version: None,
                            });
                        };

                        Some(Dependency {
                            name: name.to_string(),
                            version: Some(VersionConstraint {
                                op,
                                version: version.to_string(),
                            }),
                        })
                    } else {
                        Some(Dependency {
                            name: d.to_string(),
                            version: None,
                        })
                    }
                })
                .collect()
        })
        .unwrap_or_default()
    }

    fn parse_simple_list(list: Option<&String>) -> Vec<String> {
        list.map(|s| {
            s.split(", ")
                .map(|p| p.split_whitespace().next().unwrap_or(p).to_string())
                .collect()
        })
        .unwrap_or_default()
    }

    pub fn get(&self, name: &str) -> Option<&PackageInfo> {
        self.packages.get(name)
    }

    pub fn search(&self, query: &str) -> Vec<&PackageInfo> {
        let query = query.to_lowercase();
        self.packages
            .values()
            .filter(|p| {
                p.name.to_lowercase().contains(&query)
                    || p.description.to_lowercase().contains(&query)
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.packages.len()
    }
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &PackageInfo> {
        self.packages.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_depends() {
        let deps_str = "libc, libcurl (>= 7.80.0), zlib (= 1.2.11)".to_string();
        let deps = PackageIndex::parse_depends(Some(&deps_str));
        
        assert_eq!(deps.len(), 3);
        assert_eq!(deps[0].name, "libc");
        assert!(deps[0].version.is_none());
        
        assert_eq!(deps[1].name, "libcurl");
        assert_eq!(deps[1].version.as_ref().unwrap().op, VersionOp::Ge);
        assert_eq!(deps[1].version.as_ref().unwrap().version, "7.80.0");
        
        assert_eq!(deps[2].name, "zlib");
        assert_eq!(deps[2].version.as_ref().unwrap().op, VersionOp::Eq);
    }
    
    #[test]
    fn test_parse_simple_list() {
        let provides = "editor, vi".to_string();
        let list = PackageIndex::parse_simple_list(Some(&provides));
        assert_eq!(list, vec!["editor".to_string(), "vi".to_string()]);
    }
}
