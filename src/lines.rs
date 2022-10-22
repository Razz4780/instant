use std::fmt::{self, Display, Formatter};

/// Column and line into the Instant program.
#[derive(Debug, PartialEq, Eq)]
pub struct Position {
    line: usize,
    column: usize,
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "(line {}, column {})", self.line, self.column)
    }
}

/// Struct for finding [Position] of the given byte offset.
pub struct Lines {
    breaks: Vec<usize>,
}

impl Lines {
    /// Creates a new instance of this struct based on the given input.
    pub fn new(input: &str) -> Self {
        let breaks = input
            .chars()
            .enumerate()
            .filter_map(|(p, c)| if c == '\n' { Some(p) } else { None })
            .collect();

        Self { breaks }
    }

    /// Returns [Position] of the given byte offset.
    pub fn position(&self, offset: usize) -> Position {
        let line_idx = match self.breaks.binary_search(&offset) {
            Ok(i) => i,
            Err(i) => i,
        };

        match line_idx {
            0 => Position {
                line: 1,
                column: offset + 1,
            },
            i => Position {
                line: i + 1,
                column: offset - self.breaks[i - 1],
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lines() {
        let lines = Lines::new("aaa\na\n\naaaa\n");
        assert_eq!(lines.position(0), Position { line: 1, column: 1 });
        assert_eq!(lines.position(1), Position { line: 1, column: 2 });
        assert_eq!(lines.position(4), Position { line: 2, column: 1 });
        assert_eq!(lines.position(7), Position { line: 4, column: 1 });
        assert_eq!(lines.position(10), Position { line: 4, column: 4 });
    }
}
