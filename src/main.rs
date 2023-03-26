use std::{fs, path::Path};

use clap::Parser;
use error::{CliResult, CliError};
use ntex::{
  web::{HttpServer, App},
  server::Server,
};

use crate::{error::FromIo, config::Config};

mod cli;
mod error;
mod config;
mod models;
mod services;

fn setup_server(config: &Config) -> CliResult<Server> {
  let state_config = config.clone();
  let mut server = HttpServer::new(move || {
    App::new()
      .state(state_config.clone())
      .configure(services::configure)
  });

  match config.host.as_str() {
    url if url.starts_with("http://") => {
      let url = url.trim_start_matches("http://");
      server = server.bind(url).map_err(|err| {
        err.map_err_context(|| format!("Failed to bind to {url}"))
      })?;
    }
    _ => {
      return Err(error::CliError::new(
        format!("Invalid host url: {}", config.host),
        2,
      ));
    }
  }

  Ok(server.run())
}

fn setup_config() -> CliResult<Config> {
  let args = cli::Cli::parse();

  let context_dir = Path::new(&args.conf).parent().ok_or(CliError::new(
    format!(
      "Failed to get parent directory of config file {}",
      args.conf
    ),
    2,
  ))?;

  let config_path = Path::new(&args.conf).canonicalize().map_err(|err| {
    err.map_err_context(|| {
      format!("Failed to resolve config path at {}", args.conf)
    })
  })?;
  let config_content = fs::read_to_string(&config_path).map_err(|err| {
    err.map_err_context(|| {
      format!("Failed to read config file {}", config_path.display())
    })
  })?;
  let mut config =
    serde_yaml::from_str::<Config>(&config_content).map_err(|err| {
      err.map_err_context(|| {
        format!("Failed to parse config file {}", config_path.display())
      })
    })?;

  config.path = Path::new(&context_dir)
    .join(&config.path)
    .canonicalize()
    .map_err(|err| {
      err.map_err_context(|| {
        format!("Failed to resolve config directory at {}", config.path)
      })
    })?
    .display()
    .to_string();

  config.directory = Path::new(&context_dir)
    .join(&config.directory)
    .canonicalize()
    .map_err(|err| {
      err.map_err_context(|| {
        format!("Failed to resolve config directory at {}", config.path)
      })
    })?
    .display()
    .to_string();

  Ok(config)
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
  let config = match setup_config() {
    Err(err) => err.exit(),
    Ok(config) => config,
  };

  let server = match setup_server(&config) {
    Err(err) => err.exit(),
    Ok(server) => server,
  };

  println!("Server listening on: {}", config.host);

  server.await?;

  Ok(())
}
