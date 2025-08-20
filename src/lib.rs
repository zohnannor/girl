#![cfg_attr(doc,  doc = include_str!("../README.md"))]
//! TODO docs

use bevy as _;
use girl as _;

mod unused {
    //! Not actually used, for surviving MSRV checks only.
    use cmake as _;
    use erased_serde as _;
    use indexmap as _;
    use lock_api as _;
    use log as _;
    #[cfg(test)]
    use tracing_subscriber as _;
}
