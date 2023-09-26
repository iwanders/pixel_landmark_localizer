use pixel_landmark_localizer as pll;
use pixel_landmark_localizer::CaptureAdapted;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The config file specifying the map and landmarks.
    map: String,

    /// Output path for updated map.
    #[arg(long, short)]
    output: Option<String>,

    /// Whether or not to run the landmark erosion to refine them.
    #[arg(long, short, default_value = "false")]
    erode: bool,

    /// Whether or not to search for all landmarks each frame.
    #[arg(long, short, default_value = "false")]
    mapping: bool,

    /// Path to read the capture configuration from.
    #[arg(long, short)]
    capture_config: Option<String>,
}

pub fn main() -> Result<(), pixel_landmark_localizer::Error> {
    let args = Args::parse();
    println!("args: {args:?}");

    let path = &std::path::PathBuf::from(&args.map);

    let roi = pll::test_roi();
    let map = pll::config::load_map(path)?;

    let mut eroded_landmarks: Vec<image::RgbaImage> = map
        .landmark_ids()
        .drain(..)
        .map(|l| map.landmark(&l).to_rgba())
        .collect();

    let localizer = pll::Localizer::new(map, Default::default(), Default::default());

    let mut localizer = localizer;

    let capture_config_path = if let Some(capture_config) = args.capture_config {
        std::path::PathBuf::from(capture_config)
    } else {
        std::path::PathBuf::from("config").join(format!("{}.yaml", std::env::consts::OS))
    };

    let capture_config =
        pll::config::read_deserializable::<pll::capture::Config>(&capture_config_path)?;
    let mut capture = pll::capture::CaptureGrabber::new(capture_config);

    capture.update_resolution();

    loop {
        let res = capture.capture();

        if res.is_none() {
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }

        // Then, we can grab the actual image.
        let screenshot = res.unwrap();

        let start = std::time::Instant::now();

        if let Some(loc) = localizer.localize(&screenshot.as_adapted(), &roi) {
            if args.erode {
                for landmark in loc.matches.iter() {
                    use image::GenericImageView;
                    // get view of the landmark.
                    let adapted = screenshot.as_adapted();
                    let eroded = &mut eroded_landmarks[landmark.location.id.to_index()];
                    let view = adapted.view(
                        landmark.screen_position.0.x as u32,
                        landmark.screen_position.0.y as u32,
                        eroded.width(),
                        eroded.height(),
                    );
                    // now here, iterate over the view and eroded, make eroded transparent if pixel is different.

                    let mut changed = false;
                    for (eroded_pixel, (_, _, current)) in eroded.pixels_mut().zip(view.pixels()) {
                        if &eroded_pixel.0[0..3] != &current.0[0..3] {
                            println!("eroded: {eroded_pixel:?}, current: {current:?}");
                            *eroded_pixel = image::Rgba([0; 4]);
                            changed = true;
                        }
                    }
                    if changed {
                        eroded.save(format!(
                            "/tmp/eroded_{}.png",
                            localizer
                                .map()
                                .landmark(&landmark.location.id)
                                .name()
                                .expect("should have name")
                        ))?;
                    }
                }
            }

            println!(
                "location: {:?} with {} landmarks",
                loc.position, loc.consistent_count
            );

            if args.mapping {
                let r = localizer.mapping(&screenshot.as_adapted(), &roi);
                if !r.is_empty() {
                    for loc in r {
                        println!("New location: {loc:?}");
                    }
                    println!("{}", pll::config::save_map_string(localizer.map())?);
                    if let Some(output_path) = &args.output {
                        pll::config::save_map(
                            &std::path::PathBuf::from(output_path),
                            localizer.map(),
                        )?
                    }
                }
            }
            println!("took {}", start.elapsed().as_secs_f64());
        } else {
            let reloc = localizer.relocalize(&screenshot.as_adapted(), &roi);
            if !reloc.is_none() {
                println!("   reloc: {reloc:?}");
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
