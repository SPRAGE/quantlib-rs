//! Analytical exotic option pricing engines.
//!
//! Self-contained implementations of closed-form exotic option pricing formulas:
//!
//! * **Simple chooser** — Rubinstein (1991) chooser option
//! * **Complex chooser** — Rubinstein (1991) complex chooser option
//! * **Compound option** — Geske (1979) compound option (option on an option)
//! * **Two-asset correlation** — two-asset correlation option
//! * **Holder-extendible** — holder-extendible option
//! * **Writer-extendible** — writer-extensible option

mod bivariate_normal;
mod simple_chooser;
mod complex_chooser;
mod compound_option;
mod two_asset_correlation;
mod holder_extensible;
mod writer_extensible;

pub use simple_chooser::AnalyticSimpleChooserEngine;
pub use complex_chooser::AnalyticComplexChooserEngine;
pub use compound_option::AnalyticCompoundOptionEngine;
pub use two_asset_correlation::AnalyticTwoAssetCorrelationEngine;
pub use holder_extensible::AnalyticHolderExtensibleOptionEngine;
pub use writer_extensible::AnalyticWriterExtensibleOptionEngine;
