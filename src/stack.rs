//! Indent-stack state for the layout algorithm.

#[derive(Clone, Debug, Default)]
pub(crate) struct IndentStack {
    frames: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CloseStep {
    /// Pop the frame and emit a virtual close.
    Pop,
    /// Emit a virtual separator (sibling at same indent).
    Separator,
    /// No layout action needed (column is deeper than the current frame, the
    /// stack is at its floor, or the stack is empty).
    None,
}

impl IndentStack {
    pub(crate) fn push(&mut self, col: usize) {
        self.frames.push(col);
    }

    pub(crate) fn top(&self) -> Option<usize> {
        self.frames.last().copied()
    }

    pub(crate) fn pop(&mut self) -> Option<usize> {
        self.frames.pop()
    }

    pub(crate) fn depth(&self) -> usize {
        self.frames.len()
    }

    /// Compute the next layout action when a token appears at `col` after a
    /// line break. `floor` frames at the bottom are never popped (Eager mode
    /// keeps its top-level block open until end of input). Returns:
    /// - `Pop` if the topmost frame is deeper than `col` and above the floor.
    /// - `Separator` if the topmost frame is exactly at `col`.
    /// - `None` otherwise.
    pub(crate) fn step(&self, col: usize, floor: usize) -> CloseStep {
        match self.top() {
            Some(top) if top > col && self.depth() > floor => CloseStep::Pop,
            Some(top) if top == col => CloseStep::Separator,
            _ => CloseStep::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CloseStep, IndentStack};

    #[test]
    fn empty_stack_is_none() {
        let s = IndentStack::default();
        assert_eq!(s.step(0, 0), CloseStep::None);
        assert_eq!(s.step(99, 0), CloseStep::None);
    }

    #[test]
    fn deeper_is_none() {
        let mut s = IndentStack::default();
        s.push(2);
        assert_eq!(s.step(4, 0), CloseStep::None);
    }

    #[test]
    fn equal_is_separator() {
        let mut s = IndentStack::default();
        s.push(2);
        assert_eq!(s.step(2, 0), CloseStep::Separator);
    }

    #[test]
    fn shallower_is_pop() {
        let mut s = IndentStack::default();
        s.push(4);
        assert_eq!(s.step(2, 0), CloseStep::Pop);
    }

    #[test]
    fn floor_frame_is_never_popped() {
        let mut s = IndentStack::default();
        s.push(4);
        assert_eq!(s.step(2, 1), CloseStep::None);
        s.push(8);
        assert_eq!(s.step(2, 1), CloseStep::Pop);
    }
}
