#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}

impl Coordinate {
    pub fn dist_sq(&self) -> i32 {
        self.x * self.x + self.y * self.y
    }
}

impl std::ops::Sub<Coordinate> for Coordinate {
    type Output = Coordinate;
    fn sub(self, rhs: Coordinate) -> Self::Output {
        Coordinate {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Neg for Coordinate {
    type Output = Coordinate;
    fn neg(self) -> Self::Output {
        Coordinate {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl std::ops::Add<Coordinate> for Coordinate {
    type Output = Coordinate;
    fn add(self, rhs: Coordinate) -> Self::Output {
        Coordinate {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

/// Struct to represent a rectangle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
        }
    }
}

impl std::ops::Sub<Coordinate> for Rect {
    type Output = Rect;
    fn sub(self, rhs: Coordinate) -> Self::Output {
        Rect {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            w: self.w,
            h: self.h,
        }
    }
}

impl std::ops::Sub<Rect> for Coordinate {
    type Output = Rect;
    fn sub(self, rhs: Rect) -> Self::Output {
        Rect {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            w: rhs.w,
            h: rhs.h,
        }
    }
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

    pub fn spiral(&self) -> std::iter::FromFn<Box<dyn FnMut() -> Option<Coordinate>>> {
        // https://stackoverflow.com/a/398302
        let x_odd = self.w % 2 != 0;
        let y_odd = self.h % 2 != 0;
        let w = self.w as i32;
        let h = self.h as i32;
        let x_orig = self.x
            + if x_odd {
                self.w as i32 / 2
            } else {
                self.w as i32 / 2 - 1
            };
        let y_orig = self.y
            + if y_odd {
                self.h as i32 / 2
            } else {
                self.h as i32 / 2 - 1
            };
        let x_min = -w / 2 - if x_odd { 1 } else { 0 };
        let y_min = -h / 2 - if y_odd { 1 } else { 0 };
        let x_max = w / 2 + if x_odd { 1 } else { 0 };
        let y_max = h / 2 + if y_odd { 1 } else { 0 };
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut dx: i32 = 0;
        let mut dy: i32 = -1;
        let imax = w.max(h);
        let imax = imax * imax;
        // println!("imax: {imax}");
        let mut i = 0;
        let z = move || {
            loop {
                if !(i < imax) {
                    return None;
                }
                let mut actual_coord = None;
                if (x_min < x && x <= x_max) && (y_min < y && y <= y_max) {
                    // do things with x,y
                    // println!("{i}, {x}, {y}");
                    actual_coord = Some(Coordinate {
                        x: x + x_orig,
                        y: y + y_orig,
                    });
                }
                if (x == y) || (x < 0 && x == -y) || (x > 0 && x == 1 - y) {
                    let old_dx = dx;
                    dx = -dy;
                    dy = old_dx;
                }
                x += dx;
                y += dy;
                i += 1;
                // println!("                    {i}, {x}, {y}");
                if actual_coord.is_some() {
                    return actual_coord;
                }
            }
        };
        let boxed_fn: Box<dyn FnMut() -> Option<Coordinate>> = Box::new(z);
        std::iter::from_fn(boxed_fn)
    }

    pub fn indices(&self) -> Vec<Coordinate> {
        let mut v = Vec::with_capacity((self.w * self.h) as usize);
        for y in (self.y)..(self.y + self.h as i32) {
            for x in (self.x)..(self.x + self.w as i32) {
                v.push(Coordinate { x, y });
            }
        }
        v
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_spiral(rect: &Rect, print: bool) {
        if print {
            println!("Rect: {rect:?}");
            for s in rect.spiral() {
                println!("{s:?}");
            }
        }

        let mut spiral_vec = rect.spiral().collect::<Vec<Coordinate>>();
        let mut square_vec = rect.indices();

        for c in square_vec.clone() {
            if print {
                println!("c: {c:?}");
                println!("spiral_vec: {spiral_vec:?}");
            }
            let spiral_pos = spiral_vec
                .iter()
                .position(|z| z == &c)
                .expect("could not find coord");
            let square_pos = square_vec
                .iter()
                .position(|z| z == &c)
                .expect("could not find coord");
            spiral_vec.remove(spiral_pos);
            square_vec.remove(square_pos);
        }
        assert!(spiral_vec.is_empty());
        assert!(square_vec.is_empty());
    }

    #[test]
    fn test_spiral_3() {
        let r = Rect {
            x: 0,
            y: 0,
            w: 3,
            h: 3,
        };
        assert_spiral(&r, false);

        let r = Rect {
            x: 0,
            y: 0,
            w: 2,
            h: 2,
        };
        assert_spiral(&r, false);

        let r = Rect {
            x: 0,
            y: 0,
            w: 2,
            h: 3,
        };
        assert_spiral(&r, false);

        let r = Rect {
            x: 0,
            y: 0,
            w: 3,
            h: 2,
        };
        assert_spiral(&r, false);

        let r = Rect {
            x: -1,
            y: 0,
            w: 3,
            h: 2,
        };
        assert_spiral(&r, false);

        let r = Rect {
            x: 0,
            y: 1,
            w: 3,
            h: 2,
        };
        assert_spiral(&r, false);

        let r = Rect {
            x: 0,
            y: 0,
            w: 4,
            h: 4,
        };
        assert_spiral(&r, false);

        let r = Rect {
            x: -2,
            y: -2,
            w: 4,
            h: 4,
        };
        assert_spiral(&r, true);
    }
}
