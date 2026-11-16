//! Helpers and fixtures for reject log tests.

use wb_core::{BoardId, Time};
use wb_net::config::VERSION;
use wb_net::{Reject, RejectLog};

/// One reject of every reason.
pub const EVERY_REJECT: [Reject; 4] = [
    Reject::WrongMagic,
    Reject::UnknownVersion { got: VERSION + 1 },
    Reject::Undecodable,
    Reject::ForeignBoard {
        got: BoardId([0; 8]),
    },
];

/// Returns the text of the log lines due at `ms`.
pub fn lines_at(log: &mut RejectLog, ms: u64) -> Vec<String> {
    log.tick(Time(ms))
        .into_iter()
        .map(|(_, line)| line)
        .collect()
}

/// Formats one reject log line.
pub fn line(dropped: u64, reject: Reject) -> String {
    format!("dropped {dropped} datagrams, newest: {reject}")
}
