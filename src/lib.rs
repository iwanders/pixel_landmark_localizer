// ffmpeg -i 2023-08-13_23-03-24.mp4 -r 60 frames/frame%04d.png

pub type Error = Box<dyn std::error::Error>;

mod mock;
mod util;
pub use util::Rect;

pub fn main() -> Result<(), Error> {
    let mut mock_capture = mock::MockScreenCapture::new(std::path::Path::new("/tmp/frames/"))?;

    let _frame1 = mock_capture.next_frame()?;
    let _roi = util::Rect {
        x: 0,
        y: 0,
        w: 640,
        h: 408,
    };

    // let mut odom = FeatureOdom::new(roi);

    // let reference_points = odom.get_features(&frame1);
    // odom.set_reference(&reference_points);

    // for i in 0..100 {
    // let _frame2 = mock_capture.next_frame()?;
    // let delta = odom.update(&frame2);
    // println!("i: {i:?} delta: {delta:?}  pos: {:?}", odom.position());
    // }

    Ok(())
}

use image::Rgba;
fn pixel_diff_squared(a: &Rgba<u8>, b: &Rgba<u8>) -> u16 {
    // println!("a: {a:?}, b:{b:?}");
    a.0.iter()
        .zip(b.0.iter())
        .map(|(pa, pb)| (pa.max(pb) - pa.min(pb)) as u16)
        .sum()
}

pub fn main_landmark() -> Result<(), Error> {
    use image::GenericImageView;
    type RgbaSubImage<'a> = image::SubImage<&'a image::ImageBuffer<image::Rgba<u8>, Vec<u8>>>;

    let image_path = std::path::PathBuf::from("../screenshots/Screenshot447.png");
    let screenshot = image::open(&image_path)?.to_rgba8();

    let image_path = std::path::PathBuf::from("../screenshots/landmark_1.png");
    let l1 = image::open(&image_path)?.to_rgba8();

    let block = Rect {
        x: 0,
        y: 0,
        w: 640,
        h: 408,
    };
    let Rect { x, y, w, h } = block;
    let block_roi = screenshot.view(x, y, w, h);
    // let block_roi = screenshot.view(0, 0, screenshot.width(), screenshot.height());

    block_roi.to_image().save("/tmp/block_image.png")?;

    fn find_match(block: &RgbaSubImage, lm: &image::RgbaImage) -> u32 {
        let pixel_diff_limit = 16;
        let sum_limit = 32;
        for y in 0..(block.height() - lm.height()) {
            'b: for x in 0..(block.width() - lm.width()) {
                // here, iterate through the landmark.
                let mut sum = 0;
                for ly in 0..lm.height() {
                    for lx in 0..lm.width() {
                        let b = &block.get_pixel(x + lx, y + ly);
                        let l = &lm.get_pixel(lx, ly);
                        let diff = pixel_diff_squared(b, l);
                        // println!("{x},{y}, {lx},{ly} -> {diff}");
                        if diff > pixel_diff_limit {
                            continue 'b;
                        }
                        sum += diff;
                    }
                }
                if sum < sum_limit {
                    println!("Found landmark at {x}, {y} with {sum} ");
                }
            }
        }
        0
    }

    let start = std::time::Instant::now();

    let best = find_match(&block_roi, &l1);
    println!("best: {best:?}, took {}", start.elapsed().as_secs_f64());

    Ok(())
}
