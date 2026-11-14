use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn serde_enum(_args: TokenStream, input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as syn::ItemEnum);
	let enum_name = &input.ident;
	let variant_count = input.variants.len();
	let variant_names = input.variants.iter().map(|variant| &variant.ident);

	let all_impl = quote! {
		impl #enum_name {
			pub const ALL: [Self; #variant_count] = [#(Self::#variant_names),*];
		}
	};

	let output = quote! {
		#[derive(
			serde::Serialize,
			serde::Deserialize,
			Clone,
			PartialEq,
			Eq,
			PartialOrd,
			Ord,
			Hash,
			Debug,
			Copy,
			strum::Display,
			strum::EnumString,
			strum::EnumIter,
		)]
		#input

		#all_impl
	};

	TokenStream::from(output)
}

#[proc_macro_attribute]
pub fn sql_enum(_args: TokenStream, input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as syn::ItemEnum);
	let enum_name = &input.ident;

	let sql_impl = quote! {
		impl rusqlite::types::ToSql for #enum_name {
			fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
				Ok(self.to_string().into())
			}
		}

		impl rusqlite::types::FromSql for #enum_name {
			fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
				value.as_str()?.parse()
					.map_err(|e| rusqlite::types::FromSqlError::Other(Box::new(e)))
			}
		}
	};

	let output = quote! {
		#input
		#sql_impl
	};

	TokenStream::from(output)
}
