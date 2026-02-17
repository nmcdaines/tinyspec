use std::process;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::engine::ArgValueCompleter;

mod spec;

#[derive(Parser)]
#[command(
    name = "tinyspec",
    version,
    about = "A tiny framework for writing specs"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Set up Claude Code slash command skills and print shell completion instructions
    Init {
        /// Overwrite existing command files with the latest skill prompts
        #[arg(short, long)]
        force: bool,
    },

    /// Create a new spec
    New {
        /// Spec name in kebab-case
        spec_name: String,
        /// Use a named template (from .specs/templates/ or ~/.config/tinyspec/templates/)
        #[arg(short, long)]
        template: Option<String>,
    },

    /// List all specs
    List,

    /// Display the contents of a spec
    View {
        /// Spec name
        #[arg(add = ArgValueCompleter::new(spec::complete_spec_names))]
        spec_name: String,
    },

    /// Open a spec in your default editor
    Edit {
        /// Spec name
        #[arg(add = ArgValueCompleter::new(spec::complete_spec_names))]
        spec_name: String,
    },

    /// Delete a spec
    Delete {
        /// Spec name
        #[arg(add = ArgValueCompleter::new(spec::complete_spec_names))]
        spec_name: String,
    },

    /// Mark a task as complete
    Check {
        /// Spec name
        #[arg(add = ArgValueCompleter::new(spec::complete_spec_names))]
        spec_name: String,
        /// Task ID (e.g. A, A.1, B)
        task_id: String,
    },

    /// Mark a task as incomplete
    Uncheck {
        /// Spec name
        #[arg(add = ArgValueCompleter::new(spec::complete_spec_names))]
        spec_name: String,
        /// Task ID (e.g. A, A.1, B)
        task_id: String,
    },

    /// Format a spec's Markdown (or all specs with --all)
    Format {
        /// Spec name (omit if using --all)
        #[arg(add = ArgValueCompleter::new(spec::complete_spec_names), required_unless_present = "all")]
        spec_name: Option<String>,
        /// Format all specs
        #[arg(long)]
        all: bool,
    },

    /// Show completion progress for a spec (or all specs)
    Status {
        /// Spec name (shows all specs if omitted)
        #[arg(add = ArgValueCompleter::new(spec::complete_spec_names))]
        spec_name: Option<String>,
    },

    /// Manage repository configuration (~/.tinyspec/config.yaml)
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// List available spec templates
    Templates,

    /// Launch a real-time TUI dashboard showing spec progress
    Dashboard,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Map a repository name to a folder path
    Set {
        /// Repository name
        repo_name: String,
        /// Folder path
        path: String,
    },
    /// List all repository mappings
    List,
    /// Remove a repository mapping
    Remove {
        /// Repository name
        repo_name: String,
    },
}

fn main() {
    clap_complete::CompleteEnv::with_factory(Cli::command).complete();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { force } => spec::init(force),
        Commands::New {
            spec_name,
            template,
        } => spec::new_spec(&spec_name, template.as_deref()),
        Commands::List => spec::list(),
        Commands::View { spec_name } => spec::view(&spec_name),
        Commands::Edit { spec_name } => spec::edit(&spec_name),
        Commands::Delete { spec_name } => spec::delete(&spec_name),
        Commands::Check { spec_name, task_id } => spec::check_task(&spec_name, &task_id, true),
        Commands::Uncheck { spec_name, task_id } => spec::check_task(&spec_name, &task_id, false),
        Commands::Format { spec_name, all } => {
            if all {
                spec::format_all_specs()
            } else {
                spec::format_spec(spec_name.as_deref().unwrap())
            }
        }
        Commands::Status { spec_name } => spec::status(spec_name.as_deref()),
        Commands::Config { action } => match action {
            ConfigAction::Set { repo_name, path } => spec::config_set(&repo_name, &path),
            ConfigAction::List => spec::config_list(),
            ConfigAction::Remove { repo_name } => spec::config_remove(&repo_name),
        },
        Commands::Templates => spec::list_templates(),
        Commands::Dashboard => spec::dashboard::run(),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
