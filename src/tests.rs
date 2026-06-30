use std::{sync::LazyLock, time::Duration};

use crate::{Error, LRCTool, LineTag, LyricsAccess, SegmentTag, SyncedLyrics};

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
        let parsed = SyncedLyrics::parse(input).unwrap();
        let parsed_twice = SyncedLyrics::parse(&parsed.clone().serialize())
            .unwrap()
            .serialize();
        assert_eq!(parsed.serialize(), parsed_twice);
    }
}

#[test]
fn line_tag_check_timestamp_order() {
    assert!(LineTag::default().check_timestamp_order().is_ok());
    assert!(
        LineTag {
            timestamp: Duration::from_secs(3),
            segments: vec![SegmentTag {
                timestamp: Duration::from_secs(3),
                content: String::new()
            }]
        }
        .check_timestamp_order()
        .is_ok()
    );
    assert!(
        LineTag {
            timestamp: Duration::from_secs(3),
            segments: vec![SegmentTag {
                timestamp: Duration::from_secs(4),
                content: String::new()
            }]
        }
        .check_timestamp_order()
        .is_ok()
    );
    assert!(
        LineTag {
            timestamp: Duration::from_secs(3),
            segments: vec![
                SegmentTag {
                    timestamp: Duration::from_secs(3),
                    content: String::new()
                },
                SegmentTag {
                    timestamp: Duration::from_secs(5),
                    content: String::new()
                }
            ]
        }
        .check_timestamp_order()
        .is_ok()
    );
    assert_eq!(
        LineTag {
            timestamp: Duration::from_secs(3),
            segments: vec![
                SegmentTag {
                    timestamp: Duration::from_secs(4),
                    content: String::new()
                },
                SegmentTag {
                    timestamp: Duration::from_secs(4),
                    content: String::new()
                }
            ]
        }
        .check_timestamp_order(),
        Err(Error::InvalidTimestampOrder {
            index: 1,
            message: format!(
                "Expected a timestamp later than {:?}",
                Duration::from_secs(4)
            )
        })
    );
}

#[test]
fn synced_lyrics_check_timestamp_order() {
    assert!(SyncedLyrics::default().check_timestamp_order().is_ok());
    assert!(
        SyncedLyrics::new(vec![LineTag::default()])
            .check_timestamp_order()
            .is_ok()
    );
    assert!(
        SyncedLyrics::new(vec![
            LineTag {
                timestamp: Duration::from_secs(5),
                segments: Vec::new()
            },
            LineTag {
                timestamp: Duration::from_secs(6),
                segments: Vec::new()
            }
        ])
        .check_timestamp_order()
        .is_ok()
    );
    assert!(
        SyncedLyrics::new(vec![
            LineTag {
                timestamp: Duration::from_secs(5),
                segments: vec![
                    SegmentTag {
                        timestamp: Duration::from_secs(6),
                        content: String::new()
                    },
                    SegmentTag {
                        timestamp: Duration::from_secs(7),
                        content: String::new()
                    }
                ]
            },
            LineTag {
                timestamp: Duration::from_secs(8),
                segments: vec![
                    SegmentTag {
                        timestamp: Duration::from_secs(8),
                        content: String::new()
                    },
                    SegmentTag {
                        timestamp: Duration::from_secs(9),
                        content: String::new()
                    }
                ]
            },
            LineTag {
                timestamp: Duration::from_secs(10),
                segments: Vec::new()
            }
        ])
        .check_timestamp_order()
        .is_ok()
    );
    assert_eq!(
        SyncedLyrics::new(vec![
            LineTag {
                timestamp: Duration::from_secs(5),
                segments: vec![SegmentTag {
                    timestamp: Duration::from_secs(6),
                    content: String::new()
                }]
            },
            LineTag {
                timestamp: Duration::from_secs(6),
                segments: Vec::new()
            }
        ])
        .check_timestamp_order(),
        Err(Error::InvalidTimestampOrder {
            index: 1,
            message: format!(
                "Expected a timestamp later than {:?}",
                Duration::from_secs(6)
            )
        })
    );
}

#[test]
#[cfg(feature = "parser")]
fn parse_invalid_timestamp_order_fail() {
    assert!(SyncedLyrics::parse("[00:02.10] a\n[00:02.20] b").is_ok());
    assert_eq!(
        SyncedLyrics::parse("[00:02.00] a\n[00:02.00] a"),
        Err(Error::InvalidTimestampOrder {
            index: 1,
            message: format!(
                "Expected a timestamp later than {:?}",
                Duration::from_secs(2)
            )
        })
    );
    assert_eq!(
        SyncedLyrics::parse("[00:02.00] a\n[00:01.00] b"),
        Err(Error::InvalidTimestampOrder {
            index: 1,
            message: format!(
                "Expected a timestamp later than {:?}",
                Duration::from_secs(2)
            )
        })
    );
    assert_eq!(
        SyncedLyrics::parse("[00:02.00] <00:02.30> a <00:02.20> b \n[00:02.90] c"),
        Err(Error::InvalidTimestampOrder {
            index: 0,
            message: format!(
                "Invalid tag timestamp at index 1: Expected a timestamp later than {:?}",
                Duration::from_secs_f32(2.3)
            )
        })
    );
}

#[test]
fn line_tag_add_segment() {
    let mut line = LineTag::default();
    line.timestamp(Duration::from_secs_f32(0.5));
    let segment = SegmentTag {
        timestamp: Duration::from_secs(1),
        content: String::new(),
    };
    assert!(line.segment(segment.clone()).is_ok());
    assert_eq!(
        line,
        LineTag {
            timestamp: Duration::from_secs_f32(0.5),
            segments: vec![segment]
        }
    );

    let mut line = LineTag::default();
    line.timestamp(Duration::from_secs(1));
    let segment = SegmentTag {
        timestamp: Duration::from_secs(1),
        content: String::new(),
    };
    assert!(line.segment(segment.clone()).is_ok(),);
    assert_eq!(
        line,
        LineTag {
            timestamp: Duration::from_secs(1),
            segments: vec![segment]
        }
    );

    let mut line = LineTag::default();
    line.timestamp(Duration::from_secs(1));
    let segment = SegmentTag {
        timestamp: Duration::from_secs_f32(0.5),
        content: String::new(),
    };
    assert_eq!(
        line.segment(segment),
        Err(Error::InvalidTimestampOrder {
            index: 0,
            message: format!(
                "Expected a timestamp later than or equal to {:?}",
                Duration::from_secs(1)
            )
        })
    );
    assert_eq!(
        line,
        LineTag {
            timestamp: Duration::from_secs(1),
            segments: Vec::new()
        }
    );
}

#[test]
fn line_tag_add_segments() {
    let mut line = LineTag::default();
    let segments = &[SegmentTag::default()];
    assert!(line.segments(segments).is_ok());
    assert_eq!(
        line,
        LineTag {
            timestamp: Duration::default(),
            segments: vec![SegmentTag::default()]
        }
    );

    let mut line = LineTag::default();
    let segments = &[SegmentTag {
        timestamp: Duration::from_secs(1),
        content: String::new(),
    }];
    assert!(line.segments(segments).is_ok());
    assert_eq!(
        line,
        LineTag {
            timestamp: Duration::default(),
            segments: vec![SegmentTag {
                timestamp: Duration::from_secs(1),
                content: String::new()
            }]
        }
    );

    let mut line = LineTag::default();
    line.timestamp(Duration::from_secs(1));
    let segments = &[SegmentTag::default()];
    assert_eq!(
        line.segments(segments),
        Err(Error::InvalidTimestampOrder {
            index: 0,
            message: format!(
                "Expected a timestamp later than or equal to {:?}",
                Duration::from_secs(1)
            )
        })
    );
    assert_eq!(
        line,
        LineTag {
            timestamp: Duration::from_secs(1),
            segments: Vec::new()
        }
    );

    let mut line = LineTag::default();
    let segments = &[
        SegmentTag::default(),
        SegmentTag {
            timestamp: Duration::from_secs(1),
            content: String::new(),
        },
    ];
    assert!(line.segments(segments).is_ok());
    assert_eq!(
        line,
        LineTag {
            timestamp: Duration::default(),
            segments: segments.to_vec(),
        }
    );

    let mut line = LineTag::default();
    let segments = &[
        SegmentTag {
            timestamp: Duration::from_secs(1),
            content: String::new(),
        },
        SegmentTag {
            timestamp: Duration::from_secs(2),
            content: String::new(),
        },
    ];
    assert!(line.segments(segments).is_ok());
    assert_eq!(
        line,
        LineTag {
            timestamp: Duration::default(),
            segments: segments.to_vec(),
        }
    );

    let mut line = LineTag::default();
    line.timestamp(Duration::from_secs(1));
    assert_eq!(
        line.segments(&[
            SegmentTag::default(),
            SegmentTag {
                timestamp: Duration::from_secs(1),
                content: String::new()
            }
        ]),
        Err(Error::InvalidTimestampOrder {
            index: 0,
            message: format!(
                "Expected a timestamp later than or equal to {:?}",
                Duration::from_secs(1)
            )
        })
    );

    let mut line = LineTag::default();
    line.timestamp(Duration::from_secs(1));
    assert_eq!(
        line.segments(&[
            SegmentTag {
                timestamp: Duration::from_secs(1),
                content: String::new()
            },
            SegmentTag::default(),
        ]),
        Err(Error::InvalidTimestampOrder {
            index: 1,
            message: format!(
                "Expected a timestamp later than {:?}",
                Duration::from_secs(1)
            )
        })
    );

    let mut line = LineTag::default();
    line.timestamp(Duration::from_secs(1));
    assert_eq!(
        line.segments(&[
            SegmentTag {
                timestamp: Duration::from_secs(1),
                content: String::new()
            },
            SegmentTag {
                timestamp: Duration::from_secs(1),
                content: String::new()
            },
        ]),
        Err(Error::InvalidTimestampOrder {
            index: 1,
            message: format!(
                "Expected a timestamp later than {:?}",
                Duration::from_secs(1)
            )
        })
    );
}

#[test]
fn synced_lyrics_add_line() {
    let mut lyrics = SyncedLyrics::default();
    let mut line = LineTag::default();
    line.segments(&[
        SegmentTag::default(),
        SegmentTag {
            timestamp: Duration::from_secs(1),
            content: String::new(),
        },
    ])
    .unwrap();

    let expected = SyncedLyrics {
        lines: vec![line.clone()],
        ..Default::default()
    };

    lyrics.line(line.clone()).unwrap();
    assert_eq!(lyrics, expected);

    let line1 = LineTag {
        timestamp: Duration::from_secs(2),
        segments: vec![
            SegmentTag {
                timestamp: Duration::from_secs(2),
                content: String::new(),
            },
            SegmentTag {
                timestamp: Duration::from_secs(2),
                content: String::new(),
            },
        ],
    };
    assert_eq!(
        lyrics.line(line1),
        Err(Error::InvalidTimestampOrder {
            index: 0,
            message: format!(
                "Invalid tag timestamp at index 1: Expected a timestamp later than {:?}",
                Duration::from_secs(2)
            )
        })
    );
    assert_eq!(lyrics, expected);

    assert_eq!(
        lyrics.line(LineTag::default()),
        Err(Error::InvalidTimestampOrder {
            index: 0,
            message: format!(
                "Expected a timestamp later than {:?}",
                Duration::from_secs(1)
            )
        })
    );
    assert_eq!(lyrics, expected);
}

#[test]
fn synced_lyrics_add_lines() {
    let mut lyrics = SyncedLyrics::default();
    let lines = &[LineTag {
        timestamp: Duration::default(),
        segments: vec![SegmentTag::default()],
    }];
    lyrics.lines(lines).unwrap();
    assert_eq!(
        lyrics,
        SyncedLyrics {
            lines: lines.to_vec(),
            ..Default::default()
        }
    );
    let expected = lyrics.clone();

    assert_eq!(
        lyrics.lines(&[LineTag {
            timestamp: Duration::default(),
            segments: Vec::new(),
        }]),
        Err(Error::InvalidTimestampOrder {
            index: 0,
            message: format!("Expected a timestamp later than {:?}", Duration::default())
        })
    );
    assert_eq!(lyrics, expected);

    assert_eq!(
        lyrics.lines(&[LineTag {
            timestamp: Duration::from_secs(1),
            segments: vec![SegmentTag::default()]
        }]),
        Err(Error::InvalidTimestampOrder {
            index: 0,
            message: format!(
                "Invalid tag timestamp at index 0: Expected a timestamp later than or equal to {:?}",
                Duration::from_secs(1)
            )
        })
    );
    assert_eq!(lyrics, expected);

    assert_eq!(
        lyrics.lines(&[LineTag {
            timestamp: Duration::from_secs(1),
            segments: vec![
                SegmentTag {
                    timestamp: Duration::from_secs(1),
                    content: String::new()
                },
                SegmentTag {
                    timestamp: Duration::from_secs(1),
                    content: String::new()
                }
            ]
        }]),
        Err(Error::InvalidTimestampOrder {
            index: 0,
            message: format!(
                "Invalid tag timestamp at index 1: Expected a timestamp later than {:?}",
                Duration::from_secs(1)
            )
        })
    );
    assert_eq!(lyrics, expected);

    assert_eq!(
        lyrics.lines(&[
            LineTag {
                timestamp: Duration::from_secs(1),
                segments: vec![SegmentTag {
                    timestamp: Duration::from_secs(1),
                    content: String::new()
                },]
            },
            LineTag {
                timestamp: Duration::from_secs(1),
                segments: Vec::new()
            }
        ]),
        Err(Error::InvalidTimestampOrder {
            index: 1,
            message: format!(
                "Expected a timestamp later than {:?}",
                Duration::from_secs(1)
            )
        })
    );
}

#[test]
fn line_tag_lyrics_at() {
    let line = LineTag {
        timestamp: Duration::from_secs(1),
        segments: vec![
            SegmentTag {
                timestamp: Duration::from_secs(2),
                content: String::new(),
            },
            SegmentTag {
                timestamp: Duration::from_secs(3),
                content: String::new(),
            },
        ],
    };
    assert_eq!(line.active_tag(Duration::default()), None);
    assert_eq!(line.active_tag(Duration::from_secs(1)), None);
    assert_eq!(line.active_tag(Duration::from_secs_f32(1.5)), None);
    assert_eq!(line.active_tag(Duration::from_secs(2)), Some(0));
    assert_eq!(line.active_tag(Duration::from_secs_f32(2.5)), Some(0));
    assert_eq!(line.active_tag(Duration::from_secs(3)), Some(1));
    assert_eq!(line.active_tag(Duration::from_secs(u64::MAX)), Some(1));

    let line = LineTag {
        timestamp: Duration::from_secs(1),
        segments: vec![SegmentTag {
            timestamp: Duration::from_secs(1),
            content: String::new(),
        }],
    };
    assert_eq!(line.active_tag(Duration::default()), None);
    assert_eq!(line.active_tag(Duration::from_secs(1)), Some(0));
}

#[test]
fn synced_lyrics_lyrics_at() {
    let lyrics = SyncedLyrics::new(vec![
        LineTag::new(Duration::from_secs(1), String::new()),
        LineTag::new(Duration::from_secs(3), String::new()),
        LineTag::new(Duration::from_secs(7), String::new()),
    ]);
    assert_eq!(lyrics.active_tag(Duration::default()), None);
    assert_eq!(lyrics.active_tag(Duration::from_secs_f32(0.5)), None);
    assert_eq!(lyrics.active_tag(Duration::from_secs(1)), Some(0));
    assert_eq!(lyrics.active_tag(Duration::from_secs(2)), Some(0));
    assert_eq!(lyrics.active_tag(Duration::from_secs(3)), Some(1));
    assert_eq!(lyrics.active_tag(Duration::from_secs(6)), Some(1));
    assert_eq!(lyrics.active_tag(Duration::from_secs(7)), Some(2));
    assert_eq!(lyrics.active_tag(Duration::from_secs(u64::MAX)), Some(2));

    let lyrics = SyncedLyrics::new(vec![
        LineTag::new(Duration::from_secs_f32(1.2), String::new()),
        LineTag {
            timestamp: Duration::from_secs(2),
            segments: Vec::new(),
        },
        LineTag::new(Duration::from_secs_f32(4.1), String::new()),
    ]);

    assert_eq!(lyrics.active_tag(Duration::from_secs(3)), Some(1));
}
