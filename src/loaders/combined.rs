use crate::asset::loader::{AssetCreationHandler, AssetIdentifier, AssetLoader};
use any_handle::AnyHandle;
use std::any::Any;

pub struct CombinedLoader<Handler: AssetCreationHandler> {
    loaders: Vec<Box<dyn AssetLoader<Handler>>>,
}

impl<Handler: AssetCreationHandler> CombinedLoader<Handler> {
    pub fn new() -> Self {
        CombinedLoader {
            loaders: Vec::new(),
        }
    }

    pub fn with<T: AssetLoader<Handler> + 'static>(mut self, loader: T) -> Self {
        self.loaders.push(Box::new(loader));
        self
    }
}

impl<Handler: AssetCreationHandler> AssetLoader<Handler> for CombinedLoader<Handler> {
    fn load_asset(&self, identifier: &AssetIdentifier) -> Option<AnyHandle<dyn Any>> {
        for loader in &self.loaders {
            if let Some(asset) = loader.load_asset(identifier) {
                return Some(asset);
            }
        }

        return None;
    }
}
