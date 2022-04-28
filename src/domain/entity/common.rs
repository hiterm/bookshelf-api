#[macro_export]
macro_rules! impl_string_value_object {
    ( $type:tt ) => {
        impl $type {
            pub fn new(id: String) -> Result<Self, DomainError> {
                let object = Self { value: id };
                object.validate()?;
                Ok(object)
            }

            pub fn as_str(&self) -> &str {
                &self.value
            }

            pub fn into_string(self) -> String {
                self.value
            }
        }
    };
}
