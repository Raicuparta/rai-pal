extern crate proc_macro;
extern crate quote;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn serializable_event(_args: TokenStream, input: TokenStream) -> TokenStream {
	let mut output = TokenStream::from(quote! {
	  #[derive(serde::Serialize, serde::Deserialize, specta::Type, Clone, Debug, tauri_specta::Event)]
	});
	output.extend(input);
	output
}

#[proc_macro_attribute]
pub fn serializable_struct(_args: TokenStream, input: TokenStream) -> TokenStream {
	let mut output = TokenStream::from(quote! {
		#[derive(serde::Serialize, serde::Deserialize, specta::Type, Clone, Debug)]
		#[serde(rename_all = "camelCase")]
	});
	output.extend(input);
	output
}
