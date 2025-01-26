#[macro_export]
macro_rules! derive_schema {
    () => {};

    ($(#[$meta:meta])* $vis:vis struct $name:ident;) => {
        $(#[$meta])*
        $vis struct $name;

        const _: () = {
            struct Expression(Vec<u32>);

            impl $crate::Expression for Expression {
                type Target = $name;

                async fn write(self, write: &mut (impl ::tokio::io::AsyncWriteExt + ::std::marker::Unpin)) -> ::std::io::Result<()> {
                    write.write_u8($crate::expression_discriminant::PATH).await?;
                    write
                        .write_u32(self.0.len().try_into().map_err(|_| {
                            ::std::io::Error::new(
                                ::std::io::ErrorKind::OutOfMemory,
                                ::std::concat!(::std::env!("CARGO_CRATE_NAME"), ": ", "path expression length doesn't fit into a 32 bit unsigned integer")
                            )
                        })?)
                        .await?;

                    for segment in &self.0 {
                        write.write_u32(*segment).await?;
                    }

                    Ok(())
                }
            }

            impl $crate::FromPath for Expression {
                fn from_path(path: ::std::vec::Vec<u32>) -> Self {
                    Self(path)
                }
            }

            impl $crate::Schema for $name {
                type Expression = Expression;

                fn write_schema(
                    write: &mut (impl ::tokio::io::AsyncWriteExt + ::std::marker::Unpin + ::std::marker::Send),
                ) -> impl ::std::future::Future<Output = ::std::io::Result<()>> + ::std::marker::Send {
                    write.write_u8($crate::schema_discriminant::UNIT)
                }

                fn write_value(
                    &self,
                    _write: &mut (impl ::tokio::io::AsyncWriteExt + ::std::marker::Unpin + ::std::marker::Send),
                ) -> impl ::std::future::Future<Output = ::std::io::Result<()>> + ::std::marker::Send {
                    ::std::future::ready(Ok(()))
                }

                fn read_value(
                    _read: &mut (impl ::tokio::io::AsyncReadExt + ::std::marker::Unpin + ::std::marker::Send),
                ) -> impl ::std::future::Future<Output = ::std::io::Result<Self>> + ::std::marker::Send {
                    ::std::future::ready(Ok(Self))
                }
            }
        };
    };

    ($(#[$meta:meta])* $vis:vis struct $name:ident { $($(#[$field_meta:meta])* $field_vis:vis $field:ident: $type:ty),* $(,)? } $($tail:tt)*) => {
        $(#[$meta])*
        $vis struct $name {
            $($(#[$field_meta])* $field_vis $field: $type,)*
        }

        const _: () = {
            struct Expression {
                $($(#[$field_meta])* $field_vis $field: <$type as $crate::Schema>::Expression,)*
                __internal_path: Vec<u32>,
            }

            impl $crate::Expression for Expression {
                type Target = $name;

                async fn write(self, write: &mut (impl ::tokio::io::AsyncWriteExt + ::std::marker::Unpin)) -> ::std::io::Result<()> {
                    write.write_u8($crate::expression_discriminant::PATH).await?;
                    write
                        .write_u32(self.__internal_path.len().try_into().map_err(|_| {
                            ::std::io::Error::new(
                                ::std::io::ErrorKind::OutOfMemory,
                                ::std::concat!(::std::env!("CARGO_CRATE_NAME"), ": ", "path expression length doesn't fit into a 32 bit unsigned integer")
                            )
                        })?)
                        .await?;

                    for segment in &self.__internal_path {
                        write.write_u32(*segment).await?;
                    }

                    Ok(())
                }
            }

            impl $crate::FromPath for Expression {
                fn from_path(path: ::std::vec::Vec<u32>) -> Self {
                    let mut i = 0;

                    #[allow(unused_assignments)]
                    Self {
                        $($field: <$type as $crate::Schema>::Expression::from_path((path.iter().copied().chain([i]).collect(), i += 1).0),)*
                        __internal_path: path,
                    }
                }
            }

            impl $crate::Schema for $name {
                type Expression = Expression;

                fn write_schema(
                    write: &mut (impl ::tokio::io::AsyncWriteExt + ::std::marker::Unpin + ::std::marker::Send),
                ) -> impl ::std::future::Future<Output = ::std::io::Result<()>> + ::std::marker::Send {
                    async {
                        write.write_u8($crate::schema_discriminant::PRODUCT).await?;

                        write.write_u32(0 $(+ {
                            #[allow(unused_variables)] let $field = ();
                            1
                        })*).await?;

                        $(<$type as $crate::Schema>::write_schema(write).await?;)*

                        Ok(())
                    }
                }

                fn write_value(
                    &self,
                    write: &mut (impl ::tokio::io::AsyncWriteExt + ::std::marker::Unpin + ::std::marker::Send),
                ) -> impl ::std::future::Future<Output = ::std::io::Result<()>> + ::std::marker::Send {
                    async {
                        $(self.$field.write_value(write).await?;)*
                        Ok(())
                    }
                }

                fn read_value(
                    read: &mut (impl ::tokio::io::AsyncReadExt + ::std::marker::Unpin + ::std::marker::Send),
                ) -> impl ::std::future::Future<Output = ::std::io::Result<Self>> + ::std::marker::Send {
                    async {
                        Ok(Self {
                            $($field: <$type as $crate::Schema>::read_value(read).await?,)*
                        })
                    }
                }
            }
        };

        $crate::derive_schema!($($tail)*);
    };
}
