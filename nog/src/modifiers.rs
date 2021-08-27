#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Modifiers {
    pub lalt: bool,
    pub ralt: bool,
    pub shift: bool,
    pub win: bool,
    pub ctrl: bool,
}

impl Modifiers {
    pub const BIT_COUNT: u32 = 5;
    /// how many fields does modifiers have
    pub fn get_id(&self) -> usize {
        [
            self.lalt as u8,
            self.ralt as u8,
            self.shift as u8,
            self.win as u8,
            self.ctrl as u8,
        ]
        .iter()
        .enumerate()
        .fold(0usize, |acc, (idx, x)| {
            acc + *x as usize * 10usize.pow(idx as u32)
        })
    }
}
