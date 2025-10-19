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
        "u8" => 32, 
        "u16" => 16, 
        "u32" => 8, 
        "u64" => 4, 
        "u128" => 2,
        _ => panic!("Type not supported for simd"),     
    }
}

fn generate_add_assign(
    ty: &proc_macro2::TokenStream,
    _modulus: u128,
    lanes: usize,
) -> proc_macro2::TokenStream {
    quote! {
        #[inline(always)]
        fn simd_add_assign(a: &mut [SmallFp<Self>], b: &[SmallFp<Self>]) {

            assert_eq!(a.len(), b.len(), "slices must have equal length");
            if a.is_empty() { return; }

            let modulus_simd = Simd::<#ty, #lanes>::splat(Self::MODULUS);
            let overflow_simd = Simd::<#ty, #lanes>::splat(Self::T::MAX - Self::MODULUS + 1);
            
            let (a_chunks, a_remainder) = cast_slice_mut(a).as_chunks_mut::<#lanes>();
            let (b_chunks, b_remainder) = cast_slice(b).as_chunks::<#lanes>();

            for (a_chunk, b_chunk) in a_chunks.iter_mut().zip(b_chunks.iter()) {
                let a_simd = Simd::<#ty, #lanes>::from_array(*a_chunk);
                let b_simd = Simd::<#ty, #lanes>::from_array(*b_chunk);
                let mut sum = a_simd + b_simd;
                
                sum = (a_simd.simd_ge(sum)).select(sum + overflow_simd, sum);
                let reduced = (sum.simd_ge(modulus_simd)).select(sum - modulus_simd, sum);
                *a_chunk = reduced.to_array();
            }
            
            for (a_val, b_val) in a_remainder.iter_mut().zip(b_remainder.iter()) {
                let (mut val, overflow) = a_val.overflowing_add(*b_val);

                val += (overflow as #ty) * (Self::T::MAX - Self::MODULUS + 1);
                let m = val >= Self::MODULUS;
                *a_val = val - (m as #ty) * Self::MODULUS;
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
        fn simd_mul_assign(a: &mut [SmallFp<Self>], b: &[SmallFp<Self>]) {
            assert_eq!(a.len(), b.len(), "slices must have equal length");
            if a.is_empty() { return; }

            let modulus_simd = Simd::<#upcast_ty, #lanes>::splat(Self::MODULUS as #upcast_ty);
            
            let (a_chunks, a_remainder) = cast_slice_mut(a).as_chunks_mut::<#lanes>();
            let (b_chunks, b_remainder) = cast_slice(b).as_chunks::<#lanes>();

            for (a_chunk, b_chunk) in a_chunks.iter_mut().zip(b_chunks.iter()) {
                let a_simd = Simd::<#ty, #lanes>::from_array(*a_chunk).cast::<#upcast_ty>();
                let b_simd = Simd::<#ty, #lanes>::from_array(*b_chunk).cast::<#upcast_ty>();

                let prod = ((a_simd * b_simd) % modulus_simd).cast::<#ty>();
                *a_chunk = prod.to_array();
            }
            
            for (a_val, b_val) in a_remainder.iter_mut().zip(b_remainder.iter()) {
                let prod = (*a_val as #upcast_ty) * (*b_val as #upcast_ty);
                let reduced_prod = prod % (Self::MODULUS as #upcast_ty);
                *a_val = reduced_prod as #ty;
            }
        }
    }
}
