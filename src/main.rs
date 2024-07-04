use api_methods::SteamAPIHandler;
use config3::{Config, ConfigWrapper};

mod api_methods;
mod config3;
mod menu;
mod steam;

#[tokio::main]
async fn main() {
    let connection = Config::load_database().unwrap();

    let steam_api = api_methods::SteamAPI::new(None);

    // This should only return an error if SQLite had a problem preparing a statement.
    let config = Config::new(connection).unwrap();
    let user_profile = steam_api.get_player_info(&config)
        .await
        .expect("Unable to retrieve player info.");
    println!("Welcome back {}", user_profile.personaname);
    println!("Rust Steam Launcher");

    menu::launch_menu(&config, &steam_api).await;
}
