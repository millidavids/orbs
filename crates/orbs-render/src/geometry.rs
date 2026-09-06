//! Grid coordinates.
//!
//! Everything here counts *cells*, never pixels. Pixels are a frontend concern
//! and appear only in [`crate::viewport`](crate::scale_for), which says how big
//! a cell is on a given window — never how many of them there are.
//!
//! All rectangles are in absolute grid coordinates. Sub-regions are not
//! re-based, so a [`Rect`] handed out by layout can be passed straight to a
//! painter without a translation step — the class of bug that translation
//! introduces is not worth the ergonomics it buys.

/// A cell position: columns and rows from the top-left of the grid.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos {
    /// Distance from the left edge, in cells.
    pub col: u16,
    /// Distance from the top edge, in cells.
    pub row: u16,
}

impl Pos {
    /// The top-left cell.
    pub const ORIGIN: Self = Self { col: 0, row: 0 };

    /// A position.
    #[must_use]
    pub const fn new(col: u16, row: u16) -> Self {
        Self { col, row }
    }
}

/// The dimensions of a cell grid.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridSize {
    /// Width in cells.
    pub cols: u16,
    /// Height in cells.
    pub rows: u16,
}

impl GridSize {
    /// A grid size.
    #[must_use]
    pub const fn new(cols: u16, rows: u16) -> Self {
        Self { cols, rows }
    }

    /// Total cell count.
    #[must_use]
    pub fn area(self) -> usize {
        usize::from(self.cols) * usize::from(self.rows)
    }

    /// Whether either dimension is zero, making the grid unpaintable.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.cols == 0 || self.rows == 0
    }

    /// Whether `pos` addresses a cell inside this grid.
    #[must_use]
    pub const fn contains(self, pos: Pos) -> bool {
        pos.col < self.cols && pos.row < self.rows
    }

    /// Whether this grid is at least as large as `floor` in both dimensions.
    ///
    /// Used against [`crate::MIN_GRID`] to decide whether a window can host the
    /// game at all.
    #[must_use]
    pub const fn fits(self, floor: Self) -> bool {
        self.cols >= floor.cols && self.rows >= floor.rows
    }

    /// The whole grid as a rectangle at the origin.
    #[must_use]
    pub const fn to_rect(self) -> Rect {
        Rect::new(0, 0, self.cols, self.rows)
    }
}

/// A rectangular region of the grid, in absolute coordinates.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rect {
    /// Left edge.
    pub col: u16,
    /// Top edge.
    pub row: u16,
    /// Width in cells.
    pub cols: u16,
    /// Height in cells.
    pub rows: u16,
}

impl Rect {
    /// A rectangle occupying no cells.
    pub const EMPTY: Self = Self {
        col: 0,
        row: 0,
        cols: 0,
        rows: 0,
    };

    /// A rectangle.
    #[must_use]
    pub const fn new(col: u16, row: u16, cols: u16, rows: u16) -> Self {
        Self {
            col,
            row,
            cols,
            rows,
        }
    }

    /// One past the rightmost column.
    #[must_use]
    pub const fn right(self) -> u16 {
        self.col.saturating_add(self.cols)
    }

    /// One past the bottom row.
    #[must_use]
    pub const fn bottom(self) -> u16 {
        self.row.saturating_add(self.rows)
    }

    /// Whether the rectangle covers no cells.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.cols == 0 || self.rows == 0
    }

    /// The rectangle's dimensions.
    #[must_use]
    pub const fn size(self) -> GridSize {
        GridSize::new(self.cols, self.rows)
    }

    /// The top-left cell.
    #[must_use]
    pub const fn origin(self) -> Pos {
        Pos::new(self.col, self.row)
    }

    /// Whether `pos` falls inside the rectangle.
    #[must_use]
    pub const fn contains(self, pos: Pos) -> bool {
        pos.col >= self.col
            && pos.col < self.right()
            && pos.row >= self.row
            && pos.row < self.bottom()
    }

    /// The overlap between two rectangles, or [`Rect::EMPTY`] if they are
    /// disjoint.
    ///
    /// This is how every painting operation is clipped: a painter intersects its
    /// requested region with its own bounds, so writing outside a pane is
    /// impossible rather than merely discouraged.
    #[must_use]
    pub fn intersection(self, other: Self) -> Self {
        let col = self.col.max(other.col);
        let row = self.row.max(other.row);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if right <= col || bottom <= row {
            Self::EMPTY
        } else {
            Self::new(col, row, right - col, bottom - row)
        }
    }

    /// The smallest rectangle holding both.
    ///
    /// An empty operand is ignored rather than dragging the result to the
    /// origin: [`Rect::EMPTY`] is `(0, 0, 0, 0)`, so a naive bounding box of
    /// *nothing* and a pane halfway down the screen reaches up to the top-left
    /// corner and covers everything between. `tween::edge` records the same trap
    /// for the same constant.
    ///
    /// It is a **bounding box**, not a set union — the region between two
    /// disjoint rectangles is included. Every caller unions rectangles that abut,
    /// which is the case where the two agree.
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return self;
        }
        let col = self.col.min(other.col);
        let row = self.row.min(other.row);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Self::new(col, row, right - col, bottom - row)
    }

    /// The rectangle shrunk by `by` cells on every side.
    ///
    /// Saturates to [`Rect::EMPTY`]-sized rather than underflowing, so insetting
    /// a one-cell region is harmless.
    #[must_use]
    pub const fn inset(self, by: u16) -> Self {
        let both = by.saturating_mul(2);
        Self {
            col: self.col.saturating_add(by),
            row: self.row.saturating_add(by),
            cols: self.cols.saturating_sub(both),
            rows: self.rows.saturating_sub(both),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersection_clips_to_the_overlap() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(5, 5, 10, 10);
        assert_eq!(a.intersection(b), Rect::new(5, 5, 5, 5));
    }

    #[test]
    fn disjoint_rectangles_intersect_to_nothing() {
        let a = Rect::new(0, 0, 4, 4);
        let b = Rect::new(10, 10, 4, 4);
        assert!(a.intersection(b).is_empty());
    }

    #[test]
    fn touching_edges_do_not_overlap() {
        let a = Rect::new(0, 0, 5, 5);
        let b = Rect::new(5, 0, 5, 5);
        assert!(a.intersection(b).is_empty());
    }

    #[test]
    fn inset_saturates_instead_of_underflowing() {
        assert_eq!(Rect::new(0, 0, 1, 1).inset(4), Rect::new(4, 4, 0, 0));
    }

    #[test]
    fn contains_excludes_the_far_edges() {
        let r = Rect::new(2, 3, 4, 5);
        assert!(r.contains(Pos::new(2, 3)));
        assert!(r.contains(Pos::new(5, 7)));
        assert!(!r.contains(Pos::new(6, 7)));
        assert!(!r.contains(Pos::new(5, 8)));
    }

    #[test]
    fn fits_compares_both_dimensions() {
        let floor = GridSize::new(80, 22);
        assert!(GridSize::new(80, 22).fits(floor));
        assert!(!GridSize::new(80, 21).fits(floor));
        assert!(!GridSize::new(79, 22).fits(floor));
    }
}
