// auto_builder_derive/src/gen.rs
// Code generation logic for AutoBuilder proc macro.

use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput, Data, Fields};

use crate::parse::{is_option, is_vec, option_inner_type, get_skip_expr, get_setter_set_name, get_setter_push_name, get_setter_push_many_name};

/// Main code generation entry point for the AutoBuilder macro.
/// Generates the builder struct, all setters, and the build method.
pub fn expand_autobuilder(input: TokenStream) -> TokenStream {
    // Parse the input struct
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;
    let builder_name = format_ident!("{}Builder", struct_name);

    // Extract named fields from the struct
    let fields = match input.data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Named(ref fields_named) => &fields_named.named,
            _ => panic!("AutoBuilder only supports named fields"),
        },
        _ => panic!("AutoBuilder only supports structs"),
    };

    // These vectors collect the generated code for the builder struct
    let mut builder_fields = Vec::new(); // Fields in the builder struct
    let mut setters = Vec::new();        // Setter methods
    let mut build_fields = Vec::new();   // Fields for the final build() call
    let mut field_idents = Vec::new();   // Field names for builder initialization
    let mut skipped_with_value = Vec::new();    // Skipped fields with a value
    let mut skipped_without_value = Vec::new(); // Skipped fields without a value

    // Iterate over each field in the struct
    for f in fields.iter() {
        let name = &f.ident;
        let ty = &f.ty;
        // Handle #[builder(skip)] and #[builder(skip = ...)]
        if let Some(skip) = get_skip_expr(&f.attrs) {
            if let Some(expr) = skip {
                // Skipped with a value: set in build_fields
                build_fields.push(quote! {
                    #name: #expr
                });
                skipped_with_value.push(name);
            } else {
                // Skipped without a value: will use Default::default()
                skipped_without_value.push(name);
            }
            continue;
        }
        field_idents.push(name);
        // Handle Vec fields with special push/set/extend setters
        if let Some(inner_ty) = is_vec(ty) {
            // Generate three methods: push (add_item), extend (add_items), set (set_items)
            let push_name = get_setter_push_name(&f.attrs, &format_ident!("add_item"));
            let push_many_name = get_setter_push_many_name(&f.attrs, &format_ident!("add_items"));
            let set_name = get_setter_set_name(&f.attrs, &format_ident!("items"));
            builder_fields.push(quote! { #name: Option<Vec<#inner_ty>> });
            setters.push(quote! {
                pub fn #push_name(&mut self, value: #inner_ty) -> &mut Self {
                    if self.#name.is_none() {
                        self.#name = Some(Vec::new());
                    }
                    self.#name.as_mut().unwrap().push(value);
                    self
                }
                pub fn #push_many_name(&mut self, values: Vec<#inner_ty>) -> &mut Self {
                    if self.#name.is_none() {
                        self.#name = Some(Vec::new());
                    }
                    self.#name.as_mut().unwrap().extend(values);
                    self
                }
                pub fn #set_name(&mut self, value: Vec<#inner_ty>) -> &mut Self {
                    self.#name = Some(value);
                    self
                }
            });
            // In build(), use .unwrap_or_default() for Vec fields
            build_fields.push(quote! {
                #name: self.#name.clone().unwrap_or_default()
            });
        } else if is_option(ty) {
            // Handle Option<T> fields: setter sets Some(value), build uses unwrap_or(None)
            let inner_ty = option_inner_type(ty).unwrap();
            // Support #[builder(setter = ...)] for Option fields
            let setter_name = {
                let mut setter_name = None;
                for attr in &f.attrs {
                    if attr.path().is_ident("builder") {
                        if let Ok(punct) = attr.parse_args_with(syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated) {
                            for meta in punct {
                                if meta.path().is_ident("setter") {
                                    if let syn::Meta::NameValue(ref nv) = meta {
                                        if let syn::Expr::Lit(expr_lit) = &nv.value {
                                            if let syn::Lit::Str(litstr) = &expr_lit.lit {
                                                setter_name = Some(format_ident!("{}", litstr.value()));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                setter_name.unwrap_or_else(|| name.as_ref().unwrap().clone())
            };
            builder_fields.push(quote! { #name: Option<#ty> });
            setters.push(quote! {
                pub fn #setter_name(&mut self, value: #inner_ty) -> &mut Self {
                    self.#name = Some(Some(value));
                    self
                }
            });
            build_fields.push(quote! {
                #name: self.#name.clone().unwrap_or(None)
            });
        } else {
            // Handle regular fields (required or with default)
            // Parse all builder keys in one pass: setter, default, etc.
            let mut setter_name = None;
            let mut default_expr = None;
            for attr in &f.attrs {
                if attr.path().is_ident("builder") {
                    if let Ok(punct) = attr.parse_args_with(syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated) {
                        for meta in punct {
                            if meta.path().is_ident("setter") {
                                if let syn::Meta::NameValue(ref nv) = meta {
                                    if let syn::Expr::Lit(expr_lit) = &nv.value {
                                        if let syn::Lit::Str(litstr) = &expr_lit.lit {
                                            setter_name = Some(format_ident!("{}", litstr.value()));
                                        }
                                    }
                                }
                            }
                            if meta.path().is_ident("default") {
                                if let syn::Meta::NameValue(ref nv) = meta {
                                    default_expr = Some(nv.value.clone());
                                }
                            }
                        }
                    }
                }
            }
            let setter_name = setter_name.unwrap_or_else(|| name.as_ref().unwrap().clone());
            // Generate the setter method for this field
            builder_fields.push(quote! { #name: Option<#ty> });
            setters.push(quote! {
                pub fn #setter_name(&mut self, value: #ty) -> &mut Self {
                    self.#name = Some(value);
                    self
                }
            });
            // In build(), use default if present, else require the field
            if let Some(expr) = default_expr {
                build_fields.push(quote! {
                    #name: self.#name.clone().unwrap_or_else(|| #expr)
                });
            } else {
                build_fields.push(quote! {
                    #name: self.#name.clone().ok_or_else(|| format!("Field '{}' is missing", stringify!(#name)))?
                });
            }
        }
    }

    // Compose the build() method, using Default if any fields were skipped without a value
    let build_struct = if !skipped_without_value.is_empty() {
        quote! {
            Ok(#struct_name {
                #(#build_fields,)*
                ..Default::default()
            })
        }
    } else {
        quote! {
            Ok(#struct_name {
                #(#build_fields,)*
            })
        }
    };

    // Generate the builder struct and its impl
    let expanded = quote! {
        pub struct #builder_name {
            #(#builder_fields,)*
        }
        impl #builder_name {
            pub fn new() -> Self {
                Self {
                    #(#field_idents: None,)*
                }
            }
            #(#setters)*
            pub fn build(&self) -> Result<#struct_name, String> {
                #build_struct
            }
        }
    };
    TokenStream::from(expanded)
} 