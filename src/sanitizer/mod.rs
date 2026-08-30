#[cfg(test)]
mod tests;

/// Strips tags from lyrics.
///
/// This function is primarily useful for removing malformed tags from lyrics that couldn't be
/// parsed if you want to avoid showing users the malformed data.
///
/// It's capable of removing those kinds of malformed data from lyrics:
/// - Unknown ID tags
/// - Valid but incorrectly ordered timed tags
///
/// It will fail to handle data that contains syntax errors.
#[cfg_attr(
    feature = "parser",
    doc = r#"

For lyrics correctly parsed from certain input, this function should return the same result as
[`SyncedLyrics::to_unsynced`](struct.SyncedLyrics.html#impl-LyricsAccess-for-SyncedLyrics) for that
input. The difference is that to unsync [`SyncedLyrics`](crate::SyncedLyrics), the data has to be
parsed first.
# Examples
For valid lyrics, the result should be the same as unsyncing the parsed input:
```
# use lrc_rs::{SyncedLyrics, LyricsAccess, strip_tags};
let data = "[00:05.20]Hello\n\
[00:11.70]World\n\
[00:17.10]La <00:17.80>la <00:18.50> la";
let parsed = SyncedLyrics::parse(data).unwrap();
assert_eq!(parsed.to_unsynced(), strip_tags(data).unwrap());
```

Though the function is most effective for removing tags from malformed data:
```
# use std::time::Duration;
# use lrc_rs::{Error, SyncedLyrics, LyricsAccess, strip_tags, TimestampConstraint, TimestampError};
let data = "[00:03.10]Line 1\n\
[00:02.50]Line 2\n\
[00:05.80]Line 3";

// Malformed data can't be parsed due to an invalid timestamp order:
assert_eq!(
    SyncedLyrics::parse(data),
    Err(Error::Timestamp(TimestampError {
        line: Some(1),
        segment: None,
        expected: TimestampConstraint::GreaterThan(Duration::from_secs_f32(3.10)),
        actual: Duration::from_secs_f32(2.5),
    }))
);

// But we can still strip the malformed tags so that the lyrics look decent:
let expected = String::from("Line 1\n\
Line 2\n\
Line 3");
assert_eq!(
    strip_tags(data),
    Ok(expected)
);
```"#
)]
pub fn strip_tags(i: &str) -> Result<String, crate::Error> {
    let lines = match crate::parser::parse(i) {
        Ok(i) => i,
        Err(e) => {
            return Err(crate::Error::Nom {
                input: e.input.to_owned(),
                error: e.code.into(),
            });
        }
    };
    let stripped_lines: Vec<_> = lines
        .into_iter()
        .map(|l| match l {
            crate::parser::Line::ID(_) => None,
            crate::parser::Line::Comment(_) => None,
            crate::parser::Line::Tag(tag) => {
                let segments: Vec<_> = tag.segments.into_iter().map(|s| s.content).collect();
                Some(segments.join(""))
            }
        })
        .flatten()
        .collect();

    Ok(stripped_lines.join("\n"))
}
