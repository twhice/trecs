use proc_macro::TokenStream;
use syn::__private::quote::{format_ident, quote};

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

    std::fs::write("/home/twhice/Desktop/proc.txt", format!("{}", result)).unwrap();
    result.into()
}
