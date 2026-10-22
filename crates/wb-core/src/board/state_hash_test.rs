use super::StateHash;
use crate::{OpId, PeerId};

fn id(peer: u64, seq: u64) -> OpId {
    OpId {
        peer: PeerId(peer),
        seq,
    }
}

/// Inserts each id as added, then as tombstoned.
fn hash_of(ids: impl IntoIterator<Item = OpId>) -> StateHash {
    let mut hash = StateHash::default();
    for id in ids {
        hash.insert_added(id);
        hash.insert_tombstoned(id);
    }
    hash
}

#[test]
fn insertion_order_does_not_matter() {
    let ids: Vec<OpId> = (0..50).map(|seq| id(seq % 3, seq)).collect();
    assert_eq!(hash_of(ids.iter().copied()), hash_of(ids.into_iter().rev()));
}

#[test]
fn added_and_tombstoned_ids_hash_apart() {
    let (mut added, mut tombstoned) = (StateHash::default(), StateHash::default());
    added.insert_added(id(1, 2));
    tombstoned.insert_tombstoned(id(1, 2));
    assert_ne!(added, tombstoned);
}

#[test]
fn ids_differing_in_one_field_hash_apart() {
    let base = hash_of([id(1, 2)]);
    assert_ne!(base, hash_of([id(9, 2)]), "peer");
    assert_ne!(base, hash_of([id(1, 9)]), "seq");
    assert_ne!(base, hash_of([id(2, 1)]), "fields swapped");
}

#[test]
fn every_insert_changes_the_value() {
    let mut hash = StateHash::default();
    let mut values = vec![hash.value()];
    for seq in 0..100 {
        hash.insert_added(id(0, seq));
        values.push(hash.value());
        hash.insert_tombstoned(id(0, seq));
        values.push(hash.value());
    }
    values.sort_unstable();
    values.dedup();
    assert_eq!(values.len(), 201);
}
