#![doc = include_str!("../README.md")]
#![feature(doc_cfg)]
#[cfg(feature = "parser")]
mod parser;
#[cfg(test)]
mod tests;

use std::time::Duration;

#[cfg(feature = "parser")]
use nom::Finish;
#[cfg(feature = "parser")]
pub use nom::error::ErrorKind;

#[cfg(feature = "log")]
use log::warn;

#[cfg(feature = "parser")]
fn duration_offset<'a>(dur: Duration, offset_ms: i64) -> Result<Duration, Error> {
    match offset_ms.try_into() {
        Ok(offset) => match dur.checked_add(Duration::from_millis(offset)) {
            Some(dur) => Ok(dur),
            None => Err(Error::TimestampOffsetOverflow),
        },
        Err(_) => match dur.checked_sub(Duration::from_millis(offset_ms.unsigned_abs())) {
            Some(dur) => Ok(dur),
            None => Err(Error::TimestampOffsetOverflow),
        },
    }
}

// Ensures that the number of digits in a fraction is always 2, so for example:
// 139 -> 14
// 1 -> 10
fn normalize_fraction(frac: u32) -> u32 {
    let digits = frac.checked_ilog10().unwrap_or(0) + 1;
    match digits {
        1 => frac * 10,
        2 => frac,
        _ => (frac as f64 / 10u32.pow(digits - 2) as f64).round() as u32,
    }
}

// Formats [`Duration`] as mm:ss:cs for timed tags
fn duration_to_timestamp(dur: Duration) -> String {
    let secs = dur.as_secs_f64();
    let cs = normalize_fraction((dur.subsec_millis() as f64 / 10.0).round() as u32);
    let mins = (secs / 60.0).floor();
    let secs = (secs - mins * 60.0).floor();
    format!("{mins:02}:{secs:02}.{cs:02}")
}

fn duration_to_standard_timestamp<'a>(dur: Duration) -> String {
    format!("[{}]", duration_to_timestamp(dur))
}

fn duration_to_a2_timestamp(dur: Duration) -> String {
    format!("<{}>", duration_to_timestamp(dur))
}

/// Error indicating why lyrics couldn't be parsed as LRC.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Error {
    /// An overflow occurred while offsetting timestamps with the value of the LRC `offset` tag
    /// (eg. `[offset: -100]`).
    ///
    /// Try adjusting the offset value or removing the offset tag entirely.
    #[cfg(feature = "parser")]
    TimestampOffsetOverflow,
    /// Encountered an ID tag with an unknown key.
    ///
    /// Remove the broken tag.
    #[cfg(feature = "parser")]
    UnknownKey {
        /// The key that wasn't recognized by the parser.
        key: String,
    },
    /// Tag timestamps were not ordered correctly.
    InvalidTagOrder {
        /// The index pointing at an element in the input at which the order breaks.
        index: usize,
        /// Message specifying the expected value.
        message: String,
    },
    /// Parsing failed due to a syntax error.
    #[cfg(feature = "parser")]
    Nom {
        /// The input for which the error occurred.
        input: String,
        /// The error code.
        error: nom::error::ErrorKind,
    },
}

#[cfg(feature = "parser")]
impl<'a> From<nom::error::Error<&str>> for Error {
    fn from(value: nom::error::Error<&str>) -> Self {
        Self::Nom {
            input: value.input.to_string(),
            error: value.code,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "parser")]
            Self::TimestampOffsetOverflow => {
                write!(f, "An overflow occurred while offsetting a timestamp")
            }
            #[cfg(feature = "parser")]
            Self::UnknownKey { key } => write!(f, "Unknown ID tag key: \"{key}\""),
            Self::InvalidTagOrder { index, message } => {
                write!(f, "Invalid tag timestamp at index {index}: {message}")
            }
            #[cfg(feature = "parser")]
            Self::Nom { input, error } => {
                write!(f, "Couldn't parse the lyrics at `{input}`: {error:?}`")
            }
        }
    }
}

/// Accessor trait for synced lyrics.
pub trait LyricsAccess: Sized {
    /// Returns unsynced lyrics without timestamps or additional metadata.
    fn to_unsynced(self) -> String;
    /// Returns the index of the active tag or [`None`] if no tag is active for the given
    /// timestamp.
    ///
    /// **WARNING**: This function might return wrong results if the segment timestamp order is nor
    /// correct.
    fn active_tag(&self, timestamp: Duration) -> Option<usize>;
    /// Checks if timed tag timestamps are ordered correctly.
    ///
    /// The first [segment tag](SegmentTag) may have the same timestamp as its [line](LineTag).
    fn check_timestamp_order<'a>(&'a self) -> Result<(), Error>;
}

/// Segment of lyrics in a [single line](LineTag), associated with a timestamp.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct SegmentTag {
    /// The timestamp at which the segment starts.
    pub timestamp: Duration,
    /// The content of the segment.
    pub content: String,
}

#[cfg(feature = "parser")]
impl<'a> From<parser::TimestampedSegment<'a>> for SegmentTag {
    fn from(value: parser::TimestampedSegment) -> Self {
        Self {
            timestamp: value.timestamp,
            content: value.content.to_string(),
        }
    }
}

impl SegmentTag {
    fn serialize(self, a2_tag: bool) -> String {
        match a2_tag {
            true => duration_to_a2_timestamp(self.timestamp) + " " + &self.content,
            false => self.content,
        }
    }

    /// Checks if the segment is active at the given timestamp.
    pub fn is_active(&self, timestamp: Duration) -> bool {
        self.timestamp <= timestamp
    }

    /// Set the timestamp of the segment.
    pub fn timestamp(&mut self, timestamp: Duration) -> &mut Self {
        self.timestamp = timestamp;
        self
    }

    /// Set the content of the segment.
    pub fn content(&mut self, content: String) -> &mut Self {
        self.content = content;
        self
    }
}

/// A single line in the [synced lyrics](SyncedLyrics).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct LineTag {
    /// The timestamp at which the line starts.
    ///
    /// Can be the same as or earlier than the timestamp of first segment (if the A2 extension is
    /// used).
    pub timestamp: Duration,
    /// Timestamped segments of the line.
    ///
    /// With regular LRC files, this will contain at most one element. If the enhanced LRC format is
    /// used, it may contain more elements.
    pub segments: Vec<SegmentTag>,
}

#[cfg(feature = "parser")]
impl<'a> From<parser::TimestampedTag<'a>> for LineTag {
    fn from(value: parser::TimestampedTag) -> Self {
        Self {
            timestamp: value.timestamp,
            segments: value.segments.into_iter().map(|s| s.into()).collect(),
        }
    }
}

impl LyricsAccess for LineTag {
    fn to_unsynced(self) -> String {
        let segments: Vec<_> = self.segments.into_iter().map(|s| s.content).collect();
        segments.join("")
    }

    fn active_tag(&self, timestamp: Duration) -> Option<usize> {
        if self.segments.is_empty() {
            None
        } else if timestamp < self.timestamp {
            None
        } else {
            for (i, segment) in self.segments.iter().rev().enumerate() {
                if segment.is_active(timestamp) {
                    return Some(self.segments.len() - 1 - i);
                }
            }
            None
        }
    }

    // NOTE: If a line has only one segment, its timestamp doesn't have to be the same as the
    // timestamp of the line. The window between the line timestamp and the timestamp of the first
    // tag is when no segment is active. It does indicate that the A2 extension is active, though.
    fn check_timestamp_order<'a>(&'a self) -> Result<(), Error> {
        if self.segments.is_empty() {
            Ok(())
        } else {
            let mut ts;
            if self.segments[0].timestamp < self.timestamp {
                return Err(Error::InvalidTagOrder {
                    index: 0,
                    message: format!(
                        "Expected a timestamp later than or equal to {:?}",
                        self.timestamp
                    ),
                });
            } else if self.segments.len() == 1 {
                return Ok(());
            } else {
                ts = &self.segments[0].timestamp;
            }
            for (i, segment) in self.segments[1..self.segments.len()].iter().enumerate() {
                if segment.timestamp <= *ts {
                    return Err(Error::InvalidTagOrder {
                        index: i + 1,
                        message: format!("Expected a timestamp later than {ts:?}"),
                    });
                } else {
                    ts = &segment.timestamp;
                }
            }
            Ok(())
        }
    }
}

impl LineTag {
    #[cfg(feature = "parser")]
    fn offset<'a>(&mut self, offset_ms: i64) -> Result<(), Error> {
        self.timestamp = duration_offset(self.timestamp, offset_ms)?;
        for segment in self.segments.iter_mut() {
            segment.timestamp = duration_offset(segment.timestamp, offset_ms)?;
        }
        Ok(())
    }

    fn last_timestamp(&self) -> &Duration {
        match self.segments.last() {
            Some(d) => &d.timestamp,
            None => &self.timestamp,
        }
    }

    fn serialize(self) -> String {
        if self.segments.len() > 1
            || self.segments.len() == 1 && (self.segments[0].timestamp > self.timestamp)
        {
            duration_to_standard_timestamp(self.timestamp)
                + " "
                + &self
                    .segments
                    .into_iter()
                    .map(|s| s.serialize(true))
                    .collect::<Vec<_>>()
                    .join("")
        } else if self.segments.len() == 1 {
            duration_to_standard_timestamp(self.timestamp)
                + " "
                + &self
                    .segments
                    .into_iter()
                    .map(|s| s.serialize(false))
                    .collect::<Vec<_>>()
                    .join("")
        } else {
            duration_to_standard_timestamp(self.timestamp)
        }
    }

    /// Create a new line tag with a single segment with the same timestamp.
    ///
    /// # Examples
    /// ```rust
    /// # use std::time::Duration;
    /// # use lrc_rs::{LineTag, SegmentTag};
    /// assert_eq!(
    ///     LineTag::new(Duration::from_secs(1), "La la la".to_string()),
    ///     // You can save a lot of boilerplate code
    ///     LineTag {
    ///         timestamp: Duration::from_secs(1),
    ///         segments: vec![SegmentTag {
    ///             timestamp: Duration::from_secs(1),
    ///             content: "La la la".to_string()
    ///         }]
    ///     }
    /// );
    /// ```
    pub fn new(timestamp: Duration, content: String) -> Self {
        Self {
            timestamp,
            segments: vec![SegmentTag { timestamp, content }],
        }
    }

    /// Set the timestamp of the line.
    pub fn timestamp(&mut self, timestamp: Duration) -> &mut Self {
        self.timestamp = timestamp;
        self
    }

    /// Add a segment to the line.
    ///
    /// Prefer using this method over manually adding segments to lines as it ensures that the
    /// timestamp order stays correct. If you need to add segments manually, use the
    /// [`check_timestamp_order`](Self::check_timestamp_order) method to verify timestamp order.
    pub fn segment<'a>(&'a mut self, segment: SegmentTag) -> Result<&'a mut Self, Error> {
        self.segments(&[segment])
    }

    /// Add multiple segments to the line.
    ///
    /// Prefer using this method over manually adding segments to lines as it ensures that the
    /// timestamp order stays correct. If you need to add segments manually, use the
    /// [`check_timestamp_order`](Self::check_timestamp_order) method to verify timestamp order.
    pub fn segments<'a>(&'a mut self, segments: &[SegmentTag]) -> Result<&'a mut Self, Error> {
        if segments.is_empty() {
            Ok(self)
        } else if self.segments.is_empty() {
            if segments[0].timestamp < self.timestamp {
                return Err(Error::InvalidTagOrder {
                    index: 0,
                    message: format!(
                        "Expected a timestamp later than or equal to {:?}",
                        self.timestamp
                    ),
                });
            }
            let mut ts = &self.timestamp;
            for (i, segment) in segments[1..segments.len()].iter().enumerate() {
                if segment.timestamp <= *ts {
                    return Err(Error::InvalidTagOrder {
                        index: i + 1,
                        message: format!("Expected a timestamp later than {ts:?}"),
                    });
                } else {
                    ts = &segment.timestamp;
                }
            }
            self.segments.extend_from_slice(segments);
            Ok(self)
        } else {
            let mut ts = self.last_timestamp();
            for (i, segment) in segments.iter().enumerate() {
                if segment.timestamp <= *ts {
                    return Err(Error::InvalidTagOrder {
                        index: i,
                        message: format!("Expected a timestamp later than {ts:?}"),
                    });
                } else {
                    ts = &segment.timestamp;
                }
            }
            self.segments.extend_from_slice(segments);
            Ok(self)
        }
    }
}

/// Info on the player or editor that created the LRC file.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct LRCTool {
    /// Name of the program.
    pub name: String,
    /// Version of the program.
    pub version: Option<String>,
}

impl LRCTool {
    /// Set the name of the program.
    pub fn name(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }

    /// Set the version of the program.
    pub fn version(&mut self, version: Option<String>) -> &mut Self {
        self.version = version;
        self
    }
}

/// Lyrics grouped into timestamped segments with additional metadata.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct SyncedLyrics {
    /// Title of the song.
    pub title: Option<String>,
    /// Artist performing the song.
    pub artist: Option<String>,
    /// Album the song is from.
    pub album: Option<String>,
    /// Author of the song.
    pub author: Option<String>,
    /// Lyricist of the song.
    pub lyricist: Option<String>,
    /// Length of the song.
    pub length: Option<Duration>,
    /// Author of the LRC file (not the song).
    pub file_author: Option<String>,
    /// Info on the player or editor that created the LRC file.
    pub tool: Option<LRCTool>,
    /// Comments found in the lyrics.
    pub comments: Vec<String>,
    /// LRC segments grouped by lines.
    pub lines: Vec<LineTag>,
}

impl LyricsAccess for SyncedLyrics {
    fn to_unsynced(self) -> String {
        let lines: Vec<_> = self.lines.into_iter().map(|l| l.to_unsynced()).collect();
        lines.join("\n")
    }

    fn active_tag(&self, timestamp: Duration) -> Option<usize> {
        if self.lines.is_empty() {
            None
        } else {
            for (i, line) in self.lines.iter().rev().enumerate() {
                if line.timestamp <= timestamp {
                    return Some(self.lines.len() - 1 - i);
                }
            }
            None
        }
    }

    fn check_timestamp_order<'a>(&'a self) -> Result<(), Error> {
        if self.lines.is_empty() {
            return Ok(());
        } else {
            if let Err(e) = self.lines[0].check_timestamp_order() {
                return Err(Error::InvalidTagOrder {
                    index: 0,
                    message: format!("{e}"),
                });
            }
            let mut ts = self.lines[0].last_timestamp();
            for (i, line) in self.lines[1..self.lines.len()].iter().enumerate() {
                if let Err(e) = line.check_timestamp_order() {
                    return Err(Error::InvalidTagOrder {
                        index: i + 1,
                        message: format!("{e}"),
                    });
                } else if line.timestamp <= *ts {
                    return Err(Error::InvalidTagOrder {
                        index: i + 1,
                        message: format!("Expected a timestamp later than {ts:?}"),
                    });
                } else {
                    ts = line.last_timestamp();
                }
            }
            Ok(())
        }
    }
}

impl SyncedLyrics {
    #[cfg(feature = "parser")]
    fn parse_len<'a>(i: &'a str) -> Result<Duration, Error> {
        use nom::{Parser, combinator::eof};

        match (parser::timestamp, eof).parse(i).finish() {
            Ok((_, (len, _))) => Ok(len),
            Err(e) => {
                #[cfg(feature = "log")]
                warn!("Couldn't parse timestamp value: {e}");
                Err(Error::from(e))
            }
        }
    }

    #[cfg(feature = "parser")]
    fn parse_offset<'a>(i: &'a str) -> Result<i64, Error> {
        match parser::offset(i).finish() {
            Ok((_, offset)) => Ok(offset),
            Err(e) => {
                #[cfg(feature = "log")]
                warn!("Couldn't parse offset value: {e}");
                Err(Error::from(e))
            }
        }
    }

    fn serialize_id_tag(key: &str, value: &str, newline: bool) -> String {
        if newline {
            format!("\n[{key}:{value}]")
        } else {
            format!("[{key}:{value}]")
        }
    }

    fn format_duration_mm_ss(dur: Duration) -> String {
        let secs = dur.as_secs();
        let mins = secs / 60;
        let secs = secs - mins * 60;
        format!("{mins}:{secs}")
    }

    // /// Comments found in the lyrics.
    // pub comments: Vec<String>,
    // /// LRC segments grouped by lines.
    // pub lines: Vec<LineTag>,

    /// Set the title of the song.
    pub fn title(&mut self, title: Option<String>) -> &mut Self {
        self.title = title;
        self
    }

    /// Set the artist performing the song.
    pub fn artist(&mut self, artist: Option<String>) -> &mut Self {
        self.artist = artist;
        self
    }

    /// Set the album the song is from.
    pub fn album(&mut self, album: Option<String>) -> &mut Self {
        self.album = album;
        self
    }

    /// Set the author of the song.
    pub fn author(&mut self, author: Option<String>) -> &mut Self {
        self.author = author;
        self
    }

    /// Set the lyricist of the song.
    pub fn lyricist(&mut self, lyricist: Option<String>) -> &mut Self {
        self.lyricist = lyricist;
        self
    }

    /// Set the length of the song.
    pub fn length(&mut self, length: Option<Duration>) -> &mut Self {
        self.length = length;
        self
    }

    /// Set the author of the LRC file (not the song).
    pub fn file_author(&mut self, file_author: Option<String>) -> &mut Self {
        self.file_author = file_author;
        self
    }

    /// Set the info on the player or editor that created the LRC file.
    pub fn tool(&mut self, tool: Option<LRCTool>) -> &mut Self {
        self.tool = tool;
        self
    }

    /// Add a comment to the lyrics.
    ///
    /// **NOTE**: In serialized content, comments will be put between ID tags and timed tags.
    pub fn comment(&mut self, comment: String) -> &mut Self {
        self.comments.push(comment);
        self
    }

    /// Add multiple comments to the lyrics.
    ///
    /// **NOTE**: In serialized content, comments will be put between ID tags and timed tags.
    pub fn comments(&mut self, comments: &[String]) -> &mut Self {
        self.comments.extend_from_slice(comments);
        self
    }

    /// Add a line to the lyrics.
    ///
    /// Prefer using this method over manually adding lines to lyrics as it ensures that the
    /// timestamp order stays correct. If you need to add lines manually, use the
    /// [`check_timestamp_order`](Self::check_timestamp_order) method to verify timestamp order.
    pub fn line(&mut self, line: LineTag) -> Result<&mut Self, Error> {
        self.lines(&[line])
    }

    /// Add multiple lines to the lyrics.
    ///
    /// Prefer using this method over manually adding lines to lyrics as it ensures that the
    /// timestamp order stays correct. If you need to add lines manually, use the
    /// [`check_timestamp_order`](Self::check_timestamp_order) method to verify timestamp order.
    pub fn lines(&mut self, lines: &[LineTag]) -> Result<&mut Self, Error> {
        if lines.is_empty() {
            return Ok(self);
        } else if let Err(e) = lines[0].check_timestamp_order() {
            return Err(Error::InvalidTagOrder {
                index: 0,
                message: format!("{e}"),
            });
        }
        if !self.lines.is_empty() {
            let ts = *self.lines[self.lines.len() - 1].last_timestamp();
            if lines[0].timestamp <= ts {
                return Err(Error::InvalidTagOrder {
                    index: 0,
                    message: format!("Expected a timestamp later than {:?}", ts),
                });
            }
        }
        let mut ts = lines[0].last_timestamp();
        for (i, line) in lines[1..lines.len()].iter().enumerate() {
            if let Err(e) = line.check_timestamp_order() {
                return Err(Error::InvalidTagOrder {
                    index: i + 1,
                    message: format!("{e}"),
                });
            }
            if line.timestamp <= *ts {
                return Err(Error::InvalidTagOrder {
                    index: i + 1,
                    message: format!("Expected a timestamp later than {:?}", ts),
                });
            }
            ts = &line.timestamp;
        }
        self.lines.extend_from_slice(lines);
        Ok(self)
    }

    /// Create an empty synced lyrics struct with some timed tags.
    pub fn new(tags: Vec<LineTag>) -> Self {
        Self {
            title: None,
            artist: None,
            album: None,
            author: None,
            lyricist: None,
            length: None,
            file_author: None,
            tool: None,
            comments: Vec::new(),
            lines: tags,
        }
    }

    /// Checks if the lyrics contain any tags from the A2 extension.
    ///
    /// If the enhanced LRC format is used, [line tags](LineTag) may contain more than one segment,
    /// and the first [segment](SegmentTag) in [line tags](LineTag) may have a timestamp that is
    /// later than the line timestamp.
    pub fn is_enhanced_lrc(&self) -> bool {
        for line in &self.lines {
            if line.segments.is_empty() {
                continue;
            } else if line.segments.len() > 1 || line.timestamp < line.segments[0].timestamp {
                return true;
            }
        }
        false
    }

    /// Serialize the struct to LRC format.
    ///
    /// **NOTE**: This method may not always produce the same output as the input parsed to create
    /// the data structure. For instance, the original placement of ID tags and comments is not
    /// preserved, and the offset tag is omitted as it is applied by the parser.
    #[cfg_attr(
        feature = "parser",
        doc = r#"

It should, however, always produce the same result for input serialized from its output.

For instance:
```rust
# use lrc_rs::SyncedLyrics;
let input = include_str!("../assets/example.lrc");
let parsed = SyncedLyrics::parse(&input).unwrap();
let parsed_twice = SyncedLyrics::parse(&parsed.clone().serialize())
    .unwrap()
    .serialize();
assert_eq!(parsed.serialize(), parsed_twice);
 ```"#
    )]
    pub fn serialize(self) -> String {
        let mut result = String::new();
        if let Some(title) = self.title {
            result += &Self::serialize_id_tag("ti", &title, false);
        }
        if let Some(artist) = self.artist {
            result += &Self::serialize_id_tag("ar", &artist, !result.is_empty());
        }
        if let Some(album) = self.album {
            result += &Self::serialize_id_tag("al", &album, !result.is_empty());
        }
        if let Some(author) = self.author {
            result += &Self::serialize_id_tag("au", &author, !result.is_empty());
        }
        if let Some(lyricist) = self.lyricist {
            result += &Self::serialize_id_tag("lr", &lyricist, !result.is_empty());
        }
        if let Some(length) = self.length {
            result += &Self::serialize_id_tag(
                "length",
                Self::format_duration_mm_ss(length).as_ref(),
                !result.is_empty(),
            );
        }
        if let Some(author) = self.file_author {
            result += &Self::serialize_id_tag("by", &author, !result.is_empty());
        }
        if let Some(tool) = self.tool {
            result += &Self::serialize_id_tag("tool", &tool.name, !result.is_empty());
            if let Some(tool_version) = tool.version {
                result += &Self::serialize_id_tag("ve", &tool_version, true);
            }
        }
        let comments = self
            .comments
            .into_iter()
            .map(|c| Self::serialize_id_tag("#", &c, false))
            .collect::<Vec<_>>()
            .join("\n");
        let lines = self
            .lines
            .into_iter()
            .map(|l| l.serialize())
            .collect::<Vec<_>>()
            .join("\n");
        if !comments.is_empty() {
            if result.is_empty() {
                result += &comments;
            } else {
                result += "\n";
                result += &comments;
            }
        }
        if !lines.is_empty() {
            if result.is_empty() {
                result += &lines;
            } else {
                result += "\n";
                result += &lines;
            }
        }
        result
    }

    /// Parses LRC lyrics data.
    ///
    /// # Examples
    /// Parse some lyrics
    /// ```rust
    /// # use lrc_rs::SyncedLyrics;
    /// let content = "[ti:My awesome song]
    /// [00:02.50] La la la
    /// [00:05.10] <00:05.10> La la la <00:06.30> la la la";
    /// let lyrics = SyncedLyrics::parse(content).unwrap();
    /// ```
    ///
    /// Parser fails to parse content with an invalid ID tag
    /// ```rust
    /// # use lrc_rs::{SyncedLyrics, Error};
    /// let content = "[fun:I am an invalid ID tag]";
    /// assert_eq!(
    ///     SyncedLyrics::parse(content),
    ///     Err(Error::UnknownKey { key: "fun".to_string() })
    /// );
    /// ```
    ///
    /// Parser fails to parse content with invalid timestamp order
    /// ```rust
    /// # use std::time::Duration;
    /// # use lrc_rs::{SyncedLyrics, Error};
    /// let content = "[00:02.10] First line
    /// [00:01.90] My timestamp is wrong!";
    /// assert_eq!(
    ///     SyncedLyrics::parse(content),
    ///     Err(Error::InvalidTagOrder {
    ///         index: 1,
    ///         message: format!("Expected a timestamp later than {:?}", Duration::from_secs_f32(2.1))
    ///     })
    /// );
    /// ```
    #[cfg(feature = "parser")]
    pub fn parse<'a>(input: &'a str) -> Result<Self, Error> {
        match parser::parse(input) {
            Ok(lines) => {
                use crate::parser::Line;

                let id_tags: Vec<_> = lines
                    .iter()
                    .filter_map(|l| if let Line::ID(t) = l { Some(t) } else { None })
                    .collect();
                let comments: Vec<_> = lines
                    .iter()
                    .filter_map(|l| {
                        if let Line::Comment(c) = l {
                            Some(c.to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
                let mut title = None;
                let mut artist = None;
                let mut album = None;
                let mut author = None;
                let mut lyricist = None;
                let mut length = None;
                let mut file_author = None;
                let mut offset = None;
                let mut tool_name = None;
                let mut tool_version = None;
                for tag in id_tags {
                    match tag.key {
                        "ti" => title = Some(tag.value.to_string()),
                        "ar" => artist = Some(tag.value.to_string()),
                        "al" => album = Some(tag.value.to_string()),
                        "au" => author = Some(tag.value.to_string()),
                        "lr" => lyricist = Some(tag.value.to_string()),
                        "length" => {
                            let l = Self::parse_len(tag.value)?;
                            length = Some(l)
                        }
                        "by" => file_author = Some(tag.value.to_string()),
                        "offset" => {
                            let l = Self::parse_offset(tag.value)?;
                            offset = Some(l)
                        }
                        "re" | "tool" => tool_name = Some(tag.value.to_string()),
                        "ve" => tool_version = Some(tag.value.to_string()),
                        _ => {
                            warn!("Unknown ID tag key \"{}\"", tag.key);
                            return Err(Error::UnknownKey {
                                key: tag.key.to_string(),
                            });
                        }
                    }
                }
                let tool = match tool_name {
                    Some(name) => Some(LRCTool {
                        name: name.to_string(),
                        version: tool_version.map(|version| version.to_string()),
                    }),
                    None => {
                        if tool_version.is_some() {
                            #[cfg(feature = "log")]
                            warn!("The tool version is specified but the name isn't, ignoring");
                        }
                        None
                    }
                };
                let lines = if let Some(offset) = offset {
                    let lines: Result<Vec<_>, Error> = lines
                        .into_iter()
                        .filter_map(|l| if let Line::Tag(t) = l { Some(t) } else { None })
                        .map(|t| t.into())
                        .map(|mut t: LineTag| {
                            t.offset(offset)?;
                            Ok(t)
                        })
                        .collect();
                    lines?
                } else {
                    lines
                        .into_iter()
                        .filter_map(|l| if let Line::Tag(t) = l { Some(t) } else { None })
                        .map(|t| t.into())
                        .collect()
                };
                let mut timestamp = None;
                for (i, line) in lines.iter().enumerate() {
                    if let Err(e) = line.check_timestamp_order() {
                        return Err(Error::InvalidTagOrder {
                            index: i,
                            message: format!("{e}"),
                        });
                    }
                    if let Some(ts) = timestamp {
                        if line.last_timestamp() <= ts {
                            return Err(Error::InvalidTagOrder {
                                index: i,
                                message: format!("Expected a timestamp later than {ts:?}"),
                            });
                        }
                    }
                    timestamp = Some(line.last_timestamp())
                }
                Ok(SyncedLyrics {
                    title,
                    artist,
                    album,
                    author,
                    lyricist,
                    length,
                    tool,
                    file_author,
                    comments,
                    lines,
                })
            }
            Err(e) => {
                #[cfg(feature = "log")]
                warn!("Couldn't parse the LRC content: {e}");
                Err(Error::from(e))
            }
        }
    }
}
