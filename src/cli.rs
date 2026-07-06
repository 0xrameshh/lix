use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "lix",
    version,
    about = "Extract, clean, and format AI agent traces into ML-ready JSONL"
)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Extract and convert agent traces to training JSONL.
    /// Scans common agent log directories (~/.claude/logs, etc.) when no path given.
    Extract {
        /// Path to a trace file or directory of trace files.
        /// If omitted, scans common agent log directories.
        input: Option<PathBuf>,

        /// Output JSONL file path
        #[arg(short, long, default_value = "output.jsonl")]
        out: PathBuf,

        /// Agent provider (auto-detected by default)
        #[arg(short, long, default_value = "auto")]
        provider: String,

        /// Only process traces matching this model name (substring match)
        #[arg(short = 'm', long)]
        model: Option<String>,

        /// Skip anonymization/cleaning
        #[arg(long = "no-clean")]
        no_clean: bool,

        /// Keep incomplete traces (ending on tool result)
        #[arg(long = "keep-incomplete")]
        keep_incomplete: bool,

        /// Number of worker threads
        #[arg(short, long, env = "LIX_CONCURRENCY")]
        concurrency: Option<usize>,

        /// Write a JSON report to this path
        #[arg(short, long)]
        report: Option<PathBuf>,

        /// Suppress progress output
        #[arg(short, long)]
        quiet: bool,

        /// Stage raw traces and output to a dataset directory
        #[arg(long)]
        staging_dir: Option<PathBuf>,
    },

    /// Alias for extract --no-clean
    Convert {
        input: Option<PathBuf>,
        #[arg(short, long, default_value = "output.jsonl")]
        out: PathBuf,
        #[arg(short, long)]
        model: Option<String>,
        #[arg(short, long, env = "LIX_CONCURRENCY")]
        concurrency: Option<usize>,
        #[arg(short, long)]
        report: Option<PathBuf>,
        #[arg(short, long)]
        quiet: bool,
    },

    /// Re-clean an already-extracted JSONL file
    Clean {
        input: PathBuf,
        #[arg(short, long, default_value = "cleaned.jsonl")]
        out: PathBuf,
    },

    /// Show stats for a trace file
    Info { file: PathBuf },

    /// Discover trace files in common locations
    Find {
        /// Search in common directories (default) or all locations
        #[arg(short, long)]
        all: bool,
    },

    /// Generate synthetic trace data for testing
    Generate {
        /// Output directory for synthetic traces
        #[arg(short, long, default_value = "synthetic_traces")]
        out: PathBuf,

        /// Number of synthetic sessions to generate
        #[arg(short, long, default_value_t = 3)]
        count: usize,

        /// Provider to generate traces for (default: random)
        #[arg(short, long, default_value = "random")]
        provider: String,
    },

    /// Verify output byte-for-byte against a golden reference.
    /// Useful for regression testing.
    Verify {
        /// Path to input trace file or directory
        input: PathBuf,

        /// Path to golden output file (default: tests/goldens/<input-basename>)
        #[arg(short, long)]
        golden: Option<PathBuf>,

        /// Agent provider (auto-detected by default)
        #[arg(short, long, default_value = "auto")]
        provider: String,
    },

    /// Launch the trace studio (web UI for browsing traces)
    /// Opens a local web server for exploring and analyzing trace files.
    Studio {
        /// Path to a trace file or directory of trace files
        #[arg(default_value = ".")]
        input: PathBuf,

        /// Port for the web server
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
}
