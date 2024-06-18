use std::cell::RefCell;

use anyhow::Result;

use crate::types::{Game, Team};
use crate::{display, utils};

fn process(game: &Game, teams: &[RefCell<Team>], print: &str) {
    let mut home = teams[game.home_team_idx].borrow_mut();
    let mut away = teams[game.away_team_idx].borrow_mut();
    let expected = utils::expected(home.elo, away.elo);
    let actual = if game.home_score > game.away_score { 1.0 } else { 0.0 };
    let change = 32.0 * (actual - expected);
    let home_old = home.elo;
    let away_old = away.elo;
    home.elo += change;
    away.elo -= change;
    if print == "*" || home.name == print || away.name == print {
        display::print_game_info(game, &home, &away, expected, home_old, away_old, change);
    }
}

pub fn process_games() -> Result<()> {
    let teams = utils::load_teams_cell("files/teams.json")?;
    let games = utils::load_games("files/prev_games.json")?;
    let team = utils::ask_for_team_cell(&teams);
    for game in &games {
        process(game, &teams, &team);
    }
    let teams: Vec<_> = teams.into_iter().map(RefCell::into_inner).collect();
    utils::write_json(&teams, "files/processed.json")?;
    Ok(())
}
