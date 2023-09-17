# Pixel Landmark Localizer

This crate searches landmarks composed of pixels in an image. Landmarks can have transparency in which case only the opaque pixels are utilised. The map consists of landmarks and their expected location. At each localisation cycle the previously determined position is used to calculate the map features that are expected to be within the current view and a grid-search is performed around those expected locations in order to determine the actual position.

This system can work well if the image on the screen always shift by full pixels and they are unaffected by anti-aliasing or lighting. Map overlays that are fully opaque are good candidates.

## Performance

The pixels that make up a landmark can be ordered by longest row-sequence, this ensures that when a pixel is checked for the presence of a landmark, in general a quick rejection occurs. It is paramount to ensure that landmarks can't 'snap' to the wrong location, ideally their positioning is globally unique. Currently partially overlapping landmarks, for example where a pattern can snap to two locations that are close aren't handled.

Since landmarks allow for transparency, they can be sparse, large and unique, ensuring high confidence in the found landmarks. Determining the location requires only scanning the current landmarks that are expected, which in general can occur in less than 0.2 milliseconds. If one landmark is found, it's best guess is used to check the presence of subsequent landmarks, which - if found - completely eliminates the grid search.


## License
License is `BSD-3-Clause`.
