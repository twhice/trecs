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
    let macro_name = format_ident!("{}", parms[0]);
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

/// 将一个函数转化为系统
///
/// 函数需要满足一一些要求
#[proc_macro_attribute]
pub fn fnsystem(attr: TokenStream, fndef: TokenStream) -> TokenStream {
    assert!(attr.is_empty(), "#[fnsystem]不应带有任何属性");
    let fndef = parse_macro_input!(fndef as syn::ItemFn);
    let fn_name = fndef.sig.ident.clone();

    // 提取参数列表&&进行一些检查
    let args = {
        let sig = &fndef.sig;

        if sig.asyncness.is_some() {
            panic!("#[fnsystem]不可以在异步的函数上使用")
        }
        if sig.generics.lt_token.is_some() {
            panic!("#[fnsystem]不可以在泛型函数上使用")
        }
        if sig.constness.is_some() {
            panic!("#[fnsystem]不可以在常量函数上使用")
        }

        let args = sig.inputs.iter().collect::<Vec<_>>();

        if args
            .iter()
            .any(|arg| matches!(*arg, syn::FnArg::Receiver(_)))
        {
            panic!("#[fnsystem]不可以在Receiver的函数上使用")
        }

        args.into_iter()
            .cloned()
            .map(|arg| {
                let syn::FnArg::Typed(pt)  = arg else {panic!()};
                pt
            })
            .collect::<Vec<_>>()
    };

    let args_tys = args.into_iter().map(|pat| pat.ty);
    let args_tys2 = args_tys.clone();

    let result = quote! {
        fn #fn_name (world : &::tecs::world::World){
            #fndef
            static mut INITED : ::std::cell::OnceCell<()> = ::std::cell::OnceCell::new();
            unsafe{
                INITED.get_or_init(||{
                    let mut state = ::tecs::system::state::SystemState::new();
                    #(<#args_tys2 as ::tecs::system::fnsys::FnSystemParm>::init(&mut state)),*
                });

                #fn_name(#(<#args_tys as ::tecs::system::fnsys::FnSystemParm>::build(&world)),*);
            }
        }
    };
    result.into()
}

#[proc_macro_derive(Bundle)]
pub fn bundle(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident.clone();
    match input.data.clone() {
        syn::Data::Struct(struct_) => {
            // 1 destory
            let tys = struct_.fields.iter().cloned().map(|field| {
                let id = field.ident;
                quote! {
                    #id
                }
            });
            let tys2 = tys.clone();

            let destory = quote! {
                #[allow(non_snake_case)]
                fn destory(self) -> ::tecs::bundle::Components{
                    let #struct_name {#(#tys,)*} = self;
                    vec![#(Box::new(#tys2))*,]
                }
            };

            // 2 components_ids
            let components_ids = struct_.fields.clone().into_iter().map(|field| {
                let ty = field.ty;
                quote! {
                    <#ty as ::tecs::bundle::Component>::type_id_()
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

            let result = quote! {
                // #input
                impl ::tecs::bundle::Bundle for #struct_name{
                    #destory
                    #components_ids
                    #type_name
                    #type_id_
                }
            };
            std::fs::write("/home/twhice/Desktop/proc.txt", format!("{}", result)).unwrap();

            result.into()
        }
        _ => panic!("Bundle仅支持为结构体实现"),
    }
}

#[proc_macro_derive(Component)]
pub fn component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let type_name = input.ident.clone();

    quote! {
        // #input
        impl ::tecs::bundle::Component for #type_name{
            fn type_id_() -> ::std::any::TypeId{
                ::std::any::TypeId::of::<Self>()
            }
        }
    }
    .into()
}
