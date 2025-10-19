mod montgomery_backend;
mod standard_backend;
mod standard_backend_simd;
mod utils;

use quote::quote;

/// This function is called by the `#[derive(SmallFp)]` macro and generates
/// the implementation of the `SmallFpConfig`
pub(crate) fn small_fp_config_helper(
    modulus: u128,
    generator: u128,
    backend: String,
    enable_simd: bool,
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

    let simd_impl = if enable_simd {
        match backend.as_str() {
            "standard" => standard_backend_simd::generate_simd_impl(&ty, modulus, &config_name),
            "montgomery" => quote! {}, 
            _ => quote! {},
        }
    } else {
        quote! {}
    };

    quote! {
        impl SmallFpConfig for #config_name {
            #backend_impl
        }

        impl #config_name {
            #new_impl
        }

        #simd_impl
    }
}
