#[macro_export]
macro_rules! enum_display {
    (
        $(#[$enum_meta:meta])*
         $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident $(,)?
            )*
        }
    ) => {
        #[derive(variant_count::VariantCount)]
        $(#[$enum_meta])*
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )*
        }

        paste::paste! {
            pub const [<$name:snake:upper S>]: [&'static str;$name::VARIANT_COUNT] = [$(stringify!([<$variant:snake:lower>])),*];

            impl $name {
                pub fn as_str(&self) -> &'static str {
                    return match self {
                        $(
                            $name::$variant => stringify!([<$variant:snake:lower>]),
                        )*
                    }
                }
            }

            impl fmt::Display for $name {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    let name = match self {
                        $(
                            $name::$variant => stringify!([<$variant:snake:lower>]),
                        )*
                    };

                    write!(f, "{name}")
                }
            }
            
            impl Program for $name {
                fn to_str(self) -> &'static str {
                    self.as_str()
                }
            }
        }
    };
}
