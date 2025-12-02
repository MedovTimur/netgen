// src/tcp_echo.rs
use std::path::Path;

use anyhow::Result;
use clap::Args;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

use crate::read_mode::{ReadModeTemplateCtx, YamlReadMode};
use crate::util::{read_yaml, resolve_out_dir};

/// CLI command for generating TCP echo server.

#[derive(Debug, Args)]
pub struct EchoCmd {
    #[arg(long)]
    pub config: Option<String>,

    #[arg(short, long, default_value = "tcp-echo-server")]
    pub name: String,

    #[arg(short, long, default_value_t = 4000)]
    pub port: u16,

    #[arg(long, default_value_t = false)]
    pub tracing: bool,

    #[arg(long, default_value_t = false)]
    pub github_actions: bool,

    #[arg(long)]
    pub max_line_len: Option<usize>,

    #[arg(long)]
    pub out_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TcpEchoYamlConfig {
    pub project_name: String,
    pub port: u16,
    pub tracing: bool,
    pub read_mode: YamlReadMode,
    pub out_dir: Option<String>,
    pub github_actions: bool,
}

#[derive(Debug, Serialize)]
pub struct TcpEchoTemplateCtx {
    pub project_name: String,
    pub port: u16,
    pub tracing_enabled: bool,

    /// Everything related to read_mode (lines/fixed_size/delimited/length_prefixed)
    /// is flattened to the top level for handlebars.
    #[serde(flatten)]
    pub read_mode: ReadModeTemplateCtx,
    pub github_actions: bool,
}

impl From<TcpEchoYamlConfig> for TcpEchoTemplateCtx {
    fn from(cfg: TcpEchoYamlConfig) -> Self {
        let read_mode: ReadModeTemplateCtx = cfg.read_mode.into();

        TcpEchoTemplateCtx {
            project_name: cfg.project_name,
            port: cfg.port,
            tracing_enabled: cfg.tracing,
            read_mode,
            github_actions: cfg.github_actions
        }
    }
}

impl From<&EchoCmd> for TcpEchoTemplateCtx {
    fn from(cli: &EchoCmd) -> Self {
        // CLI currently only supports lines mode.
        let read_mode = ReadModeTemplateCtx {
            is_lines: true,
            max_line_len: cli.max_line_len,
            ..Default::default()
        };

        TcpEchoTemplateCtx {
            project_name: cli.name.clone(),
            port: cli.port,
            tracing_enabled: cli.tracing,
            read_mode,
            github_actions: cli.github_actions
        }
    }
}

/// Generate TCP echo server project from template context.
pub fn generate_tcp_echo_project(ctx: &TcpEchoTemplateCtx, out_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(out_dir.join("src"))?;

    let mut hbs = Handlebars::new();
    // Disable HTML escaping since we're generating code, not HTML
    hbs.register_escape_fn(handlebars::no_escape);

    hbs.register_template_string(
        "cargo_toml",
        include_str!("../templates/tcp_echo/Cargo.toml.hbs"),
    )?;
    hbs.register_template_string("main_rs", include_str!("../templates/tcp_echo/main.rs.hbs"))?;

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

/// Entry point for TCP echo subcommand.
pub fn run_from_cli(cmd: EchoCmd) -> Result<()> {
    if let Some(config_path) = cmd.config.as_deref() {
        let cfg: TcpEchoYamlConfig = read_yaml(config_path)?;
        let cfg_out_dir = cfg.out_dir.clone();
        let ctx: TcpEchoTemplateCtx = cfg.into();

        let out_dir_str = resolve_out_dir(cmd.out_dir.clone(), cfg_out_dir, &ctx.project_name);
        let out_dir = Path::new(&out_dir_str);

        generate_tcp_echo_project(&ctx, out_dir)?;

        println!(
            "✅ Generated TCP echo project (YAML) in {}",
            out_dir.to_string_lossy()
        );
        return Ok(());
    }

    let ctx: TcpEchoTemplateCtx = (&cmd).into();
    let out_dir_str = resolve_out_dir(cmd.out_dir.clone(), None, &ctx.project_name);
    let out_dir = Path::new(&out_dir_str);

    generate_tcp_echo_project(&ctx, out_dir)?;

    println!(
        "✅ Generated TCP echo project (CLI) in {}",
        out_dir.to_string_lossy()
    );

    Ok(())
}
