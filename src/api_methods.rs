use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::SteamInfo;

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

pub async fn get_user_games(api: &SteamInfo) -> Result<Vec<Game>, anyhow::Error> {
    let response = reqwest::get(
        format!("https://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key={}&steamid={}&include_appinfo=true&include_played_free_games=true&format=json", 
        api.api_key, 
        api.user_id
    ))
        .await
        .map_err(|err| anyhow!(format!("Request failed.:\n\t{:?}", err)))?
        .text()
        .await
        .map_err(|err| anyhow!(format!("Unable to get text response from response:\n\t{:?}", err)))?;

    let response: Value = serde_json::from_str(&response).context("Failed to parse response")?;

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

pub async fn get_player_info(steam: &SteamInfo) -> Result<PlayerInfo, anyhow::Error> {
    let response = reqwest::get(format!(
        "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002/?key={}&steamids={}",
        steam.api_key, steam.user_id
    ))
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

    let response: Value = serde_json::from_str(&response).context("Failed to parse response")?;

    let profile = response["response"]["players"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("The structure of the JSON response is invalid."))?
        .first()
        .ok_or_else(|| anyhow::anyhow!("The returned data was empty"))?;

    let output: PlayerInfo = serde_json::from_value(profile.clone())?;
    Ok(output)
}
