//! The envelope around every datagram: magic, version, then the postcard-encoded body.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wb_core::{BoardId, Op};

use crate::config::{MAGIC, MAX_DATAGRAM, VERSION};

/// One datagram's content, generic over the discovery message `D` and the sync message `S`.
///
/// On the wire it is `MAGIC`, then `VERSION` as one byte, then `body` in postcard. Magic and
/// version are not fields: `encode` always writes this build's values and `decode` accepts no
/// others.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet<D, S> {
    /// What the datagram carries.
    pub body: Body<D, S>,
}

/// What a datagram carries: discovery traffic, or a message for one board.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Body<D, S> {
    /// Discovery traffic; it belongs to no board.
    Discovery(D),
    /// A message for the board `board`; peers on other boards drop it.
    Board {
        /// The board the message is for.
        board: BoardId,
        /// The message.
        msg: BoardMsg<S>,
    },
}

/// A message for one board: a single op, or anti-entropy traffic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoardMsg<S> {
    /// One board op, as broadcast after a local change.
    Op(Op),
    /// Anti-entropy traffic.
    Sync(S),
}

/// Why `Packet::encode` produced no datagram.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum EncodeError {
    /// The datagram would be `len` bytes, over `MAX_DATAGRAM`.
    #[error("datagram of {len} bytes is over the {MAX_DATAGRAM}-byte limit")]
    TooBig {
        /// Size the datagram would have had.
        len: usize,
    },
    /// postcard could not serialize the body.
    #[error("body does not serialize")]
    Serialize,
}

/// Why a datagram was dropped. `Packet::decode` checks the first three in this order, so
/// garbage never reaches postcard; `Inbox` adds `ForeignBoard`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum Reject {
    /// The datagram does not start with `MAGIC`: not our protocol.
    #[error("wrong magic")]
    WrongMagic,
    /// Our magic, but version `got` instead of `VERSION`: usually a different build.
    #[error("unknown version {got}, this build speaks {VERSION}")]
    UnknownVersion {
        /// Version byte the datagram carried.
        got: u8,
    },
    /// Our magic, but no version byte after it, or a body that is not exactly one valid postcard
    /// `Body`: truncated, corrupt, or with bytes left over.
    #[error("undecodable body")]
    Undecodable,
    /// A valid board message for board `got`, not ours.
    #[error("message for a foreign board")]
    ForeignBoard {
        /// Board the message was for.
        got: BoardId,
    },
}

/// A `Reject` without its data, to count drops by kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// See `Reject::WrongMagic`.
    WrongMagic,
    /// See `Reject::UnknownVersion`.
    UnknownVersion,
    /// See `Reject::Undecodable`.
    Undecodable,
    /// See `Reject::ForeignBoard`.
    ForeignBoard,
}

impl Reason {
    /// Every reason, in `Reject` order.
    pub const ALL: [Self; 4] = [
        Self::WrongMagic,
        Self::UnknownVersion,
        Self::Undecodable,
        Self::ForeignBoard,
    ];
}

impl Reject {
    /// The kind of this reject, without its data.
    #[must_use]
    pub const fn reason(&self) -> Reason {
        match self {
            Self::WrongMagic => Reason::WrongMagic,
            Self::UnknownVersion { .. } => Reason::UnknownVersion,
            Self::Undecodable => Reason::Undecodable,
            Self::ForeignBoard { .. } => Reason::ForeignBoard,
        }
    }
}

impl<D, S> Packet<D, S> {
    /// Wraps `body` in a packet.
    #[must_use]
    pub const fn new(body: Body<D, S>) -> Self {
        Self { body }
    }
}

impl<D: Serialize, S: Serialize> Packet<D, S> {
    /// The datagram bytes: header, then the body in postcard.
    ///
    /// # Errors
    ///
    /// `TooBig` if the result is over `MAX_DATAGRAM` bytes; `Serialize` if postcard fails.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut bytes = Vec::with_capacity(MAX_DATAGRAM);
        bytes.extend_from_slice(&MAGIC);
        bytes.push(VERSION);
        let bytes = postcard::to_extend(&self.body, bytes).map_err(|_| EncodeError::Serialize)?;
        if bytes.len() > MAX_DATAGRAM {
            return Err(EncodeError::TooBig { len: bytes.len() });
        }
        Ok(bytes)
    }
}

impl<D: DeserializeOwned, S: DeserializeOwned> Packet<D, S> {
    /// Reads a datagram: magic first, then version, and only then the body.
    ///
    /// Never panics, whatever the bytes.
    ///
    /// # Errors
    ///
    /// The first `Reject` reason that applies, checked in its listed order; never `ForeignBoard`,
    /// since a packet does not know which board the reader is on.
    pub fn decode(datagram: &[u8]) -> Result<Self, Reject> {
        let Some(rest) = datagram.strip_prefix(&MAGIC) else {
            return Err(Reject::WrongMagic);
        };
        let Some((&got, body)) = rest.split_first() else {
            return Err(Reject::Undecodable);
        };
        if got != VERSION {
            return Err(Reject::UnknownVersion { got });
        }
        match postcard::take_from_bytes(body) {
            Ok((body, [])) => Ok(Self::new(body)),
            _ => Err(Reject::Undecodable),
        }
    }
}
