use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_macro_input, Data, DataEnum, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Ident,
    Index, VisRestricted, Visibility,
};

fn parent_visibility<'a>(vis: Visibility) -> Box<dyn ToTokens> {
    match vis {
        Visibility::Public(_) => Box::new(vis),
        Visibility::Restricted(VisRestricted {
            pub_token,
            in_token,
            path,
            ..
        }) => Box::new(if let Some(in_token) = in_token {
            quote! { #pub_token(#in_token super::#path) }
        } else {
            quote! { #pub_token(in super::#path) }
        }),
        Visibility::Inherited => Box::new(quote!(pub(super))),
    }
}

#[proc_macro_derive(Schema)]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let vis = input.vis;

    let output = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => derive_struct_named(vis, name, fields),
            Fields::Unnamed(fields) => derive_struct_unnamed(vis, name, fields),
            Fields::Unit => derive_struct_unit(name),
        },
        Data::Enum(data) => derive_enum(name, data),
        Data::Union(_) => panic!("Cannot derive Schema for union"),
    };

    TokenStream::from(output)
}

fn derive_struct_named(
    vis: Visibility,
    name: Ident,
    input: FieldsNamed,
) -> proc_macro2::TokenStream {
    let parent_vis = parent_visibility(vis);

    let fields = input.named;

    let field_names = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap())
        .collect::<Vec<_>>();

    let field_types = fields.iter().map(|field| &field.ty).collect::<Vec<_>>();
    let field_count: u32 = fields.len().try_into().unwrap();
    let field_indexes = 0..field_count;

    quote! {
        const _: () = {
            mod __internal {
                use super::*;

                 #parent_vis struct Expression {
                    #(pub #field_names: <#field_types as ::database::Schema>::Expression,)*
                    __internal_path: Vec<u32>,
                }

                impl ::database::Expression for Expression {
                    type Target = super::#name;

                    async fn write<
                        S: ::database::__internal::tokio::io::AsyncWriteExt + ::database::__internal::tokio::io::AsyncReadExt + ::std::marker::Unpin,
                        Next: ::database::protocol::FromStream<S>
                    >(
                        self,
                        write: ::database::protocol::ExpressionWrite<S, Next>,
                    ) -> ::std::io::Result<Next> {
                        write.path(&self.__internal_path).await
                    }
                }

                impl ::database::FromPath for Expression {
                    fn from_path(path: ::std::vec::Vec<u32>) -> Self {
                        Self {
                            #(#field_names: <#field_types as ::database::Schema>::Expression::from_path(path.iter().copied().chain([#field_indexes]).collect()),)*
                            __internal_path: path,
                        }
                    }
                }

                impl ::database::Schema for super::#name {
                    type Expression = Expression;

                    fn write_schema<
                        S: ::database::__internal::tokio::io::AsyncWriteExt + ::database::__internal::tokio::io::AsyncReadExt + ::std::marker::Unpin,
                        Next: ::database::protocol::FromStream<S>
                    >(
                        write: ::database::protocol::SchemaWrite<S, Next>,
                    ) -> impl ::std::future::Future<Output = ::std::io::Result<Next>> {
                        async {
                            let fields_write = write.product(#field_count).await?;
                            #(let fields_write = <#field_types as ::database::Schema>::write_schema(fields_write.add()).await?;)*

                            Ok(fields_write.finish())
                        }
                    }

                    fn write_value<
                        S: ::database::__internal::tokio::io::AsyncWriteExt + ::database::__internal::tokio::io::AsyncReadExt + ::std::marker::Unpin,
                        Next: ::database::protocol::FromStream<S>
                    >(
                        &self,
                        write: ::database::protocol::ValueWrite<S, Next>,
                    ) -> impl ::std::future::Future<Output = ::std::io::Result<Next>> {
                        async {
                            let fields_write = write.product();
                            #(let fields_write = self.#field_names.write_value(fields_write.add()).await?;)*

                            Ok(fields_write.finish())
                        }
                    }

                    fn read_value<
                        S: ::database::__internal::tokio::io::AsyncWriteExt + ::database::__internal::tokio::io::AsyncReadExt + ::std::marker::Unpin,
                        Next: ::database::protocol::FromStream<S>
                    >(
                        read: ::database::protocol::ValueRead<S, Next>,
                    ) -> impl ::std::future::Future<Output = ::std::io::Result<(Self, Next)>> {
                        async {
                            let mut fields_read = read.product();

                            Ok((
                                Self {
                                    #(#field_names: {
                                        let (value, next) = <#field_types as ::database::Schema>::read_value(fields_read.next()).await?;
                                        fields_read = next;
                                        value
                                    },)*
                                },
                                fields_read.finish(),
                            ))
                        }
                    }
                }
            }
        };
    }
}

fn derive_struct_unnamed(
    vis: Visibility,
    name: Ident,
    input: FieldsUnnamed,
) -> proc_macro2::TokenStream {
    let fields = input.unnamed;

    let parent_vis = parent_visibility(vis);

    let field_types = fields.iter().map(|field| &field.ty).collect::<Vec<_>>();
    let field_count: u32 = fields.len().try_into().unwrap();
    let path_index = Index {
        index: field_count,
        span: Span::call_site(),
    };
    let field_numbers = 0..field_count;
    let field_indexes = (0..field_count).map(|i| Index {
        index: i as u32,
        span: Span::call_site(),
    });

    quote! {
        const _: () = {
            mod __internal {
                use super::*;

                 #parent_vis struct Expression(
                    #(pub <#field_types as ::database::Schema>::Expression,)*
                    Vec<u32>,
                );

                impl ::database::Expression for Expression {
                    type Target = super::#name;

                    async fn write<
                        S: ::database::__internal::tokio::io::AsyncWriteExt + ::database::__internal::tokio::io::AsyncReadExt + ::std::marker::Unpin,
                        Next: ::database::protocol::FromStream<S>
                    >(
                        self,
                        write: ::database::protocol::ExpressionWrite<S, Next>,
                    ) -> ::std::io::Result<Next> {
                        write.path(&self.#path_index).await
                    }
                }

                impl ::database::FromPath for Expression {
                    fn from_path(path: ::std::vec::Vec<u32>) -> Self {
                        Self (
                            #(<#field_types as ::database::Schema>::Expression::from_path(path.iter().copied().chain([#field_numbers]).collect()),)*
                            path,
                        )
                    }
                }

                impl ::database::Schema for super::#name {
                    type Expression = Expression;

                    fn write_schema<
                        S: ::database::__internal::tokio::io::AsyncWriteExt + ::database::__internal::tokio::io::AsyncReadExt + ::std::marker::Unpin,
                        Next: ::database::protocol::FromStream<S>
                    >(
                        write: ::database::protocol::SchemaWrite<S, Next>,
                    ) -> impl ::std::future::Future<Output = ::std::io::Result<Next>> {
                        async {
                            let fields_write = write.product(#field_count).await?;
                            #(let fields_write = <#field_types as ::database::Schema>::write_schema(fields_write.add()).await?;)*

                            Ok(fields_write.finish())
                        }
                    }

                    fn write_value<
                        S: ::database::__internal::tokio::io::AsyncWriteExt + ::database::__internal::tokio::io::AsyncReadExt + ::std::marker::Unpin,
                        Next: ::database::protocol::FromStream<S>
                    >(
                        &self,
                        write: ::database::protocol::ValueWrite<S, Next>,
                    ) -> impl ::std::future::Future<Output = ::std::io::Result<Next>> {
                        async {
                            let fields_write = write.product();
                            #(let fields_write = self.#field_indexes.write_value(fields_write.add()).await?;)*

                            Ok(fields_write.finish())
                        }
                    }

                    fn read_value<
                        S: ::database::__internal::tokio::io::AsyncWriteExt + ::database::__internal::tokio::io::AsyncReadExt + ::std::marker::Unpin,
                        Next: ::database::protocol::FromStream<S>
                    >(
                        read: ::database::protocol::ValueRead<S, Next>,
                    ) -> impl ::std::future::Future<Output = ::std::io::Result<(Self, Next)>> {
                        async {
                            let mut fields_read = read.product();

                            Ok((
                                Self (
                                    #({
                                        let (value, next) = <#field_types as ::database::Schema>::read_value(fields_read.next()).await?;
                                        fields_read = next;
                                        value
                                    },)*
                                ),
                                fields_read.finish(),
                            ))
                        }
                    }
                }
            }
        };
    }
}

fn derive_struct_unit(name: Ident) -> proc_macro2::TokenStream {
    quote! {
        const _: () = {
            struct Expression(Vec<u32>);

            impl ::database::Expression for Expression {
                type Target = #name;

                    async fn write<
                        S: ::database::__internal::tokio::io::AsyncWriteExt + ::database::__internal::tokio::io::AsyncReadExt + ::std::marker::Unpin,
                        Next: ::database::protocol::FromStream<S>
                    >(
                        self,
                        write: ::database::protocol::ExpressionWrite<S, Next>,
                    ) -> ::std::io::Result<Next> {
                        write.path(&self.0).await
                    }
            }

            impl ::database::FromPath for Expression {
                fn from_path(path: ::std::vec::Vec<u32>) -> Self {
                    Self(path)
                }
            }

            impl ::database::Schema for #name {
                type Expression = Expression;

                fn write_schema<
                    S: ::database::__internal::tokio::io::AsyncWriteExt + ::database::__internal::tokio::io::AsyncReadExt + ::std::marker::Unpin,
                    Next: ::database::protocol::FromStream<S>
                >(
                    write: ::database::protocol::SchemaWrite<S, Next>,
                ) -> impl ::std::future::Future<Output = ::std::io::Result<Next>> {
                    write.unit()
                }

                fn write_value<
                    S: ::database::__internal::tokio::io::AsyncWriteExt + ::database::__internal::tokio::io::AsyncReadExt + ::std::marker::Unpin,
                    Next: ::database::protocol::FromStream<S>
                >(
                    &self,
                    write: ::database::protocol::ValueWrite<S, Next>,
                ) -> impl ::std::future::Future<Output = ::std::io::Result<Next>> {
                    write.unit()
                }

                fn read_value<
                    S: ::database::__internal::tokio::io::AsyncWriteExt + ::database::__internal::tokio::io::AsyncReadExt + ::std::marker::Unpin,
                    Next: ::database::protocol::FromStream<S>
                >(
                    read: ::database::protocol::ValueRead<S, Next>,
                ) -> impl ::std::future::Future<Output = ::std::io::Result<(Self, Next)>> {
                    async { Ok((Self, read.unit().await?)) }
                }
            }
        };
    }
}

fn derive_enum(name: Ident, data: DataEnum) -> proc_macro2::TokenStream {
    let variant_count = data.variants.len() as u32;

    let write_schemas = data.variants.iter().map(|variant| match &variant.fields {
        Fields::Named(fields) if fields.named.len() != 0 => {
            let field_count = fields.named.len() as u32;
            let field_types = fields.named.iter().map(|field| &field.ty);

            quote! {
                write.write_u8(::database::schema_discriminant::PRODUCT).await?;
                write.write_u32(#field_count).await?;
                #(<#field_types as ::database::Schema>::write_schema(write).await?;)*
            }
        }
        Fields::Unnamed(fields) if fields.unnamed.len() != 0 => {
            let field_count = fields.unnamed.len() as u32;
            let field_types = fields.unnamed.iter().map(|field| &field.ty);

            quote! {
                write.write_u8(::database::schema_discriminant::PRODUCT).await?;
                write.write_u32(#field_count).await?;
                #(<#field_types as ::database::Schema>::write_schema(write).await?;)*
            }
        }
        Fields::Named(_) | Fields::Unnamed(_) | Fields::Unit => quote! {
            write.write_u8(::database::schema_discriminant::UNIT).await?;
        },
    });

    let write_values = data.variants.iter().enumerate().map(|(i, variant)| {
        let variant_name = &variant.ident;
        let discriminant = i as u32;

        match &variant.fields {
            Fields::Named(fields) => {
                let field_names = &fields
                    .named
                    .iter()
                    .map(|field| &field.ident)
                    .collect::<Vec<_>>();

                quote! {
                    Self::#variant_name { #(#field_names),* } => {
                        write.write_u32(#discriminant).await?;
                        #(#field_names.write_value(write).await?;)*
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let field_names = &(0..fields.unnamed.len())
                    .map(|index| format_ident!("field_{index}"))
                    .collect::<Vec<_>>();

                quote! {
                    Self::#variant_name(#(#field_names),*) => {
                        write.write_u32(#discriminant).await?;
                        #(#field_names.write_value(write).await?;)*
                    }
                }
            }
            Fields::Unit => quote! {
                Self::#variant_name => write.write_u32(#discriminant).await?,
            },
        }
    });

    let read_values = data.variants.iter().enumerate().map(|(i, variant)| {
        let variant_name = &variant.ident;

        let discriminant = i as u32;

        match &variant.fields {
            Fields::Named(fields) => {
                let field_names = fields.named.iter().map(|field| field.ident.as_ref().unwrap());
                let field_types = fields.named.iter().map(|field| &field.ty);

                quote! {
                    #discriminant => Self::#variant_name { #(#field_names: <#field_types as ::database::Schema>::read_value(read).await?,)* },
                }
            },
            Fields::Unnamed(fields) => {
                let field_types = fields.unnamed.iter().map(|field| &field.ty);

                quote! {
                    #discriminant => Self::#variant_name(#(<#field_types as ::database::Schema>::read_value(read).await?,)*),
                }
            }
            Fields::Unit => quote! {
                #discriminant => Self::#variant_name,
            },
        }
    });

    quote! {
        const _: () = {
            struct Expression(Vec<u32>);

            impl ::database::Expression for Expression {
                type Target = #name;

                async fn write<
                    S: ::database::__internal::tokio::io::AsyncWriteExt + ::database::__internal::tokio::io::AsyncReadExt + ::std::marker::Unpin,
                    Next: ::database::protocol::FromStream<S>
                >(
                    self,
                    write: ::database::protocol::ExpressionWrite<S, Next>,
                ) -> ::std::io::Result<Next> {
                    write.path(&self.0).await
                }
            }

            impl ::database::FromPath for Expression {
                fn from_path(path: ::std::vec::Vec<u32>) -> Self {
                    Self(path)
                }
            }

            impl ::database::Schema for #name {
                type Expression = Expression;

                fn write_schema(
                    write: &mut (impl ::database::__internal::tokio::io::AsyncWriteExt + ::std::marker::Unpin + ::std::marker::Send),
                ) -> impl ::std::future::Future<Output = ::std::io::Result<()>> + ::std::marker::Send {
                    async {
                        write.write_u8(::database::schema_discriminant::SUM).await?;
                        write.write_u32(#variant_count).await?;

                        #(#write_schemas)*

                        Ok(())
                    }
                }

                fn write_value(
                    &self,
                    write: &mut (impl ::database::__internal::tokio::io::AsyncWriteExt + ::std::marker::Unpin + ::std::marker::Send),
                ) -> impl ::std::future::Future<Output = ::std::io::Result<()>> + ::std::marker::Send {
                    async move {
                        match self {
                            #(#write_values)*
                        }

                        Ok(())
                    }
                }

                fn read_value(
                    read: &mut (impl ::database::__internal::tokio::io::AsyncReadExt + ::std::marker::Unpin + ::std::marker::Send),
                ) -> impl ::std::future::Future<Output = ::std::io::Result<Self>> + ::std::marker::Send {
                    async {
                        Ok(match read.read_u32().await? {
                            #(#read_values)*
                            _ => return Err(::std::io::Error::new(
                                ::std::io::ErrorKind::InvalidData,
                                concat!(env!("CARGO_CRATE_NAME"), ": ", "invalid discriminant in value for a sum value"),
                            )),
                        })
                    }
                }
            }
        };
    }
}
