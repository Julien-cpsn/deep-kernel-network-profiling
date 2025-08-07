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
            pub const [<$name:snake:upper S>]: [&'static str;$name::VARIANT_COUNT] = [
                $(
                    $(#[$variant_meta])*
                    stringify!([<$variant:snake:lower>])
                ),*
            ];
            pub const [<$name:snake:upper _VARIANTS>]: [$name;$name::VARIANT_COUNT] = [
                $(
                    $(#[$variant_meta])*
                    $name::$variant
                ),*
            ];

            impl $name {
                pub fn as_str(&self) -> &'static str {
                    return match self {
                        $(
                            $(#[$variant_meta])*
                            $name::$variant => stringify!([<$variant:snake:lower>]),
                        )*
                        _ => unreachable!()
                    }
                }

                pub fn as_id(&self) -> u16 {
                    *self as u16
                }

                pub fn from_id(id: u16) -> $name {
                    return match id {
                        $(
                            $(#[$variant_meta])*
                            id if id == $name::$variant as u16 => $name::$variant,
                        )*
                        _ => unreachable!()
                    }
                }
            }

            impl fmt::Display for $name {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    let name = match self {
                        $(
                            $(#[$variant_meta])*
                            $name::$variant => stringify!([<$variant:snake:lower>]),
                        )*
                        _ => unreachable!()
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
    (
        $(#[$enum_meta:meta])*
         $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident => $lib_path:expr $(,)?
            )*
        }
    ) => {
        enum_display! {
            $(#[$enum_meta])*
            $vis enum $name {
                $(
                    $(#[$variant_meta])*
                    $variant,
                )*
            }
        }

        impl $name {
            pub fn get_lib_path(self) -> &'static str {
                match self {
                    $(
                        $(#[$variant_meta])*
                        $name::$variant => $lib_path,
                    )*
                    _ => unreachable!()
                }
            }
        }
    }
}
