use anyhow::Result;
use fs_err as fs;
use std::path::Path;
use std::process;
use syn::spanned::Spanned;
use syn::{File, Item, ItemConst, ItemImpl, ItemMacro, ItemStruct, ItemType, ItemUse, Visibility};
use walkdir::WalkDir;

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
                assert!(matches!(vis, Visibility::Public(_)));
            }
            Item::Struct(ItemStruct { vis, .. }) => {
                // TODO, lots more to check
                assert!(matches!(vis, Visibility::Public(_)));
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
            item => {
                let span = item.span();
                eprintln!(
                    "error: unexpected item\n  --> {}:{}:{}",
                    path.display(),
                    span.start().line,
                    span.start().column + 1,
                );
                todo!();
                process::exit(1);
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
