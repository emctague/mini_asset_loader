//! Provides the [ZipLoader] type, which loads assets from a zip file.
//! This module uses the `zip` crate, and must be enabled via the `zip` feature on this crate.

use crate::AnyHandle;
use crate::{AssetCreationHandler, AssetLoader};
use std::any::Any;
use std::cell::RefCell;
use std::io::{Read, Seek};

/// A loader that handles loading from a zip file.
///
/// Type R is the reader type used by [zip::read::ZipArchive] to read the Zip file.
pub struct ZipLoader<R>
where
    R: Read + Seek,
{
    archive: RefCell<zip::read::ZipArchive<R>>,
}

impl<R> ZipLoader<R>
where
    R: Read + Seek,
{
    /// Initialize a new ZipLoader that will read assets as files from the given ZipArchive.
    pub fn new(archive: zip::read::ZipArchive<R>) -> Self {
        ZipLoader {
            archive: RefCell::new(archive),
        }
    }
}

/// Implements AssetLoader for the ZipLoader.
impl<R> AssetLoader for ZipLoader<R>
where
    R: Read + Seek,
{
    fn load_asset(
        &self,
        handler: &mut dyn AssetCreationHandler,
        identifier: &str,
    ) -> Option<AnyHandle<dyn Any>> {
        let res = handler.create_asset(
            identifier,
            &mut self.archive.borrow_mut().by_name(identifier).ok()?,
        )?;

        Some(AnyHandle::<dyn Any>::new(res))
    }
}
