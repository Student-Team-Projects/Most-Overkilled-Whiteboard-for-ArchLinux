//! Log lines for dropped datagrams: only on tick, throttled per reason, driven by injected time.

use wb_core::{BoardId, Time};
use wb_net::config::{REJECT_LOG_EVERY_MS, VERSION};
use wb_net::{Reject, RejectLog};

const EVERY: u64 = REJECT_LOG_EVERY_MS;

/// One reject of every reason.
const EVERY_REJECT: [Reject; 4] = [
    Reject::WrongMagic,
    Reject::UnknownVersion { got: VERSION + 1 },
    Reject::Undecodable,
    Reject::ForeignBoard {
        got: BoardId([0; 8]),
    },
];

/// Just the text of the lines due at `ms`.
fn lines_at(log: &mut RejectLog, ms: u64) -> Vec<String> {
    log.tick(Time(ms))
        .into_iter()
        .map(|(_, line)| line)
        .collect()
}

fn line(dropped: u64, reject: Reject) -> String {
    format!("dropped {dropped} datagrams, newest: {reject}")
}

#[test]
fn nothing_is_logged_before_a_tick_or_without_drops() {
    let mut log = RejectLog::new();
    assert_eq!(lines_at(&mut log, 0), Vec::<String>::new(), "no drops");
    log.record(Reject::Undecodable);
    assert_eq!(lines_at(&mut log, 0), [line(1, Reject::Undecodable)]);
    assert_eq!(
        lines_at(&mut log, 2 * EVERY),
        Vec::<String>::new(),
        "no new drops"
    );
}

#[test]
fn first_tick_after_drops_logs_every_reason() {
    let mut log = RejectLog::new();
    for reject in EVERY_REJECT {
        log.record(reject);
    }
    assert_eq!(
        lines_at(&mut log, 0),
        EVERY_REJECT.map(|reject| line(1, reject))
    );
}

/// Several intervals, ticked often: a line at most once per interval, each with the drops since
/// the line before.
#[test]
fn throttles_per_interval_and_counts_every_drop() {
    let mut log = RejectLog::new();
    let mut got = Vec::new();
    for (ms, drops) in [
        (0, 1),
        (1, 2),
        (EVERY - 1, 1),
        (EVERY, 0),
        (EVERY + 1, 4),
        (2 * EVERY, 0),
    ] {
        for _ in 0..drops {
            log.record(Reject::Undecodable);
        }
        got.push(lines_at(&mut log, ms));
    }
    let one = |dropped| vec![line(dropped, Reject::Undecodable)];
    assert_eq!(got, [one(1), vec![], vec![], one(3), vec![], one(4)]);
}

#[test]
fn reasons_throttle_independently() {
    let mut log = RejectLog::new();
    log.record(Reject::WrongMagic);
    assert_eq!(lines_at(&mut log, 0).len(), 1);
    log.record(Reject::WrongMagic);
    log.record(Reject::Undecodable);
    assert_eq!(lines_at(&mut log, 1), [line(1, Reject::Undecodable)]);
}

#[test]
fn unknown_version_line_names_both_versions() {
    let got = VERSION + 1;
    let mut log = RejectLog::new();
    log.record(Reject::UnknownVersion { got });
    assert_eq!(
        lines_at(&mut log, 0),
        [format!(
            "dropped 1 datagrams, newest: unknown version {got}, this build speaks {VERSION}"
        )]
    );
}

#[test]
fn clock_going_backwards_counts_as_no_time() {
    let mut log = RejectLog::new();
    log.record(Reject::WrongMagic);
    assert_eq!(lines_at(&mut log, EVERY).len(), 1);
    log.record(Reject::WrongMagic);
    assert_eq!(lines_at(&mut log, 0), Vec::<String>::new());
    assert_eq!(lines_at(&mut log, 2 * EVERY), [line(1, Reject::WrongMagic)]);
}
