//! The receiving end of the envelope: decode, keep only our board, count every drop.

use serde::de::DeserializeOwned;
use wb_core::{BoardId, Time};

use crate::envelope::{Body, Packet, Reason, Reject};
use crate::reject_log::RejectLog;

/// Turns datagrams into bodies for one board, and counts and logs every datagram it drops.
///
/// Discovery bodies are accepted whatever the board: discovery belongs to no board. A board
/// message is accepted only for this inbox's board. The counts are exact and never reset; the
/// log lines go through the `log` crate, thinned out by `RejectLog`: an unknown version at
/// `error` level (usually two different builds), every other reason at `warn`.
#[derive(Debug)]
pub struct Inbox {
    board: BoardId,
    dropped: [u64; Reason::ALL.len()],
    log: RejectLog,
}

impl Inbox {
    /// An inbox for the board `board`, with every count at zero.
    #[must_use]
    pub const fn new(board: BoardId) -> Self {
        Self {
            board,
            dropped: [0; Reason::ALL.len()],
            log: RejectLog::new(),
        }
    }

    /// The body of `datagram`, received at `now`, if it is ours to handle.
    ///
    /// # Errors
    ///
    /// Why it was dropped: any `Packet::decode` reject, or `ForeignBoard` for a board message
    /// that is not for this board. Each drop adds one to its reason's count and may log a line.
    pub fn accept<D: DeserializeOwned, S: DeserializeOwned>(
        &mut self,
        datagram: &[u8],
        now: Time,
    ) -> Result<Body<D, S>, Reject> {
        let result = Packet::decode(datagram).and_then(|packet| match packet.body {
            Body::Board { board, .. } if board != self.board => {
                Err(Reject::ForeignBoard { got: board })
            }
            body => Ok(body),
        });
        if let Err(reject) = &result {
            let count = &mut self.dropped[reject.reason() as usize];
            *count = count.saturating_add(1);
            if let Some(line) = self.log.record(reject, now) {
                match reject {
                    Reject::UnknownVersion { .. } => log::error!("{line}"),
                    _ => log::warn!("{line}"),
                }
            }
        }
        result
    }

    /// How many datagrams were dropped for `reason` so far.
    #[must_use]
    pub const fn dropped(&self, reason: Reason) -> u64 {
        self.dropped[reason as usize]
    }
}
