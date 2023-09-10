#![allow(dead_code)]
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

    pub fn next_frame(&mut self) -> Result<image::RgbaImage, crate::Error> {
        let image_path = std::path::PathBuf::from(&self.files[self.index]);
        let orig_image = image::open(&image_path)?.to_rgba8();
        self.index += 1;
        Ok(orig_image)
    }
}
