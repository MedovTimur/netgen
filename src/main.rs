use anyhow::Result;
use clap::{Parser, Subcommand};

use netgen::http_axum::{self, HttpAxumCmd};
use netgen::tcp_echo::{self, EchoCmd};
use netgen::tcp_worker::{self, WorkerCmd};

#[derive(Parser, Debug)]
#[command(
    name = "netgen",
    version,
    about = "Network code generator (TCP echo, worker-pool, HTTP axum, etc.)",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Generate TCP echo server
    TcpEcho(EchoCmd),

    /// Generate TCP worker-pool server
    TcpWorker(WorkerCmd),

    /// Generate HTTP service on axum
    HttpAxum(HttpAxumCmd),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Command::TcpEcho(cmd) => tcp_echo::run_from_cli(cmd),
        Command::TcpWorker(cmd) => tcp_worker::run_from_cli(cmd),
        Command::HttpAxum(cmd) => http_axum::run_from_cli(cmd),
    }
}
