#[macro_export]
macro_rules! serializable_enum {
  ($enum_name:ident { $($variant:ident),* $(,)? }) => {
      #[derive(serde::Serialize, specta::Type, Clone, PartialEq, Eq, Hash, Debug, Copy)]
      pub enum $enum_name {
          $($variant,)*
      }

      impl core::fmt::Display for $enum_name {
          fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
              write!(f, "{:?}", self)
          }
      }
  };
}
