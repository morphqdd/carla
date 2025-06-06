use proc_macro::TokenStream;
use syn::__private::quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn main(_: TokenStream, input: TokenStream) -> TokenStream {
    let func = parse_macro_input!(input as syn::ItemFn);
    let ident : syn::Ident = func.sig.ident.clone();
    quote! {
        fn main() -> carla::Result<()> {
            #func
            carla::async_io::executor::block_on(
                async { #ident().await }
            )
        }
    }.into()
}