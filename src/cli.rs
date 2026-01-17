use clap::{Command, CommandFactory, Parser, Subcommand};

use crate::command_name;

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
        /// Show usage for the profile matching this label
        #[arg(value_name = "label")]
        #[arg(long)]
        label: Option<String>,
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
    /// Export saved profiles to another tool
    Export {
        /// Export to Every Code's auth_accounts.json
        #[arg(long)]
        every_code: bool,
        /// Override CODE_HOME (defaults to $CODE_HOME or ~/.code)
        #[arg(value_name = "path")]
        #[arg(long)]
        code_home: Option<String>,
        /// Replace any existing auth_accounts.json instead of merging
        #[arg(long)]
        overwrite: bool,
    },
    /// Import profiles from another tool
    Import {
        /// Import from Every Code's auth_accounts.json
        #[arg(long)]
        every_code: bool,
        /// Override CODE_HOME (defaults to $CODE_HOME or ~/.code)
        #[arg(value_name = "path")]
        #[arg(long)]
        code_home: Option<String>,
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
        "Examples:\n  {name} save --label work\n  {name} load --label work\n  {name} list\n  {name} status\n  {name} delete --label work\n  {name} export --every-code\n  {name} import --every-code"
    )
}
