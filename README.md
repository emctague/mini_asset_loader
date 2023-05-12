# mini_asset_loader

[![Crates.io](https://img.shields.io/crates/v/mini_asset_loader?style=for-the-badge)](https://crates.io/crates/mini_asset_loader) [![docs.rs](https://img.shields.io/docsrs/mini_asset_loader?style=for-the-badge)](https://docs.rs/mini_asset_loader) [![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/emctague/mini_asset_loader/rust.yml?style=for-the-badge)](https://github.com/emctague/mini_asset_loader) [![Crates.io](https://img.shields.io/crates/l/mini_asset_loader?style=for-the-badge)](https://opensource.org/license/mit/) 

`mini_asset_loader` provides an extensible asset loading system, intended to load assets from
various sources for use in a game.

The asset types, loaded sources, caching behaviour, etc. is entirely customizable.
Assets are reference-counted and thread-safe.

A simple asset type based on `serde_json` and type tags is provided in the `asset` module.

## Example

This example makes use of the `asset` feature, which only works on nightly rust. However, substituting this
for a custom asset type will work in stable rust.

```rust
use serde::{Serialize, Deserialize};
use mini_asset_loader::loaders::{CachedLoader, CombinedLoader, DirectoryLoader};
use mini_asset_loader::TypedAssetLoader;

// Creating an asset type is as easy as making a Serializable/Deserializable struct...
#[derive(Serialize, Deserialize)]
struct StringAsset {
    value: String
}

// ...and then tagging it with these two lines:
#[typetag::serde]
impl TaggedJsonAsset for StringAsset {}



// ...Then, when we want to *load* assets...
fn main() {
    // We create our loader setup as usual...
    let mut loader = CachedLoader::new(CombinedLoader::new()
        .with(DirectoryLoader::new("assets/"))
        .with(DirectoryLoader::new("/global_assets/")));

    // Make a TaggedJsonAssetCreationHandler...
    let mut handler = TaggedJsonAssetCreationHandler::default();
    
    // And we can load our assets!
    if let Some(my_string_asset) = loader.load_typed_asset::<StringAsset>(&mut handler, "my_string_asset.json") {
        println!("String asset loaded: {}", my_string_asset.read().value);
    }
}
```