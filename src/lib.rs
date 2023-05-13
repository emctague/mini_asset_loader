//! The mini asset loader provides a simple set of utilities to enable the loading of assets
//! by their name for use in, e.g., a game.
//!
//! ## Loaders
//!
//! - The [loaders::CachedLoader] provides the ability to cache assets in-memory between loads,
//!   with full user control over when unused assets are unloaded. The [loaders::ToCached] trait,
//!   which is implemented for all loaders, allows you to use `to_cached()` to convert a loader into
//!   a cached one.
//! - A simple `Vec<Box<dyn AssetLoader>>` can search multiple loaders and load the first successful
//!   result, which is made easier via the provided `asset_loader_vec!` macro.
//! - A [PathBuf] acts as a loader to load assets from a specific path on disk.
//! - The [loaders::ZipLoader] can load assets from a ZIP file.
//! - A simple [HashMap] can be used as a loader for assets stored in memory.
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
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

/// An AssetCreationHandler is a delegate that handles the creation (usually deserialization)
/// and allocation of assets from an input byte stream.
///
/// You could compose multiple AssetCreationHandlers by creating a handler that maps to other
/// handlers based on the identifier's file extension or magic numbers in the input stream.
/// See [ExtensionMappedAssetCreationHandler].
pub trait AssetCreationHandler {
    fn create_asset(&mut self, identifier: &str, reader: &mut dyn Read) -> Option<Box<dyn Any>>;
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
/// #     fn create_asset(&mut self, identifier: &str, reader: &mut dyn Read) -> Option<Box<dyn Any>> {
/// #         None
/// #     }
/// # }
/// # struct MyMeshHandler {}
/// # impl AssetCreationHandler for MyMeshHandler {
/// #     fn create_asset(&mut self, identifier: &str, reader: &mut dyn Read) -> Option<Box<dyn Any>> {
/// #         None
/// #     }
/// # }
/// let mut handler = ExtensionMappedAssetCreationHandler::new()
///     .with("json", MyJsonHandler {}) // Use MyJsonHandler on .json files
///     .with("dae", MyMeshHandler {}); // Use MyMeshHandler on .dae files
/// ```
#[derive(Default)]
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
    fn create_asset(&mut self, identifier: &str, reader: &mut dyn Read) -> Option<Box<dyn Any>> {
        let ext = Path::new(identifier).extension()?.to_str()?;
        let handler = self.handlers.get_mut(ext)?;
        handler.create_asset(identifier, reader)
    }
}

/// An AssetLoader loads an asset given its name, with help from an [AssetCreationHandler].
/// Some AssetLoaders implement special behaviour, such as caching or combining multiple
/// child loaders.
pub trait AssetLoader {
    /// Load an asset with the given identifier.
    ///
    /// Returns None if the asset could not be found or failed to serialize.
    fn load_asset(
        &self,
        handler: &mut dyn AssetCreationHandler,
        identifier: &str,
    ) -> Option<AnyHandle<dyn Any>>;
}

/// A simple HashMap can act as a loader for a set of values in memory.
impl AssetLoader for HashMap<String, AnyHandle<dyn Any>> {
    fn load_asset(
        &self,
        _handler: &mut dyn AssetCreationHandler,
        identifier: &str,
    ) -> Option<AnyHandle<dyn Any>> {
        self.get(identifier).cloned()
    }
}

/// A Vec<Box<dyn AssetLoader>> or similar structure can act as a combined loader over its elements,
/// querying them one by one. The `boxed_vec!` macro can help with this. Prefer this over the use of
/// a CombinedLoader.
impl AssetLoader for Vec<Box<dyn AssetLoader>> {
    fn load_asset(
        &self,
        handler: &mut dyn AssetCreationHandler,
        identifier: &str,
    ) -> Option<AnyHandle<dyn Any>> {
        self.iter().find_map(|x| x.load_asset(handler, identifier))
    }
}

/// A PathBuf can act as a loader for files relative to the directory it points to.
impl AssetLoader for PathBuf {
    fn load_asset(
        &self,
        handler: &mut dyn AssetCreationHandler,
        identifier: &str,
    ) -> Option<AnyHandle<dyn Any>> {
        let mut new_path: PathBuf = self.to_path_buf();
        new_path.push(identifier);

        if !new_path.is_file() {
            return None;
        }

        let res =
            handler.create_asset(identifier, &mut BufReader::new(File::open(new_path).ok()?))?;

        Some(AnyHandle::<dyn Any>::new(res))
    }
}

/// Allows for easy creation of a vector of boxed asset loaders.
/// Use it the same as you would use `vec!`. Each element will be passed through `Box::new`.
#[macro_export]
macro_rules! asset_loader_vec {
    () => {
        Vec::<Box<dyn AssetLoader>>::new()
    };
    ($($x:expr),+ $(,)?) => {
        vec![$(Box::new($x) as Box<dyn AssetLoader>),+]
    };
}

/// A TypedAssetLoader is a simple helper for AssetLoaders that can downcast
/// them into a handle matching their type.
pub trait TypedAssetLoader {
    /// Load an asset by its identifier, trying to interpret it as the given type.
    ///
    /// Returns None if the asset could not be found, failed to serialize, or was not of the
    /// correct type.
    fn load_typed_asset<T: Any>(
        &self,
        handler: &mut dyn AssetCreationHandler,
        identifier: &str,
    ) -> Option<AnyHandle<T>>;
}

/// Catch-all TypedAssetLoader implementation for any AssetLoader.
impl<T> TypedAssetLoader for T
where
    T: AssetLoader,
{
    fn load_typed_asset<Y: Any>(
        &self,
        handler: &mut dyn AssetCreationHandler,
        identifier: &str,
    ) -> Option<AnyHandle<Y>> {
        let result = self.load_asset(handler, identifier)?;
        result.into()
    }
}
