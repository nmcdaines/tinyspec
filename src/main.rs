use std::process;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::engine::ArgValueCompleter;

mod spec;

#[derive(Parser)]
#[command(name = "tinyspec", version, about = "A tiny framework for writing specs")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Set up Claude Code slash command skills and print shell completion instructions
    Init,

    /// Create a new spec
    New {
        /// Spec name in kebab-case
        spec_name: String,
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
}

fn main() {
    clap_complete::CompleteEnv::with_factory(Cli::command).complete();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init => spec::init(),
        Commands::New { spec_name } => spec::new_spec(&spec_name),
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
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
