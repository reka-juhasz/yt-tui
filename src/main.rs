use anyhow::Result;
mod app_state;
mod authenticate;
mod colors;
mod render;
mod state;
mod tui;
mod utilities;


#[tokio::main]
async fn main() -> anyhow::Result<()> {

    tui::tui_render().await?;
 
    Ok(())
}
