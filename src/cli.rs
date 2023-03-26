use clap::Parser;

/// nhfs is a file server that serve static file and provide a way to configure the directory html rendering
#[derive(Clone, Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
  #[clap(short, long, default_value = "/etc/nhfs/config.toml")]
  pub(crate) conf: String,
}
