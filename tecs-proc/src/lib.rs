use proc_macro::TokenStream;
use syn::{
    parse_macro_input, DeriveInput, Fields, Ident,
    __private::quote::{format_ident, quote},
};

#[proc_macro_derive(Bundle)]
pub fn derive_bundle(tokens: TokenStream) -> TokenStream {
    let _tokens = tokens.clone();
    let input = parse_macro_input!(_tokens as DeriveInput);
    let type_name = input.ident;

    match input.data {
        syn::Data::Struct(data) => {
            let ts = derive_bundle_inner(type_name, data.fields);
            std::fs::write("/home/twhice/Desktop/expand.rs", ts.to_string()).unwrap();
            ts
        }

        _ => panic!("Bundle 仅对Struct有效"),
    }
}

const NR_TUPLE: usize = 10;

#[proc_macro]
pub fn derive_bundle_for_tuple(_: TokenStream) -> TokenStream {
    let mut all = quote!();
    for n in 1..NR_TUPLE {
        let mut tuple_ty = quote!();
        let mut tuple_bundle = quote!();
        let mut into_boxs = quote!();
        let mut types_ids_expr = quote!();

        for i in 0..n {
            let id = format_ident!("T{i}");
            tuple_ty.extend(quote! {
                #id,
            });
            tuple_bundle.extend(quote! {
                #id : Component,
            });

            into_boxs.extend(quote! {
                Box::new(#id),
            });
            types_ids_expr.extend(quote! {
                {<#id as Component>::COMPONENT_FLAG;TypeId::of::<#id>()},
            });
        }
        // 元组的类型
        let tuple_ty = quote! {(#tuple_ty)};
        // 元组的类型约束
        let tuple_bundle = quote! {#tuple_bundle};
        // 解构表达式
        let deconstruct = quote! {
            let #tuple_ty = self;
        };
        let into_boxs = quote! { vec![#into_boxs]};
        // mod名字
        let mod_name = format_ident!("impl_bundle_for_tuple{n}");

        let impl_ = quote! {
                #[rustfmt::skip]
                #[allow(non_snake_case)]
                mod #mod_name{
                    use crate::component::Component;
                    use crate::component::bundle::Bundle;
                    use ::std::any::{TypeId,Any};
                    static mut TYPES_IDS : Option<Vec<TypeId>> = None;
                    impl<#tuple_bundle> Bundle for #tuple_ty{
                        #[inline]
                        fn conponents_ids() -> &'static [TypeId] {
                            unsafe{
                                if TYPES_IDS.is_none() {
                                    TYPES_IDS = Some(vec![#types_ids_expr]);
                                }
                                &*(&TYPES_IDS as *const _ as *const Vec<TypeId>)
                            }
                        }
                        #[inline]
                        fn deconstruct(self) -> Vec<Box<dyn Any>> {
                            #deconstruct
                            #into_boxs
                        }
                    }
        }
        };
        all.extend(impl_);
    }
    std::fs::write("/home/twhice/Desktop/expand.rs", all.to_string()).unwrap();
    all.into()
}

fn derive_bundle_inner(type_name: Ident, fields: Fields) -> TokenStream {
    let mut components_ids = quote!();
    let mut deconstruct = quote!();
    for fields in fields {
        let ty = fields.ty;
        let ident = fields.ident;
        components_ids.extend(quote! {
            {<#ty as Component>::COMPONENT_FLAG;TypeId::of::<#ty>()},
        });
        deconstruct.extend(quote! {
            Box::new(self.#ident) as Box<dyn Any>,
        });
    }
    let mod_name = format_ident!("__impl_bundle_for_{}", type_name);

    quote!(
        #[rustfmt::skip]
        #[allow(non_snake_case)]
        mod #mod_name{
            use crate::component::Component;
            use crate::component::bundle::Bundle;
            use ::std::any::{TypeId,Any};
            static mut TYPES_IDS : Option<Vec<TypeId>> = None;
            impl Bundle for super::#type_name{
                #[inline]
                fn conponents_ids() -> &'static [TypeId] {
                    unsafe{
                        if TYPES_IDS.is_none() {
                            TYPES_IDS = Some(vec![#components_ids]);
                        }
                        &*(&TYPES_IDS as *const _ as *const Vec<TypeId>)
                    }
                }
                #[inline]
                fn deconstruct(self) -> Vec<Box<dyn Any>> {
                    vec![#deconstruct]
                }
            }
        }
    )
    .into()
}
