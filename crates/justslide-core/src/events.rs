//! Ordered, bounded event log with per-subscriber filtering. Agents only see
//! events for workspaces in their grant scope; UI-only events never reach
//! agents.

use serde::Serialize;
use serde_json::Value;
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize)]
pub struct StoredEvent {
    pub seq: u64,
    pub event: String,
    pub data: Value,
    #[serde(skip)]
    pub workspace_id: Option<String>,
    #[serde(skip)]
    pub ui_only: bool,
}

pub struct EventLog {
    next: u64,
    ring: VecDeque<StoredEvent>,
    cap: usize,
    pub tx: tokio::sync::broadcast::Sender<StoredEvent>,
}

impl EventLog {
    pub fn new(cap: usize) -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(cap.max(16));
        EventLog { next: 1, ring: VecDeque::with_capacity(cap), cap, tx }
    }

    pub fn push(&mut self, event: &str, data: Value, workspace_id: Option<String>, ui_only: bool) -> u64 {
        let ev = StoredEvent { seq: self.next, event: event.to_string(), data, workspace_id, ui_only };
        self.next += 1;
        if self.ring.len() == self.cap {
            self.ring.pop_front();
        }
        self.ring.push_back(ev.clone());
        let _ = self.tx.send(ev);
        self.next - 1
    }

    pub fn last_seq(&self) -> u64 { self.next - 1 }

    /// Returns events with seq > `after` visible to `visible`, and whether the
    /// caller fell behind the retained window (must resync via a snapshot).
    pub fn poll(&self, after: u64, max: usize, visible: impl Fn(&StoredEvent) -> bool) -> (Vec<StoredEvent>, bool) {
        let oldest = self.ring.front().map(|e| e.seq).unwrap_or(self.next);
        let resync = after + 1 < oldest && after < self.last_seq();
        let evs = self.ring.iter().filter(|e| e.seq > after && visible(e)).take(max).cloned().collect();
        (evs, resync)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_and_reports_resync() {
        let mut l = EventLog::new(4);
        for i in 0..10 { l.push("x", serde_json::json!({"i": i}), None, false); }
        let (evs, resync) = l.poll(0, 50, |_| true);
        assert!(resync);
        assert_eq!(evs.len(), 4);
        assert_eq!(evs[0].seq, 7);
        let (evs, resync) = l.poll(8, 50, |_| true);
        assert!(!resync);
        assert_eq!(evs.iter().map(|e| e.seq).collect::<Vec<_>>(), vec![9, 10]);
    }
}
