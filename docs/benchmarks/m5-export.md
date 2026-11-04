# M5 export: the JPEG encoder, and a 48-megapixel export

Status: recorded 2026-11-03 on the Windows PC · [M5 work plan, slice 2](../m5-plan.md)

The technical design asks for `jpeg-encoder`, which Seura has used since M1, to be compared with `mozjpeg`. `mozjpeg` is C with extra build tools, so the plan says it needs a clear win to change. It doesn't have one, so Seura keeps `jpeg-encoder`.

## The JPEG encoder

Each encoder compressed the same 8-bit sRGB pixels, in release builds, on one thread, with no color profile. Both kept full color detail (4:4:4), which is what `jpeg-encoder` does at quality 90. Time is the fastest of three runs. Quality is SSIM on luminance over 8×8 windows, where 1 is identical, measured by the comparison's own code.

| Photo | Encoder | Quality setting | File size | Time | SSIM |
| --- | --- | --- | --- | --- | --- |
| iPhone, 12 MP | `jpeg-encoder` | 90 | 1.38 MB | 168 ms | 0.99306 |
| | `mozjpeg` | 90 | 1.19 MB | 1367 ms | 0.99195 |
| | `mozjpeg` | 93, the closest match | 1.54 MB | 1489 ms | 0.99318 |
| iPhone, 7 MP | `jpeg-encoder` | 90 | 1.60 MB | 114 ms | 0.97905 |
| | `mozjpeg` | 90 | 1.29 MB | 1058 ms | 0.97363 |
| | `mozjpeg` | 92, the closest match | 1.45 MB | 1188 ms | 0.97977 |
| Generated, 12 MP | `jpeg-encoder` | 90 | 3.30 MB | 202 ms | 0.99959 |
| | `mozjpeg` | 90 | 2.56 MB | 2039 ms | 0.95730 |
| | `mozjpeg` | 97, the highest tried | 4.25 MB | 2660 ms | 0.99510 |

- **Size:** at the same quality setting `mozjpeg`'s files are 14 to 22% smaller, but they also score lower, since the setting doesn't mean the same thing in both. At a matching score it was 12% larger on one iPhone photo and 9% smaller on the other.
- **Time:** `mozjpeg` took about 8 to 10 times as long. This build had no SIMD, because that needs the NASM assembler, which isn't on this PC, so the gap would be smaller with it. Size doesn't depend on it.
- **Limits:** this is two real photos and one generated one, whose fine noise favors `jpeg-encoder`. SSIM isn't the measure `mozjpeg` tunes for, and nobody compared the results by eye. The result is enough to say there's no clear win, not that `jpeg-encoder` is better.

Two other settings were measured on `jpeg-encoder`:

- **Progressive** files were 4 to 14% larger at the same quality, so exports stay baseline.
- **Reduced color detail (4:2:0)** at quality 90 made files 6 to 21% smaller with almost the same SSIM, which only measures luminance. `jpeg-encoder` already switches to it below quality 90, so a file can get noticeably smaller between 90 and 89. The default stays at 90, with full color detail.

## A 48-megapixel export

One generated 48-megapixel JPEG, exported with no edits on the PC's NVIDIA RTX 4070 SUPER, in a release build. Time covers decoding, rendering, scaling, encoding and writing.

| Export | Time | File size | Peak memory |
| --- | --- | --- | --- |
| JPEG, original size | 1.80 s | 13.2 MB | 1.1 GB |
| JPEG, 2048 long edge | 1.19 s | 0.8 MB | 1.1 GB |
| PNG, original size | 10.10 s | 68.7 MB | 1.1 GB |
| PNG, 2048 long edge | 2.06 s | 4.0 MB | 1.1 GB |

Peak memory is the process's peak private memory, sampled about every 50 ms, so a shorter peak could be missed. It's about the same with every option, which suggests the peak comes before scaling and encoding.
