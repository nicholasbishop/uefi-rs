use anyhow::{bail, Result};
use fs_err as fs;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::{
    Fields, FieldsNamed, FieldsUnnamed, File, Item, ItemConst, ItemEnum, ItemExternCrate, ItemImpl,
    ItemMacro, ItemMod, ItemStruct, ItemTrait, ItemType, ItemUnion, ItemUse, Visibility,
};
use walkdir::WalkDir;

struct Error {
    msg: &'static str,
    path: PathBuf,
    line: usize,
    column: usize,
    code: String,
}

fn check_file(path: &Path, errors: &mut Vec<Error>) -> Result<()> {
    let mut add_error = |msg, spanned: &dyn Spanned| {
        let span = spanned.span();
        errors.push(Error {
            msg,
            // Getting the source path from the span is not yet stable:
            // https://github.com/rust-lang/rust/issues/54725
            path: path.to_path_buf(),
            line: span.start().line,
            column: span.start().column,
            code: span.source_text().unwrap(),
        });
    };

    let code = fs::read_to_string(path)?;

    let ast: File = syn::parse_str(&code)?;

    // Checks to add:
    // * All fields pub
    // * No fields start with `_`
    // * repr C/transparent
    // * no bool
    // * check extern fns

    // Other TODO:
    // * get rid of data_types, move stuff up
    // * get rid of allow(unused)
    // * think about lints more
    // * Actually use the crate in uefi
    // * Maybe add a macro for "extern" types
    // * Get rid of all feature cfgs
    // * get rid of deprecated

    for item in ast.items.iter() {
        match item {
            Item::Use(ItemUse { .. }) => {
                // TODO
            }
            Item::Const(ItemConst { vis, .. }) => {
                // TODO: check type too
                if !matches!(vis, Visibility::Public(_)) {
                    add_error("missing pub", item);
                }
            }
            Item::Struct(ItemStruct { vis, fields, .. }) => {
                if !matches!(vis, Visibility::Public(_)) {
                    add_error("missing pub", item);
                }

                match fields {
                    Fields::Named(FieldsNamed { named, .. }) => {
                        for field in named {
                            // TODO: dedup
                            if !matches!(field.vis, Visibility::Public(_)) {
                                add_error("missing pub", field);
                            }
                        }
                    }
                    Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        // TODO: dedup with above?
                        for field in unnamed {
                            // TODO: dedup
                            if !matches!(field.vis, Visibility::Public(_)) {
                                add_error("missing pub", field);
                            }
                        }
                    }
                    Fields::Unit => {}
                }

                // TODO, lots more to check
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
            Item::Trait(ItemTrait { .. }) => {
                // TODO
            }
            Item::ExternCrate(ItemExternCrate { .. }) => {
                // TODO
            }
            item => {
                add_error("unexpected kind of item", item);
            }
        }
    }

    Ok(())
}

pub fn check_raw() -> Result<()> {
    // TODO: will need a two-phase check?

    // TODO
    assert!(Path::new("uefi-raw").exists());

    let mut errors = Vec::new();

    for entry in WalkDir::new("uefi-raw") {
        let entry = entry?;
        let path = entry.path();

        if let Some(ext) = path.extension() {
            if ext == "rs" {
                println!("checking {}", path.display());
                check_file(path, &mut errors)?;
            }
        }
    }

    for error in &errors {
        eprintln!(
            "error: {}\n  --> {}:{}:{}\n{}",
            error.msg,
            error.path.display(),
            error.line,
            error.column + 1,
            error.code,
        );
    }

    if !errors.is_empty() {
        bail!("found {} errors", errors.len());
    }

    Ok(())
}
