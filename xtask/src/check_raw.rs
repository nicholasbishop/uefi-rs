use anyhow::{bail, Result};
use fs_err as fs;
use proc_macro2::TokenStream;
use std::path::{Path, PathBuf};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{
    parenthesized, Attribute, Field, Fields, FieldsNamed, FieldsUnnamed, File, Item, ItemConst,
    ItemEnum, ItemExternCrate, ItemImpl, ItemMacro, ItemMod, ItemStruct, ItemTrait, ItemType,
    ItemUnion, ItemUse, LitInt, Visibility,
};
use walkdir::WalkDir;

fn is_pub(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

fn get_reprs(attrs: &[Attribute]) -> Vec<&'static str> {
    let mut reprs = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("repr") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("C") {
                    reprs.push("C");
                } else if meta.path.is_ident("packed") {
                    reprs.push("packed");
                } else if meta.path.is_ident("transparent") {
                    reprs.push("transparent");
                } else if meta.path.is_ident("align") {
                    // We don't care about the value, but we have to parse it to
                    // prevent parse_nested_meta from failing.
                    let content;
                    parenthesized!(content in meta.input);
                    let _lit: LitInt = content.parse()?;
                    reprs.push("align");
                } else {
                    panic!("unknown repr: {:?}", meta.path);
                }
                Ok(())
            })
            .unwrap();
        }
    }
    reprs
}

struct Error {
    msg: &'static str,
    path: PathBuf,
    line: usize,
    column: usize,
    code: String,
}

fn allow_non_pub(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|attr| attr.path().is_ident("allow_non_pub"))
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
    // * No generics, no references
    // * No phantomdata?
    // * Ensure Option on Event/Handle types?

    // Other TODO:
    // * get rid of data_types, move stuff up
    // * get rid of allow(unused)
    // * think about lints more
    // * Actually use the crate in uefi
    // * Get rid of all feature cfgs
    // * get rid of deprecated
    // * Drop tests

    for item in ast.items.iter() {
        match item {
            Item::Use(ItemUse { .. }) => {
                // Ignore
            }
            Item::Const(ItemConst { vis, .. }) => {
                // TODO: check type too
                if !is_pub(vis) {
                    add_error("missing pub", item);
                }
            }
            Item::Struct(ItemStruct {
                attrs, fields, vis, ..
            }) => {
                let allow_non_pub = allow_non_pub(attrs);
                if !allow_non_pub && !is_pub(vis) {
                    add_error("missing pub", item);
                }

                let mut check_fields = |fields: &Punctuated<Field, Comma>| {
                    for field in fields {
                        if !allow_non_pub && !is_pub(&field.vis) {
                            add_error("missing pub", field);
                        }
                        if let Some(ident) = &field.ident {
                            if ident.to_string().starts_with("_") {
                                add_error("field name starts with `_`", ident);
                            }
                        }
                    }
                };

                match fields {
                    Fields::Named(FieldsNamed { named, .. }) => check_fields(named),
                    Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => check_fields(unnamed),
                    Fields::Unit => {}
                }

                // TODO
                let reprs = get_reprs(attrs);
                if !reprs.contains(&"C") && !reprs.contains(&"transparent") {
                    add_error("bad repr", item);
                }

                // TODO, lots more to check
            }
            Item::Impl(ItemImpl { .. }) => {
                // TODO
            }
            Item::Macro(ItemMacro { mac, .. }) => {
                if mac.path.is_ident("bitflags") {
                    // Parse just the attributes.
                    struct Attrs(Vec<Attribute>);
                    impl Parse for Attrs {
                        fn parse(input: ParseStream) -> Result<Self, syn::Error> {
                            let x = input.call(Attribute::parse_outer)?;
                            let _: TokenStream = input.parse()?;
                            Ok(Self(x))
                        }
                    }
                    let attrs: Attrs = mac.parse_body()?;
                    let reprs = get_reprs(&attrs.0);
                    if reprs != ["transparent"] {
                        add_error("bad repr", item);
                    }
                }
            }
            Item::Type(ItemType { vis, .. }) => {
                if !is_pub(vis) {
                    add_error("missing pub", item);
                }
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
            Item::Trait(ItemTrait { ident, .. }) => {
                // TODO
                let allowed = ["Identify", "ResultExt"];
                if !allowed.contains(&ident.to_string().as_str()) {
                    add_error("unexpected trait", item);
                }
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
