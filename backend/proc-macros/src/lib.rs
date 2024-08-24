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

#[proc_macro_attribute]
pub fn serializable_enum(_args: TokenStream, input: TokenStream) -> TokenStream {
	// Parse the input tokens into a syntax tree
	let input = syn::parse_macro_input!(input as syn::ItemEnum);

	// Get the enum name
	let enum_name = &input.ident;

	// Generate the impl Display block
	let display_impl = quote! {
		impl core::fmt::Display for #enum_name {
			fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
				write!(f, "{:?}", self)
			}
		}
	};

	// Generate the output tokens
	let output = quote! {
		#[derive(serde::Serialize, serde::Deserialize, specta::Type, Clone, PartialEq, Eq, Hash, Debug, Copy)]
		#input

		#display_impl
	};

	// Convert the output tokens into a TokenStream
	TokenStream::from(output)
}
