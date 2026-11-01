//! The receiving end of the envelope: decode, keep only our board, count every drop.

use serde::de::DeserializeOwned;
use wb_core::BoardId;

use crate::envelope::{Body, Packet, Reason, Reject};

/// Turns datagrams into bodies for one board and counts every datagram it drops, per reason.
///
/// Discovery bodies are accepted whatever the board: discovery belongs to no board. A board
/// message is accepted only for this inbox's board. The counts are exact and never reset.
#[derive(Debug)]
pub struct Inbox {
    board: BoardId,
    dropped: [u64; Reason::ALL.len()],
}

impl Inbox {
    /// An inbox for the board `board`, with every count at zero.
    #[must_use]
    pub const fn new(board: BoardId) -> Self {
        Self {
            board,
            dropped: [0; Reason::ALL.len()],
        }
    }

    /// The body of `datagram` if it is ours to handle.
    ///
    /// # Errors
    ///
    /// Why it was dropped: any `Packet::decode` reject, or `ForeignBoard` for a board message
    /// that is not for this board. Each drop adds one to its reason's count.
    pub fn accept<D: DeserializeOwned, S: DeserializeOwned>(
        &mut self,
        datagram: &[u8],
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
        }
        result
    }

    /// How many datagrams were dropped for `reason` so far.
    #[must_use]
    pub const fn dropped(&self, reason: Reason) -> u64 {
        self.dropped[reason as usize]
    }
}
