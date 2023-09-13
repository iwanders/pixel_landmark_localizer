use crate::map::LandmarkLocation;
use crate::map::Map;
use crate::Coordinate;
use crate::Landmark;
use crate::Rect;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Localizer {
    position: Coordinate,
    map: Map,
}

impl Localizer {
    pub fn new(map: Map, position: Coordinate) -> Self {
        Localizer { position, map }
    }

    // screen -> map: screen + self.position.
    // map -> screen: screen - self.position

    pub fn localize(&mut self, image: &image::RgbaImage, roi: &Rect) {
        // Determine the expected landmarks in the roi in map frame.
        let map_roi = *roi + self.position;

        // Expected locations in this roi:
        let expected_locations = self.map.landmarks_in(&map_roi);

        // Then, try to find the expected landmarks in the image.
        const SEARCH_DISTANCE: u32 = 15;

        let mut offsets = vec![];
        for location in expected_locations {
            let loc = self.map.location(location);
            let landmark = self.map.landmark(loc.id);
            let screen_expected_pos = loc.location - self.position;

            let search_box = Rect {
                x: screen_expected_pos.x - SEARCH_DISTANCE as i32,
                y: screen_expected_pos.y - SEARCH_DISTANCE as i32,
                w: 2 * SEARCH_DISTANCE,
                h: 2 * SEARCH_DISTANCE,
            };
            if let Some(found_pos) = Self::search_landmark(image, &search_box, landmark) {
                offsets.push((found_pos, location));
            }
        }
        println!("offsets: {offsets:?}");
    }

    pub fn search_all(&self, image: &image::RgbaImage, roi: &Rect) -> Vec<LandmarkLocation> {
        let mut res = vec![];
        for id in self.map.landmark_ids() {
            let landmark = self.map.landmark(id);
            println!("landmark: {id:?}");
            if let Some(coordinate) = Self::search_landmark(image, roi, landmark) {
                res.push(LandmarkLocation {
                    location: coordinate + self.position,
                    id,
                });
            }
        }
        res
    }

    pub fn search_landmark(
        image: &image::RgbaImage,
        search: &Rect,
        landmark: &Landmark,
    ) -> Option<Coordinate> {
        for y in (search.y)..(search.y + search.h as i32) {
            for x in (search.x)..(search.x + search.w as i32) {
                let present = landmark.present(image, (x as u32, y as u32));

                if present {
                    return Some(Coordinate { x, y });
                }
            }
        }
        None
    }
}