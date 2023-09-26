use serde::{de::DeserializeOwned, Deserialize, Serialize};

/*
    landmark_a.yaml
    landmark_a.png

    landmark.yaml:
        filename: landmark_a.png <or default to current + png>
        pixel_difference_threshold: 5

    our_map.yaml:
        name: our_map
        landmarks:
            - landmark_a
            - landmark_b
        locations:
            - name: landmark_a
              position: [100, 100]
*/

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LandmarkSpecification {
    pub pixel_difference_threshold: u16,
    pub filename: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MapSpecification {
    pub name: Option<String>,
    pub landmarks: Vec<String>,
    pub locations: Vec<(String, [i32; 2])>,
}

impl MapSpecification {
    pub fn from_map(map: &crate::Map) -> Self {
        let name = map.name();
        let get_landmark_name = |lm| map.landmark(lm).name().unwrap_or(format!("{}", lm));
        let landmark_ids = map.landmark_ids();
        let landmarks = landmark_ids.iter().map(get_landmark_name).collect();
        let locations = map
            .locations()
            .iter()
            .map(|loc| (get_landmark_name(&loc.id), [loc.location.x, loc.location.y]))
            .collect();
        MapSpecification {
            name,
            landmarks,
            locations,
        }
    }
}

use std::fs::File;
use std::io::Read;

pub fn read_deserializable<T: DeserializeOwned>(path: &std::path::Path) -> Result<T, crate::Error> {
    match File::open(path) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content)
                .expect("should be able to read the file.");
            load_yaml::<T>(&content)
        }
        Err(error) => Err(Box::from(format!(
            "failed to open {}: {}",
            path.display(),
            error,
        ))),
    }
}

pub fn load_yaml<T: DeserializeOwned>(content: &str) -> Result<T, crate::Error> {
    match serde_yaml::from_str(content) {
        Ok(parsed_config) => Ok(parsed_config),
        Err(failure_message) => Err(Box::new(failure_message)),
    }
}

pub fn load_map(path: &std::path::Path) -> Result<crate::Map, crate::Error> {
    let map_spec = read_deserializable::<MapSpecification>(path)?;
    let mut map = crate::Map::default();

    map.set_name(map_spec.name);
    let mut landmark_map = std::collections::HashMap::new();

    for landmark_name in map_spec.landmarks.iter() {
        // construct the filepath.
        let map_dir = path.parent().unwrap(); // map_spec must have been a filename.
        let landmark_path_yaml = map_dir.join(format!("{landmark_name}.yaml"));

        let (landmark_meta, landmark_filename) = if landmark_path_yaml.is_file() {
            let spec = read_deserializable::<LandmarkSpecification>(&landmark_path_yaml)?;
            let png_name = spec
                .filename
                .clone()
                .unwrap_or(format!("{landmark_name}.png"));
            (spec, png_name)
        } else {
            (
                LandmarkSpecification::default(),
                format!("{landmark_name}.png"),
            )
        };

        let landmark_path_png = map_dir.join(landmark_filename);
        let mut landmark = crate::Landmark::from_path(&landmark_path_png)?;
        landmark.set_pixel_difference_threshold(landmark_meta.pixel_difference_threshold);
        landmark.set_name(Some(landmark_name.clone()));

        landmark_map.insert(landmark_name.clone(), map.add_landmark(landmark));
    }

    for (name, coord) in map_spec.locations.iter() {
        let landmark_id = landmark_map.get(name).ok_or(crate::Error::from(format!(
            "could not find landmark {name}"
        )))?;
        let coordinate = crate::Coordinate {
            x: coord[0],
            y: coord[1],
        };
        map.add_fixed(*landmark_id, coordinate);
    }

    Ok(map)
}

pub fn save_map_string(map: &crate::Map) -> Result<String, crate::Error> {
    let map_spec = MapSpecification::from_map(map);
    Ok(serde_yaml::to_string(&map_spec)?)
}

pub fn save_map(path: &std::path::Path, map: &crate::Map) -> Result<(), crate::Error> {
    use std::io::Write;
    let mut file = std::fs::File::create(path)?;
    file.write_all(save_map_string(map)?.as_bytes())?;
    Ok(())
}
