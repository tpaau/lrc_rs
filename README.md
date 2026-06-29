# lrc_rs

Robust crate for working with synced lyrics content in the LRC format with support for the A2
extension.

Docs can be found [here](https://tpaau.github.io/lrc_rs/lrc_rs/).

## Features
- Reliable parser built with the [`nom`](https://crates.io/crates/nom) crate
- Coverage of all the ID tags and comment tags
- Support for tags from the A2 extension
- Easy serialization
- Simple conversion of synced lyrics to unsynced lyrics
- Straightforward access to lyrics data with timestamps

## Examples
Parse some LRC content
```rust
# #[cfg(feature = "parser")] {
# use lrc_rs::SyncedLyrics;
let data = "[ti:Example content]
[00:02.30] Some example lyrics
[00:07.10] <00:07.70> La <00:08.20> la <00:09.10> la <00:10.00> la <00:10.90> la
[00:11.80] Hello";
let lyrics = SyncedLyrics::parse(data).unwrap();
# }
```

Convert synced lyrics to unsynced lyrics
```rust
# use std::time::Duration;
# use lrc_rs::{SyncedLyrics, LineTag, SegmentTag};
// Needed for the `to_unsynced` method
use lrc_rs::LyricsAccess;

let lyrics = SyncedLyrics {
    // Metadata is omitted in unsynced lyrics
    title: Some("Example content".to_string()),
    lines: vec![
        LineTag {
            // Line timestamps are also ignored
            timestamp: Duration::from_secs_f32(2.3),
            segments: vec![
                SegmentTag {
                    timestamp: Duration::from_secs_f32(2.3),
                    content: "Some example lyrics".to_string()
                }
            ]
        },
        LineTag {
            timestamp: Duration::from_secs_f32(7.1),
            // Multiple segments will be joined together
            segments: vec![
                SegmentTag {
                    timestamp: Duration::from_secs_f32(7.1),
                    content: "La la ".to_string()
                },
                SegmentTag {
                    timestamp: Duration::from_secs_f32(8.4),
                    content: "la la la".to_string()
                }
            ]
        },
    ],
    ..Default::default()
};
assert_eq!(lyrics.to_unsynced(),
"Some example lyrics
La la la la la");
```

Create synced lyrics
```rust
# use std::time::Duration;
# use lrc_rs::{SyncedLyrics, LineTag, SegmentTag};
let mut lyrics = SyncedLyrics::default();

// Add some metadata
lyrics.title(Some("My awesome song".to_string()));
lyrics.file_author(Some("me!!".to_string()));

// This method checks if the timestamp order is correct, so prefer using it over manually adding tags
lyrics.line(
    LineTag::new(
        Duration::from_secs_f32(5.3),
        "La la la".to_string()
    )
).unwrap();

// And here's an example on when it can fail
assert_eq!(
    lyrics.line(LineTag::default()),
    Err(
        lrc_rs::Error::InvalidTagOrder {
            index: 0,
            message: format!("Expected a timestamp later than {:?}", Duration::from_secs_f32(5.3))
        }
    )
);
```

Access lyrics data at a specific timestamp
```ignore
TBD
```
