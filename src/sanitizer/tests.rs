use crate::{LyricsAccess, SyncedLyrics};

#[test]
fn strip_tags() {
    let data = include_str!("../../assets/example.lrc");
    let parsed = SyncedLyrics::parse(data).unwrap();
    assert_eq!(super::strip_tags(data), Ok(parsed.to_unsynced()));

    let data = include_str!("../../assets/example-w-whitespace.lrc");
    let parsed = SyncedLyrics::parse(data).unwrap();
    assert_eq!(super::strip_tags(data), Ok(parsed.to_unsynced()));
}
