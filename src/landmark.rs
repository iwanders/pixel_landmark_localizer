use screen_capture::RGB;
pub trait ToRgb {
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Landmark {
    pixels: Vec<Pixel>,
    pixel_difference_threshold: u16,
    width: u32,
    height: u32,
}

impl Landmark {
    pub fn from_image<T: image::GenericImageView<Pixel = Rgba<u8>>>(
        landmark: &T,
        pixel_difference_threshold: u16,
    ) -> Landmark {
        let mut pixels = vec![];
        let width = landmark.width();
        let height = landmark.height();
        for ly in 0..height {
            for lx in 0..width {
                let p = &landmark.get_pixel(lx, ly);
                if p.0[3] != 255 {
                    continue; // transparent pixel
                }
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
            width,
            height,
        }
    }

    pub fn present<T: image::GenericImageView<Pixel = Rgba<u8>>>(
        &self,
        img: &T,
        position: (u32, u32),
    ) -> bool {
        // Check bounds, if we don't fit on the image, we can for sure return false.
        if ((position.0 + self.width) > img.width()) || ((position.1 + self.height) > img.height())
        {
            return false;
        }

        for p in self.pixels.iter() {
            let x = position.0 + p.offset.0;
            let y = position.1 + p.offset.1;
            let pixel = img.get_pixel(x, y).to_rgb();
            let d = p.difference(&pixel);
            if d > self.pixel_difference_threshold {
                return false;
            }
        }
        true
    }

    pub fn pixels(&self) -> &[Pixel] {
        &self.pixels
    }
    pub fn pixel_difference_threshold(&self) -> u16 {
        self.pixel_difference_threshold
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
}
