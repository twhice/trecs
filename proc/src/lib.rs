use proc_macro::TokenStream;
use syn::{
    __private::quote::{format_ident, quote},
    parse_macro_input, DeriveInput,
};

/// 方便快捷地为元组类型实现某特征
///
/// 实现类似C艹变长模板参数的功能
#[proc_macro]
pub fn all_tuple(input: TokenStream) -> TokenStream {
    let input = input.to_string();
    let parms = input.split(',').collect::<Vec<_>>();
    let macro_name = format_ident!("{}", parms[0].trim());
    let num = parms[1].trim().parse::<usize>().unwrap();
    let mut result = quote!();

    let mut idents = Vec::with_capacity(num);

    for i in 0..num {
        idents.push(format_ident!("T{i}"));
    }

    for i in 0..num {
        let idents = idents[..=i].iter();
        result.extend(quote! {
            #macro_name!{#(#idents),*}
        })
    }

    result.into()
}

#[proc_macro_derive(Bundle)]
pub fn bundle(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;
    match input.data {
        syn::Data::Struct(struct_) => {
            // 1 destory
            let idents = struct_.fields.iter().cloned().map(|field| {
                let ident = field.ident;
                quote! {
                    #ident
                }
            });
            let idents2 = idents.clone();

            let destory = quote! {
                #[allow(non_snake_case)]
                fn destory(self) -> ::trecs::bundle::Components{
                    let #struct_name {#(#idents,)*} = self;
                    vec![#(Box::new(#idents2))*,]
                }
            };

            // 2 components_ids
            let components_ids = struct_.fields.clone().into_iter().map(|field| {
                let ty = field.ty;
                quote! {
                    <#ty as ::trecs::bundle::Component>::type_id_()
                }
            });
            let components_ids = quote! {
                fn components_ids() -> &'static [::std::any::TypeId]{
                    static mut COMPONNETS_IDS :
                        ::std::cell::OnceCell<Vec<::std::any::TypeId>>
                        = ::std::cell::OnceCell::new();
                    unsafe{
                        COMPONNETS_IDS.get_or_init(||{
                            vec![#(#components_ids,)*]
                        })
                    }
                }
            };
            // 3 type_name
            let type_name = quote! {
                fn type_name() -> &'static str{
                    ::std::any::type_name::<Self>()
                }
            };
            // 4 type_id_
            let type_id_ = quote! {
                fn type_id_() -> std::any::TypeId{
                    ::std::any::TypeId::of::<Self>()
                }
            };

            // droper
            let tys = struct_.fields.iter().cloned().map(|field| field.ty);

            let generator = struct_.fields.iter().scan(0, |state, _| {
                *state += 1;
                Some(format_ident!("T{}", state.to_string()))
            });
            let drop = quote! {
                fn drop(cs : ::trecs::bundle::Components) {
                    let mut iter = cs.into_iter().rev();
                    #(let #generator = iter.next().unwrap().downcast::<#tys>();)*
                }
            };

            let result = quote! {
                // #input
                #[allow(non_snake_case)]
                impl ::trecs::bundle::Bundle for #struct_name{
                    #destory
                    #components_ids
                    #drop
                    #type_name
                    #type_id_
                }
            };

            result.into()
        }
        _ => panic!("Bundle仅支持为结构体实现"),
    }
}

#[proc_macro_derive(Component)]
pub fn component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let type_name = input.ident;

    quote! {
        // #input
        impl ::trecs::bundle::Component for #type_name{
            fn type_id_() -> ::std::any::TypeId{
                ::std::any::TypeId::of::<Self>()
            }
        }
    }
    .into()
}
