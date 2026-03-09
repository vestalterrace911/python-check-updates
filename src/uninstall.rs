use anyhow::{Context, Result};
use std::path::Path;

pub fn run() -> Result<()> {
    let exe = std::env::current_exe()
        .context("Could not determine the path of the current executable")?;

    remove_exe(&exe)?;
    println!("Removed {}.", exe.display());

    if let Some(dir) = dirs::config_dir().map(|d| d.join("pycu")) {
        remove_config_at(&dir);
    }

    Ok(())
}

/// Remove a config directory. Failures are non-fatal - the binary is already gone.
pub(crate) fn remove_config_at(dir: &Path) {
    if !dir.exists() {
        return;
    }
    match std::fs::remove_dir_all(dir) {
        Ok(()) => println!("Removed config directory {}.", dir.display()),
        Err(e) => eprintln!(
            "Warning: could not remove config directory {}: {}",
            dir.display(),
            e
        ),
    }
}

#[cfg(not(target_os = "windows"))]
fn remove_exe(exe: &Path) -> Result<()> {
    // On Unix, unlinking a running executable is safe: the inode stays alive
    // until the process exits, but the directory entry is gone immediately.
    std::fs::remove_file(exe).with_context(|| format!("Failed to remove {}", exe.display()))
}

#[cfg(target_os = "windows")]
fn remove_exe(exe: &Path) -> Result<()> {
    // Windows locks running executables, so we cannot delete directly.
    // Rename to .old - the file disappears from its original location
    // immediately and the renamed file is cleaned up on next boot or
    // whenever no handle holds it open.
    let old = exe.with_extension("exe.old");
    std::fs::rename(exe, &old).with_context(|| format!("Failed to remove {}", exe.display()))?;
    // Best-effort immediate cleanup (may fail while the process is still live)
    let _ = std::fs::remove_file(&old);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_remove_config_at_existing_dir_removes_it() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join("pycu");
        std::fs::create_dir(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("config.toml"),
            "color_scheme = \"default\"\n",
        )
        .unwrap();

        remove_config_at(&config_dir);

        assert!(!config_dir.exists());
    }

    #[test]
    fn test_remove_config_at_nonexistent_dir_is_noop() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join("does_not_exist");
        // Must not panic
        remove_config_at(&config_dir);
        assert!(!config_dir.exists());
    }
}
