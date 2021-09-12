#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Direction::*;
        write!(
            f,
            "{}",
            match self {
                Left => "Left",
                Right => "Right",
                Up => "Up",
                Down => "Down",
            }
        )
    }
}

impl std::str::FromStr for Direction {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Direction::*;
        Ok(match s.to_ascii_uppercase().as_str() {
            "LEFT" => Left,
            "RIGHT" => Right,
            "UP" => Up,
            "DOWN" => Down,
            dir => return Err(format!("Unknown direction '{}'", dir)),
        })
    }
}
