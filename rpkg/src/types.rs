use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub architecture: String,
    pub filename: String,
    pub size: u64,
    pub installed_size: u64,
    pub sha256: String,
    pub depends: Vec<Dependency>,
    pub provides: Vec<String>,
    pub conflicts: Vec<String>,
    pub description: String,
    pub homepage: Option<String>,
    pub maintainer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Dependency {
    pub name: String,
    pub version: Option<VersionConstraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VersionConstraint {
    pub op: VersionOp,
    pub version: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum VersionOp {
    Eq, // =
    Ge, // >=
    Le, // <=
    Gt, // >>
    Lt, // <<
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InstalledPackage {
    pub info: PackageInfo,
    pub files: Vec<String>,
    pub install_time: u64,
    pub explicit: bool, 
    pub required_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Repository {
    pub name: String,
    pub url: String,
    pub distribution: String,
    pub components: Vec<String>,
    pub architecture: String,
}

impl Default for Repository {
    fn default() -> Self {
        Self {
            name: "termux-main".into(),
            url: "https://packages.termux.dev/apt/termux-main".into(),
            distribution: "stable".into(),
            components: vec!["main".into()],
            architecture: "aarch64".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_default() {
        let repo = Repository::default();
        assert_eq!(repo.name, "termux-main");
        assert_eq!(repo.architecture, "aarch64");
    }

    #[test]
    fn test_package_info_serialization() {
        let pkg = PackageInfo {
            name: "git".into(),
            version: "2.43.0".into(),
            architecture: "aarch64".into(),
            filename: "pool/main/g/git/git_2.43.0_aarch64.deb".into(),
            size: 1024000,
            installed_size: 5000000,
            sha256: "abcdef123456".into(),
            depends: vec![Dependency {
                name: "curl".into(),
                version: Some(VersionConstraint {
                    op: VersionOp::Ge,
                    version: "7.80".into(),
                }),
            }],
            provides: vec![],
            conflicts: vec![],
            description: "A fast, scalable, distributed revision control system".into(),
            homepage: Some("https://git-scm.com".into()),
            maintainer: Some("Termux".into()),
        };

        let json = serde_json::to_string(&pkg).expect("Failed to serialize");
        let deserialized: PackageInfo = serde_json::from_str(&json).expect("Failed to deserialize");
        
        assert_eq!(pkg, deserialized);
    }
}
