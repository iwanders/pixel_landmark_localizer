// ffmpeg -i 2023-08-13_23-03-24.mp4 -r 60 frames/frame%04d.png

pub type Error = Box<dyn std::error::Error>;

mod mock;
mod util;
pub use util::Rect;

use screen_capture::RGB;

trait ToRgb {
    fn to_rgb(&self) -> RGB;
}
impl ToRgb for Rgba<u8> {
    fn to_rgb(&self) -> RGB {
        RGB {
            r: self.0[0],
            g: self.0[1],
            b: self.0[2],
        }
    }
}

use image::Rgba;
fn pixel_diff_squared(a: &Rgba<u8>, b: &Rgba<u8>) -> u16 {
    // println!("a: {a:?}, b:{b:?}");
    a.0.iter()
        .zip(b.0.iter())
        .map(|(pa, pb)| (pa.max(pb) - pa.min(pb)) as u16)
        .sum()
}

#[derive(Debug, Clone, Copy)]
pub struct Pixel {
    pub rgb: RGB,
    pub offset: (u32, u32),
}
impl Pixel {
    pub fn difference(&self, other: &RGB) -> u16 {
        (self.rgb.r.max(other.r) - self.rgb.r.min(other.r)) as u16
            + (self.rgb.g.max(other.g) - self.rgb.g.min(other.g)) as u16
            + (self.rgb.b.max(other.b) - self.rgb.b.min(other.b)) as u16
    }
}

#[derive(Debug, Clone)]
pub struct Landmark {
    pixels: Vec<Pixel>,
    pixel_difference_threshold: u16,
    pixel_sum_threshold: u16,
    width: u32,
    height: u32,
}

impl Landmark {
    pub fn from_image<T: image::GenericImageView<Pixel = Rgba<u8>>>(
        landmark: &T,
        pixel_difference_threshold: u16,
        pixel_sum_threshold: u16,
    ) -> Landmark {
        let mut pixels = vec![];
        let width = landmark.width();
        let height = landmark.height();
        for ly in 0..height {
            for lx in 0..width {
                let p = &landmark.get_pixel(lx, ly);
                let rgb = RGB {
                    r: p.0[0],
                    g: p.0[1],
                    b: p.0[2],
                };
                pixels.push(Pixel {
                    rgb,
                    offset: (lx, ly),
                })
            }
        }
        Landmark {
            pixels,
            pixel_difference_threshold,
            pixel_sum_threshold,
            width,
            height,
        }
    }

    pub fn present<T: image::GenericImageView<Pixel = Rgba<u8>>>(
        &self,
        img: &T,
        position: (u32, u32),
    ) -> bool {
        let mut sum = 0;
        for p in self.pixels.iter() {
            let x = position.0 + p.offset.0;
            let y = position.1 + p.offset.1;
            let pixel = img.get_pixel(x, y).to_rgb();
            let d = p.difference(&pixel);
            if d > self.pixel_difference_threshold {
                return false;
            }
            sum += d;
        }
        sum < self.pixel_sum_threshold
    }

    pub fn pixels(&self) -> &[Pixel] {
        &self.pixels
    }
    pub fn pixel_difference_threshold(&self) -> u16 {
        self.pixel_difference_threshold
    }
    pub fn pixel_sum_threshold(&self) -> u16 {
        self.pixel_sum_threshold
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
}

pub fn main_landmark() -> Result<(), Error> {
    use image::GenericImageView;
    type RgbaSubImage<'a> = image::SubImage<&'a image::ImageBuffer<image::Rgba<u8>, Vec<u8>>>;

    let image_path = std::path::PathBuf::from("../screenshots/Screenshot447.png");
    let screenshot = image::open(&image_path)?.to_rgba8();

    let image_path = std::path::PathBuf::from("../screenshots/landmark_1.png");
    let l1 = image::open(&image_path)?.to_rgba8();

    let lm = Landmark::from_image(&l1, 16, 32);

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
        let pixel_diff_limit = 16;
        let sum_limit = 32;
        for y in rect.y..(rect.y + rect.h) {
            'b: for x in rect.x..(rect.x + rect.w) {
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
