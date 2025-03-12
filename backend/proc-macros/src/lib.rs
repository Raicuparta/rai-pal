extern crate proc_macro;
extern crate quote;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, parse_macro_input};

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
		impl std::fmt::Display for #enum_name {
			fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
				write!(f, "{:?}", self)
			}
		}
	};

	// Generate the output tokens
	let output = quote! {
		#[derive(serde::Serialize, serde::Deserialize, specta::Type, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Copy, rai_pal_proc_macros::EnumVariants)]
		#input

		#display_impl
	};

	// Convert the output tokens into a TokenStream
	TokenStream::from(output)
}

#[proc_macro_derive(EnumVariants)]
pub fn enum_variants(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let enum_name = &input.ident;

	// Ensure it's an enum and collect variant names
	let variants: Vec<_> = match input.data {
		Data::Enum(DataEnum { variants, .. }) => variants.iter().map(|v| v.ident.clone()).collect(),
		_ => panic!("#[derive(EnumVariants)] can only be used on enums"),
	};

	// Generate the implementation
	let output = quote! {
		impl #enum_name {
			pub fn variants() -> Vec<#enum_name> {
				vec![
					#(Self::#variants),*
				]
			}
		}
	};

	output.into()
}
