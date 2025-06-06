pub mod packet;
pub mod proto;
pub mod resource;
pub mod server_config;
pub mod time;

pub fn init_tracing() {
    #[cfg(target_os = "windows")]
    ansi_term::enable_ansi_support().unwrap_or_else(|e| eprintln!("Failed enabling ansi: {}", e));

    tracing_subscriber::fmt().without_time().init();
}
