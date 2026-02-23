//! Global library settings (translates `ql/settings.hpp`).
//!
//! [`Settings`] holds the **evaluation date** — the date at which all
//! calculations are performed.  It is stored in a `thread_local!`
//! `RefCell<Settings>`, matching QuantLib's pre-thread-safe behaviour where
//! each thread has its own independent evaluation date.
//!
//! # Scoped evaluation date
//!
//! Use [`ScopedEvaluationDate`] to temporarily set the evaluation date; the
//! previous value is restored automatically when the guard is dropped.
//!
//! ```
//! use ql_core::settings::{Settings, ScopedEvaluationDate};
//!
//! Settings::set_evaluation_date_serial(45_000);
//! {
//!     let _guard = ScopedEvaluationDate::new(46_000);
//!     assert_eq!(Settings::instance().evaluation_date_serial(), Some(46_000));
//! }
//! // Restored to previous value.
//! assert_eq!(Settings::instance().evaluation_date_serial(), Some(45_000));
//! ```

use std::cell::RefCell;

/// Per-thread settings used by the quantlib-rs library.
///
/// Currently the only setting is the **evaluation date** (today's date).
/// Instruments, term structures, and pricing engines use this date as the
/// reference point for their calculations.
#[derive(Clone, Debug)]
pub struct Settings {
    /// The current evaluation date (days since the QuantLib epoch).
    evaluation_date: Option<i32>,
}

thread_local! {
    static INSTANCE: RefCell<Settings> = const { RefCell::new(Settings {
        evaluation_date: None,
    }) };
}

impl Settings {
    /// Obtain a temporary reference to the thread-local settings.
    ///
    /// This is the primary entry point — call methods on it:
    /// ```
    /// use ql_core::Settings;
    /// let serial = Settings::instance().evaluation_date_serial();
    /// ```
    ///
    /// Note: returns a *snapshot* copy because `thread_local!` + `RefCell`
    /// does not allow returning borrows across the closure boundary.
    pub fn instance() -> Settings {
        INSTANCE.with(|s| s.borrow().clone())
    }

    /// Return the current evaluation date serial number (days since the
    /// QuantLib epoch: January 1, 1900).
    ///
    /// Returns `None` if no evaluation date has been set.
    pub fn evaluation_date_serial(&self) -> Option<i32> {
        self.evaluation_date
    }

    /// Set the evaluation date as a serial number on the current thread.
    pub fn set_evaluation_date_serial(serial: i32) {
        INSTANCE.with(|s| s.borrow_mut().evaluation_date = Some(serial));
    }

    /// Clear the evaluation date, resetting it to "use today".
    pub fn reset_evaluation_date() {
        INSTANCE.with(|s| s.borrow_mut().evaluation_date = None);
    }
}

/// RAII guard that temporarily overrides the evaluation date.
///
/// When dropped, the previous evaluation date is restored.
pub struct ScopedEvaluationDate {
    previous: Option<i32>,
}

impl ScopedEvaluationDate {
    /// Set the evaluation date to `serial` and return a guard.
    ///
    /// The previous date will be restored when the guard is dropped.
    pub fn new(serial: i32) -> Self {
        let previous = Settings::instance().evaluation_date_serial();
        Settings::set_evaluation_date_serial(serial);
        Self { previous }
    }
}

impl Drop for ScopedEvaluationDate {
    fn drop(&mut self) {
        match self.previous {
            Some(s) => Settings::set_evaluation_date_serial(s),
            None => Settings::reset_evaluation_date(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_none() {
        Settings::reset_evaluation_date();
        assert_eq!(Settings::instance().evaluation_date_serial(), None);
    }

    #[test]
    fn set_and_get() {
        Settings::set_evaluation_date_serial(45_000);
        assert_eq!(Settings::instance().evaluation_date_serial(), Some(45_000));
        Settings::reset_evaluation_date();
    }

    #[test]
    fn scoped_evaluation_date() {
        Settings::set_evaluation_date_serial(44_000);
        {
            let _guard = ScopedEvaluationDate::new(46_000);
            assert_eq!(Settings::instance().evaluation_date_serial(), Some(46_000));
        }
        assert_eq!(Settings::instance().evaluation_date_serial(), Some(44_000));
        Settings::reset_evaluation_date();
    }
}
