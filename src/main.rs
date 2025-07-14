use anyhow::Result;
mod app_state;
mod authenticate;
mod render;
mod state;
mod tui;
mod utilities;

use env_logger::{Builder, Target};
use log::LevelFilter;

use std::fs::File;
use std::io::Write;

//unsused imports are sometimes used in testing and are not to be removed
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Err(e) = state::load_and_set_token() {
        eprintln!("Failed to load token: {}", e);
    }

    tui::tui_render().await?;

    Ok(())
}
