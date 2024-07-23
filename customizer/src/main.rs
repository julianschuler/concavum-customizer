use color_eyre::{config::HookBuilder, Result};
use viewer::Window;

fn main() -> Result<()> {
    HookBuilder::new().display_env_section(false).install()?;

    let window = Window::try_new()?;
    window.run_render_loop();

    Ok(())
}
