use crate::asset::loader::{AssetCreationHandler, AssetIdentifier, AssetLoader};
use any_handle::AnyHandle;
use std::any::Any;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub struct CachedLoader<Handler: AssetCreationHandler> {
    cache: RefCell<HashMap<Box<AssetIdentifier>, AnyHandle<dyn Any>>>,
    loader: Box<dyn AssetLoader<Handler>>,
}

impl<Handler: AssetCreationHandler> CachedLoader<Handler> {
    pub fn new<T: AssetLoader<Handler> + 'static>(child: T) -> Self {
        CachedLoader {
            cache: RefCell::new(HashMap::new()),
            loader: Box::new(child),
        }
    }

    pub fn garbage_collect(&mut self) {
        // Continue to Garbage Collect until all references have been cleaned up.
        loop {
            let pre_len = self.cache.borrow().len();
            self.cache
                .borrow_mut()
                .retain(|_, v| v.reference_count() > 1);
            if pre_len == self.cache.borrow().len() {
                break;
            }
        }
    }
}

impl<Handler: AssetCreationHandler> AssetLoader<Handler> for CachedLoader<Handler> {
    fn load_asset(&self, identifier: &AssetIdentifier) -> Option<AnyHandle<dyn Any>> {
        let mut cache = self.cache.borrow_mut();
        Some(
            match cache.entry(identifier.into()) {
                Entry::Occupied(o) => o.into_mut(),
                Entry::Vacant(v) => v.insert(self.loader.load_asset(identifier)?),
            }
            .clone(),
        )
    }
}
