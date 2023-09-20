#![allow(dead_code)]

/// Something to mock the screen_capture module.
pub struct MockScreenCapture {
    files: Vec<std::path::PathBuf>,
    index: usize,
}

impl MockScreenCapture {
    pub fn new(path: &std::path::Path) -> Result<Self, crate::Error> {
        use std::fs;

        let paths = fs::read_dir(path)?;
        let mut files = vec![];
        for path in paths {
            files.push(path?.path());
        }

        files.sort();
        Ok(MockScreenCapture { files, index: 0 })
    }

    pub fn advance(&mut self) -> bool {
        self.index += 1;
        self.has_next()
    }

    pub fn frame_name(&self) -> &std::path::Path {
        &self.files[self.index]
    }

    pub fn frame(&mut self) -> Result<image::RgbaImage, crate::Error> {
        let image_path = std::path::PathBuf::from(&self.files[self.index]);
        let orig_image = image::open(&image_path)?.to_rgba8();
        Ok(orig_image)
    }

    pub fn has_next(&self) -> bool {
        self.index < self.files.len()
    }
}

/// Wrapper such that we can implement GenericImageView for the RGB buffer.
pub struct CaptureAdaptor<'a> {
    pub width: usize,
    pub height: usize,
    pub buffer: &'a [screen_capture::RGB],
}
impl<'a> image::GenericImageView for CaptureAdaptor<'a> {
    type Pixel = image::Rgba<u8>;
    fn dimensions(&self) -> (u32, u32) {
        (self.width as u32, self.height as u32)
    }
    fn bounds(&self) -> (u32, u32, u32, u32) {
        (0, 0, self.width as u32, self.height as u32)
    }
    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
        let rgb = self.buffer[y as usize * self.width + x as usize];
        let mut res = image::Rgba::<u8>::from([0; 4]);
        res.0[0] = rgb.r;
        res.0[1] = rgb.g;
        res.0[2] = rgb.b;
        res.0[3] = 0;
        res
    }
}

trait CaptureAdapted {
    fn as_adapted(&self) -> CaptureAdaptor;
}
