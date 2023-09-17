use crate::map::LandmarkLocation;
use crate::map::LocationId;
use crate::map::Map;
use crate::Coordinate;
use crate::Landmark;
use crate::Rect;
use image::Rgba;

/// A struct to keep track of a location against a map.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Localizer {
    /// Position is the location of the top left corner of the screen.
    position: Coordinate,
    map: Map,
    config: LocalizerConfig,
}

/// Helper to make screen coordinates a distinct type.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct ScreenCoordinate(pub Coordinate);

/// Configuration for the localizer.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LocalizerConfig {
    /// Amount to search around the expected value. Width of the box searched is 2*search_box.
    pub search_box: u32,
}

impl Default for LocalizerConfig {
    fn default() -> LocalizerConfig {
        LocalizerConfig { search_box: 55 }
    }
}

impl Localizer {
    /// Create a new localizer using the provided map, initial position and configuration.
    pub fn new(map: Map, position: Coordinate, config: LocalizerConfig) -> Self {
        Localizer {
            position,
            map,
            config,
        }
    }

    // screen -> map: screen + self.position.
    // map -> screen: screen - self.position

    /// Do a fresh relocalisation, doing a full search on the screen and setting the position based
    /// on the known location of any found landmark. Usually, this is performed if localisation is
    /// lost.
    pub fn relocalize<T: image::GenericImageView<Pixel = Rgba<u8>>>(
        &mut self,
        image: &T,
        roi: &Rect,
    ) -> Option<Coordinate> {
        let initial = self.search_all(image, roi);

        let mut potential_locations = vec![];
        for loc in initial {
            // we found this landmark, see where it exists on the map.
            let candidates = self.map.locations_by_landmark(loc.id);

            for candidate in candidates {
                let estimated_correction = loc.location - candidate.location;
                potential_locations.push((
                    estimated_correction.dist_sq(),
                    loc,
                    candidate.clone(),
                    estimated_correction,
                ));
            }
        }

        // Sort by lowest estimated correction, and use that value.
        potential_locations.sort_by(|a, b| a.0.cmp(&b.0));
        // println!("potential_locations: {potential_locations:#?}");

        if let Some((_, _, _, correction)) = potential_locations.first() {
            self.set_position(self.position - *correction);
            Some(self.position)
        } else {
            None
        }
    }

    /// Localize relative to the previous position, searching around expected landmarks.
    pub fn localize<T: image::GenericImageView<Pixel = Rgba<u8>>>(
        &mut self,
        image: &T,
        roi: &Rect,
    ) -> Option<Coordinate> {
        // Determine the expected landmarks in the roi in map frame.
        let map_roi = *roi + self.position;

        // Expected locations in this roi:
        let expected_locations = self.map.landmarks_in(&map_roi);

        // Then, try to find the expected landmarks in the image.
        let mut offsets: Vec<(ScreenCoordinate, LocationId)> = vec![];
        for location in expected_locations {
            let loc = self.map.location(location);
            let landmark = self.map.landmark(loc.id);
            let screen_expected_pos = loc.location - self.position;
            // println!("expected: {:?} at {screen_expected_pos:?}", loc.id);

            // Before doing a search box, lets try to see if the landmark is present where we expect
            // it, based on the previously found landmark.
            if let Some(screen_coord) = {
                if let Some(past_found) = offsets.first() {
                    let map_location = self.map.location(past_found.1);
                    let best_pos = map_location.location - past_found.0 .0;
                    let screen_expected_pos = loc.location - best_pos;
                    if landmark.present(
                        image,
                        (screen_expected_pos.x as u32, screen_expected_pos.y as u32),
                    ) {
                        Some(ScreenCoordinate(screen_expected_pos))
                        // None
                    } else {
                        None
                    }
                } else {
                    None
                }
            } {
                offsets.push((screen_coord, location));
            } else {
                // We didn't find it where we expect it based on past things.
                let search_box = Rect {
                    x: (screen_expected_pos.x - self.config.search_box as i32).max(0),
                    y: (screen_expected_pos.y - self.config.search_box as i32).max(0),
                    w: 2 * self.config.search_box,
                    h: 2 * self.config.search_box,
                };
                if let Some(found_pos) = Self::search_landmark(image, &search_box, landmark) {
                    offsets.push((found_pos, location));
                }
            }
        }

        if let Some(found) = offsets.first() {
            let map_location = self.map.location(found.1);
            self.position = map_location.location - found.0 .0;
            return Some(self.position);
        }
        None
    }

    /// Perform a mapping procedure, doing a full search for all landmarks in the provided image and
    /// adding any locations that are not yet in the map.
    pub fn mapping<T: image::GenericImageView<Pixel = Rgba<u8>>>(&mut self, image: &T, roi: &Rect) {
        let all_matches = self.search_all(image, roi);
        let mut to_insert = vec![];
        {
            let locs = self.map.locations();
            for m in all_matches.iter() {
                if !locs.contains(m) {
                    to_insert.push(m);
                }
            }
        }
        println!("Inserting: {to_insert:?}");

        for m in to_insert {
            self.map.add_fixed(m.location, m.id);
        }
    }

    /// Search all landmarks in the current screen, using the current position.
    pub fn search_all<T: image::GenericImageView<Pixel = Rgba<u8>>>(
        &self,
        image: &T,
        roi: &Rect,
    ) -> Vec<LandmarkLocation> {
        let mut res = vec![];
        for id in self.map.landmark_ids() {
            let landmark = self.map.landmark(id);
            res.extend(
                Self::search_landmarks(image, roi, landmark, usize::MAX)
                    .iter()
                    .map(|s| LandmarkLocation {
                        location: s.0 + self.position,
                        id,
                    }),
            );
        }
        res
    }

    /// Search a landmark in the image, terminating if one is found.
    pub fn search_landmark<T: image::GenericImageView<Pixel = Rgba<u8>>>(
        image: &T,
        search: &Rect,
        landmark: &Landmark,
    ) -> Option<ScreenCoordinate> {
        let r = Self::search_landmarks(image, search, landmark, 1);
        r.first().copied()
    }

    /// Search a landmark in the image, using the provided search box and limiting the search.
    pub fn search_landmarks<T: image::GenericImageView<Pixel = Rgba<u8>>>(
        image: &T,
        search: &Rect,
        landmark: &Landmark,
        limit: usize,
    ) -> Vec<ScreenCoordinate> {
        let mut res = vec![];
        for y in (search.y)..(search.y + search.h as i32) {
            for x in (search.x)..(search.x + search.w as i32) {
                let present = landmark.present(image, (x as u32, y as u32));

                if present {
                    res.push(ScreenCoordinate(Coordinate { x, y }));
                    if res.len() >= limit {
                        return res;
                    }
                }
            }
        }
        res
    }

    /// Set the current position of the localizer.
    pub fn set_position(&mut self, position: Coordinate) {
        self.position = position;
    }

    /// Return the current map.
    pub fn map(&self) -> &Map {
        &self.map
    }
}
