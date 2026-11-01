//! The datagram envelope: round trip, rejects in order, and the size limit.

use proptest::prelude::*;
use wb_net::config::{MAGIC, MAX_DATAGRAM, VERSION};
use wb_net::{Body, EncodeError, Reject};

use crate::support::config::MAX_RANDOM;
use crate::support::envelope::{TestPacket, packet};

/// A small valid datagram to corrupt one byte at a time.
fn valid() -> Vec<u8> {
    TestPacket::new(Body::Discovery(vec![1, 2, 3]))
        .encode()
        .unwrap_or_default()
}

proptest! {
    #[test]
    fn round_trip(packet in packet()) {
        let bytes = packet.encode();
        prop_assert!(bytes.is_ok(), "{bytes:?}");
        let bytes = bytes.unwrap_or_default();
        prop_assert_eq!(&bytes[..MAGIC.len()], &MAGIC[..]);
        prop_assert_eq!(bytes[MAGIC.len()], VERSION);
        prop_assert_eq!(TestPacket::decode(&bytes), Ok(packet));
    }

    /// Garbage is rejected, never a panic, with or without a valid header in front.
    #[test]
    fn random_bytes_never_panic(bytes in prop::collection::vec(any::<u8>(), 0..=MAX_RANDOM)) {
        let _ = TestPacket::decode(&bytes);
        let headed = [&MAGIC[..], &[VERSION], &bytes].concat();
        let _ = TestPacket::decode(&headed);
    }
}

#[test]
fn rejects_wrong_magic() {
    let mut bytes = valid();
    bytes[0] ^= 1;
    assert_eq!(TestPacket::decode(&bytes), Err(Reject::WrongMagic));
    assert_eq!(TestPacket::decode(&[]), Err(Reject::WrongMagic), "empty");
    // Magic is checked first: a bad version behind a bad magic still reports the magic.
    bytes[MAGIC.len()] = VERSION + 1;
    assert_eq!(
        TestPacket::decode(&bytes),
        Err(Reject::WrongMagic),
        "both wrong"
    );
}

#[test]
fn rejects_unknown_version_and_names_it() {
    let mut bytes = valid();
    let got = VERSION + 1;
    bytes[MAGIC.len()] = got;
    assert_eq!(
        TestPacket::decode(&bytes),
        Err(Reject::UnknownVersion { got })
    );
}

#[test]
fn rejects_body_that_is_not_exactly_one_body() {
    let bytes = valid();
    let cases = [
        ("truncated", bytes[..bytes.len() - 1].to_vec()),
        ("trailing byte", [&bytes[..], &[0]].concat()),
        ("magic only", MAGIC.to_vec()),
        ("header only", [&MAGIC[..], &[VERSION]].concat()),
    ];
    for (case, bytes) in cases {
        assert_eq!(
            TestPacket::decode(&bytes),
            Err(Reject::Undecodable),
            "{case}"
        );
    }
}

/// Wire size of `Discovery(payload)` with a payload of 128 to 16383 bytes: magic + version,
/// 1 byte of variant tag, 2 bytes of length varint, then the payload.
const OVERHEAD: usize = MAGIC.len() + 1 + 1 + 2;

#[test]
fn encodes_up_to_max_datagram_and_not_one_byte_more() {
    let sized = |len: usize| TestPacket::new(Body::Discovery(vec![0; len])).encode();
    let fits = sized(MAX_DATAGRAM - OVERHEAD).map(|bytes| bytes.len());
    assert_eq!(fits, Ok(MAX_DATAGRAM));
    assert_eq!(
        sized(MAX_DATAGRAM - OVERHEAD + 1),
        Err(EncodeError::TooBig {
            len: MAX_DATAGRAM + 1
        })
    );
}
