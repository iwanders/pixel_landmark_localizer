
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
            }
        }
    }

    pub fn get_features(&self, frame: &image::RgbaImage) -> Vec<Point> {
        use image::GenericImageView;

        // First, crop the roi and make it grayscale.
        let Rect{x, y, w, h} = self.region_crop;
        let roi_view = frame.view(x, y, w, h);

        let grey = image::DynamicImage::ImageRgba8(roi_view.to_image()).to_luma8();
        let res = self.fast_config.detect(&grey);


        // Now, shift the results back to the roi offset.
        let res_global = res.iter().map(|p| {
            fdf::Point {
                x: p.x + x,
                y: p.y + y,
            }
        }).collect::<Vec<_>>();

        // Next, filter by the ignore regions.
        let mut res = vec![];
        'point_iter: for p in res_global {
            for reg in self.region_ignore.iter() {
                if reg.contains(p.x, p.y) {
                    continue 'point_iter;
                }
            }
            res.push(Point{x: p.x as f32, y: p.y as f32})
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
        let global_new_points = new_points.iter().map(|p|{
            self.position + *p
        }).collect::<Vec<_>>();

        // Now that we have global points, we can match them against the reference using ICP.
        let base = self.reference.iter().map(|p| [p.x, p.y]).collect::<Vec<_>>();
        let other = global_new_points.iter().map(|p| [p.x, p.y]).collect::<Vec<_>>();
        let mut icp = IterativeClosestPoint2DTranslation::setup(&base, &other);
        for _ in 0..100 {
            icp.iterate(1, 30.0);
            // println!("t: {:?}", icp.transform());
        }
        let t = icp.transform();

        self.reference = icp.moving().iter().map(|p| {
            Point {
                x: p[0],
                y: p[1],
            }
        }).collect();
        let delta = Point{x: t[0], y: t[1]};
        self.position = self.position + delta;
        delta
    }

    pub fn position(&self) -> Point {
        self.position
    }
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

