use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::cli::ColorScheme;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Config {
    pub color_scheme: ColorScheme,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::Default,
        }
    }
}

pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("pycu").join("config.toml"))
}

/// Load config from disk. Returns `None` if the file doesn't exist yet (first run).
pub fn load() -> Result<Option<Config>> {
    let path = match config_path() {
        Some(p) => p,
        None => return Ok(Some(Config::default())),
    };

    if !path.exists() {
        return Ok(None);
    }

    load_from_path(&path).map(Some)
}

pub fn save(config: &Config) -> Result<()> {
    let path = match config_path() {
        Some(p) => p,
        None => anyhow::bail!("Could not determine config directory"),
    };

    save_to_path(config, &path)
}

/// Interactive first-run prompt. Shows the color scheme preview, asks the user to pick,
/// saves the choice, and returns the chosen `Config`.
pub fn first_run_setup() -> Result<Config> {
    crate::output::table::print_color_scheme_preview();

    print!(
        "Choose a color scheme [default/okabe-ito/traffic-light/severity/high-contrast]\n(press Enter for default): "
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let color_scheme = parse_scheme_input(&input);
    let config = Config { color_scheme };
    save(&config)?;

    let path = config_path().unwrap_or_else(|| PathBuf::from("config.toml"));
    println!("Color scheme saved to {}.", path.display());
    println!("Run `pycu --set-color-scheme <SCHEME>` at any time to change it.");
    println!();

    Ok(config)
}

// ── Internal helpers (pub(crate) for testing) ────────────────────────────────

pub(crate) fn load_from_path(path: &Path) -> Result<Config> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    toml::from_str(&contents)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))
}

pub(crate) fn save_to_path(config: &Config, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
    }
    let contents = toml::to_string(config).context("Failed to serialize config")?;
    fs::write(path, contents)
        .with_context(|| format!("Failed to write config file: {}", path.display()))
}

/// Parse a raw user input string into a `ColorScheme`, defaulting to `Default`.
pub(crate) fn parse_scheme_input(input: &str) -> ColorScheme {
    match input.trim().to_lowercase().as_str() {
        "okabe-ito" => ColorScheme::OkabeIto,
        "traffic-light" => ColorScheme::TrafficLight,
        "severity" => ColorScheme::Severity,
        "high-contrast" => ColorScheme::HighContrast,
        _ => ColorScheme::Default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_default_is_default_scheme() {
        let cfg = Config::default();
        assert_eq!(cfg.color_scheme, ColorScheme::Default);
    }

    #[test]
    fn test_config_roundtrip_default() {
        let cfg = Config {
            color_scheme: ColorScheme::Default,
        };
        let s = toml::to_string(&cfg).unwrap();
        let back: Config = toml::from_str(&s).unwrap();
        assert_eq!(back.color_scheme, ColorScheme::Default);
    }

    #[test]
    fn test_config_roundtrip_all_schemes() {
        for scheme in [
            ColorScheme::OkabeIto,
            ColorScheme::TrafficLight,
            ColorScheme::Severity,
            ColorScheme::HighContrast,
        ] {
            let cfg = Config {
                color_scheme: scheme.clone(),
            };
            let s = toml::to_string(&cfg).unwrap();
            let back: Config = toml::from_str(&s).unwrap();
            assert_eq!(back.color_scheme, scheme);
        }
    }

    #[test]
    fn test_save_to_path_creates_dirs_and_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nested").join("pycu").join("config.toml");
        let cfg = Config {
            color_scheme: ColorScheme::Severity,
        };
        save_to_path(&cfg, &path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_load_from_path_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        let cfg = Config {
            color_scheme: ColorScheme::TrafficLight,
        };
        save_to_path(&cfg, &path).unwrap();
        let back = load_from_path(&path).unwrap();
        assert_eq!(back.color_scheme, ColorScheme::TrafficLight);
    }

    #[test]
    fn test_load_from_path_invalid_toml_returns_err() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(&path, "this is not valid toml !!!").unwrap();
        let result = load_from_path(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse"));
    }

    #[test]
    fn test_load_from_path_missing_file_returns_err() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.toml");
        let result = load_from_path(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to read"));
    }

    #[test]
    fn test_parse_scheme_input_all_variants() {
        assert_eq!(parse_scheme_input("okabe-ito"), ColorScheme::OkabeIto);
        assert_eq!(
            parse_scheme_input("traffic-light"),
            ColorScheme::TrafficLight
        );
        assert_eq!(parse_scheme_input("severity"), ColorScheme::Severity);
        assert_eq!(
            parse_scheme_input("high-contrast"),
            ColorScheme::HighContrast
        );
        assert_eq!(parse_scheme_input("default"), ColorScheme::Default);
        assert_eq!(parse_scheme_input(""), ColorScheme::Default);
        assert_eq!(parse_scheme_input("unknown"), ColorScheme::Default);
    }

    #[test]
    fn test_parse_scheme_input_trims_whitespace_and_ignores_case() {
        assert_eq!(parse_scheme_input("  OKABE-ITO\n"), ColorScheme::OkabeIto);
        assert_eq!(
            parse_scheme_input("  Traffic-Light  "),
            ColorScheme::TrafficLight
        );
    }
}
