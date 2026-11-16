//! The footer toast: one line of feedback after an action, `✓` in green for
//! a success and `✗` in red for a failure.
//!
//! Ported from octorus (ushironoko/octorus, MIT), `src/app/toast.rs`.
//!
//! The rules live here and nowhere else. A success leaves after 5 seconds,
//! or at the next key, whichever comes first. A failure has no timer. It
//! leaves at a key pressed more than 2 seconds after it appeared, so keys
//! mashed around the moment it appears do not clear it. While a failure is
//! on screen, Esc only dismisses it: Esc does not go back or quit.

use std::time::{Duration, Instant};

/// How long a success toast stays when no key comes first.
const SUCCESS_LIFETIME: Duration = Duration::from_secs(5);

/// Keys pressed within this long after a failure appears leave it up. Esc is
/// the exception: it dismisses a failure at once.
const FAILURE_KEY_GUARD: Duration = Duration::from_secs(2);

/// Whether a toast reports a success or a failure. The kind decides how long
/// the toast stays on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    /// The action finished.
    Success,
    /// The action failed, or could not be done.
    Failure,
}

/// One footer notice and the moment it appeared. A new toast replaces the
/// old one and starts its own clock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    kind: ToastKind,
    message: String,
    shown_at: Instant,
}

impl Toast {
    /// A success toast, shown now.
    pub fn success(message: impl Into<String>) -> Self {
        Self::at(ToastKind::Success, message, Instant::now())
    }

    /// A failure toast, shown now.
    pub fn failure(message: impl Into<String>) -> Self {
        Self::at(ToastKind::Failure, message, Instant::now())
    }

    /// A toast that appeared at `shown_at`.
    pub(crate) fn at(kind: ToastKind, message: impl Into<String>, shown_at: Instant) -> Self {
        Self {
            kind,
            message: message.into(),
            shown_at,
        }
    }

    pub fn kind(&self) -> ToastKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    fn age(&self, now: Instant) -> Duration {
        now.saturating_duration_since(self.shown_at)
    }
}

/// Clears `slot` when its success toast has run its 5 seconds by `now`.
/// Failures have no timer.
pub fn expire(slot: &mut Option<Toast>, now: Instant) {
    let timed_out = slot
        .as_ref()
        .is_some_and(|toast| toast.kind == ToastKind::Success && toast.age(now) > SUCCESS_LIFETIME);
    if timed_out {
        *slot = None;
    }
}

/// Applies one key press to `slot` at `now`.
///
/// Returns true when the key is spent: Esc on a failure. The failure is
/// dismissed and the key must go no further, so it cannot go back or quit.
pub fn apply_key(slot: &mut Option<Toast>, esc: bool, now: Instant) -> bool {
    let Some(toast) = slot.as_ref() else {
        return false;
    };
    match toast.kind {
        ToastKind::Success => {
            *slot = None;
            false
        }
        ToastKind::Failure => {
            if esc {
                *slot = None;
                return true;
            }
            if toast.age(now) > FAILURE_KEY_GUARD {
                *slot = None;
            }
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aged(kind: ToastKind, age: Duration) -> Option<Toast> {
        let shown_at = Instant::now()
            .checked_sub(age)
            .expect("clock is old enough");
        Some(Toast::at(kind, "message", shown_at))
    }

    #[test]
    fn a_success_expires_only_after_its_lifetime() {
        let now = Instant::now();
        let mut slot = Some(Toast::at(ToastKind::Success, "ok", now));
        expire(&mut slot, now + Duration::from_secs(5));
        assert!(slot.is_some(), "exactly 5 seconds is still up");
        expire(&mut slot, now + Duration::from_millis(5001));
        assert!(slot.is_none());
    }

    #[test]
    fn a_failure_never_expires_on_its_own() {
        let now = Instant::now();
        let mut slot = Some(Toast::at(ToastKind::Failure, "boom", now));
        expire(&mut slot, now + Duration::from_secs(3600));
        assert!(slot.is_some());
    }

    #[test]
    fn a_key_on_a_success_clears_it() {
        let mut slot = aged(ToastKind::Success, Duration::ZERO);
        assert!(!apply_key(&mut slot, false, Instant::now()));
        assert!(slot.is_none());
    }

    #[test]
    fn a_key_on_a_failure_waits_out_the_guard() {
        let mut slot = aged(ToastKind::Failure, Duration::from_millis(2001));
        assert!(!apply_key(&mut slot, false, Instant::now()));
        assert!(slot.is_none(), "past the guard");

        let mut slot = aged(ToastKind::Failure, Duration::from_millis(1990));
        assert!(!apply_key(&mut slot, false, Instant::now()));
        assert!(slot.is_some());
    }

    #[test]
    fn esc_spends_a_failure_at_once() {
        let mut slot = aged(ToastKind::Failure, Duration::ZERO);
        assert!(apply_key(&mut slot, true, Instant::now()));
        assert!(slot.is_none());
    }
}
