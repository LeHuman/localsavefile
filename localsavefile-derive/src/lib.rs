extern crate proc_macro;

use darling::{ast::NestedMeta, FromMeta};
use proc_macro2::TokenStream;
use quote::quote;
use sanitize_filename::sanitize;
use syn::{
    punctuated::Punctuated, token::Comma, Data, DeriveInput, Fields, GenericParam, Generics, Ident,
    ImplGenerics, WhereClause,
};

#[derive(Default, Debug, FromMeta)]
struct LSFArgs {
    version: Option<u32>,
    path: Option<String>,
    name: Option<String>,
    persist: Option<bool>,
}

fn impl_default(
    input: &DeriveInput,
    generics: &ImplGenerics,
    name: &TokenStream,
    extra_where: &TokenStream,
    impl_common: &proc_macro2::TokenStream,
) -> proc_macro::TokenStream {
    // TODO: Add Savefile onto attrs
    quote! {
        #[derive(::savefile::prelude::Savefile)]
        #input

        #impl_common

        impl #generics ::localsavefile::LocalSaveFile for #name #extra_where {}
    }
    .into()
}

fn impl_persistent(
    input: &DeriveInput,
    generics: &ImplGenerics,
    name: &TokenStream,
    extra_where: &TokenStream,
    impl_common: &proc_macro2::TokenStream,
) -> proc_macro::TokenStream {
    let attrs = &input.attrs;

    match &input.data {
        Data::Struct(data_struct) => {
            let fields = &data_struct.fields;
            let additional_field = quote! {
                #[savefile_ignore]
                #[savefile_introspect_ignore]
                __place_localsavefile_above_any_derives: ::localsavefile::LocalSaveFileMetaData,
            };

            let new_fields = match fields {
                Fields::Named(ref named_fields) => {
                    let named = &named_fields.named;
                    let named = named.iter().collect::<Punctuated<_, Comma>>();
                    quote! {
                        {
                            #named,
                            #additional_field
                        }
                    }
                }
                Fields::Unnamed(ref unnamed_fields) => {
                    let unnamed = &unnamed_fields.unnamed;
                    let unnamed = unnamed.iter().collect::<Punctuated<_, Comma>>();
                    quote! {
                        (
                            #unnamed,
                            #additional_field
                        )
                    }
                }
                Fields::Unit => {
                    quote! {
                        {
                            #additional_field
                        }
                    }
                }
            };

            let struct_attrs = attrs.iter();

            // TODO: Add Savefile onto attrs
            quote! {
                #(#struct_attrs)*
                #[derive(::savefile::prelude::Savefile)]
                struct #name #new_fields

                #impl_common

                impl #generics ::localsavefile::LocalSaveFilePersistent for #name #extra_where{
                    fn get_file_handle_mut(&mut self) -> &mut Option<std::fs::File> {
                        &mut self.__place_localsavefile_above_any_derives.file
                    }
                }
            }
        }
        _ => unimplemented!("LocalSaveFile can only be used with structs"),
    }
    .into()
}

// https://github.com/avl/savefile/blob/1cce218a9fce5ee328e6ed0c77e020e53ef8e8d5/savefile-derive/src/common.rs#L6
pub(crate) fn get_extra_where_clauses(
    gen2: &Generics,
    where_clause: Option<&WhereClause>,
    the_trait: TokenStream,
) -> TokenStream {
    let extra_where_separator;
    if let Some(where_clause) = where_clause {
        if where_clause.predicates.trailing_punct() {
            extra_where_separator = quote!();
        } else {
            extra_where_separator = quote!(,);
        }
    } else {
        extra_where_separator = quote!(where);
    }
    let mut where_clauses = vec![];
    for param in gen2.params.iter() {
        if let GenericParam::Type(t) = param {
            let t_name = &t.ident;
            let clause = quote! {#t_name : #the_trait};
            where_clauses.push(clause);
        }
    }
    let extra_where = quote! {
        #extra_where_separator #(#where_clauses),*
    };
    extra_where
}

#[proc_macro_attribute]
pub fn localsavefile(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse_macro_input!(input);
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return proc_macro::TokenStream::from(darling::Error::from(e).write_errors());
        }
    };
    let args = match LSFArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return proc_macro::TokenStream::from(e.write_errors());
        }
    };

    let name: &Ident = &input.ident;
    let name_str = sanitize(name.to_string());
    let version = args.version.unwrap_or(0);
    let path: Option<String> = args.path;
    let persist = args.persist.unwrap_or(false);
    let struct_name = args.name;

    let get_dir_path = match path {
        Some(path) => quote! {
            fn get_dir_path() -> ::std::io::Result<::std::path::PathBuf> {
                Ok(::std::path::PathBuf::from(#path))
            }
        },
        None => quote! {},
    };

    let struct_name = if let Some(struct_name) = struct_name {
        let struct_name = sanitize(struct_name);
        quote! {#struct_name.to_string()}
    } else {
        quote! {
            let mut s = module_path!().replace("::", ".") + "." + #name_str;
            s.make_ascii_lowercase();
            s.retain(|c| !c.is_whitespace());
            ::localsavefile::sanitize(s)
        }
    };

    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let extra_where = get_extra_where_clauses(
        generics,
        where_clause,
        quote! {::savefile::prelude::Packed + ::std::default::Default + ::savefile::Serialize + ::savefile::Deserialize},
    );
    let for_name = quote! {#name #ty_generics #where_clause};

    let impl_common = quote! {
        impl #impl_generics ::localsavefile::LocalSaveFileCommon for #for_name #extra_where {
            fn get_version() -> u32 {
                #version
            }

            fn get_struct_name() -> String {
                #struct_name
            }

            fn get_pkg_name() -> String {
                let mut s = env!("CARGO_PKG_NAME").to_string();
                s.make_ascii_lowercase();
                s.retain(|c| !c.is_whitespace());
                ::localsavefile::sanitize(s)
            }

            fn get_pkg_author() -> String {
                let mut s = env!("CARGO_PKG_AUTHORS").split(',').collect::<Vec<&str>>()[0].to_string();
                s.make_ascii_lowercase();
                s.retain(|c| !c.is_whitespace());
                ::localsavefile::sanitize(s)
            }

            #get_dir_path
        }
    };

    if persist {
        impl_persistent(
            &input,
            &impl_generics,
            &for_name,
            &extra_where,
            &impl_common,
        )
    } else {
        impl_default(
            &input,
            &impl_generics,
            &for_name,
            &extra_where,
            &impl_common,
        )
    }
}
