use crate::types::{Conference, Division, Game, Team};
use crate::utils;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use reqwest::{IntoUrl, Url};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct TeamResponse {
    data: Vec<TeamData>,
    meta: MetaData,
}

#[derive(Debug, Serialize, Deserialize)]
struct GameResponse {
    data: Vec<GameData>,
    meta: MetaData,
}

#[derive(Debug, Serialize, Deserialize)]
struct MetaData {
    total_pages: u32,
    current_page: u32,
    next_page: Option<u32>,
    per_page: u32,
    total_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameData {
    id: u32,
    date: DateTime<Utc>,
    home_team_score: u32,
    visitor_team_score: u32,
    season: u32,
    period: u32,
    status: String,
    time: Option<String>,
    postseason: bool,
    home_team: TeamData,
    visitor_team: TeamData,
}

impl From<GameData> for Game {
    fn from(value: GameData) -> Self {
        Game {
            date: value.date,
            home_team_idx: (value.home_team.id - 1) as usize,
            away_team_idx: (value.visitor_team.id - 1) as usize,
            home_score: value.home_team_score,
            away_score: value.visitor_team_score,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamData {
    id: u32,
    abbreviation: String,
    city: String,
    conference: Conference,
    division: Division,
    full_name: String,
    name: String,
}

impl From<TeamData> for Team {
    fn from(value: TeamData) -> Self {
        Team {
            name: value.abbreviation,
            full_name: value.full_name,
            conference: value.conference,
            division: value.division,
            elo: 1000.0,
            season_wins: 0,
            season_loses: 0,
        }
    }
}

async fn get_json_response<T, U>(url: T) -> Result<U>
where
    T: IntoUrl,
    U: DeserializeOwned,
{
    let response = reqwest::get(url).await?;
    if !response.status().is_success() {
        return Err(anyhow!("bad response error"));
    }
    let json = response.json().await?;
    Ok(json)
}

pub async fn get_teams() -> Result<()> {
    println!("Getting team data...");
    let url = "https://www.balldontlie.io/api/v1/teams";
    let mut team_response: TeamResponse = get_json_response(url).await?;
    team_response.data.sort_by_key(|x| x.id);
    println!("{:?}", team_response.meta);
    let result: Vec<Team> = team_response.data.into_iter().map(Team::from).collect();
    utils::write_json(&result, "files/teams.json")?;
    Ok(())
}

async fn get_games(season_query: String, path: &str) -> Result<()> {
    let url_base = "https://www.balldontlie.io/api/v1/games";
    let mut query: Vec<(_, String)> = vec![("page", "0".into()), ("per_page", "100".into())];
    query.extend(season_query.split(",").map(|x| ("seasons[]", x.into())));
    let mut games: Vec<Game> = Vec::new();
    loop {
        let game_response: GameResponse =
            get_json_response(Url::parse_with_params(url_base, &query)?).await?;
        println!("{:?}", game_response.meta);
        games.extend(game_response.data.into_iter().map(Game::from));
        let Some(next) = game_response.meta.next_page else {
            break;
        };
        query[0].1 = next.to_string();
    }
    games.sort_by_key(|x| x.date);
    utils::write_json(&games, path)?;
    Ok(())
}

pub async fn get_previous_games() -> Result<()> {
    println!("Getting previous games...");
    get_games("2018,2019,2020,2021,2022".into(), "files/prev_games.json").await?;
    Ok(())
}

pub async fn get_season_games() -> Result<()> {
    println!("Getting games for this season...");
    get_games("2023".into(), "files/games.json").await?;
    Ok(())
}
