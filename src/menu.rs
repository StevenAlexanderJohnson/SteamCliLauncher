use std::io::{self, Write};

use crate::{api_methods::SteamAPIHandler, config3::Config, steam};

pub async fn launch_menu<T: SteamAPIHandler>(config: &Config, steam_api: &T) {
    loop {
        println!("What would you like to do?\n");

        println!("G: Get List of games you can launch");
        println!("Q: Quit");
        print!("Enter command: ");

        let _ = io::stdout().flush();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        match input.trim() {
            "G" | "g" => print_game_menu(config, steam_api).await,
            "Q" | "q" => break,
            "clear" => println!("{}[2J", 27 as char),
            _ => println!("That is an unrecognized input."),
        };
    }
}

pub async fn prompt_launch_game(config: &Config) -> Result<(), anyhow::Error> {
    println!("\nWhat game would you like to launch?");
    print!("App Id: ");
    let _ = io::stdout().flush();
    let mut app_id = String::new();
    io::stdin().read_line(&mut app_id).unwrap();

    steam::launch_game(config, &app_id)?;
    Ok(())
}

pub async fn print_game_menu<T: SteamAPIHandler>(config: &Config, steam_api: &T) {
    loop {
        println!("\nWhat would you like to do with games?\n");
        println!("L: List games that you have played and own");
        println!("S: Start a game");
        println!("B: Back to home menu");
        print!("Enter command: ");

        let _ = io::stdout().flush();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        match input.trim() {
            "L" | "l" => match steam_api.get_user_games(config).await {
                Ok(x) => x.iter().for_each(|x| println!("{} ({})", x.name, x.appid)),
                Err(e) => println!("An error occurred while getting your games: {}", e),
            },
            "S" | "s" => match prompt_launch_game(&config).await {
                Ok(_) => break,
                Err(err) => println!("{}", err),
            },
            _ => break,
        }
        println!()
    }
}
