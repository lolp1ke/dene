use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Field, Fields, parse_macro_input, parse_quote};

pub(crate) fn refine(
  input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  match expand(input) {
    Ok(tokens) => tokens.into(),
    Err(error) => error.into_compile_error().into(),
  }
}

fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
  let DeriveInput {
    ident,
    vis,
    generics,
    data,
    ..
  } = input;

  let fields = match data {
    Data::Struct(data) => match data.fields {
      Fields::Named(fields) => fields.named.into_iter().collect::<Vec<_>>(),
      fields => {
        return Err(syn::Error::new_spanned(
          fields,
          "Refine only supports structs with named fields",
        ));
      }
    },
    _ => {
      return Err(syn::Error::new(
        ident.span(),
        "Refine only supports structs",
      ));
    }
  };

  let refinement_ident = format_ident!("{}Refinement", ident);
  let mut refinement_generics = generics.clone();

  {
    let where_clause = refinement_generics.make_where_clause();

    for field in &fields {
      let ty = &field.ty;

      if is_nested(field) {
        where_clause.predicates.push(parse_quote!(#ty: Refine));
        where_clause.predicates.push(parse_quote!(
          <#ty as Refine>::Refinement:
            Refine<Refinement = <#ty as Refine>::Refinement>
            + ::core::clone::Clone
            + ::core::default::Default
            + IsEmpty
        ));
      } else {
        where_clause
          .predicates
          .push(parse_quote!(#ty: ::core::clone::Clone));
      }
    }
  }

  let (impl_generics, ty_generics, where_clause) =
    refinement_generics.split_for_impl();

  let refinement_fields = fields.iter().map(|field| {
    let name = field.ident.as_ref().expect("named fields were checked");
    let visibility = &field.vis;
    let ty = &field.ty;

    if is_nested(field) {
      quote! {
        #visibility #name: <#ty as Refine>::Refinement
      }
    } else {
      quote! {
        #visibility #name: ::core::option::Option<#ty>
      }
    }
  });

  let value_refinements = fields.iter().map(|field| {
    let name = field.ident.as_ref().expect("named fields were checked");

    if is_nested(field) {
      quote! {
        Refine::refine(&mut self.#name, &refinement.#name);
      }
    } else {
      quote! {
        if let ::core::option::Option::Some(value) = &refinement.#name {
          self.#name = value.clone();
        }
      }
    }
  });

  let refinement_refinements = fields.iter().map(|field| {
    let name = field.ident.as_ref().expect("named fields were checked");

    if is_nested(field) {
      quote! {
        Refine::refine(&mut self.#name, &refinement.#name);
      }
    } else {
      quote! {
        if refinement.#name.is_some() {
          self.#name = refinement.#name.clone();
        }
      }
    }
  });

  let empty_conditions = fields.iter().map(|field| {
    let name = field.ident.as_ref().expect("named fields were checked");

    if is_nested(field) {
      quote! {
        IsEmpty::is_empty(&self.#name)
      }
    } else {
      quote! {
        self.#name.is_none()
      }
    }
  });

  let default_fields = fields.iter().map(|field| {
    let name = field.ident.as_ref().expect("named fields were checked");

    quote! {
      #name: ::core::default::Default::default()
    }
  });

  Ok(quote! {
    #[derive(::core::fmt::Debug, ::core::clone::Clone)]
    #vis struct #refinement_ident
      #refinement_generics
      #where_clause
    {
      #( #refinement_fields, )*
    }

    impl #impl_generics ::core::default::Default
      for #refinement_ident #ty_generics
      #where_clause
    {
      fn default() -> Self {
        Self {
          #( #default_fields, )*
        }
      }
    }

    impl #impl_generics Refine for #ident #ty_generics
      #where_clause
    {
      type Refinement = #refinement_ident #ty_generics;

      fn refine(&mut self, refinement: &Self::Refinement) {
        #( #value_refinements )*
      }
    }

    impl #impl_generics Refine
      for #refinement_ident #ty_generics
      #where_clause
    {
      type Refinement = Self;

      fn refine(&mut self, refinement: &Self::Refinement) {
        #( #refinement_refinements )*
      }
    }

    impl #impl_generics IsEmpty
      for #refinement_ident #ty_generics
      #where_clause
    {
      fn is_empty(&self) -> bool {
        true #( && #empty_conditions )*
      }
    }

    impl #impl_generics #refinement_ident #ty_generics
      #where_clause
    {
      pub fn is_empty(&self) -> bool {
        IsEmpty::is_empty(self)
      }
    }
  })
}

fn is_nested(field: &Field) -> bool {
  field.attrs.iter().any(|attribute| {
    attribute.path().is_ident("refine")
      && matches!(attribute.meta, syn::Meta::Path(_))
  })
}
