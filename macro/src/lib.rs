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
/// pub struct Ctx {
///     geometry: GeometryCtx,
///     material: MaterialCtx,
///     mesh: MeshCtx,
///     scene: SceneCtx,
/// }
/// ```
#[proc_macro_derive(PartialBorrow, attributes(module))]
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

    let field_idents = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect_vec();
    let field_types = fields.iter().map(|f| &f.ty).collect_vec();
    let params = field_idents.iter().map(|i| Ident::new(&i.to_string(), i.span())).collect_vec();

    // Generates:
    // #[repr(C)]
    // pub struct CtxRef<geometry, material, mesh, scene> {
    //     geometry: geometry,
    //     material: material,
    //     mesh: mesh,
    //     scene: scene,
    // }
    let ref_struct = quote! {
        #[derive(Debug)]
        #[repr(C)]
        #[allow(non_camel_case_types)]
        pub struct #ref_struct_ident<#(#params),*> {
            #(pub #field_idents : #params),*
        }
    };

    // Generates:
    // impl<'t, geometry, material, mesh, scene>
    //     AsRefs<'t, CtxRef<geometry, material, mesh, scene>> for Ctx
    // where
    //     GeometryCtx: RefCast<'t, geometry>,
    //     MaterialCtx: RefCast<'t, material>,
    //     MeshCtx:     RefCast<'t, mesh>,
    //     SceneCtx:    RefCast<'t, scene>,
    // {
    //     fn as_refs_impl(&'t mut self) -> CtxRef<geometry, material, mesh, scene> {
    //         CtxRef {
    //             geometry: RefCast::ref_cast(&mut self.geometry),
    //             material: RefCast::ref_cast(&mut self.material),
    //             mesh:     RefCast::ref_cast(&mut self.mesh),
    //             scene:    RefCast::ref_cast(&mut self.scene),
    //         }
    //     }
    // }
    let impl_as_refs = quote! {
        #[allow(non_camel_case_types)]
        impl<'_t, #(#params,)*>
        #lib::AsRefs<'_t, #ref_struct_ident<#(#params,)*>> for #struct_ident
        where #(#field_types: #lib::RefCast<'_t, #params>,)* {
            #[inline(always)]
            fn as_refs_impl(& '_t mut self) -> #ref_struct_ident<#(#params,)*> {
                #ref_struct_ident {
                    #(#field_idents: #lib::RefCast::ref_cast(&mut self.#field_idents),)*
                }
            }
        }
    };

    // Generates:
    // impl Ctx {
    //     pub fn as_refs_mut(&mut self) -> CtxRef<&mut GeometryCtx, &mut MaterialCtx, &mut MeshCtx, &mut SceneCtx> {
    //         CtxRef {
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
            impl #struct_ident {
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
    // impl<geometry, material, mesh, scene>
    // HasFields for CtxRef<geometry, material, mesh, scene> {
    //     type Fields = HList![geometry, material, mesh, scene];
    // }
    let impl_into_fields = {
        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#params,)*>
            #lib::HasFields for #ref_struct_ident<#(#params,)*> {
                type Fields = #lib::HList!{#(#params,)*};
            }
        }
    };

    // Generates:
    // impl<geometry_target, material_target, mesh_target, scene_target,
    //      geometry,        material,        mesh,        scene>
    // ReplaceFields<HList![geometry_target, material_target, mesh_target, scene_target]>
    // for CtxRef<geometry, material, mesh, scene> {
    //     type Result = CtxRef<geometry_target, material_target, mesh_target, scene_target>;
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
    //     (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, $($lt2:lifetime)? * $($xs:tt)*]) => {
    //         _Ctx! { @ $lt [
    //             [lifetime_chooser!{ $lt $($lt2)? GeometryCtx }]
    //             [lifetime_chooser!{ $lt $($lt2)? MaterialCtx }]
    //             [lifetime_chooser!{ $lt $($lt2)? MeshCtx }]
    //             [lifetime_chooser!{ $lt $($lt2)? SceneCtx }]
    //         ] [$ ($xs) *] }
    //     };
    //     (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, mut * $ ($xs:tt) *]) => {
    //         _Ctx! { @ $lt [
    //             [lifetime_chooser!{ $lt $($lt2)? mut GeometryCtx }]
    //             [lifetime_chooser!{ $lt $($lt2)? mut MaterialCtx }]
    //             [lifetime_chooser!{ $lt $($lt2)? mut MeshCtx }]
    //             [lifetime_chooser!{ $lt $($lt2)? mut SceneCtx }]
    //         ] [$ ($xs) *] }
    //     };
    //
    //
    //     (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, $($lt2:lifetime)? $(ref)? geometry $ ($xs:tt) *]) => {
    //         _Ctx! { @ $lt [[lifetime_chooser!{ $lt $($lt2)? GeometryCtx}] $t1 $t2 $t3] [$ ($xs) *] }
    //     };
    //     (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, $($lt2:lifetime)? $(ref)? material $ ($xs:tt) *]) => {
    //         _Ctx! { @ $lt [$t0 [lifetime_chooser!{ $lt $($lt2)? MaterialCtx}] $t2 $t3] [$ ($xs) *] }
    //     };
    //     (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, $($lt2:lifetime)? $(ref)? mesh $ ($xs:tt) *]) => {
    //         _Ctx! { @ $lt [$t0 $t1 [lifetime_chooser!{ $lt $($lt2)? MeshCtx}] $t3] [$ ($xs) *] }
    //     };
    //     (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, $($lt2:lifetime)? $(ref)? scene $ ($xs:tt) *]) => {
    //         _Ctx! { @ $lt [$t0 $t1 $t2 [lifetime_chooser!{ $lt $($lt2)? SceneCtx}]] [$ ($xs) *] }
    //     };
    //
    //
    //     (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, $($lt2:lifetime)? mut geometry $ ($xs:tt) *]) => {
    //         _Ctx! { @ $lt [[lifetime_chooser!{ $lt $($lt2)? mut GeometryCtx}] $t1 $t2 $t3] [$ ($xs) *] }
    //     };
    //     (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, $($lt2:lifetime)?mut material $ ($xs:tt) *]) => {
    //         _Ctx! { @ $lt [$t0 [lifetime_chooser!{ $lt $($lt2)? mut MaterialCtx}] $t2 $t3] [$ ($xs) *] }
    //     };
    //     (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, $($lt2:lifetime)? mut mesh $ ($xs:tt) *]) => {
    //         _Ctx! { @ $lt [$t0 $t1 [lifetime_chooser!{ $lt $($lt2)? mut MeshCtx}] $t3] [$ ($xs) *] }
    //     };
    //     (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, $($lt2:lifetime)? mut scene $ ($xs:tt) *]) => {
    //         _Ctx! { @ $lt [$t0 $t1 $t2 [lifetime_chooser!{ $lt $($lt2)? mut SceneCtx}]] [$ ($xs) *] }
    //     };
    //
    //
    //     (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, ! geometry $ ($xs:tt) *]) => {
    //         _Ctx! { @ $lt [[Hidden<GeometryCtx>] $t1 $t2 $t3] [$ ($xs) *] }
    //     };
    //     (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, ! material $ ($xs:tt) *]) => {
    //         _Ctx! { @ $lt [$t0 [Hidden<MaterialCtx>] $t2 $t3] [$ ($xs) *] }
    //     };
    //     (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, ! mesh $ ($xs:tt) *]) => {
    //         _Ctx! { @ $lt [$t0 $t1 [Hidden<MeshCtx>] $t3] [$ ($xs) *] }
    //     };
    //     (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, ! scene $ ($xs:tt) *]) => {
    //         _Ctx! { @ $lt [$t0 $t1 $t2 [Hidden<SceneCtx>]] [$ ($xs) *] }
    //     };
    //
    //
    //     (@ $lt:lifetime [$ ([$ ($ts:tt) *]) *] [$ (,) *]) => {
    //         CtxRef < $ ($ ($ts) *), * >
    //     };
    //
    //     (@ $($ts:tt)*) => { error { $($ts)* } };
    //
    //     ($lt:lifetime $ ($ts:tt) *) => {
    //         _Ctx! { @ $lt [[Hidden<GeometryCtx>] [Hidden<MaterialCtx>] [Hidden<MeshCtx>] [Hidden<SceneCtx>]] [$($ts)*] }
    //     };
    //
    //     ($($ts:tt)*) => {
    //         _Ctx! { @ '_ [[Hidden<GeometryCtx>] [Hidden<MaterialCtx>] [Hidden<MeshCtx>] [Hidden<SceneCtx>]] [, $ ($ts) *] }
    //     };
    // }
    // pub use _Ctx as Ctx;
    let ref_macro = {
        let all_hidden = quote! {#([#lib::Hidden<#module::#field_types>])*};
        let all_ref = quote! {#([#lib::lifetime_chooser!{$lt $($lt2)? #module::#field_types}])*};
        let all_ref_mut = quote! {#([#lib::lifetime_chooser!{$lt $($lt2)? mut #module::#field_types}])*};
        let ts_idents = field_idents.iter().enumerate().map(|(i, _)| Ident::new(&format!("t{i}"), Span::call_site())).collect_vec();
        let ts = ts_idents.iter().map(|t| quote!($#t)).collect_vec();
        let struct_ident2 = Ident::new(&format!("_{}", struct_ident), struct_ident.span());
        let gen_patterns = |pattern: pm::TokenStream, f: Box<dyn Fn(&syn::Type) -> pm::TokenStream>| {
            field_idents.iter().zip(field_types.iter()).enumerate().map(|(i, (name, tp))| {
                let result = f(tp);
                let mut results = ts.iter().collect_vec();
                results[i] = &result;
                quote! { (@ $lt:lifetime [#(#ts:tt)*] [, #pattern #name $($xs:tt)*]) => {
                $crate::#struct_ident! {@ $lt [#(#results)*] [$($xs)*]} };
            }
            }).collect_vec()
        };
        let patterns_ref = gen_patterns(quote!{$($lt2:lifetime)? $(ref)?}, Box::new(|t: &syn::Type| quote!{[#lib::lifetime_chooser!{$lt $($lt2)? #module::#t}]}));
        let patterns_ref_mut = gen_patterns(quote!{$($lt2:lifetime)? mut}, Box::new(|t: &syn::Type| quote!{[#lib::lifetime_chooser!{$lt $($lt2)? mut #module::#t}]}));
        let patterns_ref_none = gen_patterns(quote!{!}, Box::new(|t: &syn::Type| quote!{[#lib::Hidden<#module::#t>]}));
        quote! {
            #[macro_export]
            macro_rules! #struct_ident2 {
                (@ $lt:lifetime [#(#ts:tt)*] [, ! * $($xs:tt)*]) => {
                    $crate::#struct_ident! {@ $lt [#all_hidden] [$($xs)*]}
                };
                (@ $lt:lifetime [#(#ts:tt)*] [, $($lt2:lifetime)? * $($xs:tt)*]) => {
                    $crate::#struct_ident! {@ $lt [#all_ref] [$($xs)*]}
                };
                (@ $lt:lifetime [#(#ts:tt)*] [, $($lt2:lifetime)? mut * $($xs:tt)*]) => {
                    $crate::#struct_ident! {@ $lt [#all_ref_mut] [$($xs)*]}
                };
                #(#patterns_ref)*
                #(#patterns_ref_mut)*
                #(#patterns_ref_none)*
                (@ $lt:lifetime [$([$($ts:tt)*])*] [$(,)*]) => { #module::#ref_struct_ident<$($($ts)*),*> };
                (@ $($ts:tt)*) => { error {$($ts)*} };

                ($lt:lifetime $($ts:tt)*) => {
                    $crate::#struct_ident! {@ $lt [#all_hidden] [$($ts)*]}
                };
                ($($ts:tt)*) => {
                    $crate::#struct_ident! {@ '_ [#all_hidden] [,$($ts)*]}
                };
            }

            pub use #struct_ident2 as #struct_ident;
        }
    };

    // Generates:
    // impl<'t1, 't2, 't3, 't4, geometry, material, mesh, scene>
    // CtxRef<geometry, material, mesh, scene> where
    // 't1: 't2,
    // 't4: 't2,
    // 't1: 't3,
    // 't4: 't3
    // {
    //     pub fn extract_geometry(&'t1 mut self)
    //         -> (&'t2 mut GeometryCtx, &'t3 mut <Self as PartialBorrow<Ctx!['t4, mut geometry]>>::Rest)
    //     where geometry: Acquire<&'t4 mut GeometryCtx> {
    //         let (a, b) = <Self as PartialBorrow<Ctx! ['t4, mut geometry]>>::split_impl(self);
    //         (a.geometry, b)
    //     }
    //
    //     ...
    //
    // }
    let impl_extract_fields = {
        let fns = field_idents.iter().zip(field_types.iter()).map(|(field, ty)| {
            let name = Ident::new(&format!("extract_{field}"), field.span());
            quote! {
                #[inline(always)]
                pub fn #name(&'_t1 mut self) -> (&'_t2 mut #ty, &'_t3 mut <Self as #lib::PartialBorrow<#struct_ident!['_t4, mut #field]>>::Rest)
                where #field: #lib::Acquire<&'_t4 mut #ty> {
                    let (a, b) = <Self as #lib::PartialBorrow<#struct_ident!['_t4, mut #field]>>::split_impl(self);
                    (a.#field, b)
                }
            }
        }).collect_vec();
        quote! {
            #[allow(non_camel_case_types)]
            impl<'_t1, '_t2, '_t3, '_t4, #(#params,)*> #ref_struct_ident<#(#params,)*> where
            '_t1: '_t2,
            '_t4: '_t2,
            '_t1: '_t3,
            '_t4: '_t3
            {
                #(#fns)*
            }
        }
    };

    let out = quote! {
        #ref_struct
        #impl_as_refs
        #impl_as_refs_mut
        #ref_macro
        #impl_extract_fields
        #impl_into_fields
        #impl_from_fields
    };

    // println!(">>> {}", out);
    TokenStream::from(out)
}
