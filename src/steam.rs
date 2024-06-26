use anyhow::anyhow;

use crate::SteamInfo;
use std::process::Command;

pub fn launch_game(steam_info: &SteamInfo, app_id: &str) -> Result<(), anyhow::Error> {
    let mut command = Command::new(&steam_info.steam_exe_path)
        .arg("-applaunch")
        .arg(app_id)
        .spawn()
        .map_err(|err| anyhow!("An error occurred while starting your game. {}", err))?;

    let result = command
        .wait()
        .map_err(|err| anyhow!("Starting game exited with error: {}", err))?;
    println!("Your game has launched with status: {}", result);
    Ok(())
}
