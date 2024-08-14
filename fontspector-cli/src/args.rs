use clap::Parser;
use fontspector_checkapi::StatusCode;

/// Quality control for OpenType fonts
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Hotfix
    #[clap(short, long)]
    pub hotfix: bool,

    /// Plugins to load
    #[clap(long, value_delimiter = ',')]
    pub plugins: Vec<String>,

    /// Profile to check
    #[clap(short, long, default_value = "universal")]
    pub profile: String,

    /// List the checks available in the selected profile
    #[clap(short = 'L', long)]
    pub list_checks: bool,

    /// Read configuration file (TOML/YAML)
    #[clap(long)]
    pub configuration: Option<String>,

    /// Explicit check-ids (or parts of their name) to be executed
    #[clap(short, long)]
    pub checkid: Option<Vec<String>>,

    /// Exclude check-ids (or parts of their name) from execution
    #[clap(short = 'x', long)]
    pub exclude_checkid: Option<Vec<String>>,

    /// Threshold for emitting process error code 1
    #[clap(short, long, arg_enum, value_parser, default_value_t=StatusCode::Fail)]
    pub error_code_on: StatusCode,

    /// Increase logging
    #[clap(short, long, parse(from_occurrences), help_heading = "Logging")]
    pub verbose: usize,

    /// Log level
    #[clap(short, long, arg_enum, value_parser, default_value_t=StatusCode::Warn, help_heading="Logging")]
    pub loglevel: StatusCode,

    /// Be quiet, don’t report anything on the terminal.
    #[clap(short, long, help_heading = "Logging")]
    pub quiet: bool,

    /// Timeout (in seconds) for network operations.
    #[clap(long, help_heading = "Network")]
    pub timeout: Option<u64>,

    /// Skip network checks
    #[clap(long, help_heading = "Network")]
    pub skip_network: bool,

    /// Input files
    pub inputs: Vec<String>,
}
