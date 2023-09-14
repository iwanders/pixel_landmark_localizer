use crate::landmark::Landmark;
use crate::util::Rect;
use crate::Coordinate;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LandmarkId(usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LocationId(usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LandmarkLocation {
    pub location: Coordinate,
    pub id: LandmarkId,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Map {
    landmarks: Vec<Landmark>,
    locations: Vec<LandmarkLocation>,
}

impl Map {
    pub fn add_landmark(&mut self, lm: Landmark) -> LandmarkId {
        let id = LandmarkId(self.landmarks.len());
        self.landmarks.push(lm);
        id
    }

    pub fn add_fixed(&mut self, location: Coordinate, id: LandmarkId) {
        self.locations.push(LandmarkLocation { location, id });
    }

    pub fn landmarks_in(&self, rect: &Rect) -> Vec<LocationId> {
        self.locations
            .iter()
            .enumerate()
            .filter_map(|(i, fixed)| {
                if rect.contains(fixed.location.x, fixed.location.y) {
                    Some(LocationId(i))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn location(&self, id: LocationId) -> &LandmarkLocation {
        &self.locations[id.0]
    }

    pub fn locations(&self) -> &[LandmarkLocation] {
        &self.locations
    }

    pub fn locations_by_landmark(&self, id: LandmarkId) -> Vec<&LandmarkLocation> {
        self.locations.iter().filter(|l| l.id == id).collect()
    }

    pub fn landmark(&self, id: LandmarkId) -> &Landmark {
        &self.landmarks[id.0]
    }

    pub fn landmark_ids(&self) -> Vec<LandmarkId> {
        (0..self.landmarks.len()).map(|i| LandmarkId(i)).collect()
    }
}
