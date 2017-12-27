use std::fs::File;
use std::result;
use std::io;
use std::io::{BufReader, BufRead};
use ggez::error::*;

pub type Result<T> = result::Result<T, DataParserErr>;

pub enum DataParserErr {
    InvalidWallData(String),
    Io(io::Error)
}

use self::DataParserErr::*;

impl From<io::Error> for DataParserErr {
    fn from(err: io::Error) -> Self {
        Io(err)
    }
}

impl From<DataParserErr> for GameError {
    fn from(err: DataParserErr) -> GameError {
        match err {
            InvalidWallData(path) => GameError::ResourceLoadError(path),
            Io(err) => GameError::ResourceLoadError(err.to_string())
        }
    }
}

pub fn parse_walls(path: &str, max_w: usize, max_h: usize) -> Result<Vec<Vec<bool>>> {
    let f = File::open("resources/".to_string() + path)?;
    let buf_reader = BufReader::new(f);
    let mut walls: Vec<Vec<bool>> = vec![];
    for (idx, line) in buf_reader.lines().enumerate() {
        if idx >= max_h {
            return Err(InvalidWallData(path.to_string()));
        }
        let line = line?;
        let mut row = vec![];
        for (idx, ch) in line.chars().enumerate() {
            if idx >= max_w {
                return Err(InvalidWallData(path.to_string()));
            }
            match ch {
                '0' => row.push(false),
                '1' => row.push(true),
                _ => return Err(InvalidWallData(path.to_string()))
            }
        }
        walls.push(row);
    }
    Ok(walls)
}