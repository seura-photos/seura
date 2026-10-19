//! Making thumbnails with the worker pool, on a temporary library.

use std::fs;
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use seura_library::{Filter, Index, Outcome, Scope, ThumbnailCache, ThumbnailWorkers};

/// Writes an RGB PNG of `width` × `height` in one color.
fn png(path: &Path, width: u32, height: u32) {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer
        .write_image_data(&vec![120; (width * height * 3) as usize])
        .unwrap();
    writer.finish().unwrap();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

/// Scans `library`, makes every photo's thumbnail, and returns the outcomes
/// by file name.
fn make_all(library: &Path, cache: &Path) -> (Index, Vec<(String, Outcome)>) {
    let mut index = Index::new();
    index.add_root(library).unwrap();
    let root = index.roots()[0].clone();
    index.scan(&root);
    let ids = index.query(Scope::All, &Filter::default());
    let jobs: Vec<_> = ids
        .iter()
        .filter_map(|id| index.thumbnail_job(*id))
        .collect();

    let (sender, receiver) = mpsc::channel();
    let workers = ThumbnailWorkers::start(ThumbnailCache::new(cache), 2, move |id, outcome| {
        sender.send((id, outcome)).unwrap();
    });
    workers.queue_background(jobs);
    let mut outcomes: Vec<(String, Outcome)> = (0..ids.len())
        .map(|_| {
            let (id, outcome) = receiver
                .recv_timeout(Duration::from_secs(60))
                .expect("a thumbnail finishes");
            if let Outcome::Ready(Some(made)) = &outcome {
                index.record_details(id, made);
            }
            (index.photo(id).unwrap().name.to_string(), outcome)
        })
        .collect();
    outcomes.sort_by(|a, b| a.0.cmp(&b.0));
    (index, outcomes)
}

#[test]
fn thumbnails_are_made_once_and_record_the_details() {
    let library = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    png(&library.path().join("wide.png"), 1600, 900);
    png(&library.path().join("Summer/tall.png"), 300, 1200);

    let (index, outcomes) = make_all(library.path(), cache.path());
    for (name, outcome) in &outcomes {
        assert!(
            matches!(outcome, Outcome::Ready(Some(_))),
            "{name}: {outcome:?}"
        );
    }
    // The index learned each photo's upright size.
    let sizes: Vec<_> = index
        .query(Scope::All, &Filter::default())
        .into_iter()
        .map(|id| index.photo(id).unwrap().dimensions)
        .collect();
    assert!(sizes.contains(&Some((1600, 900))) && sizes.contains(&Some((300, 1200))));

    // The thumbnails are WebP, at most 512 px on the long edge, in the cache.
    let thumbs = ThumbnailCache::new(cache.path());
    for id in index.query(Scope::All, &Filter::default()) {
        let job = index.thumbnail_job(id).unwrap();
        let bytes = fs::read(thumbs.path(&job.key)).unwrap();
        assert_eq!(&bytes[8..12], b"WEBP");
    }

    // A second pass finds them already made.
    let (_, outcomes) = make_all(library.path(), cache.path());
    assert!(
        outcomes
            .iter()
            .all(|(_, outcome)| *outcome == Outcome::Ready(None))
    );
}

#[test]
fn a_damaged_photo_fails_without_stopping_the_others() {
    let library = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    png(&library.path().join("good.png"), 64, 64);
    // A PNG signature followed by nothing usable.
    fs::write(
        library.path().join("bad.png"),
        b"\x89PNG\r\n\x1a\nnot really",
    )
    .unwrap();

    let (_, outcomes) = make_all(library.path(), cache.path());
    assert!(matches!(&outcomes[0], (name, Outcome::Failed(_)) if name == "bad.png"));
    assert!(matches!(&outcomes[1], (name, Outcome::Ready(Some(_))) if name == "good.png"));
}
