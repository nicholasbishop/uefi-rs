use anyhow::Result;
use syn::{File, Item, ItemConst, ItemImpl, ItemMacro, ItemStruct, ItemType, ItemUse, Visibility};

pub fn check_raw() -> Result<()> {
    // Load a single file for now. Parse it with syn. Check all the top-level stuff.
    //
    // Need at least a two-phase check...

    // TODO
    let code_text = include_str!("../../uefi-raw/src/table/boot.rs");

    let ast: File = syn::parse_str(code_text).unwrap();

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
            _ => {
                dbg!(item);
                todo!()
            }
        }
    }

    Ok(())
}
