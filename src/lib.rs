//! NetGen - Network code generator for Rust
//! 
//! This library provides functionality to generate network service code
//! including TCP echo servers, TCP worker-pool servers, and HTTP Axum services.

pub mod util;
pub mod read_mode;
pub mod tcp_echo;
pub mod tcp_worker;
pub mod http_axum;
