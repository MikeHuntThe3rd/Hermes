extern crate proc_macro;

use proc_macro::{Ident, TokenStream};
use quote::quote;
use syn::{self, Data, Expr, Fields, Lit, Meta};

#[proc_macro_derive(Bindable, attributes(ids, include_ids))]
pub fn derive_macro_ingress(input: TokenStream) -> TokenStream {
    let parsed_vals = &syn::parse(input).expect("token parsing is expected to succeed");
    return implement_trait(parsed_vals);
}

fn implement_trait(vals: &syn::DeriveInput) -> TokenStream {
    let mut ids_found: bool = false;
    let mut include_flagged: bool = false;
    let mut id_col_names: Vec<String> = vec![];

    let Data::Struct(struct_tree) = &vals.data else {
        panic!("this derive macro only supports structs");
    };

    let Fields::Named(fields) = &struct_tree.fields else {
        panic!("this derive macro only supports named fields");
    };
    
    if &vals.attrs.len() == &(0 as usize) {
        panic!("an attribute list is required");
    }

    for attrib in &vals.attrs {
        if let Meta::List(flag) = &attrib.meta && 
        flag.path.is_ident("exclude_ids") && 
        flag.tokens.to_string().contains("true") {
            include_flagged = true;
            continue;
        }

        if let Meta::NameValue(named) = &attrib.meta && 
        named.path.is_ident("ids") && 
        let Expr::Lit(cols_lit) = &named.value &&
        let Lit::Str(cols_str) = &cols_lit.lit
        {
            id_col_names = cols_str.value().split(";").map(String::from).collect();
            ids_found = true;
        }
    }

    if !ids_found {
        panic!("the id columns must be given");
    }

    let name = &vals.ident;
    let table_name = vals.ident.to_string().to_lowercase() + "s";

    let all_col_idents: Vec<syn::Ident> = fields.named
    .iter()
    .map(|field| field.ident.clone().expect("all fields are expected to be named"))
    .collect();

    let mut id_col_idents: Vec<syn::Ident> = vec![];
    let mut base_col_idents: Vec<syn::Ident> = vec![];

    let mut all_col_names: Vec<String> = vec![];
    let mut base_col_names: Vec<String> = vec![];

    for col_name in &id_col_names {
        if None == all_col_idents.iter().find(|col_ident| col_name.to_string() == col_ident.to_string()) {
            panic!("all given id-s must be valid field names for the struct");
        }
    }

    for col_ident in &all_col_idents {
        if let Some(id_name) = id_col_names.iter().find(|id_name| id_name.to_string() == col_ident.to_string()) {
            id_col_idents.push(col_ident.clone());
            all_col_names.push(col_ident.to_string());

            if include_flagged {
                base_col_idents.push(col_ident.clone());
                base_col_names.push(col_ident.to_string());
            }
        }
        else {
            all_col_names.push(col_ident.to_string());
            base_col_idents.push(col_ident.clone());
            base_col_names.push(col_ident.to_string());
        }
    }

    let static_ids = to_static_helper(id_col_names);
    let static_bases = to_static_helper(base_col_names);
    let static_all = to_static_helper(all_col_names);

   let bind_id = quote! { res #(.bind(&self.#id_col_idents))* };
    let bind_base = quote! { res #(.bind(&self.#base_col_idents))* };
    let bind_all = quote! { res #(.bind(&self.#all_col_idents))* };

    let mut code = quote! {
        impl Bindable for #name {
            fn table_name() -> &'static str {
                return #table_name;
            }

            fn id_columns() -> &'static [&'static str] {
                return &[#(#static_ids), *];
            }

            fn base_columns() -> &'static [&'static str] {
                return &[#(#static_bases), *];
            }

            fn columns() -> &'static [&'static str] {
                return &[#(#static_all), *];
            }

            fn bind_values<'lftm>(
                &'lftm self,
                query: QueryAs<'lftm, Postgres, Self, PgArguments>,
                bind_val: BindVal)
                -> QueryAs<'lftm, Postgres, Self, PgArguments> 
            {
                let mut res = query;

                match bind_val {
                    BindVal::ID => res = #bind_id,
                    BindVal::BASE => res = #bind_base,
                    BindVal::ALL => res = #bind_all,
                }

                return res;
            }
        }
    };

    return code.into();
}

fn to_static_helper(vec: Vec<String>) -> &'static [&'static str] {
    let static_chrs: Vec<&'static str> = vec.into_iter().map(|c| c.leak() as &str).collect();

    return static_chrs.leak();
}