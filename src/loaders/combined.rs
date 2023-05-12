//! Provides the [CombinedLoader] type, which can search multiple child Loaders for a file.

use crate::AnyHandle;
use crate::{AssetCreationHandler, AssetLoader};
use std::any::Any;

/// A Loader that queries several child loaders for a file.
///
/// When asked to load an asset, each child loader will be queried in-order for
/// the same asset until one provides a value.
pub struct CombinedLoader<Handler: AssetCreationHandler> {
    loaders: Vec<Box<dyn AssetLoader<Handler>>>,
}

impl<Handler: AssetCreationHandler> CombinedLoader<Handler> {
    /// Creates a new CombinedLoader with no child loaders.
    ///
    /// [with] can be used to add child loaders.
    pub fn new() -> Self {
        CombinedLoader {
            loaders: Vec::new(),
        }
    }

    /// Creates a new CombinedLoader with the given child AssetLoaders.
    pub fn new_with(loaders: Vec<Box<dyn AssetLoader<Handler>>>) -> Self {
        CombinedLoader { loaders }
    }

    /// Returns a version of this Loader with an additional child Loader.
    pub fn with<T: AssetLoader<Handler> + 'static>(mut self, loader: T) -> Self {
        self.loaders.push(Box::new(loader));
        self
    }
}

/// Implements AssetLoader for CombinedLoader.
impl<Handler: AssetCreationHandler> AssetLoader<Handler> for CombinedLoader<Handler> {
    fn load_asset(&self, handler: &mut Handler, identifier: &str) -> Option<AnyHandle<dyn Any>> {
        for loader in &self.loaders {
            if let Some(asset) = loader.load_asset(handler, identifier) {
                return Some(asset);
            }
        }

        return None;
    }
}
