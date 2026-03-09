use anyhow::{Context, Result};

pub fn run() -> Result<()> {
    let exe = std::env::current_exe()
        .context("Could not determine the path of the current executable")?;

    remove_exe(&exe)?;
    println!("Removed {}.", exe.display());

    remove_config();

    Ok(())
}

/// Remove the `pycu/` config directory (contains `config.toml`).
/// Failures are non-fatal - the binary is already gone at this point.
fn remove_config() {
    let config_dir = dirs::config_dir().map(|d| d.join("pycu"));

    match config_dir {
        None => {}
        Some(dir) if !dir.exists() => {}
        Some(dir) => match std::fs::remove_dir_all(&dir) {
            Ok(()) => println!("Removed config directory {}.", dir.display()),
            Err(e) => eprintln!(
                "Warning: could not remove config directory {}: {}",
                dir.display(),
                e
            ),
        },
    }
}

#[cfg(not(target_os = "windows"))]
fn remove_exe(exe: &std::path::Path) -> Result<()> {
    // On Unix, unlinking a running executable is safe: the inode stays alive
    // until the process exits, but the directory entry is gone immediately.
    std::fs::remove_file(exe).with_context(|| format!("Failed to remove {}", exe.display()))
}

#[cfg(target_os = "windows")]
fn remove_exe(exe: &std::path::Path) -> Result<()> {
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
