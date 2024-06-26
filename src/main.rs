mod api_methods;
mod menu;
mod steam;

use std::env;

pub struct SteamInfo {
    pub api_key: String,
    pub user_id: String,
    pub steam_exe_path: String,
}


#[tokio::main]
async fn main() {
    let api_key = env::args().skip(1).collect::<Vec<String>>();
    if api_key.len() != 1 {
        println!("ONLY PROVIDE THE API KEY AFTER THE EXECUTABLE.");
        return
    }

    let steam_info = SteamInfo {
        api_key: api_key.first().expect("API KEY not provided.").to_owned(),
        user_id: "76561198106848715".to_owned(),
        steam_exe_path: "C:\\Program Files (x86)\\Steam\\Steam.exe".to_owned(),
    };

    let user_profile = api_methods::get_player_info(&steam_info)
        .await
        .expect("Unable to retrieve player info.");
    println!("Welcome back {}", user_profile.personaname);
    println!("Rust Steam Launcher");

    menu::launch_menu(&steam_info).await;
}
