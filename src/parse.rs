// auto_builder_derive/src/parse.rs
// Attribute and field parsing helpers for AutoBuilder proc macro.

use syn::{Attribute, Expr, Type, PathArguments, Lit};
use quote::format_ident;

/// Returns true if the type is Option<T>
pub fn is_option(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.first() {
            if seg.ident == "Option" {
                if let PathArguments::AngleBracketed(_) = &seg.arguments {
                    return true;
                }
            }
        }
    }
    false
}

/// Returns Some(inner type) if the type is Vec<T>, else None
pub fn is_vec(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.first() {
            if seg.ident == "Vec" {
                if let PathArguments::AngleBracketed(args) = &seg.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return Some(inner_ty);
                    }
                }
            }
        }
    }
    None
}

/// Returns Some(inner type) if the type is Option<T>, else None
pub fn option_inner_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.first() {
            if seg.ident == "Option" {
                if let PathArguments::AngleBracketed(ref args) = seg.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return Some(inner_ty);
                    }
                }
            }
        }
    }
    None
}

/// Parses #[builder(skip)] and #[builder(skip = ...)] attributes.
/// Returns Some(Some(expr)) for #[builder(skip = ...)], Some(None) for #[builder(skip)], or None if not present.
pub fn get_skip_expr(attrs: &[Attribute]) -> Option<Option<Expr>> {
    for attr in attrs {
        if attr.path().is_ident("builder") {
            let mut skip_val: Option<Option<Expr>> = None;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") {
                    if let Ok(val) = meta.value() {
                        if let Ok(parsed) = syn::parse_str::<Expr>(&val.to_string()) {
                            skip_val = Some(Some(parsed));
                        }
                    } else {
                        skip_val = Some(None);
                    }
                }
                Ok(())
            });
            if skip_val.is_some() {
                return skip_val;
            }
        }
    }
    None
}

// --- Vec setter name helpers (for gen.rs) ---

/// Parses #[builder(setter_set = ...)] or returns set_{field} as default
pub fn get_setter_set_name(attrs: &[Attribute], default: &syn::Ident) -> syn::Ident {
    for attr in attrs {
        if attr.path().is_ident("builder") {
            let mut setter_name = None;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("setter_set") {
                    if let Ok(val) = meta.value() {
                        if let Ok(Lit::Str(litstr)) = val.parse() {
                            setter_name = Some(format_ident!("{}", litstr.value()));
                        }
                    }
                }
                Ok(())
            });
            if let Some(name) = setter_name {
                return name;
            }
        }
    }
    format_ident!("set_{}", default)
}

/// Parses #[builder(setter_push = ...)] or returns add_item as default
pub fn get_setter_push_name(attrs: &[Attribute], default: &syn::Ident) -> syn::Ident {
    for attr in attrs {
        if attr.path().is_ident("builder") {
            let mut setter_name = None;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("setter_push") {
                    if let Ok(val) = meta.value() {
                        if let Ok(Lit::Str(litstr)) = val.parse() {
                            setter_name = Some(format_ident!("{}", litstr.value()));
                        }
                    }
                }
                Ok(())
            });
            if let Some(name) = setter_name {
                return name;
            }
        }
    }
    default.clone()
}

/// Parses #[builder(setter_push_many = ...)] or returns add_items as default
pub fn get_setter_push_many_name(attrs: &[Attribute], default: &syn::Ident) -> syn::Ident {
    for attr in attrs {
        if attr.path().is_ident("builder") {
            let mut setter_name = None;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("setter_push_many") {
                    if let Ok(val) = meta.value() {
                        if let Ok(Lit::Str(litstr)) = val.parse() {
                            setter_name = Some(format_ident!("{}", litstr.value()));
                        }
                    }
                }
                Ok(())
            });
            if let Some(name) = setter_name {
                return name;
            }
        }
    }
    default.clone()
} 