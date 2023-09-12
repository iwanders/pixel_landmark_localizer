// ffmpeg -i 2023-08-13_23-03-24.mp4 -r 60 frames/frame%04d.png

pub type Error = Box<dyn std::error::Error>;

mod landmark;
mod mock;
mod util;
pub use util::{Coordinate, Rect};

pub use landmark::Landmark;
pub mod localizer;
pub mod map;
use localizer::Localizer;
use map::Map;

pub fn image_to_landmark(path: &std::path::Path) -> Result<Landmark, Error> {
    // let image_path = std::path::PathBuf::from("../screenshots/landmark_3.png");
    let l1 = image::open(&path)?.to_rgba8();
    Ok(Landmark::from_image(&l1, 6))
}

fn find_match(block: &image::RgbaImage, rect: &Rect, lm: &Landmark) -> u32 {
    for y in rect.y..(rect.y + rect.h as i32) {
        for x in rect.x..(rect.x + rect.w as i32) {
            let present = lm.present(block, (x as u32, y as u32));

            if present {
                println!("Found landmark at {x}, {y} ");
            }
        }
    }
    0
}

pub fn main_landmark() -> Result<(), Error> {
    let image_path = std::path::PathBuf::from("../screenshots/Screenshot446.png");
    let screenshot = image::open(&image_path)?.to_rgba8();

    let lm1 = image_to_landmark(&std::path::PathBuf::from("../screenshots/landmark_1.png"))?;
    let lm2 = image_to_landmark(&std::path::PathBuf::from("../screenshots/landmark_2.png"))?;
    let mut test_map = Map::default();

    let lm1 = test_map.add_landmark(lm1);
    let lm2 = test_map.add_landmark(lm2);

    let roi = Rect {
        x: 0,
        y: 0,
        w: 640,
        h: 408,
    };

    let mut localizer = Localizer::new(test_map, Default::default());

    let start = std::time::Instant::now();
    // localizer.localize(&screenshot, &roi);
    let res = localizer.search_all(&screenshot, &roi);
    println!("Res: {res:?}");
    println!("took {}", start.elapsed().as_secs_f64());

    // let best = find_match(, test_map.landmark(lm2));

    Ok(())
}
