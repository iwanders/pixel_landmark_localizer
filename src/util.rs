
/// Struct to represent a rectangle.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    /// Return whether this rectangle overlaps with the provided rectangle. Including boundary.
    pub fn overlaps(&self, b: &Rect) -> bool {
        self.right() >= b.left()
            && b.right() >= self.left()
            && self.top() >= b.bottom()
            && b.top() >= self.bottom()
    }
    /// Return whether this rectangle overlaps with the provided rectangle. Excluding boundary.
    pub fn overlaps_excluding(&self, b: &Rect) -> bool {
        self.right() > b.left()
            && b.right() > self.left()
            && self.top() > b.bottom()
            && b.top() > self.bottom()
    }

    pub fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.left() && x <= self.right() && y >= self.bottom() && y <= self.top()
    }

    /// The highest y value of the rectangle (bottom in image coordinates!)
    pub fn top(&self) -> u32 {
        self.y + self.h
    }

    /// The lowest y value of the rectangle (top in image coordinates!)
    pub fn bottom(&self) -> u32 {
        self.y
    }

    /// The lowest x value of the rectangle.
    pub fn left(&self) -> u32 {
        self.x
    }

    /// The highest x value of the rectangle.
    pub fn right(&self) -> u32 {
        self.x + self.w
    }

    /// The width of the rectangle.
    pub fn width(&self) -> u32 {
        self.w
    }

    /// The height of the rectangle.
    pub fn height(&self) -> u32 {
        self.h
    }
}
