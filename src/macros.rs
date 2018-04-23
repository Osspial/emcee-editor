macro_rules! id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(::uuid::Uuid);

        impl $name {
            #[inline(always)]
            pub fn new() -> $name {
                $name(::uuid::Uuid::new(::uuid::UuidVersion::Random).unwrap())
            }
        }
    }
}
