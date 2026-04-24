use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StickerPackManifest {
    pub id: String,
    pub name: String,
    pub author: String,
    pub license: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub stickers: Vec<StickerManifest>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StickerManifest {
    pub id: String,
    pub name: String,
    pub file: PathBuf,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StickerPack {
    pub manifest: StickerPackManifest,
    pub root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sticker {
    pub pack_id: String,
    pub pack_name: String,
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub tags: Vec<String>,
}

impl StickerPack {
    pub fn load(manifest_path: &Path) -> Result<Self> {
        let manifest_text = fs::read_to_string(manifest_path)
            .with_context(|| format!("failed to read {}", manifest_path.display()))?;
        let manifest: StickerPackManifest = toml::from_str(&manifest_text)
            .with_context(|| format!("failed to parse {}", manifest_path.display()))?;
        let root = manifest_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        Ok(Self { manifest, root })
    }

    pub fn stickers(&self) -> Vec<Sticker> {
        self.manifest
            .stickers
            .iter()
            .map(|sticker| Sticker {
                pack_id: self.manifest.id.clone(),
                pack_name: self.manifest.name.clone(),
                id: sticker.id.clone(),
                name: sticker.name.clone(),
                path: self.root.join(&sticker.file),
                tags: sticker.tags.clone(),
            })
            .collect()
    }
}

impl Sticker {
    pub fn qualified_id(&self) -> String {
        format!("{}/{}", self.pack_id, self.id)
    }

    pub fn matches_query(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.id.to_lowercase().contains(&query)
            || self.name.to_lowercase().contains(&query)
            || self.qualified_id().to_lowercase().contains(&query)
            || self
                .tags
                .iter()
                .any(|tag| tag.to_lowercase().contains(&query))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_pack_manifest_and_resolves_sticker_paths() {
        let dir = tempfile::tempdir().expect("temp dir");
        fs::create_dir_all(dir.path().join("stickers")).expect("stickers dir");
        fs::write(
            dir.path().join("sticker-pack.toml"),
            r#"
id = "test"
name = "Test Pack"
author = "tests"
license = "MIT"

[[stickers]]
id = "ok"
name = "OK"
file = "stickers/ok.png"
tags = ["reaction"]
"#,
        )
        .expect("manifest");

        let pack = StickerPack::load(&dir.path().join("sticker-pack.toml")).expect("load pack");
        let stickers = pack.stickers();

        assert_eq!(pack.manifest.id, "test");
        assert_eq!(stickers[0].qualified_id(), "test/ok");
        assert!(stickers[0].path.ends_with("stickers/ok.png"));
    }
}
