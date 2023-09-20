use pixel_landmark_localizer as pll;

pub fn main() -> Result<(), pixel_landmark_localizer::Error> {
    let path = &std::path::PathBuf::from(std::env::args().nth(1).expect("should have argument"));
    let roi = pll::test_roi();
    let map = pll::config::load_map(path)?;
    let localizer = pll::Localizer::new(map, Default::default(), Default::default());

    let mut localizer = localizer;
    let mut capture = pixel_landmark_localizer::get_configured_capture();
    loop {
        let res = capture.capture_image();

        if !res {
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }

        // Then, we can grab the actual image.
        let img = capture.get_image();

        let start = std::time::Instant::now();
        let screenshot = pll::CaptureWrap {
            width: img.get_width() as usize,
            height: img.get_height() as usize,
            buffer: img.get_data().unwrap(),
        };

        if let Some(loc) = localizer.localize(&screenshot, &roi) {
            println!(
                "location: {:?} with {} landmarks",
                loc.position, loc.consistent_count
            );
            localizer.mapping(&screenshot, &roi);
            println!("took {}", start.elapsed().as_secs_f64());
        } else {
            let reloc = localizer.relocalize(&screenshot, &roi);
            println!("   reloc: {reloc:?}");
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
