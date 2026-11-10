pub const CHAR_EMPTY_LINE: char = '\u{2022}';
//pub const CHAR_BREAK_SPACE: char = '\u{202F}'; // Narrow No-Break Space
pub const CHAR_LF: char = '\n';
pub const CHAR_PAREN: char = ')';
pub const CHAR_PERIOD: char = '.';
pub const CHAR_SPACE: char = '\u{0020}'; // Zero Width Space
                                         //pub const ZERO_WIDTH_SPACE: char = '\u{200B}'; // Zero Width Space

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmd {
    Add(usize, char),
    Remove(usize, char),
}
