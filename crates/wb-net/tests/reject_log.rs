//! Throttled log lines for dropped datagrams, driven by injected time.

use wb_core::{BoardId, Time};
use wb_net::config::{REJECT_LOG_EVERY_MS, VERSION};
use wb_net::{Reject, RejectLog};

const EVERY: u64 = REJECT_LOG_EVERY_MS;

/// Records `reject` at each time in `times` and returns the lines, `None` where skipped.
fn lines(log: &mut RejectLog, reject: &Reject, times: &[u64]) -> Vec<Option<String>> {
    times
        .iter()
        .map(|&ms| log.record(reject, Time(ms)))
        .collect()
}

/// One reject of every reason.
const EVERY_REJECT: [Reject; 4] = [
    Reject::WrongMagic,
    Reject::UnknownVersion { got: VERSION + 1 },
    Reject::Undecodable,
    Reject::ForeignBoard {
        got: BoardId([0; 8]),
    },
];

#[test]
fn first_drop_of_each_reason_logs_at_once() {
    let mut log = RejectLog::new();
    for reject in EVERY_REJECT {
        assert_eq!(
            log.record(&reject, Time(0)),
            Some(format!("dropped datagram: {reject}")),
            "{reject:?}"
        );
    }
}

/// Over several intervals: a line at most once per interval, and each says how many it skipped.
#[test]
fn throttles_per_interval_and_counts_skipped_drops() {
    let mut log = RejectLog::new();
    let got = lines(
        &mut log,
        &Reject::Undecodable,
        &[0, 1, 2, EVERY - 1, EVERY, EVERY + 1, 2 * EVERY, 4 * EVERY],
    );
    let line = |suffix: &str| Some(format!("dropped datagram: undecodable body{suffix}"));
    assert_eq!(
        got,
        [
            line(""),
            None,
            None,
            None,
            line(" (+3 more)"),
            None,
            line(" (+1 more)"),
            line(""),
        ]
    );
}

#[test]
fn reasons_throttle_independently() {
    let mut log = RejectLog::new();
    assert!(log.record(&Reject::WrongMagic, Time(0)).is_some());
    assert!(log.record(&Reject::WrongMagic, Time(1)).is_none());
    assert!(
        log.record(&Reject::Undecodable, Time(1)).is_some(),
        "own throttle"
    );
}

#[test]
fn unknown_version_line_names_both_versions() {
    let got = VERSION + 1;
    let line = RejectLog::new().record(&Reject::UnknownVersion { got }, Time(0));
    assert_eq!(
        line,
        Some(format!(
            "dropped datagram: unknown version {got}, this build speaks {VERSION}"
        ))
    );
}

#[test]
fn clock_going_backwards_counts_as_no_time() {
    let mut log = RejectLog::new();
    let got = lines(&mut log, &Reject::WrongMagic, &[EVERY, 0, 2 * EVERY]);
    let line = |suffix: &str| Some(format!("dropped datagram: wrong magic{suffix}"));
    assert_eq!(got, [line(""), None, line(" (+1 more)")]);
}
