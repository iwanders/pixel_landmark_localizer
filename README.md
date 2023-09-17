# Pixel Landmark Localizer

This crate searches landmarks composed of pixels in an image. Landmarks can have transparency in which case only the opaque pixels are utilised. The map consists of landmarks and their expected location. At each localisation cycle the previously determined position is used to calculate the map features that are expected to be within the current view and a grid-search is performed around those expected locations in order to determine the actual position.

The pixels that make up a landmark are ordered by longest row-sequence, this ensures that when a pixel is checked for the presence of a landmark, in general a quick rejection occurs.

This system can work well if the image on the screen always shift by full pixels and they are unaffected by anti-aliasing or lighting. Map overlays that are fully opaque are good candidates.

## License
License is `BSD-3-Clause`.
