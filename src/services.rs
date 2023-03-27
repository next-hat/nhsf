use std::fs;
use std::path::Path;
use chrono::{DateTime, Local};

use ntex::web;
use ntex::http::StatusCode;
use ntex_files::NamedFile;

use crate::config::Config;
use crate::error::{HttpError, HttpResult, AsHttpError, FromIo};
use crate::models::{FileContext, DirTemplateContext};

#[web::get("/{all}*")]
async fn serve(
  req: web::HttpRequest,
  path: web::types::Path<String>,
  config: web::types::State<Config>,
) -> HttpResult {
  if path.starts_with("static") {
    let asset = Path::new(&config.path)
      .join(&*path)
      .canonicalize()
      .map_err(|err| {
        err
          .map_err_context(|| format!("Failed to resolve path /{path}"))
          .set_status(StatusCode::NOT_FOUND)
      })?;

    // Protect against directory traversing
    if !asset.starts_with(&config.path) {
      return Err(HttpError {
        msg: format!("Path /{} is not found", asset.display(),),
        status: StatusCode::NOT_FOUND,
      });
    }

    let asset_path = asset.clone();
    let res =
      web::block(move || NamedFile::open(asset))
        .await
        .map_err(|err| {
          err
            .map_err_context(|| {
              format!("Failed to open file {}", asset_path.display())
            })
            .set_status(StatusCode::NOT_FOUND)
        })?;
    return Ok(res.into_response(&req));
  }

  let path = Path::new(&config.directory)
    .join(&*path)
    .canonicalize()
    .map_err(|err| {
      err
        .map_err_context(|| format!("Failed to resolve path /{path}"))
        .set_status(StatusCode::NOT_FOUND)
    })?;

  // Protect against directory traversing
  if !path.starts_with(&config.directory) {
    return Err(HttpError {
      msg: format!(
        "Path /{path} is not in the directory {directory}",
        path = path.display(),
        directory = config.directory
      ),
      status: StatusCode::NOT_FOUND,
    });
  }

  // If the path is a directory we serve and html file from the dir.stpl template
  if path.is_dir() {
    let dir = path.clone();
    let err_dir = dir.clone();
    let files =
      web::block(move || fs::read_dir(dir))
        .await
        .map_err(move |err| {
          err
            .map_err_context(|| {
              format!("Failed to read directory {}", err_dir.display())
            })
            .set_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?;
    let template_path = Path::new(&config.path)
      .join("templates/dir.html")
      .canonicalize()
      .map_err(|err| {
        err
          .map_err_context(|| {
            format!("Failed to resolve templates/dir.html in {}", config.path)
          })
          .set_status(StatusCode::INTERNAL_SERVER_ERROR)
      })?;

    let template = template_path.clone();
    let template_str = web::block(move || fs::read_to_string(template))
      .await
      .map_err(|err| {
      err
        .map_err_context(|| {
          format!("Failed to read template file {}", template_path.display())
        })
        .set_status(StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    let mut dir_template_ctx = DirTemplateContext::default();

    for file in files {
      let Ok(file) = file else {
        continue;
      };
      let Ok(meta) = file.metadata() else {
        continue;
      };
      let Ok(modified) = meta.modified() else {
        continue;
      };
      let modified: DateTime<Local> = modified.into();

      let extension = file
        .path()
        .extension()
        .map(|ext| ext.to_string_lossy().to_string());

      let mut icon = if let Some(extension) = extension {
        config
          .icons
          .clone()
          .unwrap_or_default()
          .get(&extension)
          .cloned()
      } else {
        None
      };

      if icon.is_none() {
        icon = config
          .icons
          .clone()
          .unwrap_or_default()
          .get(&file.file_name().to_string_lossy().to_string())
          .cloned()
      }

      let file = FileContext {
        icon,
        path: file
          .path()
          .to_string_lossy()
          .to_string()
          .replace(&config.directory, ""),
        is_directory: meta.is_dir(),
        is_file: meta.is_file(),
        last_modified: modified.format("%Y-%m-%d %H:%M:%S").to_string(),
        size: meta.len().to_string(),
        name: file.file_name().to_string_lossy().to_string(),
      };
      dir_template_ctx.files.push(file);
    }
    dir_template_ctx.files.sort_by(|a, b| a.name.cmp(&b.name));
    let template = template_path.clone();
    let html = mustache::compile_str(&template_str).map_err(move |err| {
      err
        .map_err_context(|| {
          format!("Failed to compile template file {}", template.display())
        })
        .set_status(StatusCode::INTERNAL_SERVER_ERROR)
    })?;
    let template = template_path.clone();
    let render =
      html
        .render_to_string(&dir_template_ctx)
        .map_err(move |err| {
          err
            .map_err_context(|| {
              format!("Failed to render template file {}", template.display())
            })
            .set_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?;
    return Ok(
      web::HttpResponse::Ok()
        .content_type("text/html")
        .body(render),
    );
  }

  if path.is_file() {
    let file = path.clone();
    let err_file = file.clone();
    let res =
      web::block(move || NamedFile::open(file))
        .await
        .map_err(move |err| {
          err
            .map_err_context(|| {
              format!("Failed to open file {}", err_file.display())
            })
            .set_status(StatusCode::NOT_FOUND)
        })?;
    return Ok(res.into_response(&req));
  }

  // Should most likely never happen
  Ok(web::HttpResponse::NotFound().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg.service(serve);
}
