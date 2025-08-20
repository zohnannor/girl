#![cfg_attr(doc,  doc = include_str!("../README.md"))]
//! ## Feature flags
#![cfg_attr(
    feature = "document-features",
    cfg_attr(doc, doc = ::document_features::document_features!())
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

// extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

use bevy as _;
use girl as _;

mod unused {
    //! Not actually used, for surviving MSRV checks only.
    use cmake as _;
    // Only used for documentation.
    #[cfg(feature = "document-features")]
    use document_features as _;
    use erased_serde as _;
    use indexmap as _;
    use lock_api as _;
    use log as _;
    // Not actually used, dev-dependency for example/demo.
    #[cfg(test)]
    use tracing_subscriber as _;
}
