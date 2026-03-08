use std::io::Read;

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::version::compare::is_newer;

const REPO_OWNER: &str = "Logic-py";
const REPO_NAME: &str = "python-check-updates";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

pub async fn run(client: &reqwest::Client) -> Result<()> {
    println!("Current version: {}", CURRENT_VERSION);
    println!("Checking for updates...");

    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        REPO_OWNER, REPO_NAME
    );

    let release: Release = client
        .get(&url)
        .send()
        .await
        .context("Failed to reach GitHub API")?
        .error_for_status()
        .context("GitHub API returned an error")?
        .json()
        .await?;

    let latest = release.tag_name.trim_start_matches('v');

    if !is_newer(latest, CURRENT_VERSION) {
        println!("Already up to date ({}).", CURRENT_VERSION);
        return Ok(());
    }

    println!("Updating {} → {}...", CURRENT_VERSION, latest);

    let asset_name = platform_asset_name()?;

    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .with_context(|| {
            format!(
                "No pre-built binary found for this platform ({})",
                asset_name
            )
        })?;

    // Find the checksums file published alongside the binaries
    let checksum_asset = release
        .assets
        .iter()
        .find(|a| a.name == "checksums.sha256")
        .context(
            "checksums.sha256 not found in release assets - cannot verify download integrity",
        )?;

    println!("Downloading {}...", asset_name);
    let bytes = client
        .get(&asset.browser_download_url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    // Verify SHA256 checksum before touching the filesystem
    let checksums_text = client
        .get(&checksum_asset.browser_download_url)
        .send()
        .await
        .context("Failed to download checksums.sha256")?
        .error_for_status()?
        .text()
        .await?;

    verify_checksum(&bytes, &asset_name, &checksums_text).context(
        "SHA256 checksum verification failed - download may be corrupted or tampered with",
    )?;

    let new_binary =
        extract_binary(&bytes, &asset_name).context("Failed to extract binary from archive")?;

    let current_exe =
        std::env::current_exe().context("Could not determine current executable path")?;

    replace_exe(&current_exe, &new_binary).context("Failed to replace executable")?;

    println!("Done! pycu is now at {}.", latest);
    Ok(())
}

/// Verify that `bytes` match the expected SHA256 hash recorded in the checksums file.
fn verify_checksum(bytes: &[u8], asset_name: &str, checksums_text: &str) -> Result<()> {
    let expected = checksums_text
        .lines()
        .find_map(|line| {
            // Format: "<hash>  <filename>" or "<hash> <filename>"
            let mut parts = line.splitn(2, ' ');
            let hash = parts.next()?.trim();
            let name = parts.next()?.trim();
            if name == asset_name {
                Some(hash.to_string())
            } else {
                None
            }
        })
        .with_context(|| {
            format!(
                "No checksum entry found for {} in checksums.sha256",
                asset_name
            )
        })?;

    let actual = hex_digest(bytes);

    if actual != expected {
        bail!(
            "checksum mismatch for {}:\n  expected: {}\n  actual:   {}",
            asset_name,
            expected,
            actual
        );
    }

    Ok(())
}

fn hex_digest(data: &[u8]) -> String {
    use std::fmt::Write as _;
    let hash = Sha256::digest(data);
    let mut out = String::with_capacity(64);
    for byte in &hash {
        // Writing to a String is infallible; the Result can be safely discarded.
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// Returns the archive filename for the currently running platform.
fn platform_asset_name() -> Result<String> {
    let target = if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "x86_64-unknown-linux-musl"
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        "aarch64-unknown-linux-musl"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "x86_64-apple-darwin"
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "aarch64-apple-darwin"
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "x86_64-pc-windows-msvc"
    } else {
        bail!(
            "No pre-built binary for this platform. Build from source: \
             https://github.com/{}/{}",
            REPO_OWNER,
            REPO_NAME
        );
    };

    let ext = if cfg!(target_os = "windows") {
        "zip"
    } else {
        "tar.gz"
    };
    Ok(format!("pycu-{}.{}", target, ext))
}

fn extract_binary(bytes: &[u8], asset_name: &str) -> Result<Vec<u8>> {
    if asset_name.ends_with(".tar.gz") {
        extract_from_tar_gz(bytes)
    } else {
        extract_from_zip(bytes)
    }
}

fn extract_from_tar_gz(bytes: &[u8]) -> Result<Vec<u8>> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let mut archive = Archive::new(GzDecoder::new(bytes));

    for entry in archive.entries()? {
        let mut entry = entry?;
        let name = entry
            .path()?
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        if name == "pycu" {
            let mut data = Vec::new();
            entry.read_to_end(&mut data)?;
            return Ok(data);
        }
    }

    bail!("pycu binary not found inside archive")
}

fn extract_from_zip(bytes: &[u8]) -> Result<Vec<u8>> {
    use std::io::Cursor;
    use zip::ZipArchive;

    let mut archive = ZipArchive::new(Cursor::new(bytes))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        if name == "pycu.exe" || name.ends_with("/pycu.exe") {
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            return Ok(data);
        }
    }

    bail!("pycu.exe not found inside archive")
}

/// Atomically replace the running executable with `new_binary`.
#[cfg(not(target_os = "windows"))]
fn replace_exe(current_exe: &std::path::Path, new_binary: &[u8]) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    // Write to a sibling temp file, then rename atomically
    let tmp = current_exe.with_extension("tmp");
    std::fs::write(&tmp, new_binary)?;
    std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))?;
    std::fs::rename(&tmp, current_exe)?;
    Ok(())
}

/// On Windows you cannot overwrite a running exe, but you can rename it.
/// Rename the current .exe → .old, write the new one, clean up .old (best-effort).
#[cfg(target_os = "windows")]
fn replace_exe(current_exe: &std::path::Path, new_binary: &[u8]) -> Result<()> {
    let old = current_exe.with_extension("exe.old");
    std::fs::rename(current_exe, &old)?;
    std::fs::write(current_exe, new_binary)?;
    let _ = std::fs::remove_file(&old); // may still be locked; fine
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_digest_known_value() {
        // SHA-256("hello") is a well-known constant
        let digest = hex_digest(b"hello");
        assert_eq!(
            digest,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_hex_digest_empty() {
        let digest = hex_digest(&[]);
        assert_eq!(digest.len(), 64);
        assert!(digest.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_verify_checksum_valid() {
        let bytes = b"test content";
        let hash = hex_digest(bytes);
        let checksums = format!("{}  pycu-x86_64-unknown-linux-musl.tar.gz\n", hash);
        assert!(
            verify_checksum(bytes, "pycu-x86_64-unknown-linux-musl.tar.gz", &checksums).is_ok()
        );
    }

    #[test]
    fn test_verify_checksum_wrong_hash() {
        let bytes = b"test content";
        let checksums =
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef  myfile.tar.gz\n";
        assert!(verify_checksum(bytes, "myfile.tar.gz", checksums).is_err());
    }

    #[test]
    fn test_verify_checksum_missing_entry() {
        let bytes = b"test content";
        let hash = hex_digest(bytes);
        let checksums = format!("{}  other-file.tar.gz\n", hash);
        let result = verify_checksum(bytes, "myfile.tar.gz", &checksums);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_checksum_single_space_separator() {
        // Some tools emit "hash filename" (one space) rather than "hash  filename"
        let bytes = b"data";
        let hash = hex_digest(bytes);
        let checksums = format!("{} myfile.tar.gz\n", hash);
        assert!(verify_checksum(bytes, "myfile.tar.gz", &checksums).is_ok());
    }

    #[test]
    fn test_platform_asset_name_format() {
        let name = platform_asset_name().unwrap();
        assert!(name.starts_with("pycu-"));
        assert!(name.ends_with(".tar.gz") || name.ends_with(".zip"));
    }

    #[test]
    fn test_extract_from_tar_gz() {
        use flate2::Compression;
        use flate2::write::GzEncoder;

        let buf = Vec::new();
        let enc = GzEncoder::new(buf, Compression::default());
        let mut tar = tar::Builder::new(enc);

        let content = b"fake pycu binary";
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        tar.append_data(&mut header, "pycu", content.as_slice())
            .unwrap();
        let gz_bytes = tar.into_inner().unwrap().finish().unwrap();

        let result = extract_from_tar_gz(&gz_bytes).unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn test_extract_from_tar_gz_missing_binary() {
        use flate2::Compression;
        use flate2::write::GzEncoder;

        let buf = Vec::new();
        let enc = GzEncoder::new(buf, Compression::default());
        let mut tar = tar::Builder::new(enc);

        let content = b"some other file";
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append_data(&mut header, "readme.txt", content.as_slice())
            .unwrap();
        let gz_bytes = tar.into_inner().unwrap().finish().unwrap();

        assert!(extract_from_tar_gz(&gz_bytes).is_err());
    }

    #[test]
    fn test_extract_from_zip() {
        use std::io::{Cursor, Write};
        use zip::write::SimpleFileOptions;

        let buf = Vec::new();
        let cursor = Cursor::new(buf);
        let mut zip = zip::ZipWriter::new(cursor);
        zip.start_file("pycu.exe", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"fake exe content").unwrap();
        let bytes = zip.finish().unwrap().into_inner();

        let result = extract_from_zip(&bytes).unwrap();
        assert_eq!(result, b"fake exe content");
    }

    #[test]
    fn test_extract_from_zip_missing_binary() {
        use std::io::{Cursor, Write};
        use zip::write::SimpleFileOptions;

        let buf = Vec::new();
        let cursor = Cursor::new(buf);
        let mut zip = zip::ZipWriter::new(cursor);
        zip.start_file("readme.txt", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"not the binary").unwrap();
        let bytes = zip.finish().unwrap().into_inner();

        assert!(extract_from_zip(&bytes).is_err());
    }

    #[test]
    fn test_extract_binary_dispatches_tar_gz() {
        use flate2::Compression;
        use flate2::write::GzEncoder;

        let buf = Vec::new();
        let enc = GzEncoder::new(buf, Compression::default());
        let mut tar = tar::Builder::new(enc);
        let content = b"binary";
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        tar.append_data(&mut header, "pycu", content.as_slice())
            .unwrap();
        let gz_bytes = tar.into_inner().unwrap().finish().unwrap();

        let result = extract_binary(&gz_bytes, "pycu-x86_64-unknown-linux-musl.tar.gz").unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn test_extract_binary_dispatches_zip() {
        use std::io::{Cursor, Write};
        use zip::write::SimpleFileOptions;

        let buf = Vec::new();
        let cursor = Cursor::new(buf);
        let mut zip = zip::ZipWriter::new(cursor);
        zip.start_file("pycu.exe", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"binary").unwrap();
        let bytes = zip.finish().unwrap().into_inner();

        let result = extract_binary(&bytes, "pycu-x86_64-pc-windows-msvc.zip").unwrap();
        assert_eq!(result, b"binary");
    }
}
