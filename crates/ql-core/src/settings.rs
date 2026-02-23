//! Global library settings (translates `ql/settings.hpp`).
//!
//! [`Settings`] holds the **evaluation date** — the date at which all
//! calculations are performed.  It is a process-wide singleton accessed via
//! a `std::sync::OnceLock`.
//!
//! Thread safety: the evaluation date is stored behind a `Mutex` so that it
//! can be changed from any thread.  Each test that changes the evaluation date
//! should restore it when done (or use a dedicated test date).

use std::sync::{Mutex, OnceLock};

/// Process-wide settings used by the quantlib-rs library.
///
/// Currently the only setting is the **evaluation date** (today's date).
/// Instruments, term structures, and pricing engines use this date as the
/// reference point for their calculations.
pub struct Settings {
    /// The current evaluation date (days since the QuantLib epoch).
    evaluation_date: Mutex<Option<i32>>,
}

static INSTANCE: OnceLock<Settings> = OnceLock::new();

impl Settings {
    /// Return a reference to the global singleton.
    pub fn instance() -> &'static Settings {
        INSTANCE.get_or_init(|| Settings {
            evaluation_date: Mutex::new(None),
        })
    }

    /// Return the current evaluation date serial number (days since the
    /// QuantLib epoch: January 1, 1900).
    ///
    /// Returns `None` if no evaluation date has been set.
    pub fn evaluation_date_serial(&self) -> Option<i32> {
        *self
            .evaluation_date
            .lock()
            .expect("Settings mutex poisoned")
    }

    /// Set the evaluation date as a serial number.
    pub fn set_evaluation_date_serial(&self, serial: i32) {
        *self
            .evaluation_date
            .lock()
            .expect("Settings mutex poisoned") = Some(serial);
    }

    /// Clear the evaluation date, resetting it to "use today".
    pub fn reset_evaluation_date(&self) {
        *self
            .evaluation_date
            .lock()
            .expect("Settings mutex poisoned") = None;
    }
}
