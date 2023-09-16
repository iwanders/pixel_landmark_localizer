// ffmpeg -i 2023-08-13_23-03-24.mp4 -r 60 frames/frame%04d.png

/*
Make landmarks by:
    1. Difference in gimp between two screenshots
    2. Select by color, click a fully black pixel.
    3. Make new channel, check initialise from selection.
    4. Repeat as needed, building channels with masks where pixels are zero.
    5. Subtract all masks, these pixels don't change.
    6. Create template from the pixels that are selected.
*/

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
    let roi = Rect {
        x: 0,
        y: 0,
        w: 640,
        h: 408,
    };

    let mut lm0 = image_to_landmark(&std::path::PathBuf::from("../screenshots/landmark_1.png"))?;
    let mut lm1 = image_to_landmark(&std::path::PathBuf::from("../screenshots/landmark_2.png"))?;
    let mut lm2 = image_to_landmark(&std::path::PathBuf::from("../screenshots/landmark_5.png"))?;

    lm0.optimize_pixels_row_seq();
    lm1.optimize_pixels_row_seq();
    lm2.optimize_pixels_row_seq();

    let mut test_map = Map::default();

    let lm0 = test_map.add_landmark(lm0);
    let lm1 = test_map.add_landmark(lm1);
    let lm2 = test_map.add_landmark(lm2);

    // add lm1 to the fixed location at the origin.
    // test_map.add_fixed(Coordinate{x: 100, y: 100}, lm1);

    // test_map.add_fixed(Coordinate{x: 0, y: 0}, lm1);
    test_map.add_fixed(Coordinate { x: -45, y: -36 }, lm0);
    test_map.add_fixed(Coordinate { x: -103, y: -45 }, lm1);
    test_map.add_fixed(Coordinate { x: -58, y: -9 }, lm2);

    let mut localizer = Localizer::new(test_map, Default::default(), Default::default());

    let mut capture =
        mock::MockScreenCapture::new(&std::path::PathBuf::from("../screenshots/run1/leg_2/"))?;
    let screenshot = capture.frame()?;
    localizer.relocalize(&screenshot, &roi);
    let loc = localizer.localize(&screenshot, &roi);

    // let mut res = localizer.search_all(&screenshot, &roi);
    // println!("Res: {res:?}");
    // localizer.mapping(&screenshot, &roi);

    // let lm2_found = Localizer::search_landmarks(&screenshot, &roi, &lm1, 10000);
    // println!("lm2_found: {lm2_found:?}");

    // let image_path = std::path::PathBuf::from("../screenshots/run1/leg_1/Screenshot456.png");
    // let screenshot = image::open(&image_path)?.to_rgba8();
    while capture.advance() {
        let screenshot = capture.frame()?;
        println!("Frame: {:?}", capture.frame_name());
        let start = std::time::Instant::now();

        if let Some(loc) = localizer.localize(&screenshot, &roi) {
            println!("location: {loc:?}");
            // localizer.mapping(&screenshot, &roi);
            println!("took {}", start.elapsed().as_secs_f64());
        } else {
            let reloc = localizer.relocalize(&screenshot, &roi);
            println!("   reloc: {reloc:?}");
        }
        // let mut res = localizer.search_all(&screenshot, &roi);
        // println!("Res: {res:?}");
    }

    // let start = std::time::Instant::now();
    // let loc = localizer.localize(&screenshot, &roi);
    // println!("location: {loc:?}");

    // println!("took {}", start.elapsed().as_secs_f64());

    // let mut res = localizer.search_all(&screenshot, &roi);
    // println!("Res: {res:?}");

    // localizer.mapping(&screenshot, &roi);
    // println!("localizer: {localizer:?}");

    println!("Map: {:#?}", localizer.map().locations());

    // let best = find_match(, test_map.landmark(lm2));

    Ok(())
}
