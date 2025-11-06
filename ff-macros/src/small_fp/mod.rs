mod montgomery_backend;
mod standard_backend;
mod utils;

use quote::quote;

/// This function is called by the `#[derive(SmallFp)]` macro and generates
/// the implementation of the `SmallFpConfig`
pub(crate) fn small_fp_config_helper(
    modulus: u128,
    generator: u128,
    backend: String,
    config_name: proc_macro2::Ident,
) -> proc_macro2::TokenStream {
    let ty = match modulus {
        m if m < 1u128 << 8 => quote! { u8 },
        m if m < 1u128 << 16 => quote! { u16 },
        m if m < 1u128 << 32 => quote! { u32 },
        m if m < 1u128 << 64 => quote! { u64 },
        _ => quote! { u128 },
    };

    let backend_impl = match backend.as_str() {
        "standard" => standard_backend::backend_impl(&ty, modulus, generator),
        "montgomery" => {
            if modulus >= 1u128 << 127 {
                panic!(
                    "SmallFpConfig montgomery backend supports only moduli < 2^127. Use MontConfig with BigInt instead of SmallFp."
                )
            }
            montgomery_backend::backend_impl(&ty, modulus, generator)
        },

        _ => panic!("Unknown backend type: {}", backend),
    };

    let new_impl = match backend.as_str() {
        "standard" => standard_backend::new(),
        "montgomery" => montgomery_backend::new(modulus, ty.clone()),
        _ => panic!("Unknown backend type: {}", backend),
    };

    let k_bits = 128 - modulus.leading_zeros();    
    let ty_str = ty.to_string();

    let (mul_ty, mask) = match ty_str.as_str() {
        "u8" => (quote! {u16}, {
            let m = (1u16 << k_bits) - 1;
            quote! { #m }
        }),
        "u16" => (quote! {u32}, {
            let m = (1u32 << k_bits) - 1;
            quote! { #m }
        }),
        "u32" => (quote! {u64}, {
            let m = (1u64 << k_bits) - 1;
            quote! { #m }
        }),
        _ => (quote! {u128}, {
            let m = (1u128 << k_bits) - 1;
            quote! { #m }
        })
    };

    let helper = quote! {
        #[inline(always)]
        pub const fn mac(a: #mul_ty, b: #mul_ty, carry: &mut #ty) -> #ty {
            let tmp = a * b;
            *carry = (tmp >> #k_bits) as #ty;
            (tmp & #mask) as #ty
        }

        #[inline(always)]
        pub const fn mac_discard(a: #ty, b: #ty, c: #ty, carry: &mut #ty){
            let tmp = (a as #mul_ty) + ((b as #mul_ty) * (c as #mul_ty));
            *carry = (tmp >> #k_bits) as #ty;
        }
    };


    quote! {
        impl SmallFpConfig for #config_name {
            #backend_impl
        }

        impl #config_name {
            #new_impl

            #helper
        }
    }
}
