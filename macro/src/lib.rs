use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Data, Fields, Path};
use itertools::Itertools;
use proc_macro2::{Span};
use proc_macro2 as pm;


// =============
// === Utils ===
// =============

/// Get the current crate name;
fn crate_name() -> Ident {
    let macro_lib = env!("CARGO_PKG_NAME");
    let suffix = "-macro";
    if !macro_lib.ends_with(suffix) { panic!("Internal error.") }
    let crate_name = &macro_lib[..macro_lib.len() - suffix.len()].replace('-',"_");
    Ident::new(crate_name, Span::call_site())
}

/// Extract the module macro attribute.
fn extract_module_attr(input: &DeriveInput) -> Path {
    let mut module: Option<Path> = None;
    for attr in &input.attrs {
        if attr.path().is_ident("module") {
            let tokens = attr.meta.require_list().unwrap().tokens.clone();
            if let Ok(path) = syn::parse2::<Path>(tokens) {
                module = Some(path);
            }
        }
    }
    module.expect("The 'module' attribute is required.")
}


// =============
// === Macro ===
// =============

/// Derive impl. Comments in the code show expansion of the following example struct:
/// ```ignore
/// pub struct Ctx<'v, V: Debug> {
///     version: &'v V,
///     pub geometry: GeometryCtx,
///     pub material: MaterialCtx,
///     pub mesh: MeshCtx,
///     pub scene: SceneCtx,
/// }
/// ```
#[proc_macro_derive(Partial, attributes(module))]
pub fn partial_borrow_derive(input: TokenStream) -> TokenStream {
    let lib = crate_name();
    let input = parse_macro_input!(input as DeriveInput);
    let module = extract_module_attr(&input);

    let struct_ident = input.ident;
    let ref_struct_ident = Ident::new(&format!("{struct_ident}Ref"), struct_ident.span());

    let fields = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            fields.named.iter().collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let generics = input.generics.params.iter().collect_vec();
    let struct_lifetimes = generics.iter().filter_map(|g| {
        if let syn::GenericParam::Lifetime(lt) = g {
            Some(lt)
        } else {
            None
        }
    }).collect_vec();

    let struct_params = generics.iter().filter_map(|g| {
        if let syn::GenericParam::Type(ty) = g {
            Some(ty.ident.clone())
        } else {
            None
        }
    }).collect_vec();

    let struct_inline_bounds = generics.iter().filter_map(|g| {
        if let syn::GenericParam::Type(ty) = g {
            if !ty.bounds.is_empty() {
                Some(ty)
            } else {
                None
            }
        } else {
            None
        }
    }).collect_vec();

    let struct_where_bounds = input.generics.where_clause.as_ref().map(|w| w.predicates.iter().collect_vec()).unwrap_or_default();

    let struct_bounds = struct_inline_bounds.iter().map(|ty| {
        quote! {#ty}
    }).chain(struct_where_bounds.iter().map(|p| quote! {#p})).collect_vec();

    let field_vis = fields.iter().map(|f| f.vis.clone()).collect_vec();
    let field_idents = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect_vec();
    let field_types = fields.iter().map(|f| &f.ty).collect_vec();
    let params = field_idents.iter().map(|i| Ident::new(&i.to_string(), i.span())).collect_vec();

    // Generates:
    // #[repr(C)]
    // pub struct CtxRef<version, geometry, material, mesh, scene> {
    //     version: version,
    //     pub geometry: geometry,
    //     pub material: material,
    //     pub mesh: mesh,
    //     pub scene: scene,
    // }
    let ref_struct = quote! {
        #[derive(Debug)]
        #[repr(C)]
        #[allow(non_camel_case_types)]
        pub struct #ref_struct_ident<#(#params,)*> {
            #(#field_vis #field_idents : #params),*
        }
    };

    // Generates:
    // impl<version_target, geometry_target, material_target, mesh_target, scene_target,
    //      version,        geometry,        material,        mesh,        scene>
    // PartialInferenceGuide<
    //       CtxRef<version_target, geometry_target, material_target, mesh_target, scene_target>
    // > for CtxRef<version,        geometry,        material,        mesh,        scene> {}
    let impl_inference_guide = {
        let target_params = params.iter().map(|i| Ident::new(&format!("{i}_target"), i.span())).collect_vec();
        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#params,)* #(#target_params,)*>
            #lib::PartialInferenceGuide<#ref_struct_ident<#(#target_params,)*>>
            for #ref_struct_ident<#(#params,)*> {}
        }
    };

    // Generates:
    // impl<'t, 'v, V, version, geometry, material, mesh, scene>
    // AsRefs<'t, CtxRef<version, geometry, material, mesh, scene>> for Ctx<'v, V>
    // where
    //     V:           Debug,
    //     (&'v V):     RefCast<'t, version>,
    //     GeometryCtx: RefCast<'t, geometry>,
    //     MaterialCtx: RefCast<'t, material>,
    //     MeshCtx:     RefCast<'t, mesh>,
    //     SceneCtx:    RefCast<'t, scene>,
    // {
    //     fn as_refs_impl(&'t mut self) -> CtxRef<version, geometry, material, mesh, scene> {
    //         CtxRef {
    //             version:  RefCast::ref_cast(&mut self.version),
    //             geometry: RefCast::ref_cast(&mut self.geometry),
    //             material: RefCast::ref_cast(&mut self.material),
    //             mesh:     RefCast::ref_cast(&mut self.mesh),
    //             scene:    RefCast::ref_cast(&mut self.scene),
    //         }
    //     }
    // }
    let impl_as_refs = quote! {
        #[allow(non_camel_case_types)]
        impl<'_t, #(#struct_lifetimes,)* #(#struct_params,)* #(#params,)*>
        #lib::AsRefs<'_t, #ref_struct_ident<#(#params,)*>> for #struct_ident<#(#struct_lifetimes,)* #(#struct_params,)*>
        where
            #(#struct_bounds,)*
            #(#field_types: #lib::RefCast<'_t, #params>,)*
        {
            #[inline(always)]
            fn as_refs_impl(& '_t mut self) -> #ref_struct_ident<#(#params,)*> {
                #ref_struct_ident {
                    #(#field_idents: #lib::RefCast::ref_cast(&mut self.#field_idents),)*
                }
            }
        }
    };

    // Generates:
    // impl<'v, V> Ctx<'v, V>
    // where V: Debug
    // {
    //     pub fn as_refs_mut(&mut self) -> CtxRef<&mut &'v V, &mut GeometryCtx, &mut MaterialCtx, &mut MeshCtx, &mut SceneCtx> {
    //         CtxRef {
    //             version:  &mut self.version,
    //             geometry: &mut self.geometry,
    //             material: &mut self.material,
    //             mesh:     &mut self.mesh,
    //             scene:    &mut self.scene,
    //         }
    //     }
    // }
    let impl_as_refs_mut = {
        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#struct_lifetimes,)* #(#struct_params,)*> #struct_ident<#(#struct_lifetimes,)* #(#struct_params,)*>
            where
                #(#struct_bounds,)*
            {
                #[inline(always)]
                pub fn as_refs_mut(&mut self) -> #ref_struct_ident<#(&mut #field_types,)*> {
                    #ref_struct_ident {
                        #(#field_idents: &mut self.#field_idents,)*
                    }
                }
            }
        }
    };


    // Generates:
    // impl<'v, V: Debug> HasFields for Ctx<'v, V> {
    //     type Fields = HList![&'v V, GeometryCtx, MaterialCtx, MeshCtx, SceneCtx];
    // }
    let impl_has_fields = {
        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#struct_lifetimes,)* #(#struct_params,)*>
            #lib::HasFields for #struct_ident<#(#struct_lifetimes,)* #(#struct_params,)*>
            where #(#struct_bounds,)* {
                type Fields = #lib::HList!{#(#field_types,)*};
            }
        }
    };

    // Generates:
    // impl<version, geometry, material, mesh, scene>
    // HasFields for CtxRef<version, geometry, material, mesh, scene> {
    //     type Fields = HList![version, geometry, material, mesh, scene];
    // }
    let impl_ref_has_fields = {
        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#params,)*>
            #lib::HasFields for #ref_struct_ident<#(#params,)*> {
                type Fields = #lib::HList!{#(#params,)*};
            }
        }
    };

    // Generates:
    // impl<version_target, geometry_target, material_target, mesh_target, scene_target,
    //      version,        geometry,        material,        mesh,        scene>
    // ReplaceFields<HList![version_target, geometry_target, material_target, mesh_target, scene_target]>
    // for CtxRef<version, geometry, material, mesh, scene> {
    //     type Result = CtxRef<version_target, geometry_target, material_target, mesh_target, scene_target>;
    // }
    let impl_from_fields = {
        let target_params = params.iter().map(|i| Ident::new(&format!("{i}_target"), i.span())).collect_vec();
        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#params,)* #(#target_params,)*>
            #lib::ReplaceFields<#lib::HList!{#(#target_params,)*}> for #ref_struct_ident<#(#params,)*> {
                type Result = #ref_struct_ident<#(#target_params,)*>;
            }
        }
    };

    // Generates:
    // #[macro_export]
    // macro_rules! _Ctx {
    //     (@ [$($ps:tt)*] $lt:lifetime $ts:tt [, $($lt2:lifetime)? * $($xs:tt)*]) => {
    //         _Ctx! { @ [$($ps)*] $lt [
    //             [lifetime_chooser!{ [$lt $($lt2)?] FieldAt<hlist::N0, Ctx<$($ps)*>> }]
    //             [lifetime_chooser!{ [$lt $($lt2)?] FieldAt<hlist::N1, Ctx<$($ps)*>> }]
    //             [lifetime_chooser!{ [$lt $($lt2)?] FieldAt<hlist::N2, Ctx<$($ps)*>> }]
    //             [lifetime_chooser!{ [$lt $($lt2)?] FieldAt<hlist::N3, Ctx<$($ps)*>> }]
    //             [lifetime_chooser!{ [$lt $($lt2)?] FieldAt<hlist::N4, Ctx<$($ps)*>> }]
    //         ] [$ ($xs) *] }
    //     };
    //     (@ [$($ps:tt)*] $lt:lifetime $ts:tt [, $($lt2:lifetime)? mut * $ ($xs:tt) *]) => {
    //         _Ctx! { @ [$($ps)*] $lt [
    //             [lifetime_chooser!{ [$lt $($lt2)?] mut FieldAt<hlist::N0, Ctx<$($ps)*>> }]
    //             [lifetime_chooser!{ [$lt $($lt2)?] mut FieldAt<hlist::N1, Ctx<$($ps)*>> }]
    //             [lifetime_chooser!{ [$lt $($lt2)?] mut FieldAt<hlist::N2, Ctx<$($ps)*>> }]
    //             [lifetime_chooser!{ [$lt $($lt2)?] mut FieldAt<hlist::N3, Ctx<$($ps)*>> }]
    //             [lifetime_chooser!{ [$lt $($lt2)?] mut FieldAt<hlist::N4, Ctx<$($ps)*>> }]
    //         ] [$ ($xs) *] }
    //     };
    //
    //
    //     (@ [$($ps:tt)*] $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt $t4:tt] [, $($lt2:lifetime)? $(ref)? version $ ($xs:tt) *]) => {
    //         _Ctx! { @ [$($ps)*] $lt [[lifetime_chooser!{ [$lt $($lt2)?] FieldAt<hlist::N0, Ctx<$($ps)*>>}] $t1 $t2 $t3 $t4] [$ ($xs) *] }
    //     };
    //     (@ [$($ps:tt)*] $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt $t4:tt] [, $($lt2:lifetime)? $(ref)? geometry $ ($xs:tt) *]) => {
    //         _Ctx! { @ [$($ps)*] $lt [$t0 [lifetime_chooser!{ [$lt $($lt2)?] FieldAt<hlist::N1, Ctx<$($ps)*>>}] $t2 $t3 $t4] [$ ($xs) *] }
    //     };
    //     (@ [$($ps:tt)*] $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt $t4:tt] [, $($lt2:lifetime)? $(ref)? material $ ($xs:tt) *]) => {
    //         _Ctx! { @ [$($ps)*] $lt [$t0 $t1 [lifetime_chooser!{ [$lt $($lt2)?] FieldAt<hlist::N2, Ctx<$($ps)*>>}] $t3 $t4] [$ ($xs) *] }
    //     };
    //     (@ [$($ps:tt)*] $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt $t4:tt] [, $($lt2:lifetime)? $(ref)? mesh $ ($xs:tt) *]) => {
    //         _Ctx! { @ [$($ps)*] $lt [$t0 $t1 $t2 [lifetime_chooser!{ [$lt $($lt2)?] FieldAt<hlist::N3, Ctx<$($ps)*>>}] $t4] [$ ($xs) *] }
    //     };
    //     (@ [$($ps:tt)*] $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt $t4:tt] [, $($lt2:lifetime)? $(ref)? scene $ ($xs:tt) *]) => {
    //         _Ctx! { @ [$($ps)*] $lt [$t0 $t1 $t2 $t3 [lifetime_chooser!{ [$lt $($lt2)?] FieldAt<hlist::N4, Ctx<$($ps)*>>}]] [$ ($xs) *] }
    //     };
    //
    //
    //     (@ [$($ps:tt)*] $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt $t4:tt] [, $($lt2:lifetime)? mut version $ ($xs:tt) *]) => {
    //         _Ctx! { @ [$($ps)*] $lt [[lifetime_chooser!{ [$lt $($lt2)?] mut FieldAt<hlist::N0, Ctx<$($ps)*>>}] $t1 $t2 $t3 $t4] [$ ($xs) *] }
    //     };
    //     (@ [$($ps:tt)*] $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt $t4:tt] [, $($lt2:lifetime)? mut geometry $ ($xs:tt) *]) => {
    //         _Ctx! { @ [$($ps)*] $lt [$t0 [lifetime_chooser!{ [$lt $($lt2)?] mut FieldAt<hlist::N1, Ctx<$($ps)*>>}] $t2 $t3 $t4] [$ ($xs) *] }
    //     };
    //     (@ [$($ps:tt)*] $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt $t4:tt] [, $($lt2:lifetime)?mut material $ ($xs:tt) *]) => {
    //         _Ctx! { @ [$($ps)*] $lt [$t0 $t1 [lifetime_chooser!{ [$lt $($lt2)?] mut FieldAt<hlist::N2, Ctx<$($ps)*>>}] $t3 $t4] [$ ($xs) *] }
    //     };
    //     (@ [$($ps:tt)*] $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt $t4:tt] [, $($lt2:lifetime)? mut mesh $ ($xs:tt) *]) => {
    //         _Ctx! { @ [$($ps)*] $lt [$t0 $t1 $t2 [lifetime_chooser!{ [$lt $($lt2)?] mut FieldAt<hlist::N3, Ctx<$($ps)*>>}] $t4] [$ ($xs) *] }
    //     };
    //     (@ [$($ps:tt)*] $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt $t4:tt] [, $($lt2:lifetime)? mut scene $ ($xs:tt) *]) => {
    //         _Ctx! { @ [$($ps)*] $lt [$t0 $t1 $t2 $t3 [lifetime_chooser!{ [$lt $($lt2)?] mut FieldAt<hlist::N4, Ctx<$($ps)*>>}]] [$ ($xs) *] }
    //     };
    //
    //
    //     (@ [$($ps:tt)*] $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt $t4:tt] [, ! version $ ($xs:tt) *]) => {
    //         _Ctx! { @ [$($ps)*] $lt [[Hidden<FieldAt<hlist::N0, Ctx<$($ps)*>>>] $t1 $t2 $t3 $t4] [$ ($xs) *] }
    //     };
    //
    //     (@ [$($ps:tt)*] $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt $t4:tt] [, ! geometry $ ($xs:tt) *]) => {
    //         _Ctx! { @ [$($ps)*] $lt [$t0 [Hidden<FieldAt<hlist::N1, Ctx<$($ps)*>>>] $t2 $t3 $t4] [$ ($xs) *] }
    //     };
    //     (@ [$($ps:tt)*] $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt $t4:tt] [, ! material $ ($xs:tt) *]) => {
    //         _Ctx! { @ [$($ps)*] $lt [$t0 $t1 [Hidden<FieldAt<hlist::N2, Ctx<$($ps)*>>>] $t3 $t4] [$ ($xs) *] }
    //     };
    //     (@ [$($ps:tt)*] $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt $t4:tt] [, ! mesh $ ($xs:tt) *]) => {
    //         _Ctx! { @ [$($ps)*] $lt [$t0 $t1 $t2 [Hidden<FieldAt<hlist::N3, Ctx<$($ps)*>>>] $t4] [$ ($xs) *] }
    //     };
    //     (@ [$($ps:tt)*] $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt $t4:tt] [, ! scene $ ($xs:tt) *]) => {
    //         _Ctx! { @ [$($ps)*] $lt [$t0 $t1 $t2 $t3 [Hidden<FieldAt<hlist::N4, Ctx<$($ps)*>>>]] [$ ($xs) *] }
    //     };
    //
    //
    //     (@ $ps:tt $lt:lifetime [$ ([$ ($ts:tt) *]) *] [$ (,) *]) => {
    //         CtxRef < $ ($ ($ts) *), * >
    //     };
    //
    //     (@ $($ts:tt)*) => { MACRO_EXPAND_ERROR! { $($ts)* } };
    //
    //     ([$($ps:tt)*] $lt:lifetime $ ($ts:tt) *) => {
    //         _Ctx! { @ [$($ps)*] $lt [
    //             [Hidden<FieldAt<hlist::N0, Ctx<$($ps)*>>>]
    //             [Hidden<FieldAt<hlist::N1, Ctx<$($ps)*>>>]
    //             [Hidden<FieldAt<hlist::N2, Ctx<$($ps)*>>>]
    //             [Hidden<FieldAt<hlist::N3, Ctx<$($ps)*>>>]
    //             [Hidden<FieldAt<hlist::N4, Ctx<$($ps)*>>>]
    //         ] [$($ts)*] }
    //     };
    //
    //     ([$($ps:tt)*] $($ts:tt)*) => {
    //         _Ctx! { @ [$($ps)*] '_ [
    //             [Hidden<FieldAt<hlist::N0, Ctx<$($ps)*>>>]
    //             [Hidden<FieldAt<hlist::N1, Ctx<$($ps)*>>>]
    //             [Hidden<FieldAt<hlist::N2, Ctx<$($ps)*>>>]
    //             [Hidden<FieldAt<hlist::N3, Ctx<$($ps)*>>>]
    //             [Hidden<FieldAt<hlist::N4, Ctx<$($ps)*>>>]
    //         ] [, $ ($ts) *] }
    //     };
    // }
    // pub use _Ctx as Ctx;
    let ref_macro = {
        let fields_at = (0..field_idents.len()).map(|i| {
            let n = Ident::new(&format!("N{}", i), Span::call_site());
            quote! {#lib::FieldAt<#lib::hlist::#n, #struct_ident<$($ps)*>>}
        }
        ).collect_vec();

        let all_hidden = quote! {#([#lib::Hidden<#fields_at>])*};
        let all_ref = quote! {#([#lib::lifetime_chooser!{[$lt $($lt2)?] #fields_at}])*};
        let all_ref_mut = quote! {#([#lib::lifetime_chooser!{[$lt $($lt2)?] mut #fields_at}])*};
        let ts_idents = field_idents.iter().enumerate().map(|(i, _)| Ident::new(&format!("t{i}"), Span::call_site())).collect_vec();
        let ts = ts_idents.iter().map(|t| quote!($#t)).collect_vec();
        let struct_ident2 = Ident::new(&format!("_{}", struct_ident), struct_ident.span());
        let gen_patterns = |pattern: pm::TokenStream, f: Box<dyn Fn(&pm::TokenStream) -> pm::TokenStream>| {
            field_idents.iter().zip(fields_at.iter()).enumerate().map(|(i, (name, tp))| {
                let result = f(tp);
                let mut results = ts.iter().collect_vec();
                results[i] = &result;
                quote! {
                    (@ [$($ps:tt)*] $lt:lifetime [#(#ts:tt)*] [, #pattern #name $($xs:tt)*]) => {
                        $crate::#struct_ident! {@ [$($ps)*]  $lt [#(#results)*] [$($xs)*]}
                    };
                }
            }).collect_vec()
        };
        let patterns_ref = gen_patterns(quote!{$($lt2:lifetime)? $(ref)?}, Box::new(|t: &pm::TokenStream| quote!{[#lib::lifetime_chooser!{[$lt $($lt2)?] #t}]}));
        let patterns_ref_mut = gen_patterns(quote!{$($lt2:lifetime)? mut}, Box::new(|t: &pm::TokenStream| quote!{[#lib::lifetime_chooser!{[$lt $($lt2)?] mut #t}]}));
        let patterns_ref_none = gen_patterns(quote!{!}, Box::new(|t: &pm::TokenStream| quote!{[#lib::Hidden<#t >]}));
        quote! {
            #[macro_export]
            macro_rules! #struct_ident2 {
                (@ [$($ps:tt)*]  $lt:lifetime [#(#ts:tt)*] [, ! * $($xs:tt)*]) => {
                    $crate::#struct_ident! {@ [$($ps)*]  $lt [#all_hidden] [$($xs)*]}
                };
                (@ [$($ps:tt)*]  $lt:lifetime [#(#ts:tt)*] [, $($lt2:lifetime)? * $($xs:tt)*]) => {
                    $crate::#struct_ident! {@ [$($ps)*]  $lt [#all_ref] [$($xs)*]}
                };
                (@ [$($ps:tt)*]  $lt:lifetime [#(#ts:tt)*] [, $($lt2:lifetime)? mut * $($xs:tt)*]) => {
                    $crate::#struct_ident! {@ [$($ps)*]  $lt [#all_ref_mut] [$($xs)*]}
                };
                #(#patterns_ref)*
                #(#patterns_ref_mut)*
                #(#patterns_ref_none)*
                (@ [$($ps:tt)*]  $lt:lifetime [$([$($ts:tt)*])*] [$(,)*]) => { #module::#ref_struct_ident<$($($ts)*),*> };
                (@ [$($ps:tt)*]  $($ts:tt)*) => { MACRO_EXPANSION_ERROR!($($ts)*) };

                ([$($ps:tt)*] $lt:lifetime, $($ts:tt)*) => {
                    $crate::#struct_ident! {@ [$($ps)*] $lt [#all_hidden] [, $($ts)*]}
                };
                ([$($ps:tt)*] $($ts:tt)*) => {
                    $crate::#struct_ident! {@ [$($ps)*] '_ [#all_hidden] [,$($ts)*]}
                };
            }

            pub use #struct_ident2 as #struct_ident;
        }
    };

    // Generates:
    // #[allow(non_camel_case_types)]
    // impl<'t, version, geometry, material, mesh, scene>
    //     CtxRef<version, geometry, material, mesh, scene>
    // where
    // {
    //     #[inline(always)]
    //     pub fn extract_version(&'t mut self) -> (
    //         <version as RefFlatten<'t>>::Output,
    //         &'t mut CtxRef<Acquired<version, version>, geometry, material, mesh, scene>
    //     ) where version: Acquire<version> + RefFlatten<'t> {
    //         let rest = unsafe { &mut *(self as *mut _ as *mut _) };
    //         (self.version.ref_flatten(), rest)
    //     }
    //
    //     ...
    // }
    let impl_extract_fields = {
        let idents_str = field_idents.iter().map(|t| t.to_string()).collect_vec();
        let fns = idents_str.iter().map(|field_str| {
            let params = idents_str.iter().map(|i| {
                if i == field_str {
                    let ident = Ident::new(i, Span::call_site());
                    quote!{#lib::Acquired<#ident, #ident>}
                } else {
                    let ident = Ident::new(i, Span::call_site());
                    quote!{#ident}
                }
            }).collect_vec();
            let field = Ident::new(&field_str, Span::call_site());
            let name = Ident::new(&format!("extract_{field}"), field.span());
            quote! {
                #[inline(always)]
                pub fn #name(&'t mut self) -> (
                    <#field as #lib::RefFlatten<'t>>::Output,
                    &'t mut #ref_struct_ident<#(#params,)*>
                ) where #field: #lib::Acquire<#field> + #lib::RefFlatten<'t> {
                    let rest = unsafe { &mut *(self as *mut _ as *mut _) };
                    (self.#field.ref_flatten(), rest)
                }
            }
        }).collect_vec();
        quote! {
            #[allow(non_camel_case_types)]
            impl<'t, #(#params,)*> #ref_struct_ident<#(#params,)*> where
            {
                #(#fns)*
            }
        }
    };

    let out = quote! {
        #ref_struct
        #impl_inference_guide
        #impl_as_refs
        #impl_as_refs_mut
        #impl_has_fields
        #impl_ref_has_fields
        #impl_from_fields
        #ref_macro
        #impl_extract_fields
    };

    // println!(">>> {}", out);
    TokenStream::from(out)
}
