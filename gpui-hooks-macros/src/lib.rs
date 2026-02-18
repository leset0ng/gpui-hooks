use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_attribute]
pub fn hook_element(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let attrs = &input.attrs;
    let vis = &input.vis;
    let ident = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Extract original fields
    let mut fields = Vec::new();
    let mut original_field_names = Vec::new();
    let mut original_field_types = Vec::new();

    let data_struct = match &input.data {
        syn::Data::Struct(data_struct) => data_struct,
        _ => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                "#[hook_element] can only be used on structs",
            )
            .to_compile_error()
            .into();
        }
    };

    let fields_named = match &data_struct.fields {
        syn::Fields::Named(fields_named) => fields_named,
        _ => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                "#[hook_element] can only be used on structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    for field in &fields_named.named {
        // Save fields for regeneration
        fields.push(field);

        // Extract field names and types for Default implementation
        if let Some(field_ident) = &field.ident {
            original_field_names.push(field_ident);
            original_field_types.push(&field.ty);
        }
    }

    // Generate new struct definition, add hooks field and implement Render
    let expanded = quote! {
        #(#attrs)*
        #vis struct #ident #generics {
            #(#fields,)*
            _hooks: ::std::cell::RefCell<::std::vec::Vec<::std::boxed::Box<dyn gpui_hooks::hooks::Hook>>>,
            _hook_index: ::std::cell::Cell<usize>,
            _prev: ::std::cell::Cell<usize>,
        }

        impl #impl_generics ::std::default::Default for #ident #ty_generics #where_clause {
            fn default() -> Self {
                Self {
                    #(
                        #original_field_names: ::std::default::Default::default(),
                    )*
                    _hooks: ::std::cell::RefCell::new(::std::vec::Vec::new()),
                    _hook_index: ::std::cell::Cell::new(0),
                    _prev: ::std::cell::Cell::new(0),
                }
            }
        }

        impl #impl_generics gpui_hooks::HookedElement for #ident #ty_generics #where_clause {
            fn _hooks_ref(&self) -> &::std::cell::RefCell<::std::vec::Vec<::std::boxed::Box<dyn gpui_hooks::hooks::Hook>>> {
                &self._hooks
            }

            fn _hook_index(&self) -> usize {
                self._hook_index.get()
            }

            fn _set_hook_index(&self, index: usize) {
                self._hook_index.set(index);
            }

            fn _prev(&self) -> usize {
                self._prev.get()
            }

            fn _set_prev(&self, prev: usize) {
                self._prev.set(prev);
            }
        }

        // 自动实现 GPUI 的 Render trait
        impl #impl_generics gpui::Render for #ident #ty_generics #where_clause {
            fn render(&mut self, window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> impl gpui::IntoElement {
                gpui_hooks::execute_hooked_render(self, window, cx)
            }
        }
    };

    TokenStream::from(expanded)
}
