use std::{sync::LazyLock, time::Duration};

use crate::{LRCTool, LineTag, LyricsAccess, SegmentTag, SyncedLyrics};

static PARSED_EXAMPLE: LazyLock<SyncedLyrics> = LazyLock::new(|| SyncedLyrics {
    title: Some("example".to_string()),
    artist: Some("tpaau".to_string()),
    album: Some("lrc_rs".to_string()),
    author: Some("aaa".to_string()),
    lyricist: Some("help".to_string()),
    length: Some(Duration::from_secs_f32(25217.0)),
    file_author: Some("Helix".to_string()),
    tool: Some(LRCTool {
        name: "me1".to_string(),
        version: Some("1.0.0".to_string()),
    }),
    comments: vec!["Hello, this is a comment".to_string()],
    lines: vec![
        LineTag {
            // I can't just do `Duration::from_secs_f32(<timestamp> + <offset>)` due to float drift
            timestamp: Duration::from_secs_f32(12.1) + Duration::from_millis(100),
            segments: vec![SegmentTag {
                timestamp: Duration::from_secs_f32(12.1) + Duration::from_millis(100),
                content: "Hello, this is an example line that will appear at 12.1s".to_string(),
            }],
        },
        LineTag {
            timestamp: Duration::from_secs_f32(16.7) + Duration::from_millis(100),
            segments: vec![SegmentTag {
                timestamp: Duration::from_secs_f32(16.7) + Duration::from_millis(100),
                content: "You can also trim them numbers and it still works".to_string(),
            }],
        },
        LineTag {
            timestamp: Duration::from_secs_f32(22.0) + Duration::from_millis(100),
            segments: vec![
                SegmentTag {
                    timestamp: Duration::from_secs_f32(22.5) + Duration::from_millis(100),
                    content: "Line segments ".to_string(),
                },
                SegmentTag {
                    timestamp: Duration::from_secs_f32(23.9) + Duration::from_millis(100),
                    content: "can also have ".to_string(),
                },
                SegmentTag {
                    timestamp: Duration::from_secs_f32(25.1) + Duration::from_millis(100),
                    content: "timestamps :)".to_string(),
                },
            ],
        },
        LineTag {
            timestamp: Duration::from_secs_f32(28.8) + Duration::from_millis(100),
            segments: Vec::new(),
        },
    ],
});

static EXPECTED_SERIALIZED: LazyLock<SyncedLyrics> = LazyLock::new(|| SyncedLyrics {
    title: Some("example".to_string()),
    artist: Some("tpaau".to_string()),
    album: Some("lrc_rs".to_string()),
    author: Some("aaa".to_string()),
    lyricist: Some("help".to_string()),
    length: Some(Duration::from_secs_f32(25217.0)),
    file_author: Some("Helix".to_string()),
    tool: Some(LRCTool {
        name: "me".to_string(),
        version: Some("1.0.0".to_string()),
    }),
    comments: vec!["Hello, this is a comment".to_string()],
    lines: vec![
        LineTag {
            timestamp: Duration::from_secs_f32(12.1),
            segments: vec![SegmentTag {
                timestamp: Duration::from_secs_f32(12.1),
                content: "Hello, this is an example line that will appear at 12.1s".to_string(),
            }],
        },
        LineTag {
            timestamp: Duration::from_secs_f32(22.0),
            segments: vec![
                SegmentTag {
                    timestamp: Duration::from_secs_f32(22.5),
                    content: "Line segments ".to_string(),
                },
                SegmentTag {
                    timestamp: Duration::from_secs_f32(23.9),
                    content: "can also have ".to_string(),
                },
                SegmentTag {
                    timestamp: Duration::from_secs_f32(25.1),
                    content: "timestamps :)".to_string(),
                },
            ],
        },
        LineTag {
            timestamp: Duration::from_secs_f32(28.8),
            segments: Vec::new(),
        },
    ],
});

#[cfg(feature = "parser")]
#[test]
fn parse() {
    assert_eq!(
        SyncedLyrics::parse(include_str!("../assets/example.lrc")),
        Ok(PARSED_EXAMPLE.clone())
    );
}

#[test]
fn to_unsynced() {
    let unsynced = include_str!("../assets/example-w-out-sync.txt");
    assert_eq!(
        PARSED_EXAMPLE.clone().to_unsynced().to_string(),
        unsynced.to_string()
    );
}

#[test]
fn normalize_fraction() {
    assert_eq!(crate::normalize_fraction(120), 12);
    assert_eq!(crate::normalize_fraction(1), 10);
    assert_eq!(crate::normalize_fraction(149), 15);
    assert_eq!(crate::normalize_fraction(319), 32);
    assert_eq!(crate::normalize_fraction(10), 10);
    assert_eq!(crate::normalize_fraction(31), 31);
    assert_eq!(crate::normalize_fraction(1234), 12);
    assert_eq!(crate::normalize_fraction(0), 0);
}

#[test]
fn duration_to_timestamp() {
    assert_eq!(
        crate::duration_to_timestamp(Duration::from_secs_f64(75.01)),
        "01:15.10".to_string()
    );
    assert_eq!(
        crate::duration_to_timestamp(Duration::from_secs_f64(7220.319)),
        "120:20.32"
    );
}

#[test]
fn serialize_segment_tag() {
    let tag = SegmentTag {
        timestamp: Duration::from_secs_f32(67.69),
        content: "Six Seven".to_string(),
    };
    assert_eq!(tag.clone().serialize(false), "Six Seven".to_string());
    assert_eq!(tag.serialize(true), "<01:07.69> Six Seven".to_string());
}

#[test]
fn serialize_line_tag() {
    assert_eq!(
        LineTag {
            timestamp: Duration::from_secs_f32(420.1337),
            segments: Vec::new()
        }
        .serialize(),
        "[07:00.13]".to_string()
    );
    assert_eq!(
        LineTag {
            timestamp: Duration::from_secs_f32(420.1337),
            segments: vec![SegmentTag {
                timestamp: Duration::from_secs_f32(420.1337),
                content: "Standard line!".to_string()
            }]
        }
        .serialize(),
        "[07:00.13] Standard line!".to_string()
    );
    assert_eq!(
        LineTag {
            timestamp: Duration::from_secs_f32(420.1337),
            segments: vec![SegmentTag {
                timestamp: Duration::from_secs_f32(1984.2137),
                content: "Hello World!".to_string()
            }]
        }
        .serialize(),
        "[07:00.13] <33:04.21> Hello World!".to_string()
    );
    assert_eq!(
        LineTag {
            timestamp: Duration::from_secs_f32(420.1337),
            segments: vec![
                SegmentTag {
                    timestamp: Duration::from_secs_f32(1984.2137),
                    content: "Hello ".to_string()
                },
                SegmentTag {
                    timestamp: Duration::from_secs_f32(2137.1337),
                    content: "World!".to_string()
                }
            ]
        }
        .serialize(),
        "[07:00.13] <33:04.21> Hello <35:37.13> World!".to_string()
    );
}

#[test]
fn format_duration_mm_ss() {
    assert_eq!(
        SyncedLyrics::format_duration_mm_ss(Duration::from_secs(128)),
        "2:8".to_string()
    );
}

#[test]
fn serialize() {
    assert_eq!(
        EXPECTED_SERIALIZED.clone().serialize() + "\n", // include_str! seems to be adding an extra newline
        include_str!("../assets/example-expected-serialized.lrc")
    );

    #[cfg(feature = "parser")]
    {
        let input = include_str!("../assets/example.lrc");
        let parsed = SyncedLyrics::parse(&input).unwrap();
        let parsed_twice = SyncedLyrics::parse(&parsed.clone().serialize())
            .unwrap()
            .serialize();
        assert_eq!(parsed.serialize(), parsed_twice);
    }
}
