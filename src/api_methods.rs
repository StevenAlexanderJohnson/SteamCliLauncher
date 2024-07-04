use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config3::{Config, ConfigWrapper};

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerInfo {
    pub steamid: String,
    pub personaname: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    pub appid: i64,
    pub name: String,
    pub playtime_forever: i32,
}
pub trait SteamAPIHandler {
    fn new(client: Option<reqwest::Client>) -> Self;
    async fn get_user_games(&self, config: &Config) -> Result<Vec<Game>, anyhow::Error>;
    async fn get_player_info(&self, config: &Config) -> Result<PlayerInfo, anyhow::Error>;
}

pub struct SteamAPI {
    http: reqwest::Client,
}

impl SteamAPIHandler for SteamAPI {
    fn new(client: Option<reqwest::Client>) -> Self {
        SteamAPI {
            http: match client {
                Some(c) => c,
                None => reqwest::Client::new(),
            },
        }
    }
    async fn get_user_games(&self, config: &Config) -> Result<Vec<Game>, anyhow::Error> {
        let response = self.http.get(
        format!("https://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key={}&steamid={}&include_appinfo=true&include_played_free_games=true&format=json", 
        config.get_api_key()?,
        config.get_user_id()?
    )).send()
        .await
        .map_err(|err| anyhow!(format!("Request failed.:\n\t{:?}", err)))?
        .text()
        .await
        .map_err(|err| anyhow!(format!("Unable to get text response from response:\n\t{:?}", err)))?;

        let response: Value =
            serde_json::from_str(&response).context("Failed to parse response")?;

        let games = response["response"]["games"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("The structure of the JSON response is invalid."))?;

        let output = games
            .iter()
            .filter_map(|game| {
                let deserialized: Result<Game, _> = serde_json::from_value(game.clone());
                match deserialized {
                    Ok(game) if game.playtime_forever > 0 => Some(Ok(game)),
                    Ok(_) => None,
                    Err(e) => Some(Err(e)),
                }
            })
            .collect::<Result<Vec<Game>, _>>()
            .context("Failed to deserialize games array.")?;

        Ok(output)
    }

    async fn get_player_info(&self, config: &Config) -> Result<PlayerInfo, anyhow::Error> {
        let response = self.http.get(format!(
            "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002/?key={}&steamids={}",
            config.get_api_key()?,
            config.get_user_id()?
        )).send()
        .await
        .map_err(|err| anyhow!(format!("Request failed.:\n\t{:?}", err)))?
        .text()
        .await
        .map_err(|err| {
            anyhow!(format!(
                "Unable to get text response from response:\n\t{:?}",
                err
            ))
        })?;

        let response: Value =
            serde_json::from_str(&response).context("Failed to parse response")?;

        let profile = response["response"]["players"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("The structure of the JSON response is invalid."))?
            .first()
            .ok_or_else(|| anyhow::anyhow!("The returned data was empty"))?;

        let output: PlayerInfo = serde_json::from_value(profile.clone())?;
        Ok(output)
    }
}
