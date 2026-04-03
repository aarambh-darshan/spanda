//! SVG path string parser — converts SVG `d` attribute strings to [`PathCommand`]s.
//!
//! Supports the standard SVG path commands: M, L, Q, C, Z (and lowercase
//! relative variants). This enables loading paths from SVG files or design
//! tools without any DOM dependency.
//!
//! GSAP equivalent: `path: "#svgPath"` (but operates on the `d` string
//! directly instead of querying the DOM).
//!
//! # Example
//!
//! ```rust
//! use spanda::svg_path::SvgPathParser;
//! use spanda::motion_path::{CompoundPath, PathCommand};
//!
//! let commands = SvgPathParser::parse("M 0 0 C 50 100 100 100 150 0 L 200 0");
//! let path = CompoundPath::new(commands);
//! let pos = path.position(0.5);
//! ```

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use crate::motion_path::PathCommand;

/// A zero-allocation SVG path string parser.
///
/// Converts an SVG `d` attribute string into a `Vec<PathCommand>`.
///
/// ## Supported commands
///
/// | Command | Meaning | Coordinates |
/// |---------|---------|-------------|
/// | `M`/`m` | Move to | `x y` |
/// | `L`/`l` | Line to | `x y` |
/// | `H`/`h` | Horizontal line | `x` |
/// | `V`/`v` | Vertical line | `y` |
/// | `Q`/`q` | Quadratic Bezier | `cx cy x y` |
/// | `C`/`c` | Cubic Bezier | `c1x c1y c2x c2y x y` |
/// | `Z`/`z` | Close path | — |
///
/// Lowercase commands are relative to the current point;
/// uppercase are absolute.
#[derive(Debug)]
pub struct SvgPathParser;

impl SvgPathParser {
    /// Parse an SVG path `d` attribute string into a list of [`PathCommand`]s.
    ///
    /// ```rust
    /// use spanda::svg_path::SvgPathParser;
    ///
    /// let cmds = SvgPathParser::parse("M 10 20 L 30 40 Z");
    /// assert_eq!(cmds.len(), 3); // MoveTo, LineTo, Close
    /// ```
    pub fn parse(d: &str) -> Vec<PathCommand> {
        let mut commands = Vec::new();
        let mut cursor = [0.0_f32; 2];
        let mut subpath_start = [0.0_f32; 2];

        let tokens = Tokenizer::new(d);
        let mut nums: Vec<f32> = Vec::new();
        let mut current_cmd: Option<char> = None;

        for token in tokens {
            match token {
                Token::Command(c) => {
                    // Flush any pending implicit repeats
                    if let Some(cmd) = current_cmd {
                        Self::flush_nums(
                            cmd,
                            &mut nums,
                            &mut commands,
                            &mut cursor,
                            &mut subpath_start,
                        );
                    }
                    current_cmd = Some(c);
                    nums.clear();

                    if c == 'Z' || c == 'z' {
                        commands.push(PathCommand::Close);
                        cursor = subpath_start;
                        current_cmd = None;
                    }
                }
                Token::Number(n) => {
                    nums.push(n);
                    if let Some(cmd) = current_cmd {
                        let needed = Self::args_needed(cmd);
                        if needed > 0 && nums.len() >= needed {
                            Self::emit_command(
                                cmd,
                                &nums[nums.len() - needed..],
                                &mut commands,
                                &mut cursor,
                                &mut subpath_start,
                            );
                            // After first M, implicit repeats become L
                            if cmd == 'M' {
                                current_cmd = Some('L');
                            } else if cmd == 'm' {
                                current_cmd = Some('l');
                            }
                            nums.clear();
                        }
                    }
                }
            }
        }

        // Flush remaining
        if let Some(cmd) = current_cmd {
            Self::flush_nums(
                cmd,
                &mut nums,
                &mut commands,
                &mut cursor,
                &mut subpath_start,
            );
        }

        commands
    }

    /// Number of arguments needed for a given command.
    fn args_needed(cmd: char) -> usize {
        match cmd {
            'M' | 'm' | 'L' | 'l' => 2,
            'H' | 'h' | 'V' | 'v' => 1,
            'Q' | 'q' => 4,
            'C' | 'c' => 6,
            'Z' | 'z' => 0,
            _ => 0,
        }
    }

    /// Flush accumulated numbers for commands that may have implicit repeats.
    fn flush_nums(
        cmd: char,
        nums: &mut Vec<f32>,
        commands: &mut Vec<PathCommand>,
        cursor: &mut [f32; 2],
        subpath_start: &mut [f32; 2],
    ) {
        let needed = Self::args_needed(cmd);
        if needed == 0 {
            return;
        }
        while nums.len() >= needed {
            let args: Vec<f32> = nums.drain(..needed).collect();
            Self::emit_command(cmd, &args, commands, cursor, subpath_start);
        }
    }

    /// Emit a single path command from parsed arguments.
    fn emit_command(
        cmd: char,
        args: &[f32],
        commands: &mut Vec<PathCommand>,
        cursor: &mut [f32; 2],
        subpath_start: &mut [f32; 2],
    ) {
        let is_relative = cmd.is_ascii_lowercase();
        let ox = if is_relative { cursor[0] } else { 0.0 };
        let oy = if is_relative { cursor[1] } else { 0.0 };

        match cmd.to_ascii_uppercase() {
            'M' => {
                let p = [args[0] + ox, args[1] + oy];
                commands.push(PathCommand::MoveTo(p));
                *cursor = p;
                *subpath_start = p;
            }
            'L' => {
                let p = [args[0] + ox, args[1] + oy];
                commands.push(PathCommand::LineTo(p));
                *cursor = p;
            }
            'H' => {
                let x = args[0] + if is_relative { cursor[0] } else { 0.0 };
                let p = [x, cursor[1]];
                commands.push(PathCommand::LineTo(p));
                *cursor = p;
            }
            'V' => {
                let y = args[0] + if is_relative { cursor[1] } else { 0.0 };
                let p = [cursor[0], y];
                commands.push(PathCommand::LineTo(p));
                *cursor = p;
            }
            'Q' => {
                let control = [args[0] + ox, args[1] + oy];
                let end = [args[2] + ox, args[3] + oy];
                commands.push(PathCommand::QuadTo { control, end });
                *cursor = end;
            }
            'C' => {
                let control1 = [args[0] + ox, args[1] + oy];
                let control2 = [args[2] + ox, args[3] + oy];
                let end = [args[4] + ox, args[5] + oy];
                commands.push(PathCommand::CubicTo {
                    control1,
                    control2,
                    end,
                });
                *cursor = end;
            }
            _ => {}
        }
    }
}

// ── Tokenizer ────────────────────────────────────────────────────────────────

/// Tokens produced by the SVG path tokenizer.
#[derive(Debug, Clone)]
enum Token {
    Command(char),
    Number(f32),
}

/// Simple tokenizer for SVG path `d` strings.
struct Tokenizer<'a> {
    chars: &'a [u8],
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            chars: s.as_bytes(),
            pos: 0,
        }
    }

    fn skip_whitespace_and_commas(&mut self) {
        while self.pos < self.chars.len() {
            let b = self.chars[self.pos];
            if b == b' ' || b == b',' || b == b'\t' || b == b'\n' || b == b'\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn is_command(b: u8) -> bool {
        matches!(
            b,
            b'M' | b'm'
                | b'L'
                | b'l'
                | b'H'
                | b'h'
                | b'V'
                | b'v'
                | b'Q'
                | b'q'
                | b'C'
                | b'c'
                | b'Z'
                | b'z'
        )
    }

    fn parse_number(&mut self) -> Option<f32> {
        let start = self.pos;
        // optional sign
        if self.pos < self.chars.len()
            && (self.chars[self.pos] == b'-' || self.chars[self.pos] == b'+')
        {
            self.pos += 1;
        }
        // integer part
        while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        // decimal part
        if self.pos < self.chars.len() && self.chars[self.pos] == b'.' {
            self.pos += 1;
            while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }
        // exponent
        if self.pos < self.chars.len()
            && (self.chars[self.pos] == b'e' || self.chars[self.pos] == b'E')
        {
            self.pos += 1;
            if self.pos < self.chars.len()
                && (self.chars[self.pos] == b'-' || self.chars[self.pos] == b'+')
            {
                self.pos += 1;
            }
            while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }
        if self.pos > start {
            let s = core::str::from_utf8(&self.chars[start..self.pos]).ok()?;
            s.parse::<f32>().ok()
        } else {
            None
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        self.skip_whitespace_and_commas();
        if self.pos >= self.chars.len() {
            return None;
        }

        let b = self.chars[self.pos];

        if Self::is_command(b) {
            self.pos += 1;
            return Some(Token::Command(b as char));
        }

        // Try to parse a number (starts with digit, '.', '+', or '-')
        if b.is_ascii_digit() || b == b'-' || b == b'+' || b == b'.' {
            if let Some(n) = self.parse_number() {
                return Some(Token::Number(n));
            }
        }

        // Skip unknown character
        self.pos += 1;
        self.next()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_move_line_close() {
        let cmds = SvgPathParser::parse("M 0 0 L 100 0 L 100 100 Z");
        assert_eq!(cmds.len(), 4);
        match &cmds[0] {
            PathCommand::MoveTo(p) => assert!((p[0]).abs() < 1e-4 && (p[1]).abs() < 1e-4),
            _ => panic!("Expected MoveTo"),
        }
        match &cmds[1] {
            PathCommand::LineTo(p) => assert!((p[0] - 100.0).abs() < 1e-4),
            _ => panic!("Expected LineTo"),
        }
        match &cmds[3] {
            PathCommand::Close => {}
            _ => panic!("Expected Close"),
        }
    }

    #[test]
    fn parse_cubic() {
        let cmds = SvgPathParser::parse("M 0 0 C 50 100 100 100 150 0");
        assert_eq!(cmds.len(), 2);
        match &cmds[1] {
            PathCommand::CubicTo {
                control1,
                control2,
                end,
            } => {
                assert!((control1[0] - 50.0).abs() < 1e-4);
                assert!((control2[0] - 100.0).abs() < 1e-4);
                assert!((end[0] - 150.0).abs() < 1e-4);
            }
            _ => panic!("Expected CubicTo"),
        }
    }

    #[test]
    fn parse_quad() {
        let cmds = SvgPathParser::parse("M 0 0 Q 50 100 100 0");
        assert_eq!(cmds.len(), 2);
        match &cmds[1] {
            PathCommand::QuadTo { control, end } => {
                assert!((control[0] - 50.0).abs() < 1e-4);
                assert!((end[0] - 100.0).abs() < 1e-4);
            }
            _ => panic!("Expected QuadTo"),
        }
    }

    #[test]
    fn parse_relative() {
        let cmds = SvgPathParser::parse("M 10 20 l 30 40");
        assert_eq!(cmds.len(), 2);
        match &cmds[1] {
            PathCommand::LineTo(p) => {
                assert!((p[0] - 40.0).abs() < 1e-4, "Expected x=40, got {}", p[0]);
                assert!((p[1] - 60.0).abs() < 1e-4, "Expected y=60, got {}", p[1]);
            }
            _ => panic!("Expected LineTo"),
        }
    }

    #[test]
    fn parse_horizontal_vertical() {
        let cmds = SvgPathParser::parse("M 0 0 H 100 V 50");
        assert_eq!(cmds.len(), 3);
        match &cmds[1] {
            PathCommand::LineTo(p) => {
                assert!((p[0] - 100.0).abs() < 1e-4);
                assert!((p[1]).abs() < 1e-4);
            }
            _ => panic!("Expected LineTo for H"),
        }
        match &cmds[2] {
            PathCommand::LineTo(p) => {
                assert!((p[0] - 100.0).abs() < 1e-4); // x stays at 100
                assert!((p[1] - 50.0).abs() < 1e-4);
            }
            _ => panic!("Expected LineTo for V"),
        }
    }

    #[test]
    fn parse_relative_h_v() {
        let cmds = SvgPathParser::parse("M 10 20 h 30 v 40");
        assert_eq!(cmds.len(), 3);
        match &cmds[1] {
            PathCommand::LineTo(p) => {
                assert!((p[0] - 40.0).abs() < 1e-4);
                assert!((p[1] - 20.0).abs() < 1e-4);
            }
            _ => panic!("Expected LineTo for h"),
        }
        match &cmds[2] {
            PathCommand::LineTo(p) => {
                assert!((p[0] - 40.0).abs() < 1e-4);
                assert!((p[1] - 60.0).abs() < 1e-4);
            }
            _ => panic!("Expected LineTo for v"),
        }
    }

    #[test]
    fn parse_compact_notation() {
        // No spaces between commands and numbers, negative numbers as separators
        let cmds = SvgPathParser::parse("M0,0L100,50L200,0Z");
        assert_eq!(cmds.len(), 4);
        match &cmds[1] {
            PathCommand::LineTo(p) => {
                assert!((p[0] - 100.0).abs() < 1e-4);
                assert!((p[1] - 50.0).abs() < 1e-4);
            }
            _ => panic!("Expected LineTo"),
        }
    }

    #[test]
    fn parse_empty() {
        let cmds = SvgPathParser::parse("");
        assert!(cmds.is_empty());
    }

    #[test]
    fn parse_implicit_lineto() {
        // After M, extra coordinate pairs become implicit L
        let cmds = SvgPathParser::parse("M 0 0 50 50 100 0");
        assert_eq!(cmds.len(), 3); // M + L + L
        match &cmds[1] {
            PathCommand::LineTo(p) => {
                assert!((p[0] - 50.0).abs() < 1e-4);
                assert!((p[1] - 50.0).abs() < 1e-4);
            }
            _ => panic!("Expected implicit LineTo"),
        }
    }

    #[test]
    fn parse_negative_coords() {
        let cmds = SvgPathParser::parse("M -10 -20 L -30 -40");
        assert_eq!(cmds.len(), 2);
        match &cmds[0] {
            PathCommand::MoveTo(p) => {
                assert!((p[0] + 10.0).abs() < 1e-4);
                assert!((p[1] + 20.0).abs() < 1e-4);
            }
            _ => panic!("Expected MoveTo"),
        }
    }

    #[test]
    fn parse_into_compound_path() {
        use crate::motion_path::CompoundPath;

        let cmds = SvgPathParser::parse("M 0 0 C 50 100 100 100 150 0 L 200 0");
        let path = CompoundPath::new(cmds);

        let start = path.position(0.0);
        let end = path.position(1.0);
        assert!((start[0]).abs() < 2.0);
        assert!((end[0] - 200.0).abs() < 2.0);
    }

    #[test]
    fn parse_relative_cubic() {
        let cmds = SvgPathParser::parse("M 10 10 c 10 20 30 20 40 0");
        assert_eq!(cmds.len(), 2);
        match &cmds[1] {
            PathCommand::CubicTo {
                control1,
                control2,
                end,
            } => {
                assert!((control1[0] - 20.0).abs() < 1e-4);
                assert!((control1[1] - 30.0).abs() < 1e-4);
                assert!((control2[0] - 40.0).abs() < 1e-4);
                assert!((control2[1] - 30.0).abs() < 1e-4);
                assert!((end[0] - 50.0).abs() < 1e-4);
                assert!((end[1] - 10.0).abs() < 1e-4);
            }
            _ => panic!("Expected CubicTo"),
        }
    }
}
