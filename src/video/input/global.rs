extern crate alloc;

use crate::video::types::SkipTrigger;
use alloc::collections::BTreeSet;
use device_query::{DeviceQuery, DeviceState, Keycode};

/// Keys recognized by the global input tracker when determining skip gestures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum GlobalKey {
    /// Space bar press.
    Space,
    /// Enter key press.
    Enter,
    /// Escape key press.
    Escape,
}

/// Snapshot of relevant global input state.
#[derive(Default, Clone, PartialEq, Eq)]
pub(super) struct GlobalInputSnapshot {
    /// Keys detected as pressed in the latest snapshot.
    pub(super) keys: BTreeSet<GlobalKey>,
    /// Indicates whether any mouse button is pressed in the latest snapshot.
    pub(super) mouse_pressed: bool,
}

/// Provider abstraction enabling dependency injection for global input polling.
pub(super) trait GlobalInputProvider {
    /// Returns the most recent global input snapshot available from the provider.
    fn snapshot(&self) -> GlobalInputSnapshot;
}

/// Default provider backed by the `device_query` crate.
pub(super) struct DeviceQueryInputProvider {
    /// Underlying handle used by `device_query` to gather system-wide input state.
    device_state: DeviceState,
}

impl DeviceQueryInputProvider {
    /// Creates a new provider instance backed by a fresh `DeviceState`.
    pub(super) fn new() -> Self {
        Self {
            device_state: DeviceState::new(),
        }
    }
}

impl Default for DeviceQueryInputProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalInputProvider for DeviceQueryInputProvider {
    fn snapshot(&self) -> GlobalInputSnapshot {
        let mut keys = BTreeSet::new();
        for key in self.device_state.get_keys() {
            match key {
                Keycode::Space => {
                    keys.insert(GlobalKey::Space);
                }
                Keycode::Enter => {
                    keys.insert(GlobalKey::Enter);
                }
                Keycode::Escape => {
                    keys.insert(GlobalKey::Escape);
                }
                _ => {}
            }
        }

        let mouse_state = self.device_state.get_mouse();
        let mouse_pressed = mouse_state
            .button_pressed
            .into_iter()
            .any(|pressed| pressed);

        GlobalInputSnapshot {
            keys,
            mouse_pressed,
        }
    }
}

/// Tracks transitions in global input state and emits skip triggers on new gestures.
pub(super) struct GlobalInputTracker {
    /// Source of input snapshots.
    provider: Box<dyn GlobalInputProvider>,
    /// Keys observed in the previous snapshot.
    previous_keys: BTreeSet<GlobalKey>,
    /// Mouse pressed state captured in the previous snapshot.
    previous_mouse_pressed: bool,
}

impl GlobalInputTracker {
    /// Creates a tracker that uses the default device-query provider.
    pub(super) fn with_default_provider() -> Self {
        Self::new(Box::new(DeviceQueryInputProvider::default()))
    }

    /// Constructs a tracker using a custom provider implementation.
    pub(super) fn new(provider: Box<dyn GlobalInputProvider>) -> Self {
        Self {
            provider,
            previous_keys: BTreeSet::new(),
            previous_mouse_pressed: false,
        }
    }

    /// Captures an initial snapshot to establish the "no change" baseline.
    pub(super) fn initialize_baseline(&mut self) {
        let snapshot = self.provider.snapshot();
        self.previous_keys = snapshot.keys;
        self.previous_mouse_pressed = snapshot.mouse_pressed;
    }

    /// Polls the provider for a new snapshot and emits a skip trigger if input changed.
    pub(super) fn poll_trigger(&mut self) -> Option<SkipTrigger> {
        let snapshot = self.provider.snapshot();

        let keyboard_trigger = snapshot
            .keys
            .iter()
            .any(|key| !self.previous_keys.contains(key));

        let mouse_trigger = snapshot.mouse_pressed && !self.previous_mouse_pressed;

        self.previous_keys = snapshot.keys;
        self.previous_mouse_pressed = snapshot.mouse_pressed;

        if keyboard_trigger {
            return Some(SkipTrigger::Keyboard);
        }

        if mouse_trigger {
            return Some(SkipTrigger::Mouse);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::{BTreeSet, VecDeque};
    use std::cell::RefCell;

    struct MockProvider {
        snapshots: RefCell<VecDeque<GlobalInputSnapshot>>,
    }

    impl MockProvider {
        fn new(sequence: Vec<GlobalInputSnapshot>) -> Self {
            Self {
                snapshots: RefCell::new(sequence.into()),
            }
        }
    }

    impl GlobalInputProvider for MockProvider {
        fn snapshot(&self) -> GlobalInputSnapshot {
            self.snapshots.borrow_mut().pop_front().unwrap_or_default()
        }
    }

    fn snapshot_with_key(key: GlobalKey) -> GlobalInputSnapshot {
        let mut keys = BTreeSet::new();
        keys.insert(key);
        GlobalInputSnapshot {
            keys,
            mouse_pressed: false,
        }
    }

    fn snapshot_with_mouse_press() -> GlobalInputSnapshot {
        GlobalInputSnapshot {
            keys: BTreeSet::new(),
            mouse_pressed: true,
        }
    }

    #[test]
    fn tracker_detects_new_keyboard_press() {
        let snapshots = vec![
            GlobalInputSnapshot::default(),
            snapshot_with_key(GlobalKey::Space),
            snapshot_with_key(GlobalKey::Space),
        ];
        let provider = MockProvider::new(snapshots);
        let mut tracker = GlobalInputTracker::new(Box::new(provider));
        tracker.initialize_baseline();

        assert_eq!(tracker.poll_trigger(), Some(SkipTrigger::Keyboard));
        assert_eq!(tracker.poll_trigger(), None);
    }

    #[test]
    fn tracker_detects_new_mouse_press() {
        let snapshots = vec![
            GlobalInputSnapshot::default(),
            snapshot_with_mouse_press(),
            snapshot_with_mouse_press(),
        ];
        let provider = MockProvider::new(snapshots);
        let mut tracker = GlobalInputTracker::new(Box::new(provider));
        tracker.initialize_baseline();

        assert_eq!(tracker.poll_trigger(), Some(SkipTrigger::Mouse));
        assert_eq!(tracker.poll_trigger(), None);
    }

    #[test]
    fn tracker_emits_no_trigger_without_state_changes() {
        let snapshots = vec![
            GlobalInputSnapshot::default(),
            GlobalInputSnapshot::default(),
            GlobalInputSnapshot::default(),
        ];
        let provider = MockProvider::new(snapshots);
        let mut tracker = GlobalInputTracker::new(Box::new(provider));
        tracker.initialize_baseline();

        assert_eq!(tracker.poll_trigger(), None);
        assert_eq!(tracker.poll_trigger(), None);
    }
}
