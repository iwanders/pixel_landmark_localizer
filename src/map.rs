use crate::landmark::Landmark;
use crate::util::Rect;
use crate::Coordinate;
use serde::{Deserialize, Serialize};

/// Id for a particular landmark, (so the pattern).
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct LandmarkId(usize);

impl LandmarkId {
    pub fn to_index(&self) -> usize {
        self.0
    }
}

impl std::fmt::Display for LandmarkId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "landmark_{}", self.0)
    }
}

/// Id for a particular location on the map, so landmark id & position.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct LocationId(usize);

/// The specified landmark at the provided location.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct LandmarkLocation {
    /// The location of this landmark.
    pub location: Coordinate,
    /// The landmark found at this location.
    pub id: LandmarkId,
}

/// Something to describe a map of landmarks.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Map {
    /// An optional name for this map.
    name: Option<String>,
    /// The landmarks known by this map.
    landmarks: Vec<Landmark>,
    /// The placement of these landmarks on the map.
    locations: Vec<LandmarkLocation>,
}

impl Map {
    /// Add a landmark to this map, this just adds the pattern.
    pub fn add_landmark(&mut self, lm: Landmark) -> LandmarkId {
        let id = LandmarkId(self.landmarks.len());
        self.landmarks.push(lm);
        id
    }

    /// Adds a fixed location to the map, stating the provided landmark id will be present at this
    /// location.
    pub fn add_fixed(&mut self, id: LandmarkId, location: Coordinate) -> LandmarkLocation {
        let r = LandmarkLocation { location, id };
        self.locations.push(r);
        r
    }

    /// Return the locations that can be found within a certain rectangle (map coordinates).
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

    /// Return a specific location.
    pub fn location(&self, id: LocationId) -> &LandmarkLocation {
        &self.locations[id.0]
    }

    /// Return a all locations.
    pub fn locations(&self) -> &[LandmarkLocation] {
        &self.locations
    }

    /// Return a all locations that use the provided landmark.
    pub fn locations_by_landmark(&self, id: LandmarkId) -> Vec<&LandmarkLocation> {
        self.locations.iter().filter(|l| l.id == id).collect()
    }

    /// Return a landmark by id.
    pub fn landmark(&self, id: &LandmarkId) -> &Landmark {
        &self.landmarks[id.0]
    }

    /// Return all landmark ids.
    pub fn landmark_ids(&self) -> Vec<LandmarkId> {
        (0..self.landmarks.len()).map(|i| LandmarkId(i)).collect()
    }

    /// Set the map's name
    pub fn set_name(&mut self, name: Option<String>) {
        self.name = name;
    }

    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }
}
