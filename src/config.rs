use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

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

    let contents = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    let config: Config = toml::from_str(&contents)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
    Ok(Some(config))
}

pub fn save(config: &Config) -> Result<()> {
    let path = match config_path() {
        Some(p) => p,
        None => anyhow::bail!("Could not determine config directory"),
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
    }

    let contents = toml::to_string(config).context("Failed to serialize config")?;
    fs::write(&path, contents)
        .with_context(|| format!("Failed to write config file: {}", path.display()))?;
    Ok(())
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
    let input = input.trim().to_lowercase();

    let color_scheme = match input.as_str() {
        "okabe-ito" => ColorScheme::OkabeIto,
        "traffic-light" => ColorScheme::TrafficLight,
        "severity" => ColorScheme::Severity,
        "high-contrast" => ColorScheme::HighContrast,
        _ => ColorScheme::Default,
    };

    let config = Config { color_scheme };
    save(&config)?;

    let path = config_path().unwrap_or_else(|| PathBuf::from("config.toml"));
    println!("Color scheme saved to {}.", path.display());
    println!("Run `pycu --set-color-scheme <SCHEME>` at any time to change it.");
    println!();

    Ok(config)
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
    fn test_nonexistent_path_has_no_file() {
        let fake_path = PathBuf::from("/nonexistent/pycu/config.toml");
        assert!(!fake_path.exists());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");

        let cfg = Config {
            color_scheme: ColorScheme::OkabeIto,
        };
        let contents = toml::to_string(&cfg).unwrap();
        std::fs::write(&path, contents).unwrap();

        let back: Config = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(back.color_scheme, ColorScheme::OkabeIto);
    }
}
