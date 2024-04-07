#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

use proc_macro::TokenStream;
use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Constraint, Expr, GenericArgument, Ident, Result, Token, Type,
};

/// Represents the type of argument in specialized dispatch macro.
#[derive(Debug, Eq, PartialEq)]
enum DispatchArmArgTypeExpr {
    Type(Type),
    Constraint(Constraint),
}

impl Parse for DispatchArmArgTypeExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        let _ = input.parse::<Token![<]>()?;
        let arg = match GenericArgument::parse(input)? {
            GenericArgument::Type(t) => Self::Type(t),
            GenericArgument::Constraint(c) => Self::Constraint(c),
            _ => return Err(input.error("expected type or constraint")),
        };
        let _ = input.parse::<Token![>]>()?;
        Ok(arg)
    }
}

/// Parses either an identifier or an underscore for the argument in arms of specialized dispatch
/// macro.
#[derive(Debug, Eq, PartialEq)]
enum ArgNameExpr {
    Ident(Ident),
    UnderScore(Token![_]),
}

impl Parse for ArgNameExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Ident) {
            Ok(Self::Ident(input.parse()?))
        } else if input.peek(Token![_]) {
            Ok(Self::UnderScore(input.parse()?))
        } else {
            Err(input.error("expected identifier or underscore"))
        }
    }
}

impl ToTokens for ArgNameExpr {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Ident(ident) => ident.to_tokens(tokens),
            Self::UnderScore(underscore) => underscore.to_tokens(tokens),
        }
    }
}

/// Represents an arm for specialized dispatch macro.
///
/// # Example Inputs
///
/// **Generic type:** This arm type represents the blanket implementation for the default case.
///
/// ```
/// fn <T>(_: T) => format!("default value")
/// ```
///
/// **Concrete type:** This arm type represents the specialized implementation for a specific concrete
/// type.
///
/// ```
/// fn (v: u8) => format!("u8: {}")
/// ```
#[derive(Debug, Eq, PartialEq)]
struct DispatchArmExpr {
    // TODO(ozars): Make this work with nested templates and lifetime parameters.
    generic_arg: Option<DispatchArmArgTypeExpr>,
    arg_name: ArgNameExpr,
    arg_type: Type,
    body: Expr,
}

impl Parse for DispatchArmExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        let _ = input.parse::<Token![fn]>()?;
        let generic_arg = if input.peek(Token![<]) {
            Some(input.parse::<DispatchArmArgTypeExpr>()?)
        } else {
            None
        };
        let args_content;
        let _ = parenthesized!(args_content in input);
        let arg = args_content.parse()?;
        let _ = args_content.parse::<Token![:]>()?;
        let arg_type = args_content.parse()?;
        if !args_content.is_empty() {
            return Err(args_content.error("unexpected token"));
        }
        let _ = input.parse::<Token![=>]>()?;
        let body = input.parse()?;
        Ok(Self {
            generic_arg,
            arg_name: arg,
            arg_type,
            body,
        })
    }
}

/// This is entry point for handling arguments of `specialized_dispatch` macro. It parses arguments
/// of the specialized dispatch macro and expands to the corresponding implementation.
///
/// # Example Input
///
/// ```
/// arg,
/// Arg -> String,
/// fn <T>(_: T) => format!("default value"),
/// fn (v: u8) => format!("u8: {}", v),
/// fn (v: u16) => format!("u16: {}", v),
/// ```
#[derive(Debug, Eq, PartialEq)]
struct SpecializedDispatchExpr {
    arg: Expr,
    from_type: Type,
    to_type: Type,
    arms: Vec<DispatchArmExpr>,
}

impl Parse for SpecializedDispatchExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse first argument.
        let arg = input.parse()?;
        let _ = input.parse::<Token![,]>()?;
        let from_type = input.parse()?;
        let _ = input.parse::<Token![->]>()?;
        let to_type = input.parse()?;
        let _ = input.parse::<Token![,]>()?;
        let arms = Punctuated::<DispatchArmExpr, Token![,]>::parse_terminated(input)?
            .into_iter()
            .collect();
        Ok(Self {
            arg,
            from_type,
            to_type,
            arms,
        })
    }
}

impl ToTokens for SpecializedDispatchExpr {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let trait_name = Ident::new("SpecializedDispatchCall", Span2::mixed_site());
        let tpl = Ident::new("SpecializedDispatchT", Span2::mixed_site());

        let SpecializedDispatchExpr {
            arg,
            from_type,
            to_type,
            arms,
        } = self;

        let trait_decl = quote! {
            trait #trait_name<#tpl> {
                fn dispatch(_: #tpl) -> #to_type;
            }
        };

        let mut trait_impls = TokenStream2::new();

        for DispatchArmExpr {
            generic_arg,
            arg_name,
            arg_type,
            body,
        } in arms
        {
            match generic_arg {
                Some(DispatchArmArgTypeExpr::Type(ref t)) => {
                    trait_impls.extend(quote! {
                        impl<#t> #trait_name<#t> for #t {
                            default fn dispatch(#arg_name: #arg_type) -> #to_type {
                                #body
                            }
                        }
                    });
                }
                Some(DispatchArmArgTypeExpr::Constraint(ref c)) => {
                    let Constraint {
                        ident: t,
                        generics: g,
                        ..
                    } = c;
                    trait_impls.extend(quote! {
                        impl<#c> #trait_name<#t #g> for #t #g {
                            default fn dispatch(#arg_name: #arg_type) -> #to_type {
                                #body
                            }
                        }
                    });
                }
                None => {
                    trait_impls.extend(quote! {
                        impl #trait_name<#arg_type> for #arg_type {
                            fn dispatch(#arg_name: #arg_type) -> #to_type {
                                #body
                            }
                        }
                    });
                }
            }
        }

        tokens.extend(quote! {
            {
                #trait_decl
                #trait_impls
                <#from_type as #trait_name<#from_type>>::dispatch(#arg)
            }
        });
    }
}

/// Entry point for the macro. Please see [the crate documentation](`crate`) for
/// more information and example.
#[proc_macro]
pub fn specialized_dispatch(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as SpecializedDispatchExpr)
        .into_token_stream()
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn parse_arm_with_concrete_type() {
        let arm: DispatchArmExpr = parse_quote!(fn (v: u8) => format!("u8: {}", v));
        assert_eq!(
            arm,
            DispatchArmExpr {
                generic_arg: None,
                arg_name: parse_quote!(v),
                arg_type: parse_quote!(u8),
                body: parse_quote!(format!("u8: {}", v)),
            }
        );
    }

    #[test]
    fn parse_arm_with_generic_type() {
        let arm: DispatchArmExpr = parse_quote!(fn <T>(_: T) => format!("default value"));
        assert_eq!(
            arm,
            DispatchArmExpr {
                generic_arg: Some(DispatchArmArgTypeExpr::Type(parse_quote!(T))),
                arg_name: parse_quote!(_),
                arg_type: parse_quote!(T),
                body: parse_quote!(format!("default value")),
            }
        );
    }

    #[test]
    fn parse_specialized_dispatch_expr() {
        let expr: SpecializedDispatchExpr = parse_quote! {
            arg,
            Arg -> String,
            fn <T>(_: T) => format!("default value"),
            fn (v: u8) => format!("u8: {}", v),
            fn (v: u16) => format!("u16: {}", v),
        };
        assert_eq!(
            expr,
            SpecializedDispatchExpr {
                arg: parse_quote!(arg),
                from_type: parse_quote!(Arg),
                to_type: parse_quote!(String),
                arms: vec![
                    DispatchArmExpr {
                        generic_arg: Some(DispatchArmArgTypeExpr::Type(parse_quote!(T))),
                        arg_name: parse_quote!(_),
                        arg_type: parse_quote!(T),
                        body: parse_quote!(format!("default value")),
                    },
                    DispatchArmExpr {
                        generic_arg: None,
                        arg_name: parse_quote!(v),
                        arg_type: parse_quote!(u8),
                        body: parse_quote!(format!("u8: {}", v)),
                    },
                    DispatchArmExpr {
                        generic_arg: None,
                        arg_name: parse_quote!(v),
                        arg_type: parse_quote!(u16),
                        body: parse_quote!(format!("u16: {}", v)),
                    },
                ],
            }
        );
    }
}
