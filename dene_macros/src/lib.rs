use proc_macro::TokenStream;

mod refine;

#[proc_macro_derive(Refine, attributes(refine))]
pub fn refine(input: TokenStream) -> TokenStream {
  refine::refine(input)
}
