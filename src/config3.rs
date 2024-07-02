use std::{
    env, fs,
    io::{self, Write},
    path::Path,
};

use anyhow::anyhow;
use sqlite::State;

pub trait ConfigWrapper {
    fn load_database() -> Result<sqlite::Connection, anyhow::Error>;
    fn new(connection: sqlite::Connection) -> Result<Config, anyhow::Error>;
    fn get_api_key(&self) -> Result<String, anyhow::Error>;
    fn get_executable_paths(&self) -> Result<Vec<String>, anyhow::Error>;
    fn get_user_id(&self) -> Result<String, anyhow::Error>;
    fn set_api_key(&self, api_key: &str) -> Result<(), anyhow::Error>;
    fn add_executable_path(&self, path: &str) -> Result<(), anyhow::Error>;
    fn set_user_id(&self, user_id: &str) -> Result<(), anyhow::Error>;
}

pub struct Config {
    connection: sqlite::Connection,
}

impl ConfigWrapper for Config {
    fn load_database() -> Result<sqlite::Connection, anyhow::Error> {
        let mut app_data = match dirs::data_dir() {
            Some(mut path) => {
                path.push("SteamCliLauncher");
                path
            }
            None => return Err(anyhow!("Application Data directory could not be found.")),
        };

        fs::create_dir_all(app_data.as_os_str())?;
        app_data.push("config.db");

        if !Path::new(&app_data).exists() {
            fs::File::create(&app_data)?;
        }

        let connection = sqlite::Connection::open(app_data)?;
        Ok(connection)
    }

    fn new(connection: sqlite::Connection) -> Result<Self, anyhow::Error> {
        let config = Config { connection };
        {
            config
                .connection
                .execute("CREATE TABLE IF NOT EXISTS application_data (type TEXT, value TEXT)")
                .unwrap();
            config.connection
                .execute("CREATE TABLE IF NOT EXISTS executable_paths (id INTEGER PRIMARY KEY, exe_path TEXT)")
                .unwrap();
            let query = "SELECT value FROM application_data WHERE type='api_key'";
            let mut command = config.connection.prepare(query)?;
            // Will error out on first use or if the file was deleted.
            match command.next()? {
                State::Row => {
                    println!("Configuration is ready.");
                }
                State::Done => {
                    initialize_setup(&config)?;
                }
            }
        }
        Ok(config)
    }

    fn get_api_key(&self) -> Result<String, anyhow::Error> {
        let query = "SELECT value FROM application_data WHERE type = 'api_key'";
        let mut command = self.connection.prepare(query).unwrap();

        match command.next()? {
            State::Row => Ok(command.read::<String, _>("value")?),
            State::Done => Err(anyhow!("Unable to find your api_key in the database."))
        }
    }

    fn get_executable_paths(&self) -> Result<Vec<String>, anyhow::Error> {
        let query = "SELECT * FROM executable_paths";
        let mut output: Vec<String> = vec![];
        let mut command = self.connection.prepare(query).unwrap();

        while let Ok(State::Row) = command.next() {
            output.push(command.read::<String, _>("exe_path").unwrap());
        }

        Ok(output)
    }

    fn get_user_id(&self) -> Result<String, anyhow::Error> {
        let query = "SELECT value FROM application_data WHERE type = 'user_id'";
        let mut command = self.connection.prepare(query)?;

        match command.next()? {
            State::Row => Ok(command.read::<String, _>("value")?),
            State::Done => Err(anyhow!("Unable to find your user_id in the database."))
        }
    }

    fn set_api_key(&self, api_key: &str) -> Result<(), anyhow::Error> {
        let query = "INSERT INTO application_data (type, value) VALUES ('api_key', :api_key)";
        let mut command = self.connection.prepare(query).unwrap();
        command.bind((":api_key", api_key)).unwrap();

        command
            .next()
            .map_err(|e| anyhow!("Unable to update user_id in the database: {}", e))?;

        Ok(())
    }

    fn add_executable_path(&self, path: &str) -> Result<(), anyhow::Error> {
        let query = "INSERT INTO executable_paths(exe_path) VALUES (:path)";
        let mut command = self.connection.prepare(query).unwrap();
        command.bind((":path", path)).unwrap();

        command
            .next()
            .map_err(|e| anyhow!("Unable to add executable path to the database. {}", e))?;

        Ok(())
    }

    fn set_user_id(&self, user_id: &str) -> Result<(), anyhow::Error> {
        let query = "INSERT INTO application_data (type, value) values ('user_id', :user_id)";
        let mut command = self.connection.prepare(query).unwrap();
        command.bind((":user_id", user_id)).unwrap();

        command
            .next()
            .map_err(|e| anyhow!("Unable to the user_id to the database. {}", e))?;

        Ok(())
    }
}

fn initialize_setup(config: &Config) -> Result<(), anyhow::Error> {
    let default_steam_path = match env::consts::OS {
        "linux" => "~/.local/share/Steam",
        "windows" => "C:\\Program Files (x86)\\Steam\\steam.exe",
        "macos" => "~/Library/Application Support (this is hidden to users by default)",
        _ => "a folder on your machine, why aren't you using a normal OS?",
    };

    println!(
        r#"
    _       ____    __     ____
     )  ____)  /  __) \   |    
    (  (___   |  /     |  |    
     \___  \  | |      |  |    
     ____)  ) |  \__   |  |__  
    (      (___\    )_/      )_
    "#
    );

    println!("It seems like this is the first time you're launching Steam CLI Launcher!");
    println!("Before moving forward there is some configuration we have to do to make the CLI work as expected.\n");

    println!("First things first, provide the path to your Steam executable.");
    println!("This is usually in {}.", default_steam_path);
    print!("Path: ");

    let _ = io::stdout().flush();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    config.add_executable_path(&input)?;

    println!("\nIn order for the CLI tool to work, it also requires a Steam API Key.");
    println!("This is used to collect what games you own, and to look up the AppID of the game you want to launch.");
    print!("API Key: ");

    let _ = io::stdout().flush();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    config.set_api_key(&input)?;

    println!("\nYour Steam ID is also required for the CLI to work.");
    println!("This can be found on your account page in Steam.");
    print!("Steam ID: ");

    let _ = io::stdout().flush();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    config.set_user_id(&input)?;

    println!("\n");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Config, ConfigWrapper};

    fn setup_mock_db() -> sqlite::Connection {
        let connection = sqlite::Connection::open(":memory:").unwrap();

        connection
            .execute("CREATE TABLE application_data (type TEXT, value TEXT)")
            .unwrap();
        connection
            .execute("CREATE TABLE executable_paths (id INTEGER PRIMARY KEY, exe_path TEXT)")
            .unwrap();
        connection
            .execute("INSERT INTO executable_paths (exe_path) VALUES ('testing')")
            .unwrap();
        connection
            .execute("INSERT INTO executable_paths (exe_path) VALUES ('testing2')")
            .unwrap();
        connection
            .execute("INSERT INTO executable_paths (exe_path) VALUES ('testing3')")
            .unwrap();

        connection
            .execute("INSERT INTO application_data (type, value) VALUES ('api_key', '123')")
            .unwrap();

        connection
    }

    #[test]
    fn get_api_key_succeeds() {
        let connection = setup_mock_db();
        let wrapper = Config::new(connection).unwrap();

        let api_key: String = match wrapper.get_api_key() {
            Ok(x) => x,
            Err(e) => panic!("get_api_keys_succeeds paniced: {}", e),
        };

        println!("{}", &api_key);

        assert_eq!(api_key.len(), 3);
    }

    #[test]
    fn get_executable_paths_succeeds() {
        let connection = setup_mock_db();
        let wrapper = Config::new(connection).unwrap();

        let paths = match wrapper.get_executable_paths() {
            Ok(x) => x,
            Err(e) => panic!("get_executable_paths_failed: {}", e),
        };

        assert_eq!(paths.len(), 3);
    }
}
