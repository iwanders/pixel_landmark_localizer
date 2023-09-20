pub fn main() -> Result<(), pixel_landmark_localizer::Error> {
    // pixel_landmark_localizer::main_landmark()
    // pixel_landmark_localizer::main_on_capture()
    pixel_landmark_localizer::main_arg(&std::path::PathBuf::from(
        std::env::args().nth(1).expect("should have argument"),
    ))
}
