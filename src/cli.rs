use clap::{Parser, ValueEnum};
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

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Self-update is independent of any project file
    if cli.self_update {
        let client = crate::pypi::client::PypiClient::new()?.into_inner();
        return crate::self_update::run(&client).await;
    }

    if cli.uninstall {
        return crate::uninstall::run();
    }

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
        crate::output::table::print_table(&updates, false);
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
        crate::output::table::print_table(&updates, true);
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
