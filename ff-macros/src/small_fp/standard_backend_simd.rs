use quote::quote;

struct SimdOpConfig {
    // Additional setup needed before the SIMD
    setup_code: proc_macro2::TokenStream,
    
    // SIMD computation with specialized instructions
    simd_computation: proc_macro2::TokenStream,
}

fn generate_add_op_logic(
    ty: &proc_macro2::TokenStream,
    lanes: usize,
) -> SimdOpConfig {
    let setup_code = quote! {
        let modulus_simd = Simd::<#ty, #lanes>::splat(Self::MODULUS);
        let overflow_correction = Self::T::MAX - Self::MODULUS + 1;
        let overflow_correction_simd = Simd::<#ty, #lanes>::splat(overflow_correction);
        let a_val: &mut [#ty] = cast_slice_mut(a);
        let b_val: &[#ty] = cast_slice(b);
    };
    
    let simd_computation = quote! {
        let mut sum = a_packed + b_packed;
        sum = a_packed.simd_ge(sum).select(sum + overflow_correction_simd, sum);
        sum = (sum.simd_ge(modulus_simd)).select(sum - modulus_simd, sum);
        sum
    };
    
    SimdOpConfig {
        setup_code,
        simd_computation,
    }
}

fn generate_sub_op_logic(
    ty: &proc_macro2::TokenStream,
    lanes: usize,
) -> SimdOpConfig {
    let setup_code = quote! {
        let modulus_simd = Simd::<#ty, #lanes>::splat(Self::MODULUS);
        let a_val: &mut [#ty] = cast_slice_mut(a);
        let b_val: &[#ty] = cast_slice(b);
    };
    
    let simd_computation = quote! {
        let diff = a_packed.simd_lt(b_packed).select(
            modulus_simd - (b_packed - a_packed),
            a_packed - b_packed
        );
        diff
    };
    
    SimdOpConfig {
        setup_code,
        simd_computation,
    }
}

fn generate_mul_op_logic(
    ty: &proc_macro2::TokenStream,
    upcast_ty: &proc_macro2::TokenStream,
    lanes: usize,
) -> SimdOpConfig {
    let setup_code = quote! {
        let modulus_simd = Simd::<#upcast_ty, #lanes>::splat(Self::MODULUS as #upcast_ty);
        let a_val: &mut [#ty] = cast_slice_mut(a);
        let b_val: &[#ty] = cast_slice(b);
    };
    
    let simd_computation = quote! {
        let a_upcast = a_packed.cast::<#upcast_ty>();
        let b_upcast = b_packed.cast::<#upcast_ty>();
        let prod = ((a_upcast * b_upcast) % modulus_simd).cast::<#ty>();
        prod
    };
   
    SimdOpConfig {
        setup_code,
        simd_computation,
    }
}

fn generate_div_op_logic(
    ty: &proc_macro2::TokenStream,
    upcast_ty: &proc_macro2::TokenStream,
    lanes: usize,
) -> SimdOpConfig {
    let setup_code = quote! {
        let modulus_simd = Simd::<#upcast_ty, #lanes>::splat(Self::MODULUS as #upcast_ty);
        let mut b_inv = b.to_vec();
        for i in 0..b_inv.len() {
            b_inv[i] = match <Self as SmallFpConfig>::inverse(&b_inv[i]) {
                Some(inv) => inv,
                None => panic!("Division by zero in finite field"),
            }
        }
        let a_val: &mut [#ty] = cast_slice_mut(a);
        let b_val: &[#ty] = cast_slice(&b_inv);
    };
    
    let simd_computation = quote! {
        let a_upcast = a_packed.cast::<#upcast_ty>();
        let b_upcast = b_packed.cast::<#upcast_ty>();
        let prod = ((a_upcast * b_upcast) % modulus_simd).cast::<#ty>();
        prod
    };
    
    SimdOpConfig {
        setup_code,
        simd_computation,
    }
}

/// Generic function to generate pairwise SIMD operations
fn generate_pairwise_simd_op(
    fn_name: &str,
    ty: &proc_macro2::TokenStream,
    _upcast_ty: &proc_macro2::TokenStream,
    lanes: usize,
    op_config: SimdOpConfig,
) -> proc_macro2::TokenStream {
    let fn_ident = syn::Ident::new(fn_name, proc_macro2::Span::call_site());
    let setup_code = op_config.setup_code;
    let simd_computation = op_config.simd_computation;
    
    quote! {
        #[inline(always)]
        fn #fn_ident(a: &mut [SmallFp<Self>], b: &[SmallFp<Self>]) {
            assert_eq!(a.len(), b.len(), "slices must have equal length");
            if a.is_empty() { return; }

            #setup_code
            
            // main simd loop
            let chunks = a_val.len() / #lanes;
            let mut a_ptr = a_val.as_mut_ptr();
            let mut b_ptr = b_val.as_ptr();
            let mut iters = chunks;
            while iters > 0 {
                let a_packed = unsafe {
                    let a_arr = (a_ptr as *const [#ty; #lanes]).read_unaligned();
                    Simd::<#ty, #lanes>::from_array(a_arr)
                };
                let b_packed = unsafe {
                    let b_arr = (b_ptr as *const [#ty; #lanes]).read_unaligned();
                    Simd::<#ty, #lanes>::from_array(b_arr)
                };
                
                let result = { #simd_computation };

                unsafe {
                    core::ptr::copy_nonoverlapping(
                        result.as_array().as_ptr(),
                        a_ptr,
                        #lanes,
                    );
                    a_ptr = a_ptr.add(#lanes);
                    b_ptr = b_ptr.add(#lanes);
                }
                iters -= 1;
            }
            
            // handle remainder
            if a_val.len() > chunks * #lanes {
                let or = Simd::<#ty, #lanes>::splat(1);
                let start_idx = chunks * #lanes;
                let end_idx = a_val.len();
                let a_packed = Simd::<#ty, #lanes>::load_or(&a_val[start_idx..end_idx], or);
                let b_packed = Simd::<#ty, #lanes>::load_or(&b_val[start_idx..end_idx], or);
                
                let result = { #simd_computation };
                let actual_len = end_idx - start_idx;
                a_val[start_idx..end_idx].copy_from_slice(&result.as_array().as_slice()[..actual_len]);
            }
        }
    }
}


/// Generate SmallFpSimd trait implementation for standard backend
pub(crate) fn generate_simd_impl(
    ty: &proc_macro2::TokenStream,
    modulus: u128,
    config_name: &proc_macro2::Ident,
) -> proc_macro2::TokenStream {
    let lanes = determine_lane_count(ty);
    let ty_str = ty.to_string();
    let (upcast_ty, _bits) = match ty_str.as_str() {
        "u8" => (quote! {u16}, 16u32),
        "u16" => (quote! {u32}, 32u32),
        "u32" => (quote! {u64}, 64u32),
        _ => (quote! {u128}, 128u32),
    };
    
    // Generate SIMD operations using the generic framework
    let add_impl = generate_add_assign(ty, modulus, lanes);
    let sub_impl = generate_sub_assign(ty, modulus, lanes);
    let mul_impl = generate_mul_assign(ty, &upcast_ty, modulus, lanes);
    let div_impl = generate_div_assign(ty, &upcast_ty, modulus, lanes);
    
    quote! {
        #[cfg(feature = "portable_simd")]
        const _: () = {
            use ark_ff::{SmallFp, SmallFpSimd};
            use ark_std::simd::{Simd, prelude::{SimdPartialEq, SimdPartialOrd, SimdUint}};
            
            impl SmallFpSimd for #config_name {
                const SIMD_LANES: usize = #lanes;
                
                #add_impl
                #sub_impl
                #mul_impl
                #div_impl
            }

            #[inline]
            fn cast_slice_mut(elems: &mut [SmallFp<#config_name>]) -> &mut [#ty] {
                assert!(core::mem::size_of::<SmallFp<#config_name>>() == core::mem::size_of::<#ty>());
                unsafe {
                    core::slice::from_raw_parts_mut(
                        elems.as_mut_ptr() as *mut #ty,
                        elems.len(),
                    )
                }
            }

            #[inline]
            fn cast_slice(elems: &[SmallFp<#config_name>]) -> & [#ty] {
                assert!(core::mem::size_of::<SmallFp<#config_name>>() == core::mem::size_of::<#ty>());
                unsafe {
                    core::slice::from_raw_parts(
                        elems.as_ptr() as *const #ty,
                        elems.len(),
                    )
                }
            }
        };
    }
}

// Targeting the avx2 instruction set with 256-bit registers
fn determine_lane_count(ty: &proc_macro2::TokenStream) -> usize {
    match ty.to_string().as_str() {
        "u8" => 64, 
        "u16" => 64, 
        "u32" => 64, 
        _ => panic!("Type not supported for simd"),     
        // "u64" => 64, 
        // "u128" => 2,
    }
}

fn generate_add_assign(
    ty: &proc_macro2::TokenStream,
    _modulus: u128,
    lanes: usize,
) -> proc_macro2::TokenStream {
    let op_config = generate_add_op_logic(ty, lanes);
    generate_pairwise_simd_op("add_assign_simd", ty, &quote! {u128}, lanes, op_config)
}

fn generate_mul_assign(
    ty: &proc_macro2::TokenStream,
    upcast_ty: &proc_macro2::TokenStream,
    _modulus: u128,
    lanes: usize,
) -> proc_macro2::TokenStream {
    let op_config = generate_mul_op_logic(ty, upcast_ty, lanes);
    generate_pairwise_simd_op("mul_assign_simd", ty, upcast_ty, lanes, op_config)
}

fn generate_sub_assign(
    ty: &proc_macro2::TokenStream,
    _modulus: u128,
    lanes: usize,
) -> proc_macro2::TokenStream {
    let op_config = generate_sub_op_logic(ty, lanes);
    generate_pairwise_simd_op("sub_assign_simd", ty, &quote! {u128}, lanes, op_config)
}

fn generate_div_assign(
    ty: &proc_macro2::TokenStream,
    upcast_ty: &proc_macro2::TokenStream,
    _modulus: u128,
    lanes: usize,
) -> proc_macro2::TokenStream {
    let op_config = generate_div_op_logic(ty, upcast_ty, lanes);
    generate_pairwise_simd_op("div_assign_simd", ty, upcast_ty, lanes, op_config)
}
