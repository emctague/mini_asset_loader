use crate::asset::loader::{AssetCreationHandler, AssetIdentifier, AssetLoader};
use any_handle::AnyHandle;
use std::any::Any;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

pub struct DirectoryLoader<'a, Handler: AssetCreationHandler> {
    directory: PathBuf,
    handler: &'a mut Handler,
}

impl<'a, Handler: AssetCreationHandler> DirectoryLoader<'a, Handler> {
    pub fn new<T: AsRef<Path>>(dir: T, handler: &'a mut Handler) -> Self {
        DirectoryLoader {
            directory: dir.as_ref().to_path_buf(),
            handler,
        }
    }
}

impl<'a, Handler: AssetCreationHandler> AssetLoader<Handler> for DirectoryLoader<'a, Handler> {
    fn load_asset(&self, identifier: &AssetIdentifier) -> Option<AnyHandle<dyn Any>> {
        let mut new_path: PathBuf = self.directory.clone();
        new_path.push(identifier);

        if !new_path.is_file() {
            return None;
        }

        let res = self
            .handler
            .create_asset(BufReader::new(File::open(new_path).ok()?))?;

        Some(AnyHandle::<dyn Any>::new(res))
    }
}
