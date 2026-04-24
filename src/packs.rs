use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use walkdir::WalkDir;

use crate::{
    cli::ListArgs,
    manifest::{Sticker, StickerPack, StickerPackManifest},
};

const MANIFEST_FILE: &str = "sticker-pack.toml";

pub fn discover_packs(pack_dir: &Path) -> Result<Vec<StickerPack>> {
    if !pack_dir.exists() {
        return Ok(Vec::new());
    }

    let mut manifest_paths = Vec::new();
    let root_manifest = pack_dir.join(MANIFEST_FILE);
    if root_manifest.exists() {
        manifest_paths.push(root_manifest);
    }

    for entry in WalkDir::new(pack_dir)
        .min_depth(1)
        .max_depth(4)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if entry.file_type().is_file() && entry.file_name() == std::ffi::OsStr::new(MANIFEST_FILE)
        {
            let path = entry.into_path();
            if !manifest_paths.iter().any(|known| known == &path) {
                manifest_paths.push(path);
            }
        }
    }

    let mut packs = manifest_paths
        .iter()
        .map(|path| StickerPack::load(path))
        .collect::<Result<Vec<_>>>()?;

    packs.sort_by(|a, b| a.manifest.name.cmp(&b.manifest.name));
    Ok(packs)
}

pub fn print_list(pack_dir: &Path, args: ListArgs) -> Result<()> {
    let packs = discover_packs(pack_dir)?;

    if packs.is_empty() {
        println!("No sticker packs found in {}", pack_dir.display());
        return Ok(());
    }

    let query = args.query.unwrap_or_default();
    for pack in packs {
        let stickers = pack
            .stickers()
            .into_iter()
            .filter(|sticker| query.is_empty() || sticker.matches_query(&query))
            .collect::<Vec<_>>();

        if stickers.is_empty() {
            continue;
        }

        println!("{} ({})", pack.manifest.name, pack.manifest.id);
        for sticker in stickers {
            let tags = if sticker.tags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", sticker.tags.join(", "))
            };
            println!("  {:<24} {}{}", sticker.qualified_id(), sticker.name, tags);
        }
    }

    Ok(())
}

pub fn all_stickers(packs: &[StickerPack]) -> Vec<Sticker> {
    packs.iter().flat_map(StickerPack::stickers).collect()
}

pub fn resolve_sticker(query: &str, packs: &[StickerPack]) -> Result<Sticker> {
    let direct_path = PathBuf::from(query);
    if direct_path.exists() {
        return Ok(Sticker {
            pack_id: "direct".to_string(),
            pack_name: "Direct File".to_string(),
            id: direct_path
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("sticker")
                .to_string(),
            name: direct_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(query)
                .to_string(),
            path: direct_path,
            tags: Vec::new(),
        });
    }

    let stickers = all_stickers(packs);
    if let Some(sticker) = stickers
        .iter()
        .find(|sticker| sticker.id == query || sticker.qualified_id() == query)
    {
        return Ok(sticker.clone());
    }

    let matches = stickers
        .into_iter()
        .filter(|sticker| sticker.matches_query(query))
        .collect::<Vec<_>>();

    match matches.len() {
        0 => bail!("no sticker matched {query:?}"),
        1 => Ok(matches[0].clone()),
        _ => {
            println!("Multiple stickers matched {query:?}:");
            for sticker in matches.iter().take(20) {
                println!("  {:<24} {}", sticker.qualified_id(), sticker.name);
            }
            bail!("use a full sticker id such as pack/sticker")
        }
    }
}

pub fn init_pack(pack_dir: &Path, name: &str) -> Result<()> {
    let id = slugify(name);
    if id.is_empty() {
        bail!("pack name must contain at least one letter or number");
    }

    let root = pack_dir.join(&id);
    let manifest_path = root.join(MANIFEST_FILE);
    if manifest_path.exists() {
        bail!("pack already exists: {}", manifest_path.display());
    }

    fs::create_dir_all(root.join("stickers"))
        .with_context(|| format!("failed to create {}", root.display()))?;

    let manifest = StickerPackManifest {
        id,
        name: name.to_string(),
        author: "Unknown".to_string(),
        license: "MIT".to_string(),
        description: None,
        homepage: None,
        stickers: Vec::new(),
    };

    fs::write(&manifest_path, toml::to_string_pretty(&manifest)?)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;

    println!("created {}", manifest_path.display());
    Ok(())
}

fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;

    for ch in input.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }

    out.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugifies_pack_names() {
        assert_eq!(slugify("My Reactions!"), "my-reactions");
        assert_eq!(slugify("  A   B  "), "a-b");
    }
}
