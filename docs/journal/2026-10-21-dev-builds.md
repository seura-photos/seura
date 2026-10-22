# Faster development builds

- **Slow to open in development builds:** Nevada noticed a photo took several seconds to appear in the Editor. `cargo tauri dev` builds without optimization, and opening a 12-megapixel photo took 13 s that way against 0.23 s optimized. At Nevada's go-ahead, development builds now optimize the libraries and Seura's image code (0.48 s), leaving the rest unoptimized so it rebuilds quickly.
