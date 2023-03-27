use clap::Parser;

/// nhfs is a server that serve a static directory and its subdirectories with templating.
#[derive(Clone, Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
  #[clap(short, long, default_value = "/etc/nhfs/nhfs.conf")]
  pub(crate) conf: String,
}
