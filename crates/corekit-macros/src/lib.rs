//! Proc macros for corekit.

use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Expr, ExprLit, ExprPath, Fields, Generics, Ident, Item, ItemFn, ItemStruct, Lit, Meta,
    Path, Token, Type, UnOp, Visibility,
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
pub fn retry(attr: TokenStream, item: TokenStream) -> TokenStream {
    let original_item = proc_macro2::TokenStream::from(item.clone());
    let parsed_item = match syn::parse::<Item>(item) {
        Ok(item) => item,
        Err(error) => return error.into_compile_error().into(),
    };

    let input = match parsed_item {
        Item::Fn(input) => input,
        _ => {
            let error = retry_error("`#[retry]` can only be used on async functions");
            return quote! {
                #original_item
                #error
            }
            .into();
        }
    };

    let args = match syn::parse::<RetryArgs>(attr) {
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

    expand_retry(args, input).into()
}

#[proc_macro_attribute]
pub fn timeout(attr: TokenStream, item: TokenStream) -> TokenStream {
    let original_item = proc_macro2::TokenStream::from(item.clone());
    let parsed_item = match syn::parse::<Item>(item) {
        Ok(item) => item,
        Err(error) => return error.into_compile_error().into(),
    };

    let input = match parsed_item {
        Item::Fn(input) => input,
        _ => {
            let error = timeout_error("`#[timeout]` can only be used on async functions");
            return quote! {
                #original_item
                #error
            }
            .into();
        }
    };

    let args = match syn::parse::<TimeoutArgs>(attr) {
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

    expand_timeout(args, input).into()
}

#[proc_macro_attribute]
pub fn env_config(attr: TokenStream, item: TokenStream) -> TokenStream {
    let original_item = proc_macro2::TokenStream::from(item.clone());
    let parsed_item = match syn::parse::<Item>(item) {
        Ok(item) => item,
        Err(error) => return error.into_compile_error().into(),
    };

    let input = match parsed_item {
        Item::Struct(input) => input,
        _ => {
            let error = env_config_attribute_error("`#[env_config]` can only be used on structs with named fields");
            return quote! {
                #original_item
                #error
            }
            .into();
        }
    };

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

    if !matches!(input.fields, Fields::Named(_)) {
        let error = env_config_attribute_error("`#[env_config]` can only be used on structs with named fields");
        return quote! {
            #input
            #error
        }
        .into();
    }

    if has_generics(&input.generics) {
        let error = env_config_attribute_error("`#[env_config]` does not support generic structs");
        return quote! {
            #input
            #error
        }
        .into();
    }

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

struct RetryArgs {
    max_retries: usize,
    initial_delay_millis: u64,
    exponential_base: u64,
    max_delay_millis: u64,
    jitter: bool,
}

impl Default for RetryArgs {
    fn default() -> Self {
        Self {
            max_retries: 20,
            initial_delay_millis: 1_000,
            exponential_base: 2,
            max_delay_millis: 180_000,
            jitter: true,
        }
    }
}

impl Parse for RetryArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let metas = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
        let mut args = RetryArgs::default();
        let mut has_max_retries = false;
        let mut has_initial_delay = false;
        let mut has_exponential_base = false;
        let mut has_max_delay = false;
        let mut has_jitter = false;

        for meta in metas {
            match meta {
                Meta::NameValue(name_value) if name_value.path.is_ident("max_retries") => {
                    if has_max_retries {
                        return Err(syn::Error::new_spanned(
                            name_value,
                            "duplicate `#[retry(max_retries = ...)]` argument",
                        ));
                    }

                    let max_retries = match name_value.value {
                        Expr::Lit(ExprLit { lit: Lit::Int(value), .. }) => value.base10_parse::<usize>()?,
                        other => {
                            return Err(syn::Error::new_spanned(
                                other,
                                "`#[retry(max_retries = ...)]` expects a non-negative integer literal",
                            ));
                        }
                    };
                    args.max_retries = max_retries;
                    has_max_retries = true;
                }
                Meta::NameValue(name_value) if name_value.path.is_ident("initial_delay") => {
                    if has_initial_delay {
                        return Err(syn::Error::new_spanned(
                            name_value,
                            "duplicate `#[retry(initial_delay = ...)]` argument",
                        ));
                    }

                    let initial_delay = match &name_value.value {
                        Expr::Lit(ExprLit { lit: Lit::Str(value), .. }) => parse_retry_duration(
                            value.value().as_str(),
                            "`#[retry(initial_delay = ...)]` expects a duration string like \"500ms\", \"1s\", \"2m\", or \"1h\"",
                            value.span(),
                        )?,
                        other => {
                            return Err(syn::Error::new_spanned(
                                other,
                                "`#[retry(initial_delay = ...)]` expects a string literal",
                            ));
                        }
                    };
                    args.initial_delay_millis = initial_delay;
                    has_initial_delay = true;
                }
                Meta::NameValue(name_value) if name_value.path.is_ident("exponential_base") => {
                    if has_exponential_base {
                        return Err(syn::Error::new_spanned(
                            name_value,
                            "duplicate `#[retry(exponential_base = ...)]` argument",
                        ));
                    }

                    let exponential_base = match &name_value.value {
                        Expr::Lit(ExprLit { lit: Lit::Int(value), .. }) => value.base10_parse::<u64>()?,
                        other => {
                            return Err(syn::Error::new_spanned(
                                other,
                                "`#[retry(exponential_base = ...)]` expects an integer literal greater than 0",
                            ));
                        }
                    };

                    if exponential_base == 0 {
                        return Err(syn::Error::new_spanned(
                            name_value,
                            "`#[retry(exponential_base = ...)]` expects an integer literal greater than 0",
                        ));
                    }

                    args.exponential_base = exponential_base;
                    has_exponential_base = true;
                }
                Meta::NameValue(name_value) if name_value.path.is_ident("max_delay") => {
                    if has_max_delay {
                        return Err(syn::Error::new_spanned(
                            name_value,
                            "duplicate `#[retry(max_delay = ...)]` argument",
                        ));
                    }

                    let max_delay = match &name_value.value {
                        Expr::Lit(ExprLit { lit: Lit::Str(value), .. }) => parse_retry_duration(
                            value.value().as_str(),
                            "`#[retry(max_delay = ...)]` expects a duration string like \"500ms\", \"1s\", \"2m\", or \"1h\"",
                            value.span(),
                        )?,
                        other => {
                            return Err(syn::Error::new_spanned(
                                other,
                                "`#[retry(max_delay = ...)]` expects a string literal",
                            ));
                        }
                    };
                    args.max_delay_millis = max_delay;
                    has_max_delay = true;
                }
                Meta::NameValue(name_value) if name_value.path.is_ident("jitter") => {
                    if has_jitter {
                        return Err(syn::Error::new_spanned(name_value, "duplicate `#[retry(jitter = ...)]` argument"));
                    }

                    let jitter = match name_value.value {
                        Expr::Lit(ExprLit { lit: Lit::Bool(value), .. }) => value.value,
                        other => {
                            return Err(syn::Error::new_spanned(other, "`#[retry(jitter = ...)]` expects a boolean literal"));
                        }
                    };
                    args.jitter = jitter;
                    has_jitter = true;
                }
                other => {
                    return Err(syn::Error::new_spanned(other, "unsupported `#[retry]` argument"));
                }
            }
        }

        Ok(args)
    }
}

fn parse_retry_duration(value: &str, error_message: &str, span: proc_macro2::Span) -> syn::Result<u64> {
    let Some((number, multiplier)) = value
        .strip_suffix("ms")
        .map(|number| (number, 1_u64))
        .or_else(|| value.strip_suffix('s').map(|number| (number, 1_000_u64)))
        .or_else(|| value.strip_suffix('m').map(|number| (number, 60_000_u64)))
        .or_else(|| value.strip_suffix('h').map(|number| (number, 3_600_000_u64)))
    else {
        return Err(syn::Error::new(span, error_message));
    };

    if number.is_empty() || !number.chars().all(|character| character.is_ascii_digit()) {
        return Err(syn::Error::new(span, error_message));
    }

    let number = number.parse::<u64>().map_err(|_| syn::Error::new(span, error_message))?;

    number.checked_mul(multiplier).ok_or_else(|| syn::Error::new(span, error_message))
}

struct TimeoutArgs {
    duration_millis: u64,
}

impl Parse for TimeoutArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let args = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;

        if args.len() != 1 {
            return Err(syn::Error::new(
                input.span(),
                "`#[timeout(...)]` expects exactly one duration string",
            ));
        }

        let duration = args.first().expect("checked arg length");
        let duration_millis = match duration {
            Expr::Lit(ExprLit { lit: Lit::Str(value), .. }) => parse_retry_duration(
                value.value().as_str(),
                "`#[timeout(...)]` expects a duration string like \"500ms\", \"1s\", \"2m\", or \"1h\"",
                value.span(),
            )?,
            other => {
                return Err(syn::Error::new_spanned(other, "`#[timeout(...)]` expects a string literal"));
            }
        };

        Ok(Self { duration_millis })
    }
}

fn expand_retry(args: RetryArgs, input: ItemFn) -> proc_macro2::TokenStream {
    if input.sig.asyncness.is_none() {
        let error = retry_error("`#[retry]` can only be used on async functions");
        return quote! {
            #input
            #error
        };
    }

    let attrs = &input.attrs;
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let max_retries = args.max_retries;
    let initial_delay_millis = args.initial_delay_millis;
    let exponential_base = args.exponential_base;
    let max_delay_millis = args.max_delay_millis;
    let jitter = args.jitter;

    let (outer_attrs, inner_attrs) = split_retry_attrs(attrs);

    let attempt = match retry_attempt_expression(&inner_attrs, block) {
        Ok(attempt) => attempt,
        Err(error) => {
            let error = error.into_compile_error();
            return quote! {
                #input
                #error
            };
        }
    };

    quote! {
        #(#outer_attrs)*
        #vis #sig {
            let mut __corekit_retry_count: usize = 0;
            let __corekit_retry_max_delay_millis: u64 = #max_delay_millis;
            let mut __corekit_retry_delay_millis: u64 =
                ::std::cmp::min(#initial_delay_millis, __corekit_retry_max_delay_millis);

            loop {
                let __corekit_retry_result = #attempt;

                match __corekit_retry_result {
                    Ok(__corekit_retry_value) => return Ok(__corekit_retry_value),
                    Err(__corekit_retry_error) => {
                        if !::corekit::Retryable::is_retryable(&__corekit_retry_error) || __corekit_retry_count >= #max_retries {
                            return Err(__corekit_retry_error);
                        }

                        __corekit_retry_count += 1;

                        let __corekit_retry_sleep_millis = if #jitter && __corekit_retry_delay_millis > 0 {
                            let __corekit_retry_now_nanos = ::std::time::SystemTime::now()
                                .duration_since(::std::time::UNIX_EPOCH)
                                .map(|__corekit_retry_duration| __corekit_retry_duration.subsec_nanos() as u64)
                                .unwrap_or(0);

                            if __corekit_retry_delay_millis == u64::MAX {
                                __corekit_retry_now_nanos
                            } else {
                                __corekit_retry_now_nanos % (__corekit_retry_delay_millis + 1)
                            }
                        } else {
                            __corekit_retry_delay_millis
                        };

                        if __corekit_retry_sleep_millis > 0 {
                            ::corekit::__private::tokio::time::sleep(
                                ::std::time::Duration::from_millis(__corekit_retry_sleep_millis),
                            )
                            .await;
                        }

                        __corekit_retry_delay_millis = __corekit_retry_delay_millis
                            .saturating_mul(#exponential_base)
                            .min(__corekit_retry_max_delay_millis);
                    }
                }
            }
        }
    }
}

fn split_retry_attrs(attrs: &[Attribute]) -> (Vec<&Attribute>, Vec<&Attribute>) {
    attrs.iter().partition(|attr| !is_timeout_attr(attr))
}

fn is_timeout_attr(attr: &Attribute) -> bool {
    attr.path().segments.last().is_some_and(|segment| segment.ident == "timeout")
}

fn retry_attempt_expression(attrs: &[&Attribute], block: &syn::Block) -> syn::Result<proc_macro2::TokenStream> {
    match attrs {
        [] => Ok(quote! {
            async #block.await
        }),
        [timeout_attr] => {
            let args = timeout_attr.parse_args::<TimeoutArgs>()?;
            Ok(timeout_attempt(args.duration_millis, quote! { async #block }))
        }
        [_, extra, ..] => Err(syn::Error::new_spanned(
            extra,
            "`#[retry]` supports at most one `#[timeout]` attribute",
        )),
    }
}

fn retry_error(message: &str) -> proc_macro2::TokenStream {
    syn::Error::new(proc_macro2::Span::call_site(), message).into_compile_error()
}

fn expand_timeout(args: TimeoutArgs, input: ItemFn) -> proc_macro2::TokenStream {
    if input.sig.asyncness.is_none() {
        let error = timeout_error("`#[timeout]` can only be used on async functions");
        return quote! {
            #input
            #error
        };
    }

    let attrs = &input.attrs;
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let duration_millis = args.duration_millis;

    let attempt = timeout_attempt(duration_millis, quote! { async #block });

    quote! {
        #(#attrs)*
        #vis #sig {
            #attempt
        }
    }
}

fn timeout_error(message: &str) -> proc_macro2::TokenStream {
    syn::Error::new(proc_macro2::Span::call_site(), message).into_compile_error()
}

fn timeout_attempt(duration_millis: u64, future: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    quote! {
        match ::corekit::__private::tokio::time::timeout(
            ::std::time::Duration::from_millis(#duration_millis),
            #future,
        )
        .await
        {
            Ok(__corekit_timeout_result) => __corekit_timeout_result,
            Err(_) => Err(::corekit::FromTimeout::from_timeout()),
        }
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
    if !has_generics(&input.generics) {
        return Ok(());
    }

    Err(syn::Error::new_spanned(
        &input.ident,
        "`#[singleton]` does not support generic structs",
    ))
}

fn has_generics(generics: &Generics) -> bool {
    let Generics { params, where_clause, .. } = generics;

    !params.is_empty() || where_clause.is_some()
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
    has_dotenv: bool,
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
    trim: bool,
    non_empty: bool,
    min: Option<Expr>,
    max: Option<Expr>,
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
    trim: Option<Path>,
    non_empty: Option<Path>,
    min: Option<Expr>,
    max: Option<Expr>,
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
    reject_env_config_generics(input)?;

    let fields = parse_env_fields(&input)?;
    let ident = &input.ident;
    let vis = &input.vis;
    let dotenv_load = expand_dotenv_load(&args.dotenv);

    let field_loads = fields.iter().enumerate().map(|(index, field)| {
        let binding = format_ident!("__corekit_env_field_{index}");
        let env_name = &field.env_name;
        let present_load = field.present_load(quote! { value });
        let missing_value = field.missing_value();

        quote! {
            let #binding = match ::std::env::var(#env_name) {
                Ok(value) => #present_load,
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

fn env_config_attribute_error(message: &str) -> proc_macro2::TokenStream {
    syn::Error::new(proc_macro2::Span::call_site(), message).into_compile_error()
}

fn reject_env_config_generics(input: &DeriveInput) -> syn::Result<()> {
    if !has_generics(&input.generics) {
        return Ok(());
    }

    Err(syn::Error::new_spanned(
        &input.ident,
        "`EnvConfig` does not support generic structs",
    ))
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

    fn present_load(&self, raw_value: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        let ty = self.parse_ty();
        let env_name = &self.env_name;
        let present_value = self.present_value();
        let value = if self.trim {
            quote! { #raw_value.trim().to_owned() }
        } else {
            raw_value
        };
        let non_empty_check = if self.non_empty {
            quote! {
                if value.trim().is_empty() {
                    errors.push(::corekit::EnvVarError::invalid(#env_name));
                    None
                } else
            }
        } else {
            quote! {}
        };
        let range_check = self.range_check();

        quote! {
            {
                let value = #value;

                #non_empty_check {
                    match value.parse::<#ty>() {
                        Ok(value) => #range_check {
                            Some(#present_value)
                        },
                        Err(_) => {
                            errors.push(::corekit::EnvVarError::invalid(#env_name));
                            None
                        }
                    }
                }
            }
        }
    }

    fn range_check(&self) -> proc_macro2::TokenStream {
        let env_name = &self.env_name;
        let min_check = self.min.as_ref().map(|min| {
            quote! {
                if value < #min {
                    errors.push(::corekit::EnvVarError::invalid(#env_name));
                    None
                } else
            }
        });
        let max_check = self.max.as_ref().map(|max| {
            quote! {
                if value > #max {
                    errors.push(::corekit::EnvVarError::invalid(#env_name));
                    None
                } else
            }
        });

        quote! {
            #min_check #max_check
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
            EnvFieldMode::Default(default) => self.present_load(quote! { #default.to_string() }),
            EnvFieldMode::DefaultOption(default, _inner_ty) => self.present_load(quote! { #default.to_string() }),
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
            if args.global.is_some() {
                return Err(syn::Error::new_spanned(
                    name_value,
                    "duplicate `#[env_config(global = ...)]` argument",
                ));
            }

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
            if args.has_dotenv {
                return Err(syn::Error::new_spanned(
                    name_value,
                    "duplicate `#[env_config(dotenv = ...)]` argument",
                ));
            }

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
            args.has_dotenv = true;
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
            let parse_ty = match &mode {
                EnvFieldMode::Required | EnvFieldMode::Default(_) => &field.ty,
                EnvFieldMode::DefaultOption(_, inner_ty) | EnvFieldMode::Optional(inner_ty) => inner_ty,
            };

            if let Some(trim) = args.trim.as_ref() {
                if !is_string_ty(parse_ty) {
                    return Err(syn::Error::new_spanned(
                        trim,
                        "`#[env(trim)]` requires a `String` or `Option<String>` field",
                    ));
                }
            }

            if let Some(non_empty) = args.non_empty.as_ref() {
                if !is_string_ty(parse_ty) {
                    return Err(syn::Error::new_spanned(
                        non_empty,
                        "`#[env(non_empty)]` requires a `String` or `Option<String>` field",
                    ));
                }
            }

            if let Some(min) = args.min.as_ref() {
                if !is_numeric_ty(parse_ty) {
                    return Err(syn::Error::new_spanned(
                        min,
                        "`#[env(min = ...)]` requires an integer or float field",
                    ));
                }
            }

            if let Some(max) = args.max.as_ref() {
                if !is_numeric_ty(parse_ty) {
                    return Err(syn::Error::new_spanned(
                        max,
                        "`#[env(max = ...)]` requires an integer or float field",
                    ));
                }
            }

            Ok(EnvField {
                ident,
                ty: field.ty.clone(),
                env_name,
                mode,
                trim: args.trim.is_some(),
                non_empty: args.non_empty.is_some(),
                min: args.min,
                max: args.max,
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
                Meta::NameValue(name_value) if name_value.path.is_ident("min") => {
                    args.min = Some(parse_numeric_bound_expr(name_value.value, "min")?);
                }
                Meta::NameValue(name_value) if name_value.path.is_ident("max") => {
                    args.max = Some(parse_numeric_bound_expr(name_value.value, "max")?);
                }
                Meta::Path(path) if path.is_ident("optional") => {
                    args.optional = Some(path);
                }
                Meta::Path(path) if path.is_ident("trim") => {
                    args.trim = Some(path);
                }
                Meta::Path(path) if path.is_ident("non_empty") => {
                    args.non_empty = Some(path);
                }
                other => {
                    return Err(syn::Error::new_spanned(other, "unsupported `#[env]` argument"));
                }
            }
        }
    }

    Ok(args)
}

fn parse_numeric_bound_expr(value: Expr, name: &str) -> syn::Result<Expr> {
    if is_numeric_bound_expr(&value) {
        return Ok(value);
    }

    Err(syn::Error::new_spanned(
        value,
        format!("`#[env({name} = ...)]` expects a numeric literal"),
    ))
}

fn is_numeric_bound_expr(value: &Expr) -> bool {
    match value {
        Expr::Lit(ExprLit {
            lit: Lit::Int(_) | Lit::Float(_),
            ..
        }) => true,
        Expr::Unary(unary) if matches!(unary.op, UnOp::Neg(_)) => is_numeric_bound_expr(&unary.expr),
        _ => false,
    }
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

fn is_string_ty(ty: &Type) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };

    type_path.path.segments.last().is_some_and(|segment| segment.ident == "String")
}

fn is_numeric_ty(ty: &Type) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };

    type_path.path.segments.last().is_some_and(|segment| {
        matches!(
            segment.ident.to_string().as_str(),
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "f32" | "f64"
        )
    })
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
