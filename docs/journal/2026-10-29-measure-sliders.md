# Measuring the Editor's sliders (M4 slice 3b)

- **PRF-6 can be measured now.** `SEURA_MEASURE=sliders` runs a script of its own in a release build:
  - It works on a library of one photo each of 12, 24 and 48 megapixels.
  - It opens each photo in the Editor and drags each slider back and forth for two seconds, a step every frame, timing the frames.
  - Each slider stays where its drag left it, so the last drag has every operation on.
  - The drag sends only the sliders' input events, which redraw the photo, never their change events, which save it, so no edit file is written.
  - The Editor is development-only until M4's wrap-up, so this switch makes it available to the build that measures.
- **`measure.py --sliders` runs just this part,** on WebGPU and then on WebGL2, as each later slice adds operations. The full run includes it too.
- **First result, Windows** (1440 × 900 window, 120 Hz display), with brightness, contrast, white balance and saturation:
  - every slider on every photo drew about 120 frames a second on both WebGPU and WebGL2, which is every frame the display shows; the target is 30;
  - opening took 0.14 s at 12 MP, 0.20 s at 24 MP and 0.32 s at 48 MP.
  - The preview is decoded to the window's size, so the photo's size only shows in the time to open it.
- **The Mac is still to be measured,** with a release bundle there.
