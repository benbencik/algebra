use quote::quote;

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
    let mul_impl = generate_mul_assign(ty, &upcast_ty, modulus, lanes);
    
    quote! {
        #[cfg(feature = "portable_simd")]
        const _: () = {
            use ark_ff::{SmallFp, SmallFpSimd};
            use ark_std::simd::{Simd, prelude::{SimdPartialEq, SimdPartialOrd, SimdUint}};
            
            impl SmallFpSimd for #config_name {
                const SIMD_LANES: usize = #lanes;
                
                #add_impl
                #mul_impl 
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
    quote! {
        #[inline(always)]
        fn add_assign_simd(a: &mut [SmallFp<Self>], b: &[SmallFp<Self>]) {
            assert_eq!(a.len(), b.len(), "slices must have equal length");
            if a.is_empty() { return; }

            let modulus_simd = Simd::<#ty, #lanes>::splat(Self::MODULUS);
            let overflow_correction = Self::T::MAX - Self::MODULUS + 1;
            let overflow_correction_simd = Simd::<#ty, #lanes>::splat(overflow_correction);
            
            let a_val: &mut [#ty] = cast_slice_mut(a);
            let b_val: &[#ty] = cast_slice(b);
            
            let chunks = a_val.len() / #lanes;
            for i in 0..chunks {
                let start_idx = i * #lanes;
                let end_idx = start_idx + #lanes;
                let a_packed = Simd::<#ty, #lanes>::from_slice(&a_val[start_idx..end_idx]);
                let b_packed = Simd::<#ty, #lanes>::from_slice(&b_val[start_idx..end_idx]);
                let mut sum = a_packed + b_packed;
                
                sum = a_packed.simd_ge(sum).select(sum + overflow_correction_simd, sum);
                sum = (sum.simd_ge(modulus_simd)).select(sum - modulus_simd, sum);
                sum.copy_to_slice(&mut a_val[start_idx..end_idx]);
            }
            
            let remainder_idx = chunks * #lanes;
            for i in remainder_idx..a_val.len() {
                let (mut val, overflow) = a_val[i].overflowing_add(b_val[i]);
                val += (overflow as #ty) * overflow_correction;
                a_val[i] = val - ((val >= Self::MODULUS) as #ty) * Self::MODULUS;
            }
        }
    }
}

// ! this does not handle u128 
fn generate_mul_assign(
    ty: &proc_macro2::TokenStream,
    upcast_ty: &proc_macro2::TokenStream,
    _modulus: u128,
    lanes: usize,
) -> proc_macro2::TokenStream {
    quote! {
        #[inline(always)]
        fn mul_assign_simd(a: &mut [SmallFp<Self>], b: &[SmallFp<Self>]) {
            assert_eq!(a.len(), b.len(), "slices must have equal length");
            if a.is_empty() { return; }

            let modulus_simd = Simd::<#upcast_ty, #lanes>::splat(Self::MODULUS as #upcast_ty);
            let a_val = cast_slice_mut(a);
            let b_val = cast_slice(b);

            let chunks = a_val.len() / #lanes;
            for i in 0..chunks {
                let start_idx = i * #lanes;
                let end_idx = start_idx + #lanes;
                let a_simd = Simd::<#ty, #lanes>::from_slice(&a_val[start_idx..end_idx]).cast::<#upcast_ty>();
                let b_simd = Simd::<#ty, #lanes>::from_slice(&b_val[start_idx..end_idx]).cast::<#upcast_ty>();

                let prod = ((a_simd * b_simd) % modulus_simd).cast::<#ty>();
                prod.copy_to_slice(&mut a_val[start_idx..end_idx]);
            }
            
            let remainder_idx = chunks * #lanes;
            for i in remainder_idx..a_val.len() {
                let prod = (a_val[i] as #upcast_ty) * (b_val[i] as #upcast_ty);
                let reduced_prod = prod % (Self::MODULUS as #upcast_ty);
                a_val[i] = reduced_prod as #ty;
            }
        }
    }
}
