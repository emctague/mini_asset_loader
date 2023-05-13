//! Some basic types of AssetLoader.

pub mod cached;

#[cfg(feature = "zip")]
pub mod zip;

pub use cached::{CachedLoader, ToCached};

#[cfg(feature = "zip")]
pub use crate::loaders::zip::ZipLoader;
