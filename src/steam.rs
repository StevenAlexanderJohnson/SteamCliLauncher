use anyhow::{anyhow, Context};

use std::process::Command;

use crate::config3::{Config, ConfigWrapper};

pub fn launch_game(config: &Config, app_id: &str) -> Result<(), anyhow::Error> {
    let exe_paths = config.get_executable_paths()?;
    for path in exe_paths {
        if let Ok(mut command) = Command::new(path.trim()).arg("-applaunch").arg(app_id.trim()).spawn() {
            let result = command
                .wait()
                .context("Your game was found but was not able to be launched.")?;
            println!("Your game has launched with status: {}", result);
            return Ok(());
        }
    }
    Err(anyhow!("Unable to start your game. It could be that it is not installed in any of the registered Steam Libraries."))
}
