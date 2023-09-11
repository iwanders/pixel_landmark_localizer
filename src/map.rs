use crate::landmark::Landmark;
use crate::util::Rect;
use crate::Coordinate;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LandmarkId(usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LocationId(usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Fixed {
    coordinate: Coordinate,
    id: LandmarkId,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Map {
    landmarks: Vec<Landmark>,
    locations: Vec<Fixed>,
}

impl Map {
    pub fn add_landmark(&mut self, lm: Landmark) -> LandmarkId {
        let id = LandmarkId(self.landmarks.len());
        self.landmarks.push(lm);
        id
    }

    pub fn add_fixed(&mut self, coordinate: Coordinate, id: LandmarkId) {
        self.locations.push(Fixed { coordinate, id });
    }

    pub fn landmarks_in(&self, rect: &Rect) -> Vec<LocationId> {
        self.locations
            .iter()
            .enumerate()
            .filter_map(|(i, fixed)| {
                if rect.contains(fixed.coordinate.x, fixed.coordinate.y) {
                    Some(LocationId(i))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn location(&self, id: LocationId) -> &Fixed {
        &self.locations[id.0]
    }
    pub fn landmark(&self, id: LandmarkId) -> &Landmark {
        &self.landmarks[id.0]
    }
}
