use anyhow::Result;
use nba::get_data::{get_previous_games, get_season_games, get_teams};
use nba::process_data::process_games;
use nba::simulate::{simulate_postseason, simulate_season};
use nba::utils;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    if !Path::new("files/teams.json").exists() {
        get_teams().await?;
    }
    println!("Got team data");
    if !Path::new("files/prev_games.json").exists() {
        get_previous_games().await?;
    }
    println!("Got previous games");
    if !Path::new("files/processed.json").exists() {
        process_games()?;
    }
    println!("Processed games");
    if !Path::new("files/games.json").exists() || utils::ask("download current season again?") {
        get_season_games().await?;
    }
    println!("Got season games");
    loop {
        if !Path::new("files/after_season.json").exists() || utils::ask("simulate season again?") {
            simulate_season()?;
        }
        println!("Simulated season");
        simulate_postseason()?;
        if utils::ask("done?") {
            break;
        }
    }
    Ok(())
}
