// src/tcp_worker.rs
use std::path::Path;

use anyhow::Result;
use clap::Args;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

use crate::read_mode::{ReadModeTemplateCtx, YamlReadMode};
use crate::util::{read_yaml, resolve_out_dir};

/// CLI command for generating TCP worker-pool server.

#[derive(Debug, Args)]
pub struct WorkerCmd {
    #[arg(long)]
    pub config: String,

    #[arg(long)]
    pub out_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TcpWorkerYamlConfig {
    pub project_name: String,
    pub port: u16,
    pub tracing: bool,
    pub workers: usize,
    pub event_buffer: usize,
    pub read_mode: YamlReadMode,
    pub out_dir: Option<String>,
    pub github_actions: bool,
}

#[derive(Debug, Serialize)]
pub struct TcpWorkerTemplateCtx {
    pub project_name: String,
    pub port: u16,
    pub tracing_enabled: bool,
    pub workers: usize,
    pub event_buffer: usize,

    /// Everything related to read_mode is flattened to the top level.
    #[serde(flatten)]
    pub read_mode: ReadModeTemplateCtx,
    pub github_actions: bool,
}

impl From<TcpWorkerYamlConfig> for TcpWorkerTemplateCtx {
    fn from(cfg: TcpWorkerYamlConfig) -> Self {
        let read_mode: ReadModeTemplateCtx = cfg.read_mode.into();

        TcpWorkerTemplateCtx {
            project_name: cfg.project_name,
            port: cfg.port,
            tracing_enabled: cfg.tracing,
            workers: cfg.workers,
            event_buffer: cfg.event_buffer,
            read_mode,
            github_actions: cfg.github_actions
        }
    }
}

/// Generate TCP worker-pool server project from template context.
pub fn generate_tcp_worker_project(ctx: &TcpWorkerTemplateCtx, out_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(out_dir.join("src"))?;

    let mut hbs = Handlebars::new();
    // Disable HTML escaping since we're generating code, not HTML
    hbs.register_escape_fn(handlebars::no_escape);

    hbs.register_template_string(
        "cargo_toml",
        include_str!("../templates/tcp_worker/Cargo.toml.hbs"),
    )?;
    hbs.register_template_string(
        "main_rs",
        include_str!("../templates/tcp_worker/main.rs.hbs"),
    )?;

    let cargo_toml = hbs.render("cargo_toml", ctx)?;
    std::fs::write(out_dir.join("Cargo.toml"), cargo_toml)?;

    let main_rs = hbs.render("main_rs", ctx)?;
    std::fs::write(out_dir.join("src/main.rs"), main_rs)?;

    // GitHub Actions
    if ctx.github_actions {
        hbs.register_template_string(
            "github_actions",
            include_str!("../templates/ci.yml.hbs"),
        )?;
        let gha = hbs.render("github_actions", ctx)?;
        let workflows_dir = out_dir.join(".github/workflows");
        std::fs::create_dir_all(&workflows_dir)?;
        std::fs::write(workflows_dir.join("ci.yml"), gha)?;
    }

    Ok(())
}

/// Entry point for TCP worker subcommand.
pub fn run_from_cli(cmd: WorkerCmd) -> Result<()> {
    let cfg: TcpWorkerYamlConfig = read_yaml(&cmd.config)?;
    let cfg_out_dir = cfg.out_dir.clone();
    let ctx: TcpWorkerTemplateCtx = cfg.into();

    let out_dir_str = resolve_out_dir(cmd.out_dir.clone(), cfg_out_dir, &ctx.project_name);
    let out_dir = Path::new(&out_dir_str);

    generate_tcp_worker_project(&ctx, out_dir)?;

    println!(
        "✅ Generated TCP worker server project in {}",
        out_dir.to_string_lossy()
    );

    Ok(())
}
