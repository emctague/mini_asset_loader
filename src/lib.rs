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
//! The [ExtensionMappedAssetCreationHandler] is a builtin type that allows one to map different file
//! extensions on the asset identifier string to different AssetCreationHandlers. For example, one could use
//! the builtin `asset` module's creation handler for `.json`-based types, and use a separate mesh creation
//! handler for `.dae` types.
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
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

/// An AssetCreationHandler is a delegate that handles the creation (usually deserialization)
/// and allocation of assets from an input byte stream.
///
/// You could compose multiple AssetCreationHandlers by creating a handler that maps to other
/// handlers based on the identifier's file extension or magic numbers in the input stream.
/// See [ExtensionMappedAssetCreationHandler].
pub trait AssetCreationHandler {
    fn create_asset(&mut self, identifier: &str, reader: Box<dyn Read>) -> Option<Box<dyn Any>>;
}

/// Maps to multiple [AssetCreationHandler]s based on the file extension of the asset.
///
/// ## Example
///
/// ```
/// # use std::any::Any;
/// # use std::io::Read;
/// # use mini_asset_loader::{AssetCreationHandler, ExtensionMappedAssetCreationHandler};
/// # struct MyJsonHandler {}
/// # impl AssetCreationHandler for MyJsonHandler {
/// #     fn create_asset(&mut self, identifier: &str, reader: Box<dyn Read>) -> Option<Box<dyn Any>> {
/// #         None
/// #     }
/// # }
/// # struct MyMeshHandler {}
/// # impl AssetCreationHandler for MyMeshHandler {
/// #     fn create_asset(&mut self, identifier: &str, reader: Box<dyn Read>) -> Option<Box<dyn Any>> {
/// #         None
/// #     }
/// # }
/// let mut handler = ExtensionMappedAssetCreationHandler::new()
///     .with("json", MyJsonHandler {}) // Use MyJsonHandler on .json files
///     .with("dae", MyMeshHandler {}); // Use MyMeshHandler on .dae files
/// ```
pub struct ExtensionMappedAssetCreationHandler {
    handlers: HashMap<String, Box<dyn AssetCreationHandler>>,
}

impl ExtensionMappedAssetCreationHandler {
    /// Creates a default ExtensionMappedAssetCreationHandler.
    /// Use [with] to add extensions.
    pub fn new() -> Self {
        ExtensionMappedAssetCreationHandler {
            handlers: HashMap::default(),
        }
    }
    /// Returns a version of this Handler with an additional child Handler.
    pub fn with<T: AssetCreationHandler + 'static>(mut self, key: &str, handler: T) -> Self {
        self.handlers.insert(key.to_string(), Box::new(handler));
        self
    }
}

impl AssetCreationHandler for ExtensionMappedAssetCreationHandler {
    /// Handles extension-mapped asset creation.
    fn create_asset(&mut self, identifier: &str, reader: Box<dyn Read>) -> Option<Box<dyn Any>> {
        let ext = Path::new(identifier).extension()?.to_str()?;
        let handler = self.handlers.get_mut(ext)?;
        handler.create_asset(identifier, reader)
    }
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
