use crate::utils::{named_to_vec, unnamed_to_vec};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, quote_spanned};
use syn::{Data, DeriveInput, Field, Fields, Ident, Expr, Type, Token, Result, parse_quote, Variant, Error, Attribute, TypePath, Path, Meta};

/// Provides the hook to expand `#[derive(Default)]` into an implementation of `Default`
pub fn expand(input: &DeriveInput, _: &str) -> Result<TokenStream> {
    let input_type = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let (consts,funcs,body) = match input.data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Named(ref fields) => {
                let field_vec = named_to_vec(fields);
                let default_vec = process_fields(&field_vec)?;

                let consts = struct_consts(&default_vec)?;
                let funcs = struct_functions(&default_vec)?;
                let body = struct_default_body(&default_vec)?;

                (consts,funcs,body)
            },
            Fields::Unnamed(ref fields) => {
                let field_vec = unnamed_to_vec(fields);
                let default_vec = process_fields(&field_vec)?;

                let consts = struct_consts(&default_vec)?;
                let funcs = struct_functions(&default_vec)?;
                let body = unnamed_struct_default_body(&default_vec)?;

                (consts,funcs,body)
            },
            _ => panic!("Only named structs can derive a Default"),
        },
        Data::Enum(ref data_enum) => {
            let mut variant: Option<(&Variant, &Attribute)> = None;

            for ele in &data_enum.variants {
                let attr = find_attr("default", &ele.attrs)?;
                if let Some(attr) = attr {
                    variant = Some((ele, attr));
                    break;
                }
            }

            if let Some((variant, attr)) = variant {

                let var_ident = &variant.ident;

                let mut field: FieldDefault = FieldDefault {
                    ident: Some(variant.ident.clone()),
                    ty: Type::Path(TypePath{ qself: None, path: Path::from(input_type.clone())}),
                    value: parse_quote!{ Self:: #var_ident },
                    field: None,
                    func: None,
                };
                process_attrs(&vec![attr.to_owned()], 0, &mut field)?;
                let default_vec = vec![ field.clone() ];


                let consts = struct_consts(&default_vec)?;
                let funcs = struct_functions(&default_vec)?;

                let value = match (&field.field, &field.func) {
                    (Some(field),_) => quote_spanned!{field.span()=> Self::#field },
                    (_,Some(func)) => quote_spanned!{func.span()=> Self::#func() },
                    _ => (&field.value).into_token_stream(),
                };
                let body = quote!( #value );

                (consts,funcs,body)
            } else {
                panic!("Enums must define a variant as a Default")
            }
        },
        Data::Union(_) => panic!("Can't derive a Default for a Union"),
    };


    Ok(quote! {
        #[allow(missing_docs)]
        #[automatically_derived]
        impl #impl_generics #input_type #ty_generics #where_clause {
            #consts
            #funcs
        }

        #[allow(missing_docs)]
        #[automatically_derived]
        impl #impl_generics Default for #input_type #ty_generics #where_clause {
            #[inline]
            fn default() -> Self {
                #body
            }
        }
    })
}

fn struct_consts(fields: &[FieldDefault]) -> Result<TokenStream> {
    let field_decls = fields.iter().filter_map(|f| {
        let field = match &f.field {
            Some(f) => f,
            None => return None,
        };

        let value = &f.value;

        let ty = &f.ty;

        let out = quote!{
            pub const #field: #ty = #value;
        };

        Some(out)
    });

    Ok(quote! { #(#field_decls)* })
}

fn struct_functions(fields: &[FieldDefault]) -> Result<TokenStream> {
    let func_decls = fields.iter().filter_map(|f| {
        let func = match &f.func {
            Some(f) => f,
            None => return None,
        };

        let value = match &f.field {
            Some(field) => quote_spanned!{field.span()=> Self::#field },
            _ => (&f.value).into_token_stream(),
        };

        let ty = &f.ty;

        let out = quote!{
            pub fn #func () -> #ty { #value }
        };

        Some(out)
    });

    Ok(quote! { #(#func_decls)* })
}

fn struct_default_body(fields: &[FieldDefault]) -> Result<TokenStream> {
    let field_decls = fields.iter().map(|f| {
        let value = match (&f.field, &f.func) {
            (Some(field),_) => quote_spanned!{field.span()=> Self::#field },
            (_,Some(func)) => quote_spanned!{func.span()=> Self::#func() },
            _ => (&f.value).into_token_stream(),
        };

        let ident = f.ident.as_ref().unwrap();

        quote_spanned!{f.ident.as_ref().unwrap().span()=>
            #ident: #value
        }
    });

    Ok(quote! { Self { #(#field_decls),* } })
}

fn unnamed_struct_default_body(fields: &[FieldDefault]) -> Result<TokenStream> {
    let field_decls = fields.iter().map(|f| {
        let value = match (&f.field, &f.func) {
            (Some(field),_) => quote_spanned!{field.span()=> Self::#field },
            (_,Some(func)) => quote_spanned!{func.span()=> Self::#func() },
            _ => (&f.value).into_token_stream(),
        };

        quote!{
            #value
        }
    });

    Ok(quote! { Self ( #(#field_decls),* ) })
}

fn process_fields(fields: &[&Field])-> Result<Vec<FieldDefault>> {
    let vars: Result<Vec<FieldDefault>> = fields.iter().enumerate().map(|(index, f)| {
        let mut out: FieldDefault = FieldDefault {
            ident: f.ident.clone(),
            ty: f.ty.clone(),
            value: parse_quote!{ Default::default() },
            field: None,
            func: None,
        };

        process_attrs(&f.attrs, index, &mut out)?;

        Ok(out)
    }).collect();

    Ok(vars?)
}

fn process_attrs(attrs: &Vec<Attribute>, index: usize, out: &mut FieldDefault) -> Result<()> {
    let attr = find_attr("default", attrs)?;

    if let Some(attr) = attr {
        if matches!(attr.meta, Meta::List(_)) {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("value") {
                    let buf = &meta.value()?;
                    let mut value = buf.parse()?;

                    if let Expr::Lit(expr) = &value {
                        if let syn::Lit::Str(_) | syn::Lit::ByteStr(_) = expr.lit {
                            value = parse_quote!{ (#expr).into() }
                        }
                    };

                    out.value = value
                } else if meta.path.is_ident("constant") {
                    out.field = Some(if meta.input.peek(Token![=]) {
                        meta.value()?.parse()?
                    }else{
                        quote::format_ident!("DEFAULT_{}", out.ident.as_ref().map_or_else(|| format!("{}", index), |i| i.to_string().to_uppercase()))
                    });
                } else if meta.path.is_ident("function") {
                    out.func = Some(if meta.input.peek(Token![=]) {
                        meta.value()?.parse()?
                    }else{
                        quote::format_ident!("default_{}", out.ident.as_ref().map_or_else(|| format!("{}", index), |i| i.to_string().to_lowercase()))
                    });
                }

                Ok(())
            })?;
        }
    }

    Ok(())
}

fn find_attr<'a>(ident: &str, attrs: &'a Vec<Attribute>) -> Result<Option<&'a Attribute>> {
    let mut out: Option<&Attribute> = None;

    for attr in attrs {
        if !attr.path().is_ident(ident) {
            continue;
        }
        if out.is_some() {
            return Err(Error::new(attr.path().get_ident().unwrap().span(), format!("Attribute '{ident}' should be defined only once per field")));
        }

        out = Some(attr);
    }

    Ok(out)
}

#[derive(Clone, Debug)]
struct FieldDefault {
    ident: Option<Ident>,
    ty: Type,
    value: Expr,
    field: Option<Ident>,
    func: Option<Ident>,
}
