//! The mini asset loader provides a simple set of utilities to enable the loading of assets
//! by their name for use in, e.g., a game.
//!
//! ## Loaders
//!
//! - The [loaders::CachedLoader] provides the ability to cache assets in-memory between loads,
//!   with full user control over when unused assets are unloaded.
//! - The [loaders::CombinedLoader] can search multiple nested loaders for a given asset.
//! - The [loaders::DirectoryLoader] can load assets from a specific path on disk.
//! - The [loaders::ZipLoader] can load assets from a ZIP file.
//! - The [asset::TaggedJsonAsset] provides a simple pre-existing implementation of loading
//!   type-tagged assets from JSON files using `serde` and `typetag`.
//!
//! These loaders can be composed in various ways to create more advanced behaviour.
//!
//! ## Asset Creation Handlers
//!
//! An [AssetCreationHandler] implementation is required - this provides the function that actually
//! *creates* an asset, meaning this is the ideal place to implement custom deserialization, allocation,
//! etc. for your custom asset type.
//!
//! For a simple implementation of this, which provides a helpful example of the asset system in use,
//! (but which, unfortunately, requires nightly rust), see the [asset] module.
//!
//! ## Features
//!
//! - `zip` - Provides the [loaders::zip] module, containing a ZIP file loader.
//! - `asset` - Provides the [asset] module, containing a simple JSON asset implementation.

#![cfg_attr(all(feature = "asset", nightly), feature(trait_upcasting))]
#[cfg(feature = "asset")]
pub mod asset;

pub mod loaders;

pub use any_handle::AnyHandle;
use std::any::Any;
use std::io::Read;

/// An AssetCreationHandler is a delegate that handles the creation (usually deserialization)
/// and allocation of assets from an input byte stream.
pub trait AssetCreationHandler {
    fn create_asset<R: Read>(&mut self, reader: R) -> Option<Box<dyn Any>>;
}

/// An AssetLoader loads an asset given its name, with help from an [AssetCreationHandler].
/// Some AssetLoaders implement special behaviour, such as caching or combining multiple
/// child loaders.
pub trait AssetLoader<Handler: AssetCreationHandler> {
    /// Load an asset with the given identifier.
    ///
    /// Returns None if the asset could not be found or failed to serialize.
    fn load_asset(&self, handler: &mut Handler, identifier: &str) -> Option<AnyHandle<dyn Any>>;
}

/// A TypedAssetLoader is a simple helper for AssetLoaders that can downcast
/// them into a handle matching their type.
pub trait TypedAssetLoader<Handler: AssetCreationHandler> {
    /// Load an asset by its identifier, trying to interpret it as the given type.
    ///
    /// Returns None if the asset could not be found, failed to serialize, or was not of the
    /// correct type.
    fn load_typed_asset<T: Any>(
        &self,
        handler: &mut Handler,
        identifier: &str,
    ) -> Option<AnyHandle<T>>;
}

/// Catch-all TypedAssetLoader implementation for any AssetLoader.
impl<T, Handler: AssetCreationHandler> TypedAssetLoader<Handler> for T
where
    T: AssetLoader<Handler>,
{
    fn load_typed_asset<Y: Any>(
        &self,
        handler: &mut Handler,
        identifier: &str,
    ) -> Option<AnyHandle<Y>> {
        let result = self.load_asset(handler, identifier)?;
        result.into()
    }
}
