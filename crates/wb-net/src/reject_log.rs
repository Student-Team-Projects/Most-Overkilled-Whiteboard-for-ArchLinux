//! Log lines for dropped datagrams, written on tick: at most one per reason every 10 s.

use wb_core::Time;

use crate::config::REJECT_LOG_EVERY_MS;
use crate::envelope::{Reason, Reject};

/// Collects dropped datagrams and turns them into log lines on `tick`, so a flood of garbage
/// cannot flood the log.
///
/// `record` only collects; nothing is logged until `tick`. Each reason is throttled on its own:
/// on a tick, a reason with drops since its last line gets a new line, unless its last line was
/// less than `REJECT_LOG_EVERY_MS` ago; then its drops wait for a later tick. A clock that goes
/// backwards counts as no time passed.
#[derive(Debug)]
pub struct RejectLog {
    reasons: [Pending; Reason::ALL.len()],
}

/// One reason: when it last got a line, and the drops since then.
#[derive(Debug, Clone, Copy)]
struct Pending {
    last_line: Option<Time>,
    dropped: u64,
    /// The newest drop since the last line, shown in the next line.
    newest: Option<Reject>,
}

impl Pending {
    /// Never logged, nothing dropped.
    const NEW: Self = Self {
        last_line: None,
        dropped: 0,
        newest: None,
    };
}

impl RejectLog {
    /// A log with nothing collected.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            reasons: [Pending::NEW; Reason::ALL.len()],
        }
    }

    /// Collects one dropped datagram; it shows up in a line on a later `tick`.
    pub fn record(&mut self, reject: Reject) {
        let pending = &mut self.reasons[reject.reason() as usize];
        pending.dropped = pending.dropped.saturating_add(1);
        pending.newest = Some(reject);
    }

    /// The lines due at `now`, one per reason at most, in `Reason::ALL` order.
    ///
    /// A line is `dropped N datagrams, newest: <reject>`; for an unknown version the reject
    /// names both versions. Each returned line resets its reason's count.
    pub fn tick(&mut self, now: Time) -> Vec<(Reject, String)> {
        let mut lines = Vec::new();
        for pending in &mut self.reasons {
            let quiet = pending
                .last_line
                .is_some_and(|last| now.0.saturating_sub(last.0) < REJECT_LOG_EVERY_MS);
            let Some(newest) = pending.newest.filter(|_| !quiet) else {
                continue;
            };
            lines.push((
                newest,
                format!("dropped {} datagrams, newest: {newest}", pending.dropped),
            ));
            *pending = Pending {
                last_line: Some(now),
                ..Pending::NEW
            };
        }
        lines
    }
}

impl Default for RejectLog {
    fn default() -> Self {
        Self::new()
    }
}
