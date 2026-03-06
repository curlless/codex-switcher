use clap::{Command, CommandFactory, Parser, Subcommand};

use crate::switcher::command_name;

#[derive(Parser)]
#[command(author, version, about, color = clap::ColorChoice::Never)]
pub struct Cli {
    /// Disable styling and separators
    #[arg(long, global = true)]
    pub plain: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Save the current auth.json as a profile
    Save {
        /// Optional label for the profile (must be unique)
        #[arg(value_name = "label")]
        #[arg(long)]
        label: Option<String>,
    },
    /// Load a profile from the interactive list
    Load {
        /// Load the profile matching this label
        #[arg(value_name = "label")]
        #[arg(long)]
        label: Option<String>,
    },
    /// List profiles ordered by last used
    List,
    /// Show usage details for the current profile
    Status {
        /// Show usage for all saved profiles
        #[arg(long)]
        all: bool,
        /// Show usage for the currently active profile only
        #[arg(long)]
        current: bool,
        /// Show usage for the profile matching this label
        #[arg(value_name = "label")]
        #[arg(long)]
        label: Option<String>,
    },
    /// Switch to the best profile based on remaining 7d/5h usage
    Switch {
        /// Show ranking and selected profile without switching
        #[arg(long)]
        dry_run: bool,
        /// After switching, try to reload IDE processes (best effort)
        #[arg(long)]
        reload_ide: bool,
    },
    /// Run the IDE/app reload logic without switching profiles
    ReloadApp {
        /// Inspect targets and print reload guidance without terminating processes
        #[arg(long)]
        dry_run: bool,
    },
    /// Mark a saved profile so auto-switch skips it
    Reserve {
        /// Reserve the profile matching this label
        #[arg(value_name = "label")]
        #[arg(long)]
        label: Option<String>,
    },
    /// Remove the auto-switch reservation from a saved profile
    Unreserve {
        /// Unreserve the profile matching this label
        #[arg(value_name = "label")]
        #[arg(long)]
        label: Option<String>,
    },
    /// Copy profiles from another Codex directory into current storage
    Migrate {
        /// Source Codex directory (contains profiles/ and profiles.json)
        #[arg(value_name = "path")]
        #[arg(long)]
        from: Option<String>,
        /// Overwrite existing destination profiles with source files
        #[arg(long)]
        overwrite: bool,
    },
    /// Delete saved profiles from the interactive list
    Delete {
        /// Skip delete confirmation
        #[arg(long)]
        yes: bool,
        /// Delete the profile matching this label
        #[arg(value_name = "label")]
        #[arg(long)]
        label: Option<String>,
    },
    /// Relay an existing Roo/Codex callback URL to a local listener
    RelayLogin {
        /// Callback URL to relay to the local auth listener
        #[arg(value_name = "callback_url")]
        #[arg(long)]
        url: Option<String>,
    },
}

pub fn command_with_examples() -> Command {
    let name = command_name();
    let mut cmd = Cli::command();
    cmd.set_bin_name(name);
    cmd = cmd.after_help(examples_root(name));
    cmd
}

fn examples_root(name: &str) -> String {
    format!(
        "Examples:\n  {name} save --label work\n  {name} load --label work\n  {name} switch\n  {name} reload-app\n  {name} reserve --label vps-a\n  {name} unreserve --label vps-a\n  {name} migrate\n  {name} relay-login --url \"http://localhost:1455/auth/callback?code=...&state=...\"\n  {name} list\n  {name} status\n  {name} status --current\n  {name} delete --label work"
    )
}
