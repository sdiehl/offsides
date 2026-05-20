//! Byte-offset to (line, column) lookups.

#[derive(Clone, Debug)]
pub(crate) struct ColumnIndex {
    line_starts: Vec<usize>,
}

impl ColumnIndex {
    pub(crate) fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (i, b) in source.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        Self { line_starts }
    }

    pub(crate) fn line(&self, offset: usize) -> usize {
        match self.line_starts.binary_search(&offset) {
            Ok(line) => line,
            Err(line) => line - 1,
        }
    }

    /// Column of `offset`, with each tab counted as `tab_width` columns and all
    /// other bytes (including UTF-8 continuation bytes) counted as 1. Layout
    /// decisions only need ordinal comparisons, not display width.
    pub(crate) fn column(&self, source: &str, offset: usize, tab_width: usize) -> usize {
        let line = self.line(offset);
        let start = self.line_starts[line];
        let region = source.get(start..offset).unwrap_or("");
        let mut col = 0;
        for b in region.bytes() {
            col += if b == b'\t' { tab_width } else { 1 };
        }
        col
    }
}

#[cfg(test)]
mod tests {
    use super::ColumnIndex;

    #[test]
    fn line_and_column_basic() {
        let src = "abc\nde\n  f";
        let idx = ColumnIndex::new(src);
        assert_eq!(idx.line(0), 0);
        assert_eq!(idx.line(3), 0);
        assert_eq!(idx.line(4), 1);
        assert_eq!(idx.line(7), 2);
        assert_eq!(idx.column(src, 0, 1), 0);
        assert_eq!(idx.column(src, 5, 1), 1);
        assert_eq!(idx.column(src, 9, 1), 2);
    }

    #[test]
    fn tab_width() {
        let src = "\tx";
        let idx = ColumnIndex::new(src);
        assert_eq!(idx.column(src, 1, 4), 4);
        assert_eq!(idx.column(src, 1, 1), 1);
    }
}
