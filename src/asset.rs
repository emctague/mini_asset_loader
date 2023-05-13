//! A simple Asset type implementation based on JSON files.
//!
//! This module can be enabled via `asset` feature on the crate.
//! In order for this module to function correctly, you must be on Nightly,
//! thanks to `trait_upcasting`.
//!
//! ## How it works:
//!
//! ```
//! use serde::{Serialize, Deserialize};
//! use mini_asset_loader::asset::{TaggedJsonAsset, TaggedJsonAssetCreationHandler};
//! use mini_asset_loader::loaders::ToCached;
//! use mini_asset_loader::{TypedAssetLoader, asset_loader_vec, AssetLoader};
//! use std::path::PathBuf;
//!
//! // Creating an asset type is as easy as making a Serializable/Deserializable struct...
//! #[derive(Serialize, Deserialize)]
//! struct StringAsset {
//!     value: String
//! }
//!
//! /// ...and then tagging it with these two lines:
//! #[typetag::serde]
//! impl TaggedJsonAsset for StringAsset {}
//!
//!
//! // ...Then, when we want to *load* assets...
//!
//! // We create our loader setup as usual...
//! let mut loader = asset_loader_vec![
//!     PathBuf::from("assets/"),
//!     PathBuf::from("/global_assets/")
//! ].to_cached();
//!
//! // Make a TaggedJsonAssetCreationHandler...
//! let mut handler = TaggedJsonAssetCreationHandler::default();
//!
//! // And we can load our assets!
//! if let Some(my_string_asset) = loader.load_typed_asset::<StringAsset>(&mut handler, "my_string_asset.json") {
//!     println!("String asset loaded: {}", my_string_asset.read().value);
//! }
//!
//! ```

use std::any::Any;
use std::io::Read;

/// A TaggedJsonAsset is the base trait that must be implemented by any assets you want to make.
///
/// These assets must be [serde::Serialize], [serde::Deserialize], and their
/// `impl TaggedJsonAsset` must be tagged with [typetag::serde].
#[typetag::serde(tag = "type", content = "data")]
pub trait TaggedJsonAsset: Any {
    fn on_create(&mut self) {}
}

/// An AssetCreationHandler that loads JSON-based assets that implement [TaggedJsonAsset].
#[derive(Default)]
pub struct TaggedJsonAssetCreationHandler {}

/// Allows TaggedJsonAssetCreationHandler to create JSON assets. This *requires* nightly.
#[cfg(nightly)]
impl crate::AssetCreationHandler for TaggedJsonAssetCreationHandler {
    fn create_asset(&mut self, _: &str, reader: &mut dyn Read) -> Option<Box<dyn Any>> {
        let any: Box<dyn TaggedJsonAsset> = serde_json::from_reader(reader).ok()?;
        Some(any)
    }
}
