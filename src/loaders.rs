//! Some basic types of AssetLoader.

pub mod cached;
pub mod combined;
pub mod directory;

#[cfg(feature = "zip")]
pub mod zip;

pub use cached::CachedLoader;
pub use combined::CombinedLoader;
pub use directory::DirectoryLoader;

#[cfg(feature = "zip")]
pub use crate::loaders::zip::ZipLoader;
