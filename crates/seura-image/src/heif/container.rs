//! Reads what Seura needs from a HEIF container, independently of whichever
//! decoder turns the compressed data into pixels: the primary image's
//! rotation and mirroring (`irot`, `imir`) and its color profile (`colr`).
//!
//! HEIF stores orientation in the container, not as EXIF (an EXIF
//! orientation in a HEIC must be ignored, or the photo turns twice). The
//! decoders used so far (ImageIO, WIC) apply it themselves; reading it here
//! lets a decoder that doesn't be corrected (ISO/IEC 23008-12, sections
//! 6.5.10 to 6.5.12).
//! Only the `meta` box is read, and every length is checked against the data.

/// One geometric property of the primary image, in the order the file lists it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transform {
    /// Counter-clockwise rotation in degrees: 90, 180, or 270.
    Rotate(u16),
    /// Mirror about a vertical axis (left and right swap).
    MirrorHorizontal,
    /// Mirror about a horizontal axis (top and bottom swap).
    MirrorVertical,
}

/// The color description of the primary image.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Color {
    /// An embedded ICC profile.
    Icc(Vec<u8>),
    /// Coded color primaries (ITU-T H.273): 1 is sRGB/BT.709, 12 is Display P3.
    Primaries(u16),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Container {
    pub transforms: Vec<Transform>,
    pub color: Option<Color>,
}

/// How much of the file is read to find the `meta` box: plenty for any
/// real HEIC, small enough that a damaged file can't run away with memory.
pub const HEAD_LIMIT: usize = 1 << 20;

/// Reads the container from the start of a HEIF file. Anything malformed
/// yields an empty description: the photo then shows as decoded.
pub fn read(head: &[u8]) -> Container {
    parse(head).unwrap_or_default()
}

fn parse(data: &[u8]) -> Option<Container> {
    let meta = boxes(data).find(|(kind, _)| kind == b"meta")?.1;
    let meta = meta.get(4..)?; // full box: version and flags
    let primary = primary_item(boxes(meta).find(|(kind, _)| kind == b"pitm")?.1)?;
    let iprp = boxes(meta).find(|(kind, _)| kind == b"iprp")?.1;
    let properties: Vec<([u8; 4], &[u8])> =
        boxes(boxes(iprp).find(|(kind, _)| kind == b"ipco")?.1).collect();
    let mut container = Container::default();
    for (_, body) in boxes(iprp).filter(|(kind, _)| kind == b"ipma") {
        for index in associations(body, primary)? {
            let Some((property, data)) = index.checked_sub(1).and_then(|i| properties.get(i))
            else {
                continue;
            };
            match property {
                b"irot" => {
                    let angle = u16::from(*data.first()? & 0b11) * 90;
                    if angle != 0 {
                        container.transforms.push(Transform::Rotate(angle));
                    }
                }
                // Axis 1 is a left-right mirror: that's how Apple writes it
                // (for EXIF orientation 2) and how libheif and WIC read it,
                // whatever the standard's wording suggests.
                b"imir" => container.transforms.push(if data.first()? & 1 == 1 {
                    Transform::MirrorHorizontal
                } else {
                    Transform::MirrorVertical
                }),
                b"colr" if container.color.is_none() => container.color = color(data),
                _ => {}
            }
        }
    }
    Some(container)
}

/// The child boxes in `data`, as (type, body). Stops at the first malformed one.
fn boxes(data: &[u8]) -> impl Iterator<Item = ([u8; 4], &[u8])> {
    let mut rest = data;
    std::iter::from_fn(move || {
        let size = u32::from_be_bytes(rest.get(..4)?.try_into().ok()?) as usize;
        let kind: [u8; 4] = rest.get(4..8)?.try_into().ok()?;
        let (header, size) = match size {
            0 => (8, rest.len()),
            1 => (
                16,
                usize::try_from(u64::from_be_bytes(rest.get(8..16)?.try_into().ok()?)).ok()?,
            ),
            size => (8, size),
        };
        let body = rest.get(header..size)?;
        rest = &rest[size..];
        Some((kind, body))
    })
}

/// `pitm`: the primary item's id.
fn primary_item(body: &[u8]) -> Option<u32> {
    match *body.first()? {
        0 => Some(u32::from(u16::from_be_bytes(
            body.get(4..6)?.try_into().ok()?,
        ))),
        _ => Some(u32::from_be_bytes(body.get(4..8)?.try_into().ok()?)),
    }
}

/// `ipma`: the 1-based property indices associated with `item`, in order.
fn associations(body: &[u8], item: u32) -> Option<Vec<usize>> {
    let version = *body.first()?;
    let large_indices = body.get(3)? & 1 == 1;
    let count = u32::from_be_bytes(body.get(4..8)?.try_into().ok()?);
    let mut at = 8;
    for _ in 0..count {
        let id = if version < 1 {
            let id = u32::from(u16::from_be_bytes(body.get(at..at + 2)?.try_into().ok()?));
            at += 2;
            id
        } else {
            let id = u32::from_be_bytes(body.get(at..at + 4)?.try_into().ok()?);
            at += 4;
            id
        };
        let entries = *body.get(at)? as usize;
        at += 1;
        let mut indices = Vec::with_capacity(entries);
        for _ in 0..entries {
            let index = if large_indices {
                let value = u16::from_be_bytes(body.get(at..at + 2)?.try_into().ok()?);
                at += 2;
                usize::from(value & 0x7FFF)
            } else {
                let value = *body.get(at)?;
                at += 1;
                usize::from(value & 0x7F)
            };
            indices.push(index);
        }
        if id == item {
            return Some(indices);
        }
    }
    Some(Vec::new())
}

/// `colr`: an ICC profile (`prof`, `rICC`) or coded primaries (`nclx`).
fn color(body: &[u8]) -> Option<Color> {
    match body.get(..4)? {
        b"prof" | b"rICC" => Some(Color::Icc(body.get(4..)?.to_vec())),
        b"nclx" => Some(Color::Primaries(u16::from_be_bytes(
            body.get(4..6)?.try_into().ok()?,
        ))),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Builds a box from its type and body.
    pub fn bx(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut out = ((body.len() + 8) as u32).to_be_bytes().to_vec();
        out.extend(kind);
        out.extend(body);
        out
    }

    /// A minimal HEIF head: primary item 1 with the given properties.
    pub fn head(properties: &[Vec<u8>]) -> Vec<u8> {
        let ipco = bx(b"ipco", &properties.concat());
        let mut ipma = vec![0, 0, 0, 0, 0, 0, 0, 2];
        // Item 2 (a thumbnail, say) gets property 1 only; item 1 gets them all.
        ipma.extend([0, 2, 1, 0x81]);
        ipma.extend([0, 1, properties.len() as u8]);
        ipma.extend((1..=properties.len() as u8).map(|i| i | 0x80));
        let iprp = bx(b"iprp", &[ipco, bx(b"ipma", &ipma)].concat());
        let pitm = bx(b"pitm", &[0, 0, 0, 0, 0, 1]);
        let meta = bx(b"meta", &[vec![0, 0, 0, 0], pitm, iprp].concat());
        [bx(b"ftyp", b"heic\0\0\0\0mif1heic"), meta].concat()
    }

    #[test]
    fn rotation_mirroring_and_profile_are_read_in_order() {
        let data = head(&[
            bx(b"ispe", &[0; 12]),
            bx(b"colr", &[b"prof".as_slice(), b"icc bytes"].concat()),
            bx(b"irot", &[1]),
            bx(b"imir", &[1]),
        ]);
        let container = read(&data);
        assert_eq!(
            container.transforms,
            [Transform::Rotate(90), Transform::MirrorHorizontal]
        );
        assert_eq!(container.color, Some(Color::Icc(b"icc bytes".to_vec())));
    }

    #[test]
    fn coded_primaries_are_read() {
        let nclx = [b"nclx".as_slice(), &[0, 12, 0, 13, 0, 0, 0x80]].concat();
        let container = read(&head(&[bx(b"colr", &nclx)]));
        assert_eq!(container.color, Some(Color::Primaries(12)));
        assert!(container.transforms.is_empty());
    }

    #[test]
    fn properties_of_other_items_are_ignored() {
        // Property 1 (a rotation) belongs to item 2 as well as item 1; the
        // primary item's own list is what counts.
        let container = read(&head(&[bx(b"irot", &[3])]));
        assert_eq!(container.transforms, [Transform::Rotate(270)]);
    }

    #[test]
    fn damaged_files_read_as_empty() {
        let data = head(&[bx(b"irot", &[1])]);
        for cut in 0..data.len() {
            // Must never panic, whatever is cut off.
            let _ = read(&data[..cut]);
        }
        assert_eq!(read(b"not a heif file"), Container::default());
        let mut lying = data.clone();
        lying[3] = 0xFF; // ftyp claims more bytes than there are
        assert_eq!(read(&lying), Container::default());
    }
}
