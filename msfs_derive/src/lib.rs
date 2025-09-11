use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{
    Expr, Ident, ItemFn, ItemStruct, Lit, Meta, Token, Type,
    parse::{Parse, ParseStream, Result as SynResult},
    parse_macro_input,
};

/// Declare a standalone module.
/// ```rs
/// #[standalone_module]
/// async fn module(mut module: msfs::StandaloneModule) -> Result<(), Box<dyn std::error::Error>> {
///   while let Some(event) = module.next_event().await {
///     // ...
///   }
/// }
/// ```
#[proc_macro_attribute]
pub fn standalone_module(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let rusty_name = input.sig.ident.clone();
    let executor_name = format_ident!(
        "{}_executor_do_not_use_or_you_will_be_fired",
        input.sig.ident
    );

    let output = quote! {
        #input

        // SAFETY: it is safe to create references of this static since all WASM modules are single threaded
        // and there is only 1 reference in use at all times
        #[allow(non_upper_case_globals)]
        static mut #executor_name: ::msfs::StandaloneModuleExecutor = ::msfs::StandaloneModuleExecutor {
            executor: ::msfs::executor::Executor {
                handle: |m| std::boxed::Box::pin(#rusty_name(m)),
                future: None,
                tx: None,
            },
        };

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn module_init() {
            ::msfs::wrap_executor(&raw mut #executor_name, |e| e.handle_init());
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn module_deinit() {
            ::msfs::wrap_executor(&raw mut #executor_name, |e| e.handle_deinit());
        }
    };

    TokenStream::from(output)
}

struct GaugeArgs {
    name: Option<String>,
}

impl Parse for GaugeArgs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        match input.parse::<Ident>() {
            Ok(i) if i == "name" => {
                input.parse::<Token![=]>()?;
                Ok(GaugeArgs {
                    name: Some(input.parse::<Ident>()?.to_string()),
                })
            }
            _ => Ok(GaugeArgs { name: None }),
        }
    }
}

/// Declare a gauge callback. It will be automatically exported with the name
/// `NAME_gauge_callback`, where `NAME` is the name of the decorated function.
/// ```rs
/// use futures::stream::{Stream, StreamExt};
/// // Declare and export `FOO_gauge_callback`
/// #[msfs::gauge]
/// async fn FOO(mut gauge: msfs::Gauge) -> Result<(), Box<dyn std::error::Error>> {
///   while let Some(event) = gauge.next_event().await {
///     // ...
///   }
/// }
/// ```
///
/// The macro can also be given a parameter, `name`, to rename the exported function.
/// ```rs
/// // Declare and export `FOO_gauge_callback`
/// #[msfs::gauge(name=FOO)]
/// async fn xyz(...) {}
#[proc_macro_attribute]
pub fn gauge(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as GaugeArgs);
    let input = parse_macro_input!(item as ItemFn);

    let rusty_name = input.sig.ident.clone();
    let executor_name = format_ident!(
        "{}_executor_do_not_use_or_you_will_be_fired",
        input.sig.ident
    );

    let extern_name = args.name.unwrap_or_else(|| input.sig.ident.to_string());
    let extern_gauge_name = format_ident!("{}_gauge_callback", extern_name);
    let extern_mouse_name = format_ident!("{}_mouse_callback", extern_name);

    let output = quote! {
        #input

        // SAFETY: it is safe to create references of this static since all WASM modules are single threaded
        // and there is only 1 reference in use at all times
        #[allow(non_upper_case_globals)]
        static mut #executor_name: ::msfs::GaugeExecutor = ::msfs::GaugeExecutor {
            fs_ctx: None,
            executor: ::msfs::executor::Executor {
                handle: |gauge| std::boxed::Box::pin(#rusty_name(gauge)),
                tx: None,
                future: None,
            },
        };

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn #extern_gauge_name(
            ctx: ::msfs::sys::FsContext,
            service_id: std::os::raw::c_int,
            p_data: *mut std::os::raw::c_void,
        ) -> bool {
            unsafe {
                ::msfs::wrap_executor(&raw mut #executor_name, |e| e.handle_gauge(ctx, service_id, p_data))
            }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn #extern_mouse_name(
            fx: std::os::raw::c_float,
            fy: std::os::raw::c_float,
            i_flags: std::os::raw::c_uint,
        ) {
            unsafe {
                ::msfs::wrap_executor(&raw mut #executor_name, |e| e.handle_mouse(fx, fy, i_flags));
            }
         }
    };

    TokenStream::from(output)
}

fn parse_struct_fields(
    input: &mut ItemStruct,
    attributes: &[&str],
    get_type: Option<fn(&str) -> &str>,
) -> Vec<HashMap<String, String>> {
    let mut data = Vec::new();

    for (i, field) in &mut input.fields.iter_mut().enumerate() {
        let mut meta = HashMap::new();

        meta.insert(
            "field_name".to_string(),
            if let Some(ident) = &field.ident {
                ident.to_string()
            } else {
                i.to_string()
            },
        );

        if let Some(get_type) = get_type {
            let ty = match &field.ty {
                Type::Path(p) => p.path.get_ident().unwrap().to_string(),
                _ => panic!("Unsupported type"),
            };

            meta.insert("type".to_string(), get_type(ty.as_str()).to_string());
        } else {
            let t = &field.ty;
            meta.insert("type".to_string(), quote!(#t).to_string());
        }

        let mut attrs = Vec::new();
        for a in &field.attrs {
            let simish = if let Some(i) = a.path().get_ident() {
                attributes.contains(&i.to_string().as_str())
            } else {
                false
            };
            if simish {
                let (name, value) = match &a.meta {
                    Meta::NameValue(mnv) => {
                        let name = mnv.path.get_ident().unwrap().to_string();
                        let value = match &mnv.value {
                            Expr::Lit(l) => match &l.lit {
                                Lit::Str(s) => s.value(),
                                Lit::Float(f) => f.base10_digits().to_string(),
                                _ => panic!("argument must be a string or float"),
                            },
                            _ => panic!("argument must be a string or float"),
                        };
                        (name, value)
                    }
                    _ => panic!("attribute must be in for #[name = \"value\"]"),
                };

                meta.insert(name, value);
            } else {
                attrs.push(a.clone());
            }
        }
        field.attrs = attrs;

        data.push(meta);
    }

    data
}

/// Generate a struct which can be used with SimConnect's data definitions.
/// ```rs
/// #[sim_connect::data_definition]
/// struct ControlSurfaces {
///     #[name = "ELEVATOR POSITION"]
///     #[unit = "Position"]
///     elevator: f64,
///     #[name = "AILERON POSITION"]
///     #[unit = "Position"]
///     ailerons: f64,
///     #[name = "RUDDER POSITION"]
///     #[unit = "Position"]
///     rudder: f64,
/// }
///
/// sim.add_data_definition::<ControlSurfaces>();
/// ```
#[proc_macro_attribute]
pub fn sim_connect_data_definition(_args: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);
    let name = input.ident.clone();

    let data = parse_struct_fields(
        &mut input,
        &["name", "unit", "epsilon"],
        Some(|ty| match ty {
            "bool" => "INT32",
            "i32" => "INT32",
            "i64" => "INT64",
            "f32" => "FLOAT32",
            "f64" => "FLOAT64",
            "DataXYZ" => "XYZ",
            _ => panic!("Unsupported type {}", ty),
        }),
    );

    let mut array = String::from("&[\n");
    for meta in data {
        let name = meta["name"].clone();
        let unit = meta
            .get("unit")
            .unwrap_or_else(|| panic!("{} needs a #[unit] decorator", name));

        let fallback = "0.0".to_string();
        let epsilon = meta.get("epsilon").unwrap_or(&fallback);

        let ty = meta["type"].clone();
        array += &format!(
            "  ({name:?}, {unit:?}, {epsilon}, ::msfs::sys::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_{ty}),\n"
        );
    }
    array += "]";
    let array = syn::parse_str::<Expr>(&array).unwrap();

    let output = quote! {
        #[repr(C)]
        #input

        impl ::msfs::sim_connect::DataDefinition for #name {
            const DEFINITIONS: &'static [(&'static str, &'static str, f32, ::msfs::sys::SIMCONNECT_DATATYPE)] = #array;
        }
    };

    TokenStream::from(output)
}

/// Generate a struct which can be used with SimConnect's client data definitions.
/// ```rs
/// #[sim_connect::client_data_definition]
/// struct SomeData {
///     foo: u8,
///     bar: f64,
///     #[epsilon = 0.5]
///     baz: i8,
/// }
/// ```
#[proc_macro_attribute]
pub fn sim_connect_client_data_definition(_args: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);
    let name = input.ident.clone();

    let data = parse_struct_fields(&mut input, &["epsilon"], None);

    let mut array = String::from("vec![\n");

    for meta in data {
        let fallback = "0.0".to_string();
        let epsilon = meta.get("epsilon").unwrap_or(&fallback);

        array += &format!(
            "    (unsafe {{
                     let uninit = std::mem::MaybeUninit::<{struct_name}>::uninit();
                     let base = uninit.as_ptr() as *const {struct_name};
                     let field = &((*base).{field_name}) as *const _;
                     (field as usize) - (base as usize)
                 }}, std::mem::size_of::<{type}>(), {epsilon}),
            ",
            struct_name=name, field_name=meta["field_name"], type=meta["type"], epsilon=epsilon,
        );
    }

    array += "]";

    let array = syn::parse_str::<Expr>(&array).unwrap();
    let output = quote! {
        #input

        impl ::msfs::sim_connect::ClientDataDefinition for #name {
            fn get_definitions() -> Vec<(usize, usize, f32)> { #array }
        }
    };

    TokenStream::from(output)
}

/// Generate a struct which can be used with SimConnect's facility definitions.
/// ```rs
/// #[facility_definition("AIRPORT")]
/// struct AirportData {
///     #[name = "N_JETWAYS"]
///     jetway_count: i32,
///     #[name = "N_RUNWAYS"]
///     runway_count: i32,
///     #[child_facility]
///     taxi_paths: Vec<TaxiPathData>,
/// }
///
/// #[facility_definition("TAXI_PATH")]
/// #[derive(Debug, Clone)]
/// struct TaxiPathData {
///     #[name = "TYPE"]
///     taxiway_type: i32,
/// }
///
/// sim.add_facility_definition::<AirportData>();
/// ```
#[proc_macro_attribute]
pub fn sim_connect_facility_definition(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);
    let mut raw_input = input.clone();
    let input_name = input.ident.clone();

    // parse facility type string from args
    if args.is_empty() {
        panic!(
            "facility_definition requires a facility type e.g. #[facility_definition(\"AIRPORT\")]"
        );
    }

    let facility_type = {
        let lit: Lit = syn::parse(args).expect("Expected string literal");
        match lit {
            Lit::Str(s) => s.value(),
            _ => panic!(
                "facility_definition argument must be a string  e.g. #[facility_definition(\"AIRPORT\")]"
            ),
        }
    };

    let mut facility_fields = Vec::new();
    let mut facility_definitions = Vec::new();
    let mut child_fields = Vec::new();
    let mut child_facilities = Vec::new();

    // parse field attributes
    for field in &mut input.fields {
        let field_has_attrs = field.attrs.iter().any(|attr| {
            if let Some(ident) = attr.path().get_ident() {
                ident == "name" || ident == "child_facility"
            } else {
                false
            }
        });

        if !field_has_attrs {
            panic!(
                "All fields must have either a #[name = \"...\"] or #[child_facility] attribute"
            );
        }

        let mut remaining_attrs = Vec::new();

        for attr in &field.attrs {
            if let Some(ident) = attr.path().get_ident() {
                match ident.to_string().as_str() {
                    "name" => {
                        if let Meta::NameValue(mnv) = &attr.meta {
                            if let Expr::Lit(expr_lit) = &mnv.value {
                                if let Lit::Str(lit_str) = &expr_lit.lit {
                                    facility_fields.push(field.ident.clone().unwrap());
                                    facility_definitions.push(lit_str.value());
                                } else {
                                    panic!("#[name] attribute must have a string value");
                                }
                            } else {
                                panic!("#[name] attribute must have a literal value");
                            }
                        } else {
                            panic!("#[name] attribute must be in format #[name = \"value\"]");
                        }
                    }
                    "child_facility" => {
                        // ensure this is a flag attribute
                        if !matches!(&attr.meta, Meta::Path(_)) {
                            panic!("#[child_facility] should only be a flag attribute");
                        }

                        // Extract inner type from Vec<T> using syn AST
                        let inner_type = match &field.ty {
                            Type::Path(type_path) => {
                                if let Some(segment) = type_path.path.segments.last() {
                                    if segment.ident == "Vec" {
                                        if let syn::PathArguments::AngleBracketed(args) =
                                            &segment.arguments
                                        {
                                            if let Some(syn::GenericArgument::Type(inner)) =
                                                args.args.first()
                                            {
                                                quote!(#inner).to_string()
                                            } else {
                                                panic!("Vec must have a type argument");
                                            }
                                        } else {
                                            panic!("Vec must have angle bracketed arguments");
                                        }
                                    } else {
                                        panic!(
                                            "Fields with #[child_facility] must be Vec<T>, found: {}",
                                            quote!(#field.ty)
                                        );
                                    }
                                } else {
                                    panic!("Invalid type path");
                                }
                            }
                            _ => panic!(
                                "Fields with #[child_facility] must be Vec<T>, found: {}",
                                quote!(#field.ty)
                            ),
                        };

                        child_fields.push(field.ident.clone().unwrap());
                        child_facilities.push(syn::parse_str::<Type>(&inner_type).unwrap());
                    }
                    _ => {
                        remaining_attrs.push(attr.clone());
                    }
                }
            } else {
                remaining_attrs.push(attr.clone());
            }
        }

        // Remove processed attributes
        field.attrs = remaining_attrs;
    }

    // create raw struct
    raw_input.ident = format_ident!("Raw{}", input_name);
    // remove child_facility fields and all attributes from the original struct
    let filtered_fields = raw_input
        .fields
        .iter()
        .filter(|field| {
            !field.attrs.iter().any(|attr| {
                if let Some(ident) = attr.path().get_ident() {
                    ident == "child_facility"
                } else {
                    false
                }
            })
        })
        .cloned()
        .collect();

    raw_input.fields = syn::Fields::Named(syn::FieldsNamed {
        brace_token: Default::default(),
        named: filtered_fields,
    });

    // clear attributes from remaining fields
    for field in &mut raw_input.fields {
        field.attrs.clear();
    }
    raw_input.attrs = Vec::new();
    let raw_input_name = raw_input.ident.clone();

    // check if child facility types implement FacilityDefinition
    let child_facility_checks = child_facilities.iter().map(|inner_type_ident| {
        quote! {
            const _: fn() = || {
                fn assert_facility_definition<T: ::msfs::sim_connect::FacilityDefinition>() {}
                assert_facility_definition::<#inner_type_ident>();
            };
        }
    });

    let output = quote! {
        #input

        #[repr(C, packed)]
        pub #raw_input

        impl From<#raw_input_name> for #input_name {
            fn from(raw: #raw_input_name) -> Self {
                Self {
                    #(#facility_fields: raw.#facility_fields,)*
                    #(#child_fields: Vec::new(),)*
                }
            }
        }

        const _: () = {
            impl ::msfs::sim_connect::FacilityDefinition for #input_name {
                type RawType = #raw_input_name;

                fn add_facility_definitions(
                    handle: ::msfs::sys::HANDLE,
                    define_id: ::msfs::sys::SIMCONNECT_DATA_DEFINITION_ID,
                ) -> ::msfs::sim_connect::Result<()> {
                    unsafe {
                        let open_cmd_cstr = std::ffi::CString::new(format!("OPEN {}", #facility_type)).unwrap();
                         ::msfs::sim_connect::map_err(::msfs::sys::SimConnect_AddToFacilityDefinition(
                            handle,
                            define_id,
                            open_cmd_cstr.as_ptr(),
                        ))?;

                        // add field definitions
                        #(
                            let field_name_cstr = std::ffi::CString::new(#facility_definitions).unwrap();
                             ::msfs::sim_connect::map_err(::msfs::sys::SimConnect_AddToFacilityDefinition(
                                handle,
                                define_id,
                                field_name_cstr.as_ptr(),
                            ))?;
                        )*

                        // add child facility definitions
                        #(#child_facilities::add_facility_definitions(handle, define_id)?;)*

                        let close_cmd_cstr = std::ffi::CString::new(format!("CLOSE {}", #facility_type)).unwrap();
                         ::msfs::sim_connect::map_err(::msfs::sys::SimConnect_AddToFacilityDefinition(
                            handle,
                            define_id,
                            close_cmd_cstr.as_ptr(),
                        ))?;

                        Ok(())
                    }
                }
            }
        };

        #(#child_facility_checks)*
    };

    TokenStream::from(output)
}
