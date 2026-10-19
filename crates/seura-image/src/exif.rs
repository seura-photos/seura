//! The two EXIF fields thumbnails and the index need: orientation and the
//! date taken.

use exif::{In, Reader, Tag, Value};

/// What the photo's EXIF block says, if it has one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExifInfo {
    /// 1 to 8, as defined by EXIF; 1 (or absent) means upright.
    pub orientation: u8,
    /// `DateTimeOriginal`, as seconds since 1970 on the photo's own clock.
    /// EXIF records local time without a time zone, so this is for sorting
    /// and display, not for converting between zones.
    pub taken: Option<i64>,
}

/// Reads a raw EXIF block, starting at its TIFF header. Anything unreadable
/// is treated as absent: a photo with damaged metadata still shows.
pub fn read(raw: &[u8]) -> ExifInfo {
    let Ok(exif) = Reader::new().read_raw(raw.to_vec()) else {
        return ExifInfo {
            orientation: 1,
            taken: None,
        };
    };
    let orientation = exif
        .get_field(Tag::Orientation, In::PRIMARY)
        .and_then(|field| field.value.get_uint(0))
        .filter(|value| (1..=8).contains(value))
        .map_or(1, |value| value as u8);
    let taken = [Tag::DateTimeOriginal, Tag::DateTime]
        .iter()
        .find_map(|tag| match &exif.get_field(*tag, In::PRIMARY)?.value {
            Value::Ascii(parts) => parse_date(parts.first()?),
            _ => None,
        });
    ExifInfo { orientation, taken }
}

/// `2026:10:18 19:40:12` → seconds since 1970, treating the time as UTC.
fn parse_date(text: &[u8]) -> Option<i64> {
    let text = std::str::from_utf8(text).ok()?.trim_end_matches('\0');
    let number =
        |range: std::ops::Range<usize>| -> Option<i64> { text.get(range)?.trim().parse().ok() };
    let (year, month, day) = (number(0..4)?, number(5..7)?, number(8..10)?);
    let (hour, minute, second) = (number(11..13)?, number(14..16)?, number(17..19)?);
    let valid = (1..=12).contains(&month)
        && (1..=31).contains(&day)
        && hour < 24
        && minute < 60
        && second < 61
        && year > 1800;
    valid.then(|| days_from_civil(year, month, day) * 86_400 + hour * 3600 + minute * 60 + second)
}

/// Days since 1970-01-01 for a date in the proleptic Gregorian calendar
/// (Howard Hinnant's algorithm).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month_index = (month + 9) % 12;
    let day_of_year = (153 * month_index + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_convert_to_seconds() {
        assert_eq!(parse_date(b"1970:01:01 00:00:00"), Some(0));
        assert_eq!(parse_date(b"2026:10:18 19:40:12\0"), Some(1_792_352_412));
        assert_eq!(parse_date(b"2000:02:29 12:00:00"), Some(951_825_600));
        // Cameras with an unset clock write zeros or blanks.
        assert_eq!(parse_date(b"0000:00:00 00:00:00"), None);
        assert_eq!(parse_date(b"    :  :     :  :  "), None);
    }

    #[test]
    fn damaged_exif_reads_as_upright_and_undated() {
        assert_eq!(
            read(b"not exif"),
            ExifInfo {
                orientation: 1,
                taken: None
            }
        );
    }
}
