//! Proc macros for corekit.

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, ExprPath, Generics, Ident, Item, ItemStruct, Meta, Path, Token};

#[proc_macro_attribute]
pub fn singleton(attr: TokenStream, item: TokenStream) -> TokenStream {
    let original_item = proc_macro2::TokenStream::from(item.clone());
    let parsed_item = match syn::parse::<Item>(item) {
        Ok(item) => item,
        Err(error) => return error.into_compile_error().into(),
    };

    let args = match syn::parse::<SingletonArgs>(attr) {
        Ok(args) => args,
        Err(error) => {
            let error = error.into_compile_error();
            return quote! {
                #original_item
                #error
            }
            .into();
        }
    };

    expand_singleton(args, parsed_item, original_item).into()
}

#[derive(Default)]
struct SingletonArgs {
    strategy: InitStrategy,
    has_strategy_conflict: bool,
}

enum InitStrategy {
    New,
    Default,
    Custom(Path),
}

impl Default for InitStrategy {
    fn default() -> Self {
        Self::New
    }
}

impl Parse for SingletonArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let metas = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
        let mut strategy = None;
        let mut has_strategy_conflict = false;

        for meta in metas {
            match meta {
                Meta::Path(path) if path.is_ident("default") => {
                    set_strategy(&mut strategy, InitStrategy::Default, &mut has_strategy_conflict);
                }
                Meta::NameValue(name_value) if name_value.path.is_ident("init") => {
                    let path = match name_value.value {
                        Expr::Path(ExprPath { path, .. }) => path,
                        other => {
                            return Err(syn::Error::new_spanned(
                                other,
                                "`#[singleton(init = ...)]` expects a zero-argument function path",
                            ));
                        }
                    };
                    set_strategy(&mut strategy, InitStrategy::Custom(path), &mut has_strategy_conflict);
                }
                other => {
                    return Err(syn::Error::new_spanned(other, "unsupported `#[singleton]` argument"));
                }
            }
        }

        Ok(Self {
            strategy: strategy.unwrap_or_default(),
            has_strategy_conflict,
        })
    }
}

fn set_strategy(current: &mut Option<InitStrategy>, next: InitStrategy, has_conflict: &mut bool) {
    if current.is_some() {
        *has_conflict = true;
        return;
    }

    *current = Some(next);
}

fn expand_singleton(args: SingletonArgs, item: Item, original_item: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let input = match item {
        Item::Struct(input) => input,
        other => {
            let error = syn::Error::new_spanned(other, "`#[singleton]` can only be used on structs").into_compile_error();
            return quote! {
                #original_item
                #error
            };
        }
    };

    let ident = &input.ident;

    if args.has_strategy_conflict {
        let error = syn::Error::new_spanned(ident, "`#[singleton]` accepts only one init strategy").into_compile_error();
        return quote! {
            #input
            #error
        };
    }

    if let Err(error) = reject_generics(&input) {
        let error = error.into_compile_error();
        return quote! {
            #input
            #error
        };
    }

    let init = init_path(&args.strategy, ident);

    quote! {
        #input

        impl #ident {
            pub fn shared() -> &'static Self {
                static INSTANCE: ::std::sync::LazyLock<#ident> = ::std::sync::LazyLock::new(|| #init());
                &*INSTANCE
            }
        }
    }
}

fn reject_generics(input: &ItemStruct) -> syn::Result<()> {
    let Generics { params, where_clause, .. } = &input.generics;

    if params.is_empty() && where_clause.is_none() {
        return Ok(());
    }

    Err(syn::Error::new_spanned(
        &input.ident,
        "`#[singleton]` does not support generic structs",
    ))
}

fn init_path(strategy: &InitStrategy, ident: &Ident) -> proc_macro2::TokenStream {
    let span = ident.span();

    match strategy {
        InitStrategy::New => quote_spanned!(span=> #ident::new),
        InitStrategy::Default => quote_spanned!(span=> #ident::default),
        InitStrategy::Custom(path) => {
            let mut path = path.clone();
            normalize_init_path(&mut path, ident);
            quote_spanned!(span=> #path)
        }
    }
}

fn normalize_init_path(path: &mut Path, type_ident: &Ident) {
    let span = type_ident.span();

    if let Some(first) = path.segments.first_mut() {
        if first.ident == "Self" {
            first.ident = type_ident.clone();
        }
    }

    for segment in path.segments.iter_mut() {
        segment.ident.set_span(span);
    }
}
