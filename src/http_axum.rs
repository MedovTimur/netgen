use std::path::Path;

use anyhow::Result;
use clap::Args;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

use crate::util::{read_yaml, resolve_out_dir};

/// Subcommand for generating HTTP service on axum.
#[derive(Debug, Args)]
pub struct HttpAxumCmd {
    /// Path to YAML config (required).
    #[arg(long)]
    pub config: String,

    /// Override output directory.
    #[arg(long)]
    pub out_dir: Option<String>,
}

/// HTTP method in YAML config.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

/// Description of a single route in YAML.
#[derive(Debug, Deserialize)]
pub struct HttpRouteYaml {
    pub path: String,
    pub method: HttpMethod,
    /// Handler name that will be generated.
    pub handler: String,
    /// Response text (simple text).
    pub response: String,
}

/// Database config in YAML.
#[derive(Debug, Deserialize)]
pub struct DatabaseYamlConfig {
    /// Enable database initialization.
    pub enabled: bool,
    /// Database type (currently only Postgres is used, but field for future use).
    #[serde(default = "default_db_kind")]
    pub kind: String,
    /// Environment variable name with URL (e.g., DATABASE_URL).
    pub url_env: String,
    /// Maximum number of connections in pool (optional).
    pub max_connections: Option<u32>,
}

fn default_db_kind() -> String {
    "postgres".to_string()
}

#[derive(Debug, Deserialize)]
pub struct HttpAxumYamlConfig {
    pub project_name: String,
    pub port: u16,
    pub tracing: bool,
    pub routes: Vec<HttpRouteYaml>,
    pub out_dir: Option<String>,
    /// Optional database block.
    pub database: Option<DatabaseYamlConfig>,
}

/// Route as it appears in the template.
#[derive(Debug, Serialize)]
pub struct RouteTemplate {
    pub path: String,
    /// Builder function name from `axum::routing` — get / post / put / delete.
    pub method_fn: String,
    /// Handler name in handlers module.
    pub handler_name: String,
    /// Text response.
    pub response: String,
}

/// Context passed to http-axum templates.
#[derive(Debug, Serialize)]
pub struct HttpAxumTemplateCtx {
    pub project_name: String,
    pub port: u16,
    pub tracing_enabled: bool,
    pub routes: Vec<RouteTemplate>,

    // ---- Database ----
    pub db_enabled: bool,
    pub db_url_env: Option<String>,
    pub db_max_connections: Option<u32>,
}

impl From<HttpAxumYamlConfig> for HttpAxumTemplateCtx {
    fn from(cfg: HttpAxumYamlConfig) -> Self {
        let routes = cfg
            .routes
            .into_iter()
            .map(|r| {
                let method_fn = match r.method {
                    HttpMethod::GET => "get",
                    HttpMethod::POST => "post",
                    HttpMethod::PUT => "put",
                    HttpMethod::DELETE => "delete",
                }
                .to_string();

                RouteTemplate {
                    path: r.path,
                    method_fn,
                    handler_name: r.handler,
                    response: r.response,
                }
            })
            .collect::<Vec<_>>();

        let (db_enabled, db_url_env, db_max_connections) = if let Some(db) = cfg.database {
            if db.enabled {
                (
                    true,
                    Some(db.url_env),
                    db.max_connections,
                )
            } else {
                (false, None, None)
            }
        } else {
            (false, None, None)
        };

        HttpAxumTemplateCtx {
            project_name: cfg.project_name,
            port: cfg.port,
            tracing_enabled: cfg.tracing,
            routes,
            db_enabled,
            db_url_env,
            db_max_connections,
        }
    }
}

/// Generate HTTP axum service project from template context.
pub fn generate_http_axum_project(ctx: &HttpAxumTemplateCtx, out_dir: &Path) -> Result<()> {
    let src_dir = out_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    let mut hbs = Handlebars::new();
    // Disable HTML escaping since we're generating code, not HTML
    hbs.register_escape_fn(handlebars::no_escape);

    hbs.register_template_string(
        "cargo_toml",
        include_str!("../templates/http_axum/Cargo.toml.hbs"),
    )?;
    hbs.register_template_string(
        "main_rs",
        include_str!("../templates/http_axum/main.rs.hbs"),
    )?;
    hbs.register_template_string(
        "handlers_rs",
        include_str!("../templates/http_axum/handlers.rs.hbs"),
    )?;

    let cargo_toml = hbs.render("cargo_toml", ctx)?;
    std::fs::write(out_dir.join("Cargo.toml"), cargo_toml)?;

    let main_rs = hbs.render("main_rs", ctx)?;
    std::fs::write(src_dir.join("main.rs"), main_rs)?;

    let handlers_rs = hbs.render("handlers_rs", ctx)?;
    std::fs::write(src_dir.join("handlers.rs"), handlers_rs)?;

    Ok(())
}

/// Entrypoint для subcommand `http-axum`.
pub fn run_from_cli(cmd: HttpAxumCmd) -> Result<()> {
    let cfg: HttpAxumYamlConfig = read_yaml(&cmd.config)?;
    let cfg_out_dir = cfg.out_dir.clone();
    let ctx: HttpAxumTemplateCtx = cfg.into();

    let out_dir_str = resolve_out_dir(cmd.out_dir.clone(), cfg_out_dir, &ctx.project_name);
    let out_dir = Path::new(&out_dir_str);

    generate_http_axum_project(&ctx, out_dir)?;

    println!(
        "✅ Generated HTTP axum project in {}",
        out_dir.to_string_lossy()
    );

    Ok(())
}
