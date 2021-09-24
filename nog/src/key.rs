use std::{fmt::Display, str::FromStr};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Enter,
    Escape,
    Backspace,
    Tab,
    Space,
}

impl Key {
    pub fn from_usize(code: usize) -> Option<Self> {
        Some(match code {
            8 => Key::Backspace,
            9 => Key::Tab,
            13 => Key::Enter,
            20 => Key::Space,
            27 => Key::Escape,
            48 => Key::Zero,
            49 => Key::One,
            50 => Key::Two,
            51 => Key::Three,
            52 => Key::Four,
            53 => Key::Five,
            54 => Key::Six,
            55 => Key::Seven,
            56 => Key::Eight,
            57 => Key::Nine,
            65 => Key::A,
            66 => Key::B,
            67 => Key::C,
            68 => Key::D,
            69 => Key::E,
            70 => Key::F,
            71 => Key::G,
            72 => Key::H,
            73 => Key::I,
            74 => Key::J,
            75 => Key::K,
            76 => Key::L,
            77 => Key::M,
            78 => Key::N,
            79 => Key::O,
            80 => Key::P,
            81 => Key::Q,
            82 => Key::R,
            83 => Key::S,
            84 => Key::T,
            85 => Key::U,
            86 => Key::V,
            87 => Key::W,
            88 => Key::X,
            89 => Key::Y,
            90 => Key::Z,
            112 => Key::F1,
            113 => Key::F2,
            114 => Key::F3,
            115 => Key::F4,
            116 => Key::F5,
            117 => Key::F6,
            118 => Key::F7,
            119 => Key::F8,
            120 => Key::F9,
            121 => Key::F10,
            122 => Key::F11,
            123 => Key::F12,
            _ => return None,
        })
    }

    pub fn to_usize(&self) -> usize {
        match self {
            Key::Backspace => 8,
            Key::Tab => 9,
            Key::Enter => 13,
            Key::Space => 20,
            Key::Escape => 27,
            Key::Zero => 48,
            Key::One => 49,
            Key::Two => 50,
            Key::Three => 51,
            Key::Four => 52,
            Key::Five => 53,
            Key::Six => 54,
            Key::Seven => 55,
            Key::Eight => 56,
            Key::Nine => 57,
            Key::A => 65,
            Key::B => 66,
            Key::C => 67,
            Key::D => 68,
            Key::E => 69,
            Key::F => 70,
            Key::G => 71,
            Key::H => 72,
            Key::I => 73,
            Key::J => 74,
            Key::K => 75,
            Key::L => 76,
            Key::M => 77,
            Key::N => 78,
            Key::O => 79,
            Key::P => 80,
            Key::Q => 81,
            Key::R => 82,
            Key::S => 83,
            Key::T => 84,
            Key::U => 85,
            Key::V => 86,
            Key::W => 87,
            Key::X => 88,
            Key::Y => 89,
            Key::Z => 90,
            Key::F1 => 112,
            Key::F2 => 113,
            Key::F3 => 114,
            Key::F4 => 115,
            Key::F5 => 116,
            Key::F6 => 117,
            Key::F7 => 118,
            Key::F8 => 119,
            Key::F9 => 120,
            Key::F10 => 121,
            Key::F11 => 122,
            Key::F12 => 123,
        }
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Key::*;
        write!(
            f,
            "{}",
            match self {
                A => "A",
                B => "B",
                C => "C",
                D => "D",
                E => "E",
                F => "F",
                G => "G",
                H => "H",
                I => "I",
                J => "J",
                K => "K",
                L => "L",
                M => "M",
                N => "N",
                O => "O",
                P => "P",
                Q => "Q",
                R => "R",
                S => "S",
                T => "T",
                U => "U",
                V => "V",
                W => "W",
                X => "X",
                Y => "Y",
                Z => "Z",
                Zero => "0",
                One => "1",
                Two => "2",
                Three => "3",
                Four => "4",
                Five => "5",
                Six => "6",
                Seven => "7",
                Eight => "8",
                Nine => "9",
                Enter => "Enter",
                Escape => "Escape",
                Backspace => "Backspace",
                Tab => "Tab",
                Space => "Space",
                F1 => "F1",
                F2 => "F2",
                F3 => "F3",
                F4 => "F4",
                F5 => "F5",
                F6 => "F6",
                F7 => "F7",
                F8 => "F8",
                F9 => "F9",
                F10 => "F10",
                F11 => "F11",
                F12 => "F12",
            }
        )
    }
}

impl FromStr for Key {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "A" => Key::A,
            "B" => Key::B,
            "C" => Key::C,
            "D" => Key::D,
            "E" => Key::E,
            "F" => Key::F,
            "G" => Key::G,
            "H" => Key::H,
            "I" => Key::I,
            "J" => Key::J,
            "K" => Key::K,
            "L" => Key::L,
            "N" => Key::N,
            "O" => Key::O,
            "P" => Key::P,
            "Q" => Key::Q,
            "R" => Key::R,
            "S" => Key::S,
            "T" => Key::T,
            "U" => Key::U,
            "V" => Key::V,
            "W" => Key::W,
            "X" => Key::X,
            "Y" => Key::Y,
            "0" => Key::Zero,
            "1" => Key::One,
            "2" => Key::Two,
            "3" => Key::Three,
            "4" => Key::Four,
            "5" => Key::Five,
            "6" => Key::Six,
            "7" => Key::Seven,
            "8" => Key::Eight,
            "9" => Key::Nine,
            "ENTER" => Key::Enter,
            "ESCAPE" | "ESC" => Key::Escape,
            "BACKSPACE" => Key::Backspace,
            "TAB" => Key::Tab,
            "SPACE" => Key::Space,
            "F1" => Key::F1,
            "F2" => Key::F2,
            "F3" => Key::F3,
            "F4" => Key::F4,
            "F5" => Key::F5,
            "F6" => Key::F6,
            "F7" => Key::F7,
            "F8" => Key::F8,
            "F9" => Key::F9,
            "F10" => Key::F10,
            "F11" => Key::F11,
            "F12" => Key::F12,
            k => return Err(format!("Unknown key '{}'", k)),
        })
    }
}
