use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

/// config contained in `omni.toml`,
/// which also counts as project root.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub project: Project,
    #[serde(default)]
    pub dir_aliases: HashMap<String, PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub name: String,
    #[serde(default = "default_nodes_path")]
    pub nodes_path: PathBuf,
    #[serde(default = "default_build_path")]
    pub build_path: PathBuf,
}

fn default_nodes_path() -> PathBuf {
    PathBuf::from("nodes.toml")
}

fn default_build_path() -> PathBuf {
    PathBuf::from("build/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialization_all_specified() {
        let raw_toml = r#"
        [project]
        name = "my_proj"
        nodes_path = "other_nodes.toml" 
        build_path = "target" 

        [dir_aliases]
        linalg = "Linear Algebra"
        "#;

        let config: Config = toml::from_str(raw_toml).unwrap();
        assert_eq!(
            config,
            Config {
                project: Project {
                    name: "my_proj".into(),
                    nodes_path: "other_nodes.toml".into(),
                    build_path: "target".into(),
                },
                dir_aliases: HashMap::from([("linalg".into(), "Linear Algebra".into())])
            }
        )
    }

    #[test]
    fn test_config_deserialization_bare_minimum() {
        let raw_toml = r#"
        [project]
        name = "my_proj"
        "#;

        let config: Config = toml::from_str(raw_toml).unwrap();
        assert_eq!(
            config,
            Config {
                project: Project {
                    name: "my_proj".into(),
                    nodes_path: default_nodes_path(),
                    build_path: default_build_path(),
                },
                dir_aliases: HashMap::new()
            }
        )
    }
}
