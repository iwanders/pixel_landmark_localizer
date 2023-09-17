use crate::map::LandmarkLocation;
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LandmarkMatch {
    pub screen_position: ScreenCoordinate,
    pub location: LandmarkLocation,
    pub best_position: Coordinate,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct LocalisationResult {
    pub matches: Vec<LandmarkMatch>,
    pub position: Coordinate,
    pub consistent_count: usize,
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

    fn matches_to_localisation_result(matches: &[LandmarkMatch]) -> Option<LocalisationResult> {
        if matches.is_empty() {
            return None;
        }

        // Determine the coordinate for which the most landmarks agree;
        use std::collections::HashMap;
        let mut position_count: HashMap<Coordinate, usize> = HashMap::new();
        for LandmarkMatch { best_position, .. } in matches.iter() {
            *position_count.entry(*best_position).or_default() += 1;
        }

        let (position, consistent_count) =
            position_count.into_iter().max_by_key(|(_, v)| *v).unwrap();

        Some(LocalisationResult {
            matches: matches.to_vec(),
            position,
            consistent_count,
        })
    }

    /// Do a fresh relocalisation, doing a full search on the screen and setting the position based
    /// on the known location of any found landmark. Usually, this is performed if localisation is
    /// lost.
    pub fn relocalize<T: image::GenericImageView<Pixel = Rgba<u8>>>(
        &mut self,
        image: &T,
        roi: &Rect,
    ) -> Option<LocalisationResult> {
        let landmark_matches = self.search_all(image, roi);

        // let mut potential_locations = vec![];
        let mut matches: Vec<LandmarkMatch> = vec![];
        for (location, screen_position) in landmark_matches {
            // we found this landmark, see where it exists on the map.
            let candidates = self.map.locations_by_landmark(location.id);

            for candidate in candidates {
                let estimated_correction = location.location - candidate.location;
                let best_position = self.position - estimated_correction;

                matches.push(LandmarkMatch {
                    screen_position,
                    location,
                    best_position,
                });
            }
        }

        let res = Self::matches_to_localisation_result(&matches);
        if let Some(loc_res) = &res {
            self.position = loc_res.position;
        }
        res
    }

    /// Localize relative to the previous position, searching around expected landmarks.
    pub fn localize<T: image::GenericImageView<Pixel = Rgba<u8>>>(
        &mut self,
        image: &T,
        roi: &Rect,
    ) -> Option<LocalisationResult> {
        // Determine the expected landmarks in the roi in map frame.
        let map_roi = *roi + self.position;

        // Expected locations in this roi:
        let expected_locations = self.map.landmarks_in(&map_roi);

        // Then, try to find the expected landmarks in the image.
        let mut matches: Vec<LandmarkMatch> = vec![];
        for location in expected_locations {
            let loc = self.map.location(location);
            let landmark = self.map.landmark(loc.id);
            let screen_expected_pos = loc.location - self.position;

            // Before doing a search box, lets try to see if the landmark is present where we expect
            // it, based on the previously found landmark.
            if let Some((screen_coord, best_pos)) = {
                if let Some(LandmarkMatch { best_position, .. }) = matches.first() {
                    let screen_expected_pos = loc.location - *best_position;
                    if landmark.present(
                        image,
                        (screen_expected_pos.x as u32, screen_expected_pos.y as u32),
                    ) {
                        Some((ScreenCoordinate(screen_expected_pos), best_position))
                        // None
                    } else {
                        None
                    }
                } else {
                    None
                }
            } {
                matches.push(LandmarkMatch {
                    screen_position: screen_coord,
                    location: *loc,
                    best_position: *best_pos,
                });
            } else {
                // We didn't find it where we expect it based on past things.
                let search_box = Rect {
                    x: (screen_expected_pos.x - self.config.search_box as i32).max(0),
                    y: (screen_expected_pos.y - self.config.search_box as i32).max(0),
                    w: 2 * self.config.search_box,
                    h: 2 * self.config.search_box,
                };
                if let Some(found_pos) = Self::search_landmark(image, &search_box, landmark) {
                    let best_pos = loc.location - found_pos.0;
                    matches.push(LandmarkMatch {
                        screen_position: found_pos,
                        location: *loc,
                        best_position: best_pos,
                    });
                }
            }
        }

        let res = Self::matches_to_localisation_result(&matches);
        if let Some(loc_res) = &res {
            self.position = loc_res.position;
        }
        res
    }

    /// Perform a mapping procedure, doing a full search for all landmarks in the provided image and
    /// adding any locations that are not yet in the map.
    pub fn mapping<T: image::GenericImageView<Pixel = Rgba<u8>>>(&mut self, image: &T, roi: &Rect) {
        let all_matches = self.search_all(image, roi);
        let mut to_insert = vec![];
        {
            let locs = self.map.locations();
            for (m, _screen_pos) in all_matches.iter() {
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
    ) -> Vec<(LandmarkLocation, ScreenCoordinate)> {
        let mut res = vec![];
        for id in self.map.landmark_ids() {
            let landmark = self.map.landmark(id);
            res.extend(
                Self::search_landmarks(image, roi, landmark, usize::MAX)
                    .iter()
                    .map(|s| {
                        (
                            LandmarkLocation {
                                location: s.0 + self.position,
                                id,
                            },
                            *s,
                        )
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
