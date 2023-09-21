use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    pub date: DateTime<Utc>,
    pub home_team_idx: usize,
    pub away_team_idx: usize,
    pub home_score: u32,
    pub away_score: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Team {
    pub name: String,
    pub full_name: String,
    pub conference: Conference,
    pub division: Division,
    pub elo: f64,
    pub season_wins: u32,
    pub season_loses: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Conference {
    West,
    East,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Division {
    Atlantic,
    Central,
    Southeast,
    Northwest,
    Pacific,
    Southwest,
}
