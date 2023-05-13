//! Provides the [CachedLoader] type, which keeps loaded assets from its child Loader in
//! memory, freeing unused ones when `CachedLoader::garbage_collect` is called.

use crate::AnyHandle;
use crate::{AssetCreationHandler, AssetLoader};
use std::any::Any;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

/// A Loader that caches assets that it loads, allowing for quick loading of
/// the same, shared asset.
///
/// Each asset will be held onto and reference-counted. Use [garbage_collect] to destroy
/// any assets that are currently going entirely unused.
pub struct CachedLoader<Loader> {
    cache: RefCell<HashMap<Box<str>, AnyHandle<dyn Any>>>,
    loader: Loader,
}

impl<Loader> CachedLoader<Loader> {
    /// Create a new CachedLoader that caches the results of the given child loader.
    pub fn new(child: Loader) -> Self {
        CachedLoader {
            cache: RefCell::new(HashMap::new()),
            loader: child,
        }
    }

    /// Garbage collect, deleting all cached objects whose only reference is held
    /// by this CachedLoader.
    ///
    /// This iterates through the cache repeatedly to try and account for assets which
    /// were referenced by other orphaned assets.
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

/// Implement AssetLoader for the CachedLoader.
impl<Loader> AssetLoader for CachedLoader<Loader>
where
    Loader: AssetLoader,
{
    fn load_asset(
        &self,
        handler: &mut dyn AssetCreationHandler,
        identifier: &str,
    ) -> Option<AnyHandle<dyn Any>> {
        let mut cache = self.cache.borrow_mut();
        Some(
            match cache.entry(identifier.into()) {
                Entry::Occupied(o) => o.into_mut(),
                Entry::Vacant(v) => v.insert(self.loader.load_asset(handler, identifier)?),
            }
            .clone(),
        )
    }
}

/// The ToCached trait makes it easy to turn any loader into a cached loader.
/// Simply invoke to_cached() upon it.
pub trait ToCached<T: AssetLoader> {
    fn to_cached(self) -> CachedLoader<T>;
}

/// Blanket implementation of ToCached for all asset loaders.
impl<Y: AssetLoader> ToCached<Y> for Y {
    fn to_cached(self) -> CachedLoader<Y> {
        CachedLoader::new(self)
    }
}
