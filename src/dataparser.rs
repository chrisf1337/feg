use std::fs::File;
use std::result;
use std::io;
use std::io::{BufReader, BufRead};
use ggez::error::*;

pub type Result<T> = result::Result<T, DataParserErr>;

#[derive(Debug)]
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

pub fn parse_walls_from_bufread<T: BufRead + Sized>(buf_reader: &mut T,
                                                    path: &str,
                                                    max_w: usize,
                                                    max_h: usize)
    -> Result<Vec<Vec<bool>>> {
    let mut walls: Vec<Vec<bool>> = vec![vec![false; max_h]; max_w];
    for (y, line) in buf_reader.lines().enumerate() {
        if y >= max_h {
            return Err(InvalidWallData(path.to_string()));
        }
        let line = line?;
        for (x, ch) in line.chars().enumerate() {
            if x >= max_w {
                return Err(InvalidWallData(path.to_string()));
            }
            match ch {
                '0' => walls[x][y] = false,
                '1' => walls[x][y] = true,
                _ => return Err(InvalidWallData(path.to_string()))
            }
        }
    }
    Ok(walls)
}

// Opens from "resources" dir
pub fn parse_walls(path: &str, max_w: usize, max_h: usize) -> Result<Vec<Vec<bool>>> {
    let f = File::open("resources/".to_string() + path)?;
    let mut buf_reader = BufReader::new(f);
    parse_walls_from_bufread(&mut buf_reader, path, max_w, max_h)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_ok_1() {
        let walls = indoc!("
            0000
            0010
            0100
            0000
        ");
        let mut cursor = Cursor::new(walls);
        let result = parse_walls_from_bufread(&mut cursor, "", 4, 4);
        assert!(result.is_ok());
        let parsed_walls = result.unwrap();
        for (x, v) in parsed_walls.iter().enumerate() {
            for (y, is_wall) in v.iter().enumerate() {
                if (x == 2 && y == 1) || (x == 1 && y == 2) {
                    assert_eq!(*is_wall, true);
                } else {
                    assert_eq!(*is_wall, false);
                }
            }
        }
    }
}