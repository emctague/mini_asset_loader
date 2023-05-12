//! Provides the [DirectoryLoader] type, which loads assets from a directory on disk.

use crate::AnyHandle;
use crate::{AssetCreationHandler, AssetLoader};
use std::any::Any;
use std::fs::File;
use std::io::BufReader;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// A Loader that can load assets from a directory on disk.
pub struct DirectoryLoader<Handler: AssetCreationHandler> {
    directory: PathBuf,
    _phantom: PhantomData<Handler>,
}

impl<Handler: AssetCreationHandler> DirectoryLoader<Handler> {
    /// Create a new DirectoryLoader that loads from the given directory.
    ///
    /// Assets will be loaded as files relative to `dir`.
    pub fn new<T: AsRef<Path>>(dir: T) -> Self {
        DirectoryLoader {
            directory: dir.as_ref().to_path_buf(),
            _phantom: PhantomData,
        }
    }
}

/// Implements AssetLoader for DirectoryLoader.
impl<'a, Handler: AssetCreationHandler> AssetLoader<Handler> for DirectoryLoader<Handler> {
    fn load_asset(&self, handler: &mut Handler, identifier: &str) -> Option<AnyHandle<dyn Any>> {
        let mut new_path: PathBuf = self.directory.clone();
        new_path.push(identifier);

        if !new_path.is_file() {
            return None;
        }

        let res = handler.create_asset(BufReader::new(File::open(new_path).ok()?))?;

        Some(AnyHandle::<dyn Any>::new(res))
    }
}
