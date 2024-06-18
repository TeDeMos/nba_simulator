use std::cell::RefCell;
use std::fs::File;
use std::io::BufReader;
use std::{fs, io};

use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::types::{Game, Team};

pub fn write_json<T>(data: &Vec<T>, path: &str) -> Result<()>
where T: Serialize {
    let serialized = serde_json::to_string_pretty(data)?;
    fs::write(path, serialized)?;
    Ok(())
}

fn read_json<T>(path: &str) -> Result<Vec<T>>
where T: DeserializeOwned {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let result = serde_json::from_reader(reader)?;
    Ok(result)
}

pub fn load_games(path: &str) -> Result<Vec<Game>> { read_json(path) }

pub fn load_teams(path: &str) -> Result<Vec<Team>> { read_json(path) }

pub fn load_teams_cell(path: &str) -> Result<Vec<RefCell<Team>>> {
    Ok(read_json(path)?.into_iter().map(RefCell::new).collect())
}

pub fn ask_for_team(teams: &[Team]) -> String {
    ask_for_team_closure(|result| teams.iter().any(|x| x.name == result))
}

pub fn ask_for_team_cell(teams: &[RefCell<Team>]) -> String {
    ask_for_team_closure(|result| teams.iter().any(|x| x.borrow().name == result))
}

fn ask_for_team_closure<F>(mut check: F) -> String
where F: FnMut(&str) -> bool {
    println!("Enter abbreviation of a team you want to print scores off (leave empty to ignore)");
    let mut buffer = String::new();
    loop {
        io::stdin().read_line(&mut buffer).unwrap();
        let result = buffer.trim_end().to_uppercase();
        if matches!(result.as_str(), "" | "*") || check(&result) {
            return result;
        }
        println!("Didn't find team with this abbreviation, try again");
        buffer.clear();
    }
}

pub fn expected(a: f64, b: f64) -> f64 { 1.0 / (1.0 + 10f64.powf((b - a) / 400.0)) }

pub fn ask(question: &str) -> bool {
    println!("{question} (y/n)");
    let mut buffer = String::new();
    loop {
        io::stdin().read_line(&mut buffer).unwrap();
        match buffer.trim_end().chars().next() {
            Some('y') | Some('Y') => return true,
            Some('n') | Some('N') => return false,
            _ => {
                println!("Try again");
                buffer.clear()
            },
        }
    }
}
