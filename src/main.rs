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
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Include archived specs
        #[arg(long)]
        include_archived: bool,
    },

    /// Display the contents of a spec
    View {
        /// Spec name
        #[arg(add = ArgValueCompleter::new(spec::complete_spec_names))]
        spec_name: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
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
        /// Task ID (e.g. A, A.1, B, or emoji like 🧪, 🧪.1)
        task_id: String,
    },

    /// Mark a task as incomplete
    Uncheck {
        /// Spec name
        #[arg(add = ArgValueCompleter::new(spec::complete_spec_names))]
        spec_name: String,
        /// Task ID (e.g. A, A.1, B, or emoji like 🧪, 🧪.1)
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
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Include archived specs
        #[arg(long)]
        include_archived: bool,
    },

    /// Manage repository configuration (~/.tinyspec/config.yaml)
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// List available spec templates
    Templates,

    /// Launch a real-time TUI dashboard showing spec progress
    Dashboard {
        /// Include archived specs
        #[arg(long)]
        include_archived: bool,
    },

    /// Search specs by title or body content
    Search {
        /// Search query
        query: String,
        /// Narrow search to a specific group folder
        #[arg(long)]
        group: Option<String>,
        /// Filter by spec status
        #[arg(long, value_name = "STATUS")]
        status: Option<String>,
    },

    /// Move a spec to the archive
    Archive {
        /// Spec name (omit if using --all-completed)
        #[arg(add = ArgValueCompleter::new(spec::complete_spec_names), required_unless_present = "all_completed")]
        spec_name: Option<String>,
        /// Archive all completed specs
        #[arg(long)]
        all_completed: bool,
    },

    /// Move a spec out of the archive
    Unarchive {
        /// Spec name
        spec_name: String,
    },

    /// Validate spec health
    Lint {
        /// Spec name (omit to lint all specs)
        #[arg(add = ArgValueCompleter::new(spec::complete_spec_names))]
        spec_name: Option<String>,
        /// Lint all specs
        #[arg(long)]
        all: bool,
    },
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
        Commands::List {
            json,
            include_archived,
        } => spec::list(json, include_archived),
        Commands::View { spec_name, json } => spec::view(&spec_name, json),
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
        Commands::Status {
            spec_name,
            json,
            include_archived,
        } => spec::status(spec_name.as_deref(), json, include_archived),
        Commands::Config { action } => match action {
            ConfigAction::Set { repo_name, path } => spec::config_set(&repo_name, &path),
            ConfigAction::List => spec::config_list(),
            ConfigAction::Remove { repo_name } => spec::config_remove(&repo_name),
        },
        Commands::Templates => spec::list_templates(),
        Commands::Dashboard { include_archived } => spec::dashboard::run(include_archived),
        Commands::Search {
            query,
            group,
            status,
        } => spec::search(&query, group.as_deref(), status.as_deref()),
        Commands::Archive {
            spec_name,
            all_completed,
        } => {
            if all_completed {
                spec::archive_all_completed()
            } else {
                spec::archive_spec(spec_name.as_deref().unwrap())
            }
        }
        Commands::Unarchive { spec_name } => spec::unarchive_spec(&spec_name),
        Commands::Lint { spec_name, all } => spec::lint(spec_name.as_deref(), all),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
