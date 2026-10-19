# Test images

Made for Seura's tests, not taken from anyone's photo library. The project may redistribute them under the same terms as the rest of the repository (Apache-2.0).

| File | What it is |
| --- | --- |
| `quadrants.heic` | 64×48, red top left, green top right, blue bottom left, white bottom right; sRGB (an `nclx` color box with unspecified primaries). |
| `quadrants-orientation-6.heic` | The same, stored turned: `irot` 270° counter-clockwise, EXIF orientation 6. |
| `quadrants-orientation-2.heic` | The same, stored mirrored: `imir` axis 1, EXIF orientation 2. |
| `quadrants-p3.heic` | The same pixel values in Display P3 (an ICC profile). |

Generated 2026-10-18 on Nevada's MacBook Pro (macOS 26.6.2) by the Claude Code session there, with Apple's ImageIO (`CGImageDestination`, HEIC, quality 0.95) from a Swift script that draws the quadrants with Core Graphics. Each file carries its orientation twice, in the HEIF container and as EXIF, as iPhones do; decoders must use the container's and ignore the EXIF tag.

SHA-256:

- `quadrants.heic`: `aeadfbd30ae4c317bfabe953bd08823104a4548b9b004cf13d697a7b598b4b5f`
- `quadrants-orientation-6.heic`: `0bbb40cac640c3e2179d5f3990670cd0a3208d1d68f9f7d8871d8a7a51103fcc`
- `quadrants-orientation-2.heic`: `e4711e9409693439b1e5eec580b7307ffb16fe14bebb8b440f1474d961097c47`
- `quadrants-p3.heic`: `d2b83aa42863e0d42f28897fc48d0a70c24fa2c761752fa70015eb88dae1d54e`
