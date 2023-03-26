use anyhow::Result;
use fs_err as fs;
use std::path::Path;
use std::process;
use syn::spanned::Spanned;
use syn::{
    File, Item, ItemConst, ItemEnum, ItemImpl, ItemMacro, ItemMod, ItemStruct, ItemType, ItemUnion,
    ItemUse, Visibility,
};
use walkdir::WalkDir;

fn fail(err: &str, spanned: &dyn Spanned, path: &Path) -> ! {
    let span = spanned.span();
    eprintln!(
        "error: {err}\n  --> {}:{}:{}",
        // Getting the source path from the span is not yet stable:
        // https://github.com/rust-lang/rust/issues/54725
        path.display(),
        span.start().line,
        span.start().column + 1,
    );
    process::exit(1);
}

fn check_file(path: &Path) -> Result<()> {
    let code = fs::read_to_string(path)?;

    let ast: File = syn::parse_str(&code)?;

    for item in ast.items.iter() {
        match item {
            Item::Use(ItemUse { .. }) => {
                // TODO
            }
            Item::Const(ItemConst { vis, .. }) => {
                // TODO: check type too
                if !matches!(vis, Visibility::Public(_)) {
                    fail("missing pub", item, path);
                }
            }
            Item::Struct(ItemStruct { vis, .. }) => {
                // TODO, lots more to check
                if !matches!(vis, Visibility::Public(_)) {
                    fail("missing pub", item, path);
                }
            }
            Item::Impl(ItemImpl { .. }) => {
                // TODO
            }
            Item::Macro(ItemMacro { .. }) => {
                // TODO
            }
            Item::Type(ItemType { .. }) => {
                // TODO
            }
            Item::Mod(ItemMod { .. }) => {
                // TODO
            }
            Item::Enum(ItemEnum { .. }) => {
                // TODO: decide if we actually want to allow any Rust enums
            }
            Item::Union(ItemUnion { .. }) => {
                // TODO
            }
            item => {
                fail("unexpected kind of item", item, path);
            }
        }
    }

    Ok(())
}

pub fn check_raw() -> Result<()> {
    // TODO: will need a two-phase check?

    // TODO
    assert!(Path::new("uefi-raw").exists());

    for entry in WalkDir::new("uefi-raw") {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "rs" {
                println!("checking {}", path.display());
                check_file(path)?;
            }
        }
    }

    Ok(())
}
