use core::array;
use std::cell::RefCell;

use anyhow::Result;
use rand::random;

use crate::types::{Conference, Game, Team};
use crate::{display, utils};

fn partition(mut teams: Vec<Team>) -> (Vec<Team>, Vec<Team>) {
    teams.sort_by_key(|x| x.season_loses);
    teams.into_iter().partition(|x| matches!(x.conference, Conference::West))
}

fn simulate_from_teams(
    name: &str, home: &mut Team, away: &mut Team, count_wins: bool, print: &str,
) -> bool {
    let expected = utils::expected(home.elo, away.elo);
    let (actual, home_win) = if random::<f64>() < expected { (1.0, true) } else { (0.0, false) };
    let change = 32.0 * (actual - expected);
    let home_old = home.elo;
    let away_old = away.elo;
    home.elo += change;
    away.elo -= change;
    if count_wins {
        if home_win {
            home.season_wins += 1;
            away.season_loses += 1;
        } else {
            home.season_loses += 1;
            away.season_wins += 1;
        }
    }
    if print == "*" || home.name == print || away.name == print {
        display::print_simulated_game_info(
            name, home, away, home_win, expected, home_old, away_old, change,
        );
    }
    home_win
}

fn simulate_from_game(game: &Game, teams: &[RefCell<Team>], print: &str) {
    let name = game.date.to_string();
    let mut home = teams[game.home_team_idx].borrow_mut();
    let mut away = teams[game.away_team_idx].borrow_mut();
    simulate_from_teams(&name, &mut home, &mut away, true, print);
}

fn simulate_with_info(
    name: &str, mut home: Team, mut away: Team, print: &str,
) -> (Team, Team, GameData) {
    let result = simulate_from_teams(name, &mut home, &mut away, false, print);
    let data = GameData::new(&home, &away, result);
    if result {
        (home, away, data)
    } else {
        (away, home, data)
    }
}

pub fn simulate_season() -> Result<()> {
    let teams = utils::load_teams_cell("files/processed.json")?;
    let games = utils::load_games("files/games.json")?;
    display::display_by_elo(&teams);
    let team = utils::ask_for_team_cell(&teams);
    for game in &games {
        simulate_from_game(game, &teams, &team);
    }
    let teams: Vec<_> = teams.into_iter().map(RefCell::into_inner).collect();
    utils::write_json(&teams, "files/after_season.json")?;
    Ok(())
}

pub fn simulate_postseason() -> Result<()> {
    let teams = utils::load_teams("files/after_season.json")?;
    let team = utils::ask_for_team(&teams);
    let (west, east) = partition(teams);
    display::display_by_wins(&west, &east);
    let west = ConferenceBracket::new("West".into(), west);
    let east = ConferenceBracket::new("East".into(), east);
    let (west_winner, west_data) = west.simulate(&team);
    let (east_winner, east_data) = east.simulate(&team);
    let finals = Round::new("Finals".into(), west_winner, east_winner);
    let (winner, finals_data) = finals.simulate(&team);
    display::display_ladder(&west_data, &east_data, &finals_data, &winner.name)?;
    Ok(())
}

struct ConferenceBracket {
    name: String,
    teams: Vec<Team>,
}

impl ConferenceBracket {
    fn new(name: String, teams: Vec<Team>) -> Self { ConferenceBracket { name, teams } }

    fn simulate_play_in(&mut self, print: &str) -> [GameData; 3] {
        self.teams.truncate(10);
        let tenth = self.teams.pop().unwrap();
        let ninth = self.teams.pop().unwrap();
        let eighth = self.teams.pop().unwrap();
        let seventh = self.teams.pop().unwrap();
        let mut name = format!("{} play-in round 1", self.name);
        let (seventh, round_3_home, round_1_data) =
            simulate_with_info(&name, seventh, eighth, print);
        name.pop();
        name.push('2');
        let (round_3_away, _, round_2_data) = simulate_with_info(&name, ninth, tenth, print);
        name.pop();
        name.push('3');
        let (eighth, _, round_3_data) =
            simulate_with_info(&name, round_3_home, round_3_away, print);
        self.teams.push(seventh);
        self.teams.push(eighth);
        [round_1_data, round_2_data, round_3_data]
    }

    fn simulate(mut self, print: &str) -> (Team, ConferenceData) {
        let mut names = vec![
            format!("{} round 1 (1 vs 8)", self.name),
            format!("{} round 1 (2 vs 7)", self.name),
            format!("{} round 1 (3 vs 6)", self.name),
            format!("{} round 1 (4 vs 5)", self.name),
            format!("{} semifinals (1/8 vs 4/5)", self.name),
            format!("{} semifinals (2/7 vs 3/6)", self.name),
            format!("{} finals (1 vs 8)", self.name),
        ];
        let play_in_data = self.simulate_play_in(print);
        let mut data = Vec::new();
        for _ in 0..3 {
            let mut winners = Vec::new();
            while !self.teams.is_empty() {
                let round =
                    Round::new(names.remove(0), self.teams.remove(0), self.teams.pop().unwrap());
                let (winner, round_data) = round.simulate(print);
                winners.push(winner);
                data.push(round_data);
            }
            self.teams = winners;
        }
        let winner = self.teams.remove(0);
        let round_1_data: [RoundData; 4] = array::from_fn(|_| data.remove(0));
        let semis_data: [RoundData; 2] = array::from_fn(|_| data.remove(0));
        let finals_data = data.remove(0);
        let data = ConferenceData::new(play_in_data, round_1_data, semis_data, finals_data);
        (winner, data)
    }
}

pub struct ConferenceData {
    pub play_in: [GameData; 3],
    pub round_1: [RoundData; 4],
    pub semifinals: [RoundData; 2],
    pub finals: RoundData,
}

impl ConferenceData {
    fn new(
        play_in: [GameData; 3], round_1: [RoundData; 4], semifinals: [RoundData; 2],
        finals: RoundData,
    ) -> Self {
        ConferenceData { play_in, round_1, semifinals, finals }
    }
}

pub struct GameData {
    home: String,
    away: String,
    result: bool,
}

impl GameData {
    fn new(home: &Team, away: &Team, result: bool) -> Self {
        GameData { home: home.name.clone(), away: away.name.clone(), result }
    }
}

struct Round {
    name: String,
    team_a: Team,
    team_b: Team,
    team_a_wins: u32,
    team_b_wins: u32,
}

impl Round {
    fn new(name: String, team_a: Team, team_b: Team) -> Self {
        Round { name: name + " game ", team_a, team_b, team_a_wins: 0, team_b_wins: 0 }
    }

    fn play_game(&mut self, switch: bool, print: &str) -> bool {
        let result = if switch {
            simulate_from_teams(&self.name, &mut self.team_b, &mut self.team_a, false, print)
        } else {
            simulate_from_teams(&self.name, &mut self.team_a, &mut self.team_b, false, print)
        };
        if switch ^ result {
            self.team_a_wins += 1;
            self.team_a_wins == 4
        } else {
            self.team_b_wins += 1;
            self.team_b_wins == 4
        }
    }

    fn simulate(mut self, print: &str) -> (Team, RoundData) {
        let mut game = 1;
        let team_b_advantage = self.team_a.season_wins < self.team_b.season_wins;
        loop {
            self.name.push(char::from_digit(game, 10).unwrap());
            let switch = team_b_advantage ^ matches!(game, 3 | 4 | 6);
            if self.play_game(switch, print) {
                break;
            }
            self.name.pop();
            game += 1;
        }
        let data = RoundData::new(&self);
        let winner = if self.team_a_wins == 4 { self.team_a } else { self.team_b };
        (winner, data)
    }
}

#[derive(Debug)]
pub struct RoundData {
    pub team_a: String,
    pub team_b: String,
    pub team_a_wins: u32,
    pub team_b_wins: u32,
}

impl RoundData {
    fn new(round: &Round) -> Self {
        RoundData {
            team_a: round.team_a.name.clone(),
            team_b: round.team_b.name.clone(),
            team_a_wins: round.team_a_wins,
            team_b_wins: round.team_b_wins,
        }
    }
}
