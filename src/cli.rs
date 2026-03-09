use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "pycu",
    about = "Check Python dependencies for updates on PyPI",
    version  // exposes --version using the version in Cargo.toml
)]
pub struct Cli {
    /// Path to pyproject.toml or requirements.txt (auto-detected if omitted)
    #[arg(long)]
    pub file: Option<PathBuf>,

    /// Output results as JSON
    #[arg(long)]
    pub json: bool,

    /// Upgrade the file in-place with the latest versions
    #[arg(short = 'u', long)]
    pub upgrade: bool,

    /// Max concurrent PyPI requests
    #[arg(long, default_value = "10")]
    pub concurrency: usize,

    /// Only show updates of a specific bump level
    #[arg(short = 't', long, value_name = "LEVEL", default_value = "latest")]
    pub target: TargetLevel,

    /// Preview all color schemes, or set one persistently.
    /// Without a value: show all schemes visually.
    /// With a value: save that scheme and exit.
    #[arg(long, value_name = "SCHEME", num_args = 0..=1, default_missing_value = "")]
    pub set_color_scheme: Option<String>,

    /// Update pycu itself to the latest release
    #[arg(long)]
    pub self_update: bool,

    /// Remove the pycu binary from your system
    #[arg(long)]
    pub uninstall: bool,
}

#[derive(ValueEnum, Clone, PartialEq, Debug)]
pub enum TargetLevel {
    /// All updates (default)
    Latest,
    /// Only major version bumps (X.y.z)
    Major,
    /// Only minor version bumps (x.Y.z)
    Minor,
    /// Only patch version bumps (x.y.Z)
    Patch,
}

#[derive(ValueEnum, Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum ColorScheme {
    /// #D73A49 / #0366D6 / #28A745 - GitHub-style SemVer severity (default)
    Default,
    /// #E69F00 / #0072B2 / #009E73 - Okabe-Ito, color-blind safe
    OkabeIto,
    /// #E74C3C / #F1C40F / #2ECC71 - traffic-light (red/yellow/green)
    TrafficLight,
    /// #8E44AD / #3498DB / #95A5A6 - monitoring style (purple/blue/gray)
    Severity,
    /// #CC79A7 / #0072B2 / #F0E442 - maximum distinction, color-blind safe
    HighContrast,
}

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(raw) = cli.set_color_scheme {
        if raw.is_empty() {
            // --set-color-scheme used without a value: show the preview
            crate::output::table::print_color_scheme_preview();
        } else {
            // --set-color-scheme <SCHEME>: parse, save, confirm
            use clap::ValueEnum;
            let scheme = ColorScheme::from_str(&raw, true).map_err(|e| anyhow::anyhow!(
                "Unknown color scheme '{}'. {}\nRun `pycu --set-color-scheme` to see all options.",
                raw, e
            ))?;
            let config = crate::config::Config {
                color_scheme: scheme.clone(),
            };
            crate::config::save(&config)?;
            let path = crate::config::config_path().unwrap_or_else(|| PathBuf::from("config.toml"));
            println!(
                "Color scheme set to '{}' and saved to {}.",
                raw,
                path.display()
            );
        }
        return Ok(());
    }

    // Self-update is independent of any project file
    if cli.self_update {
        let client = crate::pypi::client::PypiClient::new()?.into_inner();
        return crate::self_update::run(&client).await;
    }

    if cli.uninstall {
        return crate::uninstall::run();
    }

    // Load persisted config, running first-run setup if no config exists or is unreadable
    let config = match crate::config::load() {
        Ok(Some(cfg)) => cfg,
        Ok(None) | Err(_) => crate::config::first_run_setup()?,
    };

    let file_path = match cli.file {
        Some(p) => p,
        None => resolve_default_file()?,
    };

    if !file_path.exists() {
        anyhow::bail!("File not found: {}", file_path.display());
    }

    eprintln!("Checking {}", file_path.display());

    let parser = crate::parsers::detect_parser(&file_path)?;
    let deps = parser.parse(&file_path)?;

    if deps.is_empty() {
        println!("No dependencies found.");
        return Ok(());
    }

    let client = crate::pypi::client::PypiClient::new()?;
    let all_updates = crate::version::compare::find_updates(deps, client, cli.concurrency).await?;

    use crate::version::compare::BumpKind;
    let filter_bump: Option<BumpKind> = match cli.target {
        TargetLevel::Latest => None,
        TargetLevel::Major => Some(BumpKind::Major),
        TargetLevel::Minor => Some(BumpKind::Minor),
        TargetLevel::Patch => Some(BumpKind::Patch),
    };
    let updates: Vec<_> = all_updates
        .into_iter()
        .filter(|u| filter_bump.as_ref().is_none_or(|b| &u.bump_kind == b))
        .collect();

    if cli.upgrade {
        crate::output::table::print_table(&updates, false, &config.color_scheme);
        let count = crate::upgrade::apply_upgrades(&file_path, &updates)?;
        if count > 0 {
            println!(
                "{} package{} upgraded in {}.",
                count,
                if count == 1 { "" } else { "s" },
                file_path.display()
            );
        }
        return Ok(());
    }

    if cli.json {
        crate::output::json::print_json(&updates)?;
    } else {
        crate::output::table::print_table(&updates, true, &config.color_scheme);
    }

    Ok(())
}

fn resolve_default_file() -> anyhow::Result<PathBuf> {
    for name in &["pyproject.toml", "requirements.txt"] {
        let p = PathBuf::from(name);
        if p.exists() {
            return Ok(p);
        }
    }
    anyhow::bail!(
        "No pyproject.toml or requirements.txt found in the current directory.\n\
         Use --file to specify a path."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_default_file_not_found() {
        // The project root is a Rust workspace with no Python dependency files,
        // so this must return an error.
        let result = resolve_default_file();
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("No pyproject.toml or requirements.txt found"));
    }
}
