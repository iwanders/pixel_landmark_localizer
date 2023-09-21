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
    name: Option<String>,
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
            name: None,
            pixel_difference_threshold,
            width,
            height,
        }
    }

    pub fn from_path(path: &std::path::Path) -> Result<Landmark, crate::Error> {
        // let image_path = std::path::PathBuf::from("../screenshots/landmark_3.png");
        if !path.is_file() {
            return Err(crate::Error::from(format!(
                "landmark file {path:?} not found"
            )));
        }
        let l1 = image::open(&path)?.to_rgba8();
        Ok(Self::from_image(&l1, 0))
    }

    pub fn to_rgba(&self) -> image::RgbaImage {
        let mut image = image::RgbaImage::from_pixel(self.width, self.height, image::Rgba([0; 4]));
        for p in self.pixels.iter() {
            *image.get_pixel_mut(p.offset.0, p.offset.1) = image::Rgba([p.rgb.r, p.rgb.g, p.rgb.b, 255 ]);
        }
        
        image
    }

    pub fn set_pixel_difference_threshold(&mut self, value: u16) {
        self.pixel_difference_threshold = value;
    }
    pub fn set_name(&mut self, value: Option<String>) {
        self.name = value;
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

    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    pub fn optimize_pixels_row_seq(&mut self) {
        // We want to order by longest sequence in x direction.
        if self.pixels.is_empty() {
            return;
        }

        // Sort by rows.
        let mut orig_pixels = self.pixels.clone();
        orig_pixels.sort_by(|a, b| {
            let a_yx = (a.offset.1, a.offset.0);
            let b_yx = (b.offset.1, b.offset.0);
            a_yx.cmp(&b_yx)
        });

        let mut pixel_ordering: Vec<(usize, Vec<Pixel>)> = vec![];
        let mut current_x = u32::MAX - 1;
        let mut current_y = u32::MAX - 1;

        for a in orig_pixels.iter() {
            if a.offset.1 == current_y && a.offset.0 == (current_x + 1) {
                let el = pixel_ordering.last_mut().unwrap();
                el.0 += 1;
                el.1.push(*a);
            } else {
                pixel_ordering.push((0, vec![*a]));
            }
            (current_x, current_y) = a.offset;
        }

        if false {
            println!("pixel_ordering: {pixel_ordering:?}");
            let mut total_pixels = 0;
            for z in pixel_ordering.iter() {
                total_pixels += z.1.len();
            }
            println!("Total pixels: {total_pixels}");
            println!("orig_pixels pixels: {}", orig_pixels.len());
        }
        self.pixels = pixel_ordering
            .iter()
            .map(|(_, pixel_vec)| pixel_vec.iter())
            .flatten()
            .copied()
            .collect();

        assert_eq!(self.pixels.len(), orig_pixels.len());
    }
}
