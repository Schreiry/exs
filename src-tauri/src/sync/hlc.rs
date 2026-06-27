// Перенесено из Exsul (src-tauri/src/sync/hlc.rs) без изменений.
// Hybrid Logical Clock — monotonic, causality-preserving timestamps for the
// event ledger. Format: "<ts_ms>:<counter>:<node_id>".

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct HybridLogicalClock {
    inner: Mutex<HlcState>,
    node_id: String,
}

struct HlcState {
    ts: u64,
    counter: u32,
}

impl HybridLogicalClock {
    pub fn new(node_id: String) -> Self {
        Self {
            inner: Mutex::new(HlcState { ts: 0, counter: 0 }),
            node_id,
        }
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Generate a new HLC timestamp for a local event.
    pub fn now(&self) -> String {
        let physical = Self::physical_ms();
        let mut state = self.inner.lock().unwrap_or_else(|p| p.into_inner());

        if physical > state.ts {
            state.ts = physical;
            state.counter = 0;
        } else {
            state.counter += 1;
        }

        format!("{}:{}:{}", state.ts, state.counter, self.node_id)
    }

    /// Merge with a received remote timestamp.
    #[allow(dead_code)]
    pub fn recv(&self, remote_hlc: &str) -> String {
        let (remote_ts, remote_counter, _) = Self::parse(remote_hlc);
        let physical = Self::physical_ms();
        let mut state = self.inner.lock().unwrap_or_else(|p| p.into_inner());

        let max_ts = physical.max(state.ts).max(remote_ts);

        if max_ts == state.ts && max_ts == remote_ts {
            state.counter = state.counter.max(remote_counter) + 1;
        } else if max_ts == state.ts {
            state.counter += 1;
        } else if max_ts == remote_ts {
            state.counter = remote_counter + 1;
        } else {
            state.counter = 0;
        }
        state.ts = max_ts;

        format!("{}:{}:{}", state.ts, state.counter, self.node_id)
    }

    #[allow(dead_code)]
    pub fn parse(hlc: &str) -> (u64, u32, String) {
        let parts: Vec<&str> = hlc.splitn(3, ':').collect();
        (
            parts.first().and_then(|s| s.parse().ok()).unwrap_or(0),
            parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
            parts.get(2).unwrap_or(&"").to_string(),
        )
    }

    fn physical_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_is_monotonic() {
        let hlc = HybridLogicalClock::new("node-a".into());
        let a = hlc.now();
        let b = hlc.now();
        assert_ne!(a, b, "two consecutive timestamps must differ");
    }
}
