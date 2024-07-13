use std::path::Path;

use color_eyre::{config::HookBuilder, Result};
use config::Config;
use viewer::Window;

fn main() -> Result<()> {
    HookBuilder::new().display_env_section(false).install()?;

    let config_path = Path::new("config.toml");
    let initial_config = Config::try_from_path(config_path)?;

    let window = Window::try_new()?;
    window.run_render_loop(initial_config);

    Ok(())
}
