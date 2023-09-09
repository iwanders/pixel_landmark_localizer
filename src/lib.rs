// ffmpeg -i 2023-08-13_23-03-24.mp4 -r 60 frames/frame%04d.png

use feature_detector_fast as fdf;

use registration_icp_2d::IterativeClosestPoint2DTranslation;

pub type Error = Box<dyn std::error::Error>;

mod mock;
mod util;
use util::Rect;

#[derive(Clone, Default, Debug, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
impl std::ops::Add<Point> for Point {
    type Output = Point;
    fn add(self, v: Point) -> <Self as std::ops::Add<Point>>::Output {
        Point {
            x: self.x + v.x,
            y: self.y + v.y,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FeatureOdom {
    reference: Vec<Point>,
    region_crop: Rect,
    region_ignore: Vec<Rect>,
    position: Point,

    fast_config: fdf::Config,
}

impl FeatureOdom {
    pub fn new(region_crop: util::Rect) -> Self {
        FeatureOdom {
            position: Default::default(),
            reference: vec![],
            region_crop,
            region_ignore: vec![],
            fast_config: fdf::Config {
                threshold: 16,
                count: 9,
                non_maximal_supression: fdf::NonMaximalSuppression::MaxThreshold,
            },
        }
    }

    pub fn get_features(&self, frame: &image::RgbaImage) -> Vec<Point> {
        use image::GenericImageView;

        // First, crop the roi and make it grayscale.
        let Rect { x, y, w, h } = self.region_crop;
        let roi_view = frame.view(x, y, w, h);

        let grey = image::DynamicImage::ImageRgba8(roi_view.to_image()).to_luma8();
        let res = self.fast_config.detect(&grey);

        // Now, shift the results back to the roi offset.
        let res_global = res
            .iter()
            .map(|p| fdf::Point {
                x: p.x + x,
                y: p.y + y,
            })
            .collect::<Vec<_>>();

        // Next, filter by the ignore regions.
        let mut res = vec![];
        'point_iter: for p in res_global {
            for reg in self.region_ignore.iter() {
                if reg.contains(p.x, p.y) {
                    continue 'point_iter;
                }
            }
            res.push(Point {
                x: p.x as f32,
                y: p.y as f32,
            })
        }
        res
    }

    pub fn set_reference(&mut self, points: &[Point]) {
        self.reference = points.to_vec();
    }

    pub fn update(&mut self, frame: &image::RgbaImage) -> Point {
        // Get points from the new image.
        let new_points = self.get_features(&frame);

        // Transform new points with the current transform.
        let global_new_points = new_points
            .iter()
            .map(|p| self.position + *p)
            .collect::<Vec<_>>();

        // Now that we have global points, we can match them against the reference using ICP.
        let base = self
            .reference
            .iter()
            .map(|p| [p.x, p.y])
            .collect::<Vec<_>>();
        let other = global_new_points
            .iter()
            .map(|p| [p.x, p.y])
            .collect::<Vec<_>>();
        let mut icp = IterativeClosestPoint2DTranslation::setup(&base, &other);
        for _ in 0..100 {
            icp.iterate(1, 30.0);
            // println!("t: {:?}", icp.transform());
        }
        let t = icp.transform();

        self.reference = icp
            .moving()
            .iter()
            .map(|p| Point { x: p[0], y: p[1] })
            .collect();
        let delta = Point { x: t[0], y: t[1] };
        self.position = self.position + delta;
        delta
    }

    pub fn position(&self) -> Point {
        self.position
    }
}

pub fn main_blocks() -> Result<(), Error> {
    use image::GenericImageView;
    type RgbaSubImage<'a> = image::SubImage<&'a image::ImageBuffer<image::Rgba<u8>, Vec<u8>>>;

    let image_path = std::path::PathBuf::from("../screenshots/Screenshot399.png");
    let s0 = image::open(&image_path)?.to_rgba8();
    let image_path = std::path::PathBuf::from("../screenshots/Screenshot400.png");
    let s1 = image::open(&image_path)?.to_rgba8();


    let block = Rect {
        x: 135,
        y: 742,
        w: 16,
        h: 16,
    };
    let Rect { x, y, w, h } = block;
    let block_roi = s0.view(x, y, w, h);

    block_roi.to_image().save("/tmp/block_image.png")?;

    fn score(a: &RgbaSubImage, b: &RgbaSubImage) -> u32 {
        // let mut score = 0;
        a.pixels()
            .zip(b.pixels())
            .map(|(pa, pb)| {
                // println!("pa: {pa:?}");
                let (_xa, _ya, pa) = pa;
                let (_xb, _yb, pb) = pb;
                pa.0.iter()
                    .zip(pb.0.iter())
                    .map(|(xa, xb)| (*xa as i32 - *xb as i32).abs() as u32)
                    .sum::<u32>()
            })
            .sum()
    }

    fn find_match(
        block: &RgbaSubImage,
        other: &image::RgbaImage,
        start: (u32, u32),
        x: (i32, i32),
        y: (i32, i32),
    ) -> u32 {
        let xmin = (start.0 as i32 + x.0).max(0).min(other.width() as i32) as u32;
        let xmax = (start.0 as i32 + x.1).max(0).min(other.width() as i32) as u32;
        let ymin = (start.1 as i32 + y.0).max(0).min(other.height() as i32) as u32;
        let ymax = (start.1 as i32 + y.1).max(0).min(other.height() as i32) as u32;
        println!("xmin: {xmin:?}");
        println!("xmax: {xmax:?}");
        println!("ymin: {ymin:?}");
        println!("ymax: {ymax:?}");

        // can do the moving histogram trick.
        let mut lowest = u32::MAX;
        let mut best = (0, 0);

        for y in ymin..ymax {
            for x in xmin..xmax {
                let other_sub = other.view(x, y, block.width(), block.height());
                let score = score(block, &other_sub);
                if score < lowest {
                    lowest = score;
                    best = (x, y);
                }
                // return score;
            }
        }
        println!("lowest: {lowest:?}");
        println!("Best: {best:?}");
        0
    }

    let start = std::time::Instant::now();

    let best = find_match(
        &block_roi,
        &s1,
        (block.x, block.y),
        (-100, 100),
        (-100, 100),
    );
    println!("best: {best:?}, took {}", start.elapsed().as_secs_f64());

    Ok(())
}

pub fn main() -> Result<(), Error> {
    let mut mock_capture = mock::MockScreenCapture::new(std::path::Path::new("/tmp/frames/"))?;

    let frame1 = mock_capture.next_frame()?;
    let roi = util::Rect {
        x: 492,
        y: 110,
        w: 1416,
        h: 742,
    };

    let mut odom = FeatureOdom::new(roi);

    let reference_points = odom.get_features(&frame1);
    odom.set_reference(&reference_points);

    for i in 0..100 {
        let frame2 = mock_capture.next_frame()?;
        let delta = odom.update(&frame2);
        println!("i: {i:?} delta: {delta:?}  pos: {:?}", odom.position());
    }

    Ok(())
}



pub fn main_histogram() -> Result<(), Error> {
    use image::GenericImageView;
    type RgbaSubImage<'a> = image::SubImage<&'a image::ImageBuffer<image::Rgba<u8>, Vec<u8>>>;

    let image_path = std::path::PathBuf::from("../screenshots/Screenshot399.png");
    let s0 = image::open(&image_path)?.to_rgba8();
    let image_path = std::path::PathBuf::from("../screenshots/Screenshot400.png");
    let s1 = image::open(&image_path)?.to_rgba8();

    let roi = util::Rect {
        x: 492,
        y: 110,
        w: 1416,
        h: 742,
    };

    let Rect { x, y, w, h } = roi;

    let s0_block_roi = s0.view(x, y, w, h);
    s0_block_roi.to_image().save("/tmp/s0_block_image.png")?;

    let s1_block_roi = s1.view(x, y, w, h);
    s1_block_roi.to_image().save("/tmp/s1_block_image.png")?;


    fn histogram(a: &RgbaSubImage) -> (Vec<u32>, Vec<u32>)  {
        let start = std::time::Instant::now();
        let mut hist_y = vec![0u32; a.width() as usize];
        let mut hist_x = vec![0u32; a.height() as usize];
        for iy in 0..a.height() {
            for ix in 0..a.width() {
                let value = a.get_pixel(ix as u32, iy as u32).0[1] as u32;
                hist_y[ix as usize] += value;
                hist_x[iy as usize] += value;
            }
        }
        
        println!("took {}", start.elapsed().as_secs_f64());
        (hist_x, hist_y)
    }


    let (s0_x, s0_y) = histogram(&s0_block_roi);
    let (s1_x, s1_y) = histogram(&s1_block_roi);

    println!("s0_x {s0_x:?}");
    println!("s0_y {s0_y:?}");

    fn best_cross_correlation(a: &[u32], b: &[u32]) -> (isize, u64) {
        let left_lag = -(b.len() as isize - 1);
        let right_lag = a.len() as isize;
        // let result_size = right_lag - left_lag + 1;
        let mut correlations = vec![];

        for lag in left_lag..right_lag {
            let start = lag.max(0);
            let end = a.len().min((b.len() as isize + lag) as usize);
            let mut s: u64 = 0;
            for i in start as usize..end as usize {
                // s += (a[i] * b[i - lag as usize]) as u64;
                s += (a[i] as i32 - b[i - lag as usize] as i32).abs() as u64;
            }
            correlations.push((lag, s / ((end as usize - start as usize) as u64)));
            // correlations.push((lag, s));
        }
        println!("correlations: {correlations:?}");

        let mut best_score = (0, 199999999);
        // let mut best_score = (0, 0);
        for i in 0..correlations.len() {
            let (lag, s) = correlations[i];
            if s < best_score.1 {
            // if s > best_score.1 {
                best_score = (lag, s);
            }
        }
        best_score
        // (0, 0)
    }

    let best_corr_y = best_cross_correlation(&s0_x, &s1_x);
    let best_corr_x = best_cross_correlation(&s0_y, &s1_y);
    println!("best_corr_x {best_corr_x:?}");
    println!("best_corr_y {best_corr_y:?}");

    fn combine_images(a: &RgbaSubImage, b: &RgbaSubImage, offsets: (i32, i32)) -> image::RgbaImage {
        use image::GenericImage;
        let combined_w = a.width() + b.width() * 2;
        let combined_h = a.height() + b.height() * 2;
        println!("offsets: {offsets:?}");
        let mut img = image::RgbaImage::new(combined_w, combined_h);
        img.copy_from(&a.to_image(), b.width() * 1, b.height() * 1).unwrap();
        img.copy_from(&b.to_image(), (a.width() as i32 * 1  + offsets.0) as u32, (a.height()as i32  * 1  + offsets.1) as u32).unwrap();
        img
    }

    // let xoff = -best_corr_y.0 as i32;
    // let yoff = w as i32 - best_corr_x.0 as i32;
    let xoff = best_corr_x.0 as i32;
    let yoff = best_corr_y.0 as i32;

    let c = combine_images(&s0_block_roi, &s1_block_roi, (xoff, yoff));
    c.save("/tmp/combined_c.png")?;
    
    Ok(())
}

use image::Rgba;
fn pixel_diff_squared(a: &Rgba<u8>, b: &Rgba<u8>)  -> u16 {
    println!("a: {a:?}, b:{b:?}");
    a.0.iter().zip(b.0.iter()).map(|(pa, pb)| {
        (pa.max(pb) - pa.min(pb)) as u16
    }).sum()

}

pub fn main_landmark() -> Result<(), Error> {
    use image::GenericImageView;
    type RgbaSubImage<'a> = image::SubImage<&'a image::ImageBuffer<image::Rgba<u8>, Vec<u8>>>;

    let image_path = std::path::PathBuf::from("../screenshots/Screenshot418.png");
    let screenshot = image::open(&image_path)?.to_rgba8();
    
    let image_path = std::path::PathBuf::from("../screenshots/landmark_1.png");
    let l1 = image::open(&image_path)?.to_rgba8();

    let block = Rect {
        x: 160,
        y: 80,
        w: 600,
        h: 400,
    };
    let Rect { x, y, w, h } = block;
    // let block_roi = screenshot.view(x, y, w, h);
    let block_roi = screenshot.view(0, 0, screenshot.width(), screenshot.height());

    block_roi.to_image().save("/tmp/block_image.png")?;

    fn find_match(
        block: &RgbaSubImage,
        lm: &image::RgbaImage,
    ) -> u32 {
        for y in 0..(block.height() - lm.height()) {
            'b: for x in 0..(block.width() - lm.width()) {
                if (x == 322 && y == 212) {
                    // here, iterate through the landmark.
                    let mut sum = 0;
                    for ly in 0..lm.height() {
                        for lx in 0..lm.width() {
                            let b = &block.get_pixel(x + lx, y + ly);
                            let l = &lm.get_pixel(lx, ly);
                            let diff = pixel_diff_squared(b, l);
                            println!("{x},{y}, {lx},{ly} -> {diff}");
                            if diff > 16 {
                                continue 'b;
                            }
                            sum += diff;
                        }
                    }
                    println!("Found landmark at {x}, {y} with {sum} ");
                }
            }
        }
        0
    }

    let start = std::time::Instant::now();

    let best = find_match(
        &block_roi,
        &l1,
    );
    println!("best: {best:?}, took {}", start.elapsed().as_secs_f64());

    Ok(())
}