use crate::simulate::{ConferenceData, RoundData};
use crate::types::{Game, Team};
use anyhow::Result;
use std::cell::RefCell;
use std::fs;

pub fn display_by_elo(teams: &Vec<RefCell<Team>>) {
    let mut ordered: Vec<_> = teams.iter().map(RefCell::borrow).collect();
    ordered.sort_by(|x, y| x.elo.partial_cmp(&y.elo).unwrap().reverse());
    let worse = ordered.split_off(15);
    println!("\nTeams by elo based on seasons from 18-19 to 22-23:\n");
    for (i, (better, worse)) in ordered.into_iter().zip(worse).enumerate() {
        println!(
            "{:>2}. {:<23} elo: {:>7.2} | {:>2}. {:<23} elo: {:>7.2}",
            i + 1,
            better.full_name,
            better.elo,
            i + 16,
            worse.full_name,
            worse.elo
        );
    }
    println!();
}

pub fn display_by_wins(west: &Vec<Team>, east: &Vec<Team>) {
    println!("\nStandings after season:\n");
    println!("West Conference {:>31}| East Conference", "");
    for (i, (w, e)) in west.into_iter().zip(east).enumerate() {
        println!(
            "{:>2}. {:23} {:>2}-{:<2} elo: {:>7.2} | {:>2}. {:23} {:>2}-{:<2} elo: {:>7.2}",
            i + 1,
            w.full_name,
            w.season_wins,
            w.season_loses,
            w.elo,
            i + 1,
            e.full_name,
            e.season_wins,
            e.season_loses,
            e.elo,
        );
    }
    println!();
}

pub fn print_game_info(
    game: &Game,
    home: &Team,
    away: &Team,
    expected: f64,
    home_old: f64,
    away_old: f64,
    change: f64,
) {
    println!(
        "{}: {} {:>3} - {:<3} {} | exp: {:>2}% | {:>7.2} -> {:>7.2} ({:+06.2}), {:>7.2} -> {:>7.2} ({:+06.2})",
        game.date.date_naive(),
        &home.name,
        game.home_score,
        game.away_score,
        &away.name,
        (expected * 100.0) as u32,
        home_old,
        home.elo,
        change,
        away_old,
        away.elo,
        -change
    );
}

pub fn print_simulated_game_info(
    name: &str,
    home: &Team,
    away: &Team,
    home_win: bool,
    expected: f64,
    home_old: f64,
    away_old: f64,
    change: f64,
) {
    println!(
        "{:<35}: {} {} - {} {} | exp: {:>2}% | {:>7.2} -> {:>7.2} ({:+06.2}), {:>7.2} -> {:>7.2} ({:+06.2})",
        name,
        &home.name,
        if home_win { "W" } else { "L" },
        if !home_win { "W" } else { "L" },
        &away.name,
        (expected * 100.0) as u32,
        home_old,
        home.elo,
        change,
        away_old,
        away.elo,
        -change
    );
}

pub fn display_ladder(
    west: &ConferenceData,
    east: &ConferenceData,
    finals: &RoundData,
    winner: &str,
) -> Result<()> {
    let ladder: String = fs::read_to_string("files/ladder.txt")?;
    let mut bytes = ladder.into_bytes();
    let mut replace = |value: &[u8; 3], replacement: &[u8]| {
        if let Some(pos) = bytes.windows(3).position(|w| w == value) {
            for i in 0..3 {
                bytes[pos + i] = replacement[i];
            }
        }
    };
    replace(b"FC0", winner.as_bytes());
    let mut replace_round = |mut base: [u8; 3], round: &RoundData, switch: bool| {
        let a_wins = round.team_a_wins as u8 + b'0';
        let b_wins = round.team_b_wins as u8 + b'0';
        let score = if switch {
            [b_wins, b'-', a_wins]
        } else {
            [a_wins, b'-', b_wins]
        };
        replace(&base, &score);
        base[1] -= 1;
        base[2] = (base[2] - b'0') * 2 + b'0';
        replace(&base, round.team_a.as_bytes());
        base[2] += 1;
        replace(&base, round.team_b.as_bytes());
    };
    replace_round([b'F', b'B', b'0'], finals, false);
    let iter = [(west, b'W'), (east, b'E')];
    for (conference, letter) in iter {
        replace_round([letter, b'F', b'0'], &conference.finals, false);
        for (i, data) in conference.semifinals.iter().enumerate() {
            replace_round([letter, b'D', b'0' + i as u8], data, i == 1);
        }
        for (i, data) in conference.round_1.iter().enumerate() {
            replace_round([letter, b'B', b'0' + i as u8], data, false);
        }
    }
    let ladder: String = String::from_utf8(bytes)?;
    println!("{ladder}");
    Ok(())
}
