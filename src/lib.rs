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

pub mod capture;
mod landmark;
pub use capture::CaptureAdapted;
mod util;
pub use util::{Coordinate, Rect};

pub use landmark::Landmark;
pub mod localizer;
pub mod map;
pub use localizer::Localizer;
use map::Map;

pub mod config;

pub fn get_configured_capture() -> Box<dyn screen_capture::Capture> {
    let mut capture = screen_capture::get_capture();

    let current_resolution = capture.get_resolution();
    if std::env::consts::OS == "windows" {
        capture.prepare_capture(0, 0, 0, current_resolution.width, current_resolution.height);
    } else {
        capture.prepare_capture(
            0,
            1920,
            0,
            current_resolution.width - 1920,
            current_resolution.height,
        );
    }
    capture
}

/// Clunky function to run a localisation effort against the map.
pub fn run_on_capture(localizer: Localizer, roi: Rect) -> Result<(), Error> {
    let mut localizer = localizer;
    let mut capture = get_configured_capture();
    loop {
        let res = capture.capture_image();

        if !res {
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }

        // Then, we can grab the actual image.
        let screenshot = capture.get_image();

        let start = std::time::Instant::now();

        if let Some(loc) = localizer.localize(&screenshot.as_adapted(), &roi) {
            println!(
                "location: {:?} with {} landmarks",
                loc.position, loc.consistent_count
            );
            // localizer.mapping(&screenshot, &roi);
            println!("took {}", start.elapsed().as_secs_f64());
        } else {
            let reloc = localizer.relocalize(&screenshot.as_adapted(), &roi);
            println!("   reloc: {reloc:?}");
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

pub fn main_on_capture() -> Result<(), Error> {
    let roi = test_roi();
    let test_map = test_map()?;
    let localizer = Localizer::new(test_map, Default::default(), Default::default());

    return run_on_capture(localizer, roi);
}

pub fn test_map() -> Result<Map, Error> {
    let mut lm0 = Landmark::from_path(&std::path::PathBuf::from("../screenshots/landmark_1.png"))?;
    let mut lm1 = Landmark::from_path(&std::path::PathBuf::from("../screenshots/landmark_2.png"))?;
    let mut lm2 = Landmark::from_path(&std::path::PathBuf::from("../screenshots/landmark_5.png"))?;

    let mut lm3 = Landmark::from_path(&std::path::PathBuf::from("../screenshots/landmark_3.png"))?;
    let mut lm4 = Landmark::from_path(&std::path::PathBuf::from("../screenshots/landmark_4.png"))?;

    lm0.optimize_pixels_row_seq();
    lm1.optimize_pixels_row_seq();
    lm2.optimize_pixels_row_seq();
    lm3.optimize_pixels_row_seq();
    lm4.optimize_pixels_row_seq();

    let mut test_map = Map::default();

    let lm0 = test_map.add_landmark(lm0);
    let lm1 = test_map.add_landmark(lm1);
    let lm2 = test_map.add_landmark(lm2);

    let lm3 = test_map.add_landmark(lm3);
    let lm4 = test_map.add_landmark(lm4);

    // add lm1 to the fixed location at the origin.
    // test_map.add_fixed(Coordinate{x: 100, y: 100}, lm1);

    // test_map.add_fixed(Coordinate{x: 0, y: 0}, lm1);
    test_map.add_fixed(lm0, Coordinate { x: -45, y: -36 });
    test_map.add_fixed(lm1, Coordinate { x: -103, y: -45 });
    test_map.add_fixed(lm2, Coordinate { x: -58, y: -9 });

    test_map.add_fixed(lm4, Coordinate { x: 1000, y: 1000 });
    test_map.add_fixed(lm3, Coordinate { x: 960, y: 1012 });

    Ok(test_map)
}

pub fn test_roi() -> Rect {
    Rect {
        x: 0,
        y: 0,
        w: 640,
        h: 408,
    }
}

pub fn main_landmark() -> Result<(), Error> {
    let roi = test_roi();
    let test_map = test_map()?;
    let mut localizer = Localizer::new(test_map, Default::default(), Default::default());

    let mut capture =
        capture::MockScreenCapture::new(&std::path::PathBuf::from("../screenshots/run1/"))?;
    let screenshot = capture.frame()?;
    let reloc = localizer.relocalize(&screenshot, &roi);
    println!("   reloc: {reloc:?}");
    // localizer.localize(&screenshot, &roi);

    while capture.advance() {
        let screenshot = capture.frame()?;
        println!("Frame: {:?}", capture.frame_name());
        let start = std::time::Instant::now();

        if let Some(loc) = localizer.localize(&screenshot, &roi) {
            println!(
                "location: {:?} with {} landmarks",
                loc.position, loc.consistent_count
            );
            // localizer.mapping(&screenshot, &roi);
            println!("took {}", start.elapsed().as_secs_f64());
        } else {
            let reloc = localizer.relocalize(&screenshot, &roi);
            println!("   reloc: {reloc:?}");
        }
    }

    println!("Map: {:#?}", localizer.map().locations());

    Ok(())
}

pub fn main_arg(path: &std::path::Path) -> Result<(), Error> {
    let roi = test_roi();
    let map = config::load_map(path)?;
    let localizer = Localizer::new(map, Default::default(), Default::default());

    return run_on_capture(localizer, roi);

    Ok(())
}
