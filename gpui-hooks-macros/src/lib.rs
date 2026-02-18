use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

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
            _hooks: ::std::vec::Vec<::std::boxed::Box<dyn gpui_hooks::Hook>>,
            _hook_index: usize,
            _prev: usize,
        }

        impl #impl_generics ::std::default::Default for #ident #ty_generics #where_clause {
            fn default() -> Self {
                Self {
                    #(
                        #original_field_names: ::std::default::Default::default(),
                    )*
                    _hooks: ::std::vec::Vec::new(),
                    _hook_index: 0,
                    _prev: 0,
                }
            }
        }

        impl #impl_generics gpui_hooks::HookedElement for #ident #ty_generics #where_clause {
            /// Add a hook to the hooks list
            fn _use<T>(&mut self, hook: T) -> T where T:impl gpui_hooks::Hook + 'static {
                if self._hook_index == self._hooks.len() {
                    self._hooks.push(::std::boxed::Box::new(hook));
                    hook
                } else {
                    self._hooks[self._hook_index] = ::std::boxed::Box::new(hook);
                }
            }

            /// Get an immutable reference to the hooks list
            fn _hooks(&self) -> &[::std::boxed::Box<dyn gpui_hooks::Hook>] {
                &self._hooks
            }

            /// Get a mutable reference to the hooks list
            fn _hooks_mut(&mut self) -> &mut [::std::boxed::Box<dyn gpui_hooks::Hook>] {
                &mut self._hooks
            }

            /// reset the hook index
            fn _reset(&mut self) {
                if self._prev == 0 || self._hook_index == self._prev {
                    self._prev = self._hook_index;
                } else {
                    panic!("dont use hooks in condition");
                }
                self._hook_index = 0;
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
