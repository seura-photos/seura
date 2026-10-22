# A flaky test held up 0.1.1

- **Main's CI failed once** after the 0.1.1 version bump merged: on one macOS runner, the Viewer test "zooms to 100%" found the 100% button not pressed after clicking it. The other runners, and every earlier run, passed.
- **The race, in the test, not the app:** the test clicked as soon as the photo was drawn, but the Viewer records a photo as shown a moment after drawing it, and zooming waits for that. On a fast runner the click could land in between. The "+ and −" test had the same pattern. Both now wait until Fit shows as pressed, which happens only once the photo is shown.
- **0.1.1 went ahead** after re-running the failed job, since the app's code was fine.
