//! Proc macros for corekit.

use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{
    parse_macro_input, Data, DeriveInput, Expr, ExprLit, ExprPath, Fields, Generics, Ident, Item, ItemStruct, Lit, Meta, Path, Token, Type,
    Visibility,
};
use syn::{GenericArgument, PathArguments};

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

#[proc_macro_attribute]
pub fn env_config(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let args = match parse_env_config_attr(proc_macro2::TokenStream::from(attr)) {
        Ok(args) => args,
        Err(error) => {
            let error = error.into_compile_error();
            return quote! {
                #input
                #error
            }
            .into();
        }
    };

    match expand_env_config_attribute(args, input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

#[proc_macro_derive(EnvConfig, attributes(env_config, env))]
pub fn derive_env_config(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    match expand_env_config(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
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

#[derive(Default)]
struct EnvConfigArgs {
    global: Option<Ident>,
    dotenv: DotenvMode,
}

enum DotenvMode {
    Default,
    File(String),
    Disabled,
}

impl Default for DotenvMode {
    fn default() -> Self {
        Self::Default
    }
}

struct EnvField {
    ident: Ident,
    ty: Type,
    env_name: String,
    mode: EnvFieldMode,
}

enum EnvFieldMode {
    Required,
    Default(Lit),
    DefaultOption(Lit, Type),
    Optional(Type),
}

#[derive(Default)]
struct EnvFieldArgs {
    name: Option<String>,
    default: Option<Lit>,
    optional: Option<Path>,
}

fn expand_env_config(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let args = parse_env_config_args(&input)?;
    expand_env_config_impl(&input, args)
}

fn expand_env_config_attribute(args: EnvConfigArgs, input: ItemStruct) -> syn::Result<proc_macro2::TokenStream> {
    let derive_input = derive_input_from_struct(&input);
    let generated = expand_env_config_impl(&derive_input, args)?;
    let mut emitted_input = input;
    strip_env_field_attrs(&mut emitted_input);
    emitted_input.attrs.push(syn::parse_quote!(#[allow(non_snake_case)]));

    Ok(quote! {
        #emitted_input

        #generated
    })
}

fn derive_input_from_struct(input: &ItemStruct) -> DeriveInput {
    DeriveInput {
        attrs: input.attrs.clone(),
        vis: input.vis.clone(),
        ident: input.ident.clone(),
        generics: input.generics.clone(),
        data: Data::Struct(syn::DataStruct {
            struct_token: input.struct_token,
            fields: input.fields.clone(),
            semi_token: input.semi_token,
        }),
    }
}

fn strip_env_field_attrs(input: &mut ItemStruct) {
    match &mut input.fields {
        Fields::Named(fields) => {
            for field in fields.named.iter_mut() {
                field.attrs.retain(|attr| !attr.path().is_ident("env"));
            }
        }
        Fields::Unnamed(fields) => {
            for field in fields.unnamed.iter_mut() {
                field.attrs.retain(|attr| !attr.path().is_ident("env"));
            }
        }
        Fields::Unit => {}
    }
}

fn expand_env_config_impl(input: &DeriveInput, args: EnvConfigArgs) -> syn::Result<proc_macro2::TokenStream> {
    let fields = parse_env_fields(&input)?;
    let ident = &input.ident;
    let vis = &input.vis;
    let dotenv_load = expand_dotenv_load(&args.dotenv);

    let field_loads = fields.iter().enumerate().map(|(index, field)| {
        let binding = format_ident!("__corekit_env_field_{index}");
        let env_name = &field.env_name;
        let ty = field.parse_ty();
        let present_value = field.present_value();
        let missing_value = field.missing_value();

        quote! {
            let #binding = match ::std::env::var(#env_name) {
                Ok(value) => match value.parse::<#ty>() {
                    Ok(value) => Some(#present_value),
                    Err(_) => {
                        errors.push(::corekit::EnvVarError::invalid(#env_name));
                        None
                    }
                },
                Err(::std::env::VarError::NotPresent) => {
                    #missing_value
                }
                Err(::std::env::VarError::NotUnicode(_)) => {
                    errors.push(::corekit::EnvVarError::invalid(#env_name));
                    None
                }
            };
        }
    });

    let field_inits = fields.iter().enumerate().map(|(index, field)| {
        let binding = format_ident!("__corekit_env_field_{index}");
        let ident = &field.ident;

        quote! {
            #ident: #binding.expect("validated env field")
        }
    });

    let global = args.global.map(|global| expand_env_global(vis, ident, &global));

    Ok(quote! {
        impl #ident {
            pub fn load() -> ::std::result::Result<Self, ::corekit::EnvError> {
                #dotenv_load

                let mut errors = ::std::vec::Vec::new();

                #(#field_loads)*

                if !errors.is_empty() {
                    return Err(::corekit::EnvError::new(errors));
                }

                Ok(Self {
                    #(#field_inits,)*
                })
            }
        }

        #global
    })
}

fn expand_dotenv_load(mode: &DotenvMode) -> proc_macro2::TokenStream {
    match mode {
        DotenvMode::Default => quote! {
            ::corekit::__private::dotenvy::dotenv().ok();
        },
        DotenvMode::File(filename) => quote! {
            ::corekit::__private::dotenvy::from_filename(#filename).ok();
        },
        DotenvMode::Disabled => quote! {},
    }
}

impl EnvField {
    fn parse_ty(&self) -> &Type {
        match &self.mode {
            EnvFieldMode::Required | EnvFieldMode::Default(_) => &self.ty,
            EnvFieldMode::DefaultOption(_, inner_ty) => inner_ty,
            EnvFieldMode::Optional(inner_ty) => inner_ty,
        }
    }

    fn present_value(&self) -> proc_macro2::TokenStream {
        match &self.mode {
            EnvFieldMode::Required | EnvFieldMode::Default(_) => quote! { value },
            EnvFieldMode::DefaultOption(_, _) => quote! { ::std::option::Option::Some(value) },
            EnvFieldMode::Optional(_) => quote! { ::std::option::Option::Some(value) },
        }
    }

    fn missing_value(&self) -> proc_macro2::TokenStream {
        let env_name = &self.env_name;

        match &self.mode {
            EnvFieldMode::Required => {
                quote! {
                    errors.push(::corekit::EnvVarError::missing(#env_name));
                    None
                }
            }
            EnvFieldMode::Default(default) => {
                let ty = &self.ty;

                quote! {
                    match #default.to_string().parse::<#ty>() {
                        Ok(value) => Some(value),
                        Err(_) => {
                            errors.push(::corekit::EnvVarError::invalid(#env_name));
                            None
                        }
                    }
                }
            }
            EnvFieldMode::DefaultOption(default, inner_ty) => {
                quote! {
                    match #default.to_string().parse::<#inner_ty>() {
                        Ok(value) => Some(::std::option::Option::Some(value)),
                        Err(_) => {
                            errors.push(::corekit::EnvVarError::invalid(#env_name));
                            None
                        }
                    }
                }
            }
            EnvFieldMode::Optional(_) => quote! {
                Some(::std::option::Option::None)
            },
        }
    }
}

fn expand_env_global(vis: &Visibility, env_ident: &Ident, global_ident: &Ident) -> proc_macro2::TokenStream {
    quote! {
        #[allow(non_upper_case_globals)]
        #vis static #global_ident: ::std::sync::LazyLock<#env_ident> = ::std::sync::LazyLock::new(|| {
            match #env_ident::load() {
                Ok(__corekit_env_value) => __corekit_env_value,
                Err(error) => panic!("failed to load EnvConfig: {error}"),
            }
        });
    }
}

fn parse_env_config_args(input: &DeriveInput) -> syn::Result<EnvConfigArgs> {
    let mut args = EnvConfigArgs::default();

    for attr in &input.attrs {
        if !attr.path().is_ident("env_config") {
            continue;
        }

        let metas = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for meta in metas {
            parse_env_config_meta(&mut args, meta)?;
        }
    }

    Ok(args)
}

fn parse_env_config_attr(attr: proc_macro2::TokenStream) -> syn::Result<EnvConfigArgs> {
    let mut args = EnvConfigArgs::default();
    let metas = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(attr)?;

    for meta in metas {
        parse_env_config_meta(&mut args, meta)?;
    }

    Ok(args)
}

fn parse_env_config_meta(args: &mut EnvConfigArgs, meta: Meta) -> syn::Result<()> {
    match meta {
        Meta::NameValue(name_value) if name_value.path.is_ident("global") => {
            let ident = match name_value.value {
                Expr::Path(ExprPath { path, .. }) if path.segments.len() == 1 => path.segments[0].ident.clone(),
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "`#[env_config(global = ...)]` expects an identifier",
                    ));
                }
            };
            args.global = Some(ident);
        }
        Meta::NameValue(name_value) if name_value.path.is_ident("dotenv") => {
            args.dotenv = match name_value.value {
                Expr::Lit(ExprLit { lit: Lit::Str(value), .. }) => DotenvMode::File(value.value()),
                Expr::Lit(ExprLit { lit: Lit::Bool(value), .. }) if !value.value => DotenvMode::Disabled,
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "`#[env_config(dotenv = ...)]` expects a string literal or `false`",
                    ));
                }
            };
        }
        other => {
            return Err(syn::Error::new_spanned(other, "unsupported `#[env_config]` argument"));
        }
    }

    Ok(())
}

fn parse_env_fields(input: &DeriveInput) -> syn::Result<Vec<EnvField>> {
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            Fields::Unnamed(_) | Fields::Unit => {
                return Err(syn::Error::new_spanned(
                    &input.ident,
                    "`EnvConfig` can only be derived for structs with named fields",
                ));
            }
        },
        Data::Enum(_) | Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "`EnvConfig` can only be derived for structs with named fields",
            ));
        }
    };

    fields
        .iter()
        .map(|field| {
            let ident = field.ident.clone().expect("named field has an identifier");
            let args = parse_env_field_args(field)?;
            let env_name = args.name.unwrap_or_else(|| env_name_from_field_ident(&ident));
            let option_inner_ty = option_inner_ty(&field.ty);
            let mode = match (args.default, args.optional, option_inner_ty) {
                (Some(default), None, None) => EnvFieldMode::Default(default),
                (Some(default), None, Some(inner_ty)) => EnvFieldMode::DefaultOption(default, inner_ty),
                (None, Some(_), Some(inner_ty)) | (None, None, Some(inner_ty)) => EnvFieldMode::Optional(inner_ty),
                (None, Some(optional), None) => {
                    return Err(syn::Error::new_spanned(
                        optional,
                        "`#[env(optional)]` requires an `Option<T>` field",
                    ));
                }
                (Some(_), Some(optional), _) => {
                    return Err(syn::Error::new_spanned(
                        optional,
                        "`#[env(optional)]` cannot be combined with `#[env(default = ...)]`",
                    ));
                }
                (None, None, None) => EnvFieldMode::Required,
            };

            Ok(EnvField {
                ident,
                ty: field.ty.clone(),
                env_name,
                mode,
            })
        })
        .collect()
}

fn parse_env_field_args(field: &syn::Field) -> syn::Result<EnvFieldArgs> {
    let mut args = EnvFieldArgs::default();

    for attr in &field.attrs {
        if !attr.path().is_ident("env") {
            continue;
        }

        let metas = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for meta in metas {
            match meta {
                Meta::NameValue(name_value) if name_value.path.is_ident("name") => {
                    let value = match name_value.value {
                        Expr::Lit(ExprLit { lit: Lit::Str(value), .. }) => value,
                        other => {
                            return Err(syn::Error::new_spanned(other, "`#[env(name = ...)]` expects a string literal"));
                        }
                    };
                    args.name = Some(value.value());
                }
                Meta::NameValue(name_value) if name_value.path.is_ident("default") => {
                    let value = match name_value.value {
                        Expr::Lit(ExprLit { lit, .. }) => lit,
                        other => {
                            return Err(syn::Error::new_spanned(other, "`#[env(default = ...)]` expects a literal"));
                        }
                    };
                    args.default = Some(value);
                }
                Meta::Path(path) if path.is_ident("optional") => {
                    args.optional = Some(path);
                }
                other => {
                    return Err(syn::Error::new_spanned(other, "unsupported `#[env]` argument"));
                }
            }
        }
    }

    Ok(args)
}

fn option_inner_ty(ty: &Type) -> Option<Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };

    let segment = type_path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }

    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    let Some(GenericArgument::Type(inner_ty)) = args.args.first() else {
        return None;
    };

    Some(inner_ty.clone())
}

fn env_name_from_field_ident(ident: &Ident) -> String {
    let value = ident.to_string();

    if value.chars().any(|ch| ch.is_ascii_lowercase()) {
        snake_to_screaming(&value)
    } else {
        value
    }
}

fn snake_to_screaming(value: &str) -> String {
    let mut output = String::new();
    let mut previous_was_lower_or_digit = false;

    for ch in value.chars() {
        if ch == '_' {
            output.push('_');
            previous_was_lower_or_digit = false;
            continue;
        }

        if ch.is_ascii_uppercase() && previous_was_lower_or_digit {
            output.push('_');
        }

        output.push(ch.to_ascii_uppercase());
        previous_was_lower_or_digit = ch.is_ascii_lowercase() || ch.is_ascii_digit();
    }

    output
}
