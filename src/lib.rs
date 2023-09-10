// ffmpeg -i 2023-08-13_23-03-24.mp4 -r 60 frames/frame%04d.png

pub type Error = Box<dyn std::error::Error>;

mod landmark;
mod mock;
mod util;
pub use util::Rect;

pub use landmark::Landmark;
pub mod map;

pub fn main_landmark() -> Result<(), Error> {
    let image_path = std::path::PathBuf::from("../screenshots/Screenshot439.png");
    let screenshot = image::open(&image_path)?.to_rgba8();

    let image_path = std::path::PathBuf::from("../screenshots/landmark_3.png");
    let l1 = image::open(&image_path)?.to_rgba8();

    let lm = Landmark::from_image(&l1, 6);

    let block = Rect {
        x: 0,
        y: 0,
        w: 640,
        h: 408,
    };
    // let Rect { x, y, w, h } = block;
    // let block_roi = screenshot.view(x, y, w, h);
    // let block_roi = screenshot.view(0, 0, screenshot.width(), screenshot.height());

    // block_roi.to_image().save("/tmp/block_image.png")?;

    fn find_match(block: &image::RgbaImage, rect: &Rect, lm: &Landmark) -> u32 {
        for y in rect.y..(rect.y + rect.h) {
            for x in rect.x..(rect.x + rect.w) {
                let present = lm.present(block, (x, y));

                if present {
                    println!("Found landmark at {x}, {y} ");
                }
            }
        }
        0
    }

    let start = std::time::Instant::now();

    let best = find_match(&screenshot, &block, &lm);
    println!("best: {best:?}, took {}", start.elapsed().as_secs_f64());

    Ok(())
}
