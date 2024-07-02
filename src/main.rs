use config3::{Config, ConfigWrapper};

mod api_methods;
mod config3;
mod menu;
mod steam;

#[tokio::main]
async fn main() {
    let connection = Config::load_database().unwrap();

    // This should only return an error if SQLite had a problem preparing a statement.
    let config = Config::new(connection).unwrap();
    let user_profile = api_methods::get_player_info(&config)
        .await
        .expect("Unable to retrieve player info.");
    println!("Welcome back {}", user_profile.personaname);
    println!("Rust Steam Launcher");

    menu::launch_menu(&config).await;
}
