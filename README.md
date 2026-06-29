# lrc_rs

Robust crate for working with synced lyrics content in the LRC format.

It uses the [`nom`]() crate for all the parsing, and has support for the A2 extension (enhanced
LRC).

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
# #[cfg(feature = "parser")] {
# use lrc_rs::SyncedLyrics;
use lrc_rs::LyricsAccess;
# let data = "[ti:Example content]
# [00:02.30] Some example lyrics
# [00:07.10] La la la la la
# [00:11.80] Hello";
# let lyrics = SyncedLyrics::parse(data).unwrap();
assert_eq!(lyrics.to_unsynced(),
"Some example lyrics
La la la la la
Hello");
# }
```

Create synced lyrics
```ignore
TBD
```

Access lyrics data at a specific timestamp
```ignore
TBD
```
