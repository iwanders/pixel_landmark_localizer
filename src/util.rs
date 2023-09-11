
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}


/// Struct to represent a rectangle.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl std::ops::Add<Coordinate> for Rect {
    type Output = Rect;
    fn add(self, rhs: Coordinate) -> Self::Output {
        Rect {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            w: self.w,
            h: self.h,
        }
    }
}

impl std::ops::Add<Rect> for Coordinate {
    type Output = Rect;
    fn add(self, rhs: Rect) -> Self::Output {
        Rect {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            w: rhs.w,
            h: rhs.h,
        }}
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

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.left() && x <= self.right() && y >= self.bottom() && y <= self.top()
    }

    /// The highest y value of the rectangle (bottom in image coordinates!)
    pub fn top(&self) -> i32 {
        self.y + self.h as i32
    }

    /// The lowest y value of the rectangle (top in image coordinates!)
    pub fn bottom(&self) -> i32 {
        self.y
    }

    /// The lowest x value of the rectangle.
    pub fn left(&self) -> i32 {
        self.x
    }

    /// The highest x value of the rectangle.
    pub fn right(&self) -> i32 {
        self.x + self.w as i32
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
