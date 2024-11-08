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
#[proc_macro_derive(Split, attributes(module))]
pub fn split_derive(input: TokenStream) -> TokenStream {
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
    let bounds_params_access = quote! { #(#params: #lib::Access,)* };
    let field_values = field_types.iter().zip(params.iter()).map(|(field_ty, param)| {
        quote! { #lib::Value<'_t, #param, #field_ty> }
    }).collect_vec();

    // Generates:
    // #[repr(C)]
    // pub struct CtxRef<'t, geometry, material, mesh, scene> {
    //     geometry: Value<'t, geometry, GeometryCtx>,
    //     material: Value<'t, material, MaterialCtx>,
    //     mesh: Value<'t, mesh, MeshCtx>,
    //     scene: Value<'t, scene, SceneCtx>,
    // }
    let ref_struct = quote! {
        #[derive(Debug)]
        #[repr(C)]
        #[allow(non_camel_case_types)]
        pub struct #ref_struct_ident<'_t, #(#params),*>
        where #bounds_params_access {
            #(pub #field_idents : #field_values),*
        }
    };

    // Generates:
    // impl<'t, geometry, material, mesh, scene>
    //     AsRefs<'t, CtxRef<'t, geometry, material, mesh, scene>> for Ctx
    // where
    //     geometry:    Access,
    //     material:    Access,
    //     mesh:        Access,
    //     scene:       Access,
    //     GeometryCtx: RefCast<'t, Value<'t, geometry, GeometryCtx>>,
    //     MaterialCtx: RefCast<'t, Value<'t, material, MaterialCtx>>,
    //     MeshCtx:     RefCast<'t, Value<'t, mesh,     MeshCtx>>,
    //     SceneCtx:    RefCast<'t, Value<'t, scene,    SceneCtx>>,
    // {
    //     fn as_refs_impl(&'t mut self) -> CtxRef<'t, geometry, material, mesh, scene> {
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
        #lib::AsRefs<'_t, #ref_struct_ident<'_t, #(#params,)*>> for #struct_ident
        where #bounds_params_access #(#field_types: #lib::RefCast<'_t, #field_values>,)* {
            fn as_refs_impl(& '_t mut self) -> #ref_struct_ident<'_t, #(#params,)*> {
                #ref_struct_ident {
                    #(#field_idents: #lib::RefCast::ref_cast(&mut self.#field_idents),)*
                }
            }
        }
    };

    // Generates:
    // impl Ctx {
    //     pub fn as_ref_mut<'t>(&'t mut self) -> CtxRef<'t, RefMut, RefMut, RefMut, RefMut> {
    //         CtxRef {
    //             geometry: &mut self.geometry,
    //             material: &mut self.material,
    //             mesh:     &mut self.mesh,
    //             scene:    &mut self.scene,
    //         }
    //     }
    // }
    let impl_as_ref_mut = {
        let ref_muts = params.iter().map(|_| quote!{#lib::RefMut}).collect_vec();
        quote! {
            #[allow(non_camel_case_types)]
            impl #struct_ident {
                pub fn as_ref_mut<'_t>(&'_t mut self) -> #ref_struct_ident<'_t, #(#ref_muts,)*> {
                    #ref_struct_ident {
                        #(#field_idents: &mut self.#field_idents,)*
                    }
                }
            }
        }
    };

    // Generates:
    // impl<'t, geometry_target, material_target, mesh_target, scene_target,
    //          geometry,        material,        mesh,        scene>
    // Split<CtxRef<'t, geometry_target, material_target, mesh_target, scene_target>>
    // for CtxRef<'t, geometry,        material,        mesh,        scene>
    // where
    //     geometry:        Access,
    //     material:        Access,
    //     mesh:            Access,
    //     scene:           Access,
    //     geometry_target: Access,
    //     material_target: Access,
    //     mesh_target:     Access,
    //     scene_target:    Access,
    //     geometry:        Acquire<geometry_target>,
    //     material:        Acquire<material_target>,
    //     mesh:            Acquire<mesh_target>,
    //     scene:           Acquire<scene_target>,
    // {
    //     type Rest = CtxRef<'t,
    //         Acquired<geometry, target_geometry>,
    //         Acquired<material, target_material>,
    //         Acquired<mesh,     target_mesh>,
    //         Acquired<scene,    target_scene>,
    //     >;
    // }
    let impl_split = {
        let target_params = params.iter().map(|i| Ident::new(&format!("{i}_target"), i.span())).collect_vec();
        let bounds_target_params_access = quote! { #(#target_params: #lib::Access,)* };
        quote! {
            #[allow(non_camel_case_types)]
            impl<'_t, #(#params,)* #(#target_params,)*>
            #lib::Split<#ref_struct_ident<'_t, #(#target_params,)*>> for #ref_struct_ident<'_t, #(#params,)*>
            where
                #bounds_params_access
                #bounds_target_params_access
                #(#params: #lib::Acquire<#target_params>,)*
            {
                type Rest = #ref_struct_ident<'_t, #(#lib::Acquired<#params, #target_params>,)*>;
            }
        }
    };

    // Generates:
    // #[macro_export]
    // macro_rules! _Ctx {
    //     ($lt:lifetime $ ($ts:tt) *) => {
    //         CtxImpl! { $lt [[None] [None] [None] [None]] [$($ts)*] }
    //     };
    //     ($($ts:tt)*) => {
    //         CtxImpl! { '_ [[None] [None] [None] [None]] [, $ ($ts) *] }
    //     };
    // }
    //
    // #[macro_export]
    // macro_rules! CtxImpl {
    //     ($lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [,* $($xs:tt)*]) => {
    //         CtxImpl! { $lt [[Ref] [Ref] [Ref] [Ref]] [$ ($xs) *] }
    //     };
    //     ($lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, mut * $ ($xs:tt) *]) => {
    //         CtxImpl! { $lt [[RefMut] [RefMut] [RefMut] [RefMut]] [$ ($xs) *] }
    //     };
    //
    //
    //     ($lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, $(ref)? geometry $ ($xs:tt) *]) => {
    //         CtxImpl! { $lt [[Ref] $t1 $t2 $t3] [$ ($xs) *] }
    //     };
    //     ($lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, $(ref)? material $ ($xs:tt) *]) => {
    //         CtxImpl! { $lt [$t0 [Ref] $t2 $t3] [$ ($xs) *] }
    //     };
    //     ($lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, $(ref)? mesh $ ($xs:tt) *]) => {
    //         CtxImpl! { $lt [$t0 $t1 [Ref] $t3] [$ ($xs) *] }
    //     };
    //     ($lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, $(ref)? scene $ ($xs:tt) *]) => {
    //         CtxImpl! { $lt [$t0 $t1 $t2 [Ref]] [$ ($xs) *] }
    //     };
    //
    //
    //     ($lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, mut geometry $ ($xs:tt) *]) => {
    //         CtxImpl! { $lt [[RefMut] $t1 $t2 $t3] [$ ($xs) *] }
    //     };
    //     ($lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, mut material $ ($xs:tt) *]) => {
    //         CtxImpl! { $lt [$t0 [RefMut] $t2 $t3] [$ ($xs) *] }
    //     };
    //     ($lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, mut mesh $ ($xs:tt) *]) => {
    //         CtxImpl! { $lt [$t0 $t1 [RefMut] $t3] [$ ($xs) *] }
    //     };
    //     ($lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, mut scene $ ($xs:tt) *]) => {
    //         CtxImpl! { $lt [$t0 $t1 $t2 [RefMut]] [$ ($xs) *] }
    //     };
    //
    //
    //     ($lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, ! geometry $ ($xs:tt) *]) => {
    //         CtxImpl! { $lt [[None] $t1 $t2 $t3] [$ ($xs) *] }
    //     };
    //     ($lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, ! material $ ($xs:tt) *]) => {
    //         CtxImpl! { $lt [$t0 [None] $t2 $t3] [$ ($xs) *] }
    //     };
    //     ($lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, ! mesh $ ($xs:tt) *]) => {
    //         CtxImpl! { $lt [$t0 $t1 [None] $t3] [$ ($xs) *] }
    //     };
    //     ($lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, ! scene $ ($xs:tt) *]) => {
    //         CtxImpl! { $lt [$t0 $t1 $t2 [None]] [$ ($xs) *] }
    //     };
    //
    //
    //     ($lt:lifetime [$ ([$ ($ts:tt) *]) *] [$ (,) *]) => {
    //         CtxRef < $lt, $ ($ ($ts) *), * >
    //     };
    // }
    // pub use _Ctx as Ctx;
    let ref_macro = {
        let q_none = quote! {[#lib::None]};
        let q_ref = quote! {[#lib::Ref]};
        let q_ref_mut = quote! {[#lib::RefMut]};
        let all_none = field_idents.iter().map(|_| &q_none).collect_vec();
        let all_ref = field_idents.iter().map(|_| &q_ref).collect_vec();
        let all_ref_mut = field_idents.iter().map(|_| &q_ref_mut).collect_vec();
        let ts_idents = field_idents.iter().enumerate().map(|(i, _)| Ident::new(&format!("t{i}"), Span::call_site())).collect_vec();
        let ts = ts_idents.iter().map(|t| quote!($#t)).collect_vec();
        let struct_ident2 = Ident::new(&format!("_{}", struct_ident), struct_ident.span());
        let gen_patterns = |pattern: pm::TokenStream, access: &pm::TokenStream| {
            field_idents.iter().enumerate().map(|(i, name)| {
                let mut result = ts.iter().collect_vec();
                result[i] = access;
                quote! { (@ $lt:lifetime [#(#ts:tt)*] [, #pattern #name $($xs:tt)*]) => {
                $crate::#struct_ident! {@ $lt [#(#result)*] [$($xs)*]} };
            }
            }).collect_vec()
        };
        let patterns_ref = gen_patterns(quote!{$(ref)?}, &q_ref);
        let patterns_ref_mut = gen_patterns(quote!{mut}, &q_ref_mut);
        let patterns_ref_none = gen_patterns(quote!{!}, &q_none);
        quote! {
            #[macro_export]
            macro_rules! #struct_ident2 {
                (@ $lt:lifetime [#(#ts:tt)*] [, ! * $($xs:tt)*]) => {
                    $crate::#struct_ident! {@ $lt [#(#all_none)*] [$($xs)*]}
                };
                (@ $lt:lifetime [#(#ts:tt)*] [, * $($xs:tt)*]) => {
                    $crate::#struct_ident! {@ $lt [#(#all_ref)*] [$($xs)*]}
                };
                (@ $lt:lifetime [#(#ts:tt)*] [, mut * $($xs:tt)*]) => {
                    $crate::#struct_ident! {@ $lt [#(#all_ref_mut)*] [$($xs)*]}
                };
                #(#patterns_ref)*
                #(#patterns_ref_mut)*
                #(#patterns_ref_none)*
                (@ $lt:lifetime [$([$($ts:tt)*])*] [$(,)*]) => { #module::#ref_struct_ident<$lt, $($($ts)*),*> };
                (@ $($ts:tt)*) => { error };

                ($lt:lifetime $($ts:tt)*) => {
                    $crate::#struct_ident! {@ $lt [#(#all_none)*] [$($ts)*]}
                };
                ($($ts:tt)*) => {
                    $crate::#struct_ident! {@ '_ [#(#all_none)*] [,$($ts)*]}
                };
            }

            pub use #struct_ident2 as #struct_ident;
        }
    };

    // Generates:
    // impl<'t, geometry, material, mesh, scene>
    // CtxRef<'t, geometry, material, mesh, scene>
    // where geometry: Access, material: Access, mesh: Access, scene: Access {
    //     pub fn extract_geometry(&mut self)
    //         -> (&mut GeometryCtx, &mut <Self as Split<Ctx!['t, mut geometry]>>::Rest)
    //     where geometry: Acquire<RefMut> {
    //         let (a, b) = <Self as Split<Ctx! ['t, mut geometry]>>::split_impl(self);
    //         (a.geometry, b)
    //     }
    //     ...
    // }
    let impl_extract_fields = {
        let fns = field_idents.iter().zip(field_types.iter()).map(|(field, ty)| {
            let name = Ident::new(&format!("extract_{field}"), field.span());
            quote! {
                pub fn #name(&mut self) -> (&mut #ty, &mut <Self as #lib::Split<#struct_ident!['_t, mut #field]>>::Rest)
                where #field: #lib::Acquire<#lib::RefMut> {
                    let (a, b) = <Self as #lib::Split<#struct_ident!['_t, mut #field]>>::split_impl(self);
                    (a.#field, b)
                }
            }
        }).collect_vec();
        quote! {
            #[allow(non_camel_case_types)]
            impl<'_t, #(#params,)*> #ref_struct_ident<'_t, #(#params,)*>
            where #bounds_params_access {
                #(#fns)*
            }
        }
    };

    let out = quote! {
        #ref_struct
        #impl_as_refs
        #impl_as_ref_mut
        #impl_split
        #ref_macro
        #impl_extract_fields
    };

    // println!(">>> {}", out);
    TokenStream::from(out)
}
