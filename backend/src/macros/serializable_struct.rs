#[macro_export]
macro_rules! serializable_struct {
    ($struct_name:ident { $($field:tt)* }) => {
        #[derive(serde::Serialize, specta::Type, Clone, Debug)]
        #[serde(rename_all = "camelCase")]
        pub struct $struct_name {
            $($field)*
        }
    };
}
