#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

use proc_macro::TokenStream;
use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, GenericParam, Ident, Result, Token, Type,
};

/// Parses either an identifier or an underscore for the input expression in arms of specialized
/// dispatch macro.
#[derive(Debug, Eq, PartialEq)]
enum InputExprName {
    Ident(Ident),
    Underscore(Token![_]),
}

impl Parse for InputExprName {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Ident) {
            Ok(Self::Ident(input.parse()?))
        } else if input.peek(Token![_]) {
            Ok(Self::Underscore(input.parse()?))
        } else {
            Err(input.error("expected identifier or underscore"))
        }
    }
}

impl ToTokens for InputExprName {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Ident(ident) => ident.to_tokens(tokens),
            Self::Underscore(underscore) => underscore.to_tokens(tokens),
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
    default: Option<Token![default]>,
    generic_params: Option<Punctuated<GenericParam, Token![,]>>,
    input_expr_name: InputExprName,
    input_expr_type: Type,
    body: Expr,
}

impl Parse for DispatchArmExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        let default = input.parse::<Option<Token![default]>>()?;
        let _ = input.parse::<Token![fn]>()?;
        let generic_params = if input.peek(Token![<]) {
            let _ = input.parse::<Token![<]>()?;
            let generic_params =
                Punctuated::<GenericParam, Token![,]>::parse_separated_nonempty(input)?;
            let _ = input.parse::<Token![>]>()?;
            Some(generic_params)
        } else {
            None
        };
        let input_expr_content;
        let _ = parenthesized!(input_expr_content in input);
        let input_expr_name = input_expr_content.parse()?;
        let _ = input_expr_content.parse::<Token![:]>()?;
        let input_expr_type = input_expr_content.parse()?;
        if !input_expr_content.is_empty() {
            return Err(input_expr_content.error("unexpected token"));
        }
        let _ = input.parse::<Token![=>]>()?;
        let body = input.parse()?;
        Ok(Self {
            default,
            generic_params,
            input_expr_name,
            input_expr_type,
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
/// E -> String,
/// fn <T>(_: T) => format!("default value"),
/// fn (v: u8) => format!("u8: {}", v),
/// fn (v: u16) => format!("u16: {}", v),
/// expr,
/// ```
#[derive(Debug, Eq, PartialEq)]
struct SpecializedDispatchExpr {
    from_type: Type,
    to_type: Type,
    arms: Vec<DispatchArmExpr>,
    input_expr: Expr,
}

fn parse_punctuated_arms(input: &ParseStream) -> Result<Punctuated<DispatchArmExpr, Token![,]>> {
    let mut arms = Punctuated::new();
    loop {
        if input.peek(Token![default]) || input.peek(Token![fn]) {
            arms.push(input.parse()?);
        } else {
            break;
        }
        if input.peek(Token![,]) && (input.peek2(Token![default]) || input.peek2(Token![fn])) {
            let _ = input.parse::<Token![,]>()?;
        } else {
            break;
        }
    }
    Ok(arms)
}

impl Parse for SpecializedDispatchExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        let from_type = input.parse()?;
        let _ = input.parse::<Token![->]>()?;
        let to_type = input.parse()?;
        let _ = input.parse::<Token![,]>()?;
        let arms = parse_punctuated_arms(&input)?.into_iter().collect();
        let _ = input.parse::<Token![,]>()?;
        let input_expr = input.parse()?;
        let _ = input.parse::<Token![,]>().ok();
        Ok(Self {
            from_type,
            to_type,
            arms,
            input_expr,
        })
    }
}

/// Generates local helper trait declaration that will be used for specialized dispatch.
fn generate_trait_declaration(trait_name: &Ident, return_type: &Type) -> TokenStream2 {
    let tpl = Ident::new("T", Span2::mixed_site());
    quote! {
        trait #trait_name<#tpl> {
            fn dispatch(_: #tpl) -> #return_type;
        }
    }
}

/// Generates implementation of the helper trait for specialized dispatch arms. This covers both
/// generic case(s) and concrete case(s).
fn generate_trait_implementation(
    default: Option<&Token![default]>,
    trait_name: &Ident,
    generic_params: Option<&Punctuated<GenericParam, Token![,]>>,
    input_expr_type: &Type,
    input_expr_name: &InputExprName,
    return_type: &Type,
    body: &Expr,
) -> TokenStream2 {
    let generics = generic_params.map(|g| quote! {<#g>});
    quote! {
        impl #generics #trait_name<#input_expr_type> for #input_expr_type {
            #default fn dispatch(#input_expr_name: #input_expr_type) -> #return_type {
                #body
            }
        }
    }
}

/// Generates the dispatch call to the helper trait.
fn generate_dispatch_call(from_type: &Type, trait_name: &Ident, input_expr: &Expr) -> TokenStream2 {
    quote! {
        <#from_type as #trait_name<#from_type>>::dispatch(#input_expr)
    }
}

impl ToTokens for SpecializedDispatchExpr {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let trait_name = Ident::new("SpecializedDispatchCall", Span2::mixed_site());
        let trait_decl = generate_trait_declaration(&trait_name, &self.to_type);

        let mut trait_impls = TokenStream2::new();

        for arm in &self.arms {
            trait_impls.extend(generate_trait_implementation(
                arm.default.as_ref(),
                &trait_name,
                arm.generic_params.as_ref(),
                &arm.input_expr_type,
                &arm.input_expr_name,
                &self.to_type,
                &arm.body,
            ));
        }

        let dispatch_call = generate_dispatch_call(&self.from_type, &trait_name, &self.input_expr);

        tokens.extend(quote! {
            {
                #trait_decl
                #trait_impls
                #dispatch_call
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
                default: None,
                generic_params: None,
                input_expr_name: parse_quote!(v),
                input_expr_type: parse_quote!(u8),
                body: parse_quote!(format!("u8: {}", v)),
            }
        );
    }

    #[test]
    fn parse_arm_with_generic_type() {
        let arm: DispatchArmExpr = parse_quote!(default fn <T>(_: T) => format!("default value"));
        assert_eq!(
            arm,
            DispatchArmExpr {
                default: Some(Default::default()),
                generic_params: Some(parse_quote!(T)),
                input_expr_name: parse_quote!(_),
                input_expr_type: parse_quote!(T),
                body: parse_quote!(format!("default value")),
            }
        );
    }

    #[test]
    fn parse_specialized_dispatch_expr() {
        let expr: SpecializedDispatchExpr = parse_quote! {
            E -> String,
            default fn <T>(_: T) => format!("default value"),
            fn (v: u8) => format!("u8: {}", v),
            fn (v: u16) => format!("u16: {}", v),
            expr,
        };
        assert_eq!(
            expr,
            SpecializedDispatchExpr {
                from_type: parse_quote!(E),
                to_type: parse_quote!(String),
                arms: vec![
                    DispatchArmExpr {
                        default: Some(Default::default()),
                        generic_params: Some(parse_quote!(T)),
                        input_expr_name: parse_quote!(_),
                        input_expr_type: parse_quote!(T),
                        body: parse_quote!(format!("default value")),
                    },
                    DispatchArmExpr {
                        default: None,
                        generic_params: None,
                        input_expr_name: parse_quote!(v),
                        input_expr_type: parse_quote!(u8),
                        body: parse_quote!(format!("u8: {}", v)),
                    },
                    DispatchArmExpr {
                        default: None,
                        generic_params: None,
                        input_expr_name: parse_quote!(v),
                        input_expr_type: parse_quote!(u16),
                        body: parse_quote!(format!("u16: {}", v)),
                    },
                ],
                input_expr: parse_quote!(expr),
            }
        );
    }
}
