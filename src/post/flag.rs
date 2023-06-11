use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Copy, strum::Display, strum::EnumIter)]
pub enum PostFlagEnum {
    Hidden = 1,
    // 0x01
    Draft = 1 << 1,
    // 0x02
    LatexEnabled = 1 << 2, // 0x04
}

pub struct PostFlag(u32);

impl PostFlag {
    pub fn new() -> Self {
        PostFlag(0)
    }

    pub fn is_hidden(&self) -> bool {
        self.0 & (PostFlagEnum::Hidden as u32) != 0
    }

    pub fn is_draft(&self) -> bool {
        self.0 & (PostFlagEnum::Draft as u32) != 0
    }

    pub fn is_latex_enabled(&self) -> bool {
        self.0 & (PostFlagEnum::LatexEnabled as u32) != 0
    }

    pub fn set_hidden(&mut self) {
        self.0 |= PostFlagEnum::Hidden as u32;
    }

    pub fn set_draft(&mut self) {
        self.0 |= PostFlagEnum::Draft as u32;
    }

    pub fn set_latex_enabled(&mut self) {
        self.0 |= PostFlagEnum::LatexEnabled as u32;
    }
}

impl Display for PostFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use strum::IntoEnumIterator;

        let mut v = vec![];
        for flag in PostFlagEnum::iter() {
            if self.0 & (flag as u32) != 0 {
                v.push(flag.to_string());
            }
        }
        let result = format!("[{}]", v.join(", "));
        std::fmt::Display::fmt(&result, f)
    }
}

impl Debug for PostFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}
