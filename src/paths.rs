use std::{
    env,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;

pub fn resolve_pack_dir(explicit: Option<&Path>) -> PathBuf {
    if let Some(path) = explicit {
        return path.to_path_buf();
    }

    if let Ok(path) = env::var("TERMINAL_STICKERS_HOME") {
        let path = PathBuf::from(path);
        if !path.as_os_str().is_empty() {
            return path;
        }
    }

    if let Ok(cwd) = env::current_dir() {
        let local_packs = cwd.join("packs");
        if local_packs.exists() {
            return local_packs;
        }
    }

    if let Ok(exe) = env::current_exe() {
        if let Some(bin_dir) = exe.parent() {
            let installed_packs = bin_dir
                .join("..")
                .join("share")
                .join("terminal-stickers")
                .join("packs");
            if installed_packs.exists() {
                return installed_packs;
            }
        }
    }

    ProjectDirs::from("com", "danielvictorino", "terminal-stickers")
        .map(|dirs| dirs.data_local_dir().join("packs"))
        .unwrap_or_else(|| PathBuf::from("packs"))
}
