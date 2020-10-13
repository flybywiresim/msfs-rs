extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{
    parse::{Parse, ParseStream, Result as SynResult},
    parse_macro_input, Expr, Ident, ItemFn, ItemStruct, Lit, Meta, Token, Type,
};

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

    let rusty_name = format_ident!("{}", input.sig.ident);
    let extern_name = format_ident!(
        "{}_gauge_callback",
        args.name.unwrap_or_else(|| input.sig.ident.to_string())
    );

    let output = quote! {
        #input

        #[no_mangle]
        pub extern "C" fn #extern_name(ctx: ::msfs::sys::FsContext, service_id: u32, _: *mut u8) -> bool {
            fn remap(
                gauge: ::msfs::msfs::Gauge,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + 'static>> {
                Box::pin(#rusty_name(gauge))
            }
            static mut executor: ::msfs::msfs::GaugeExecutor = ::msfs::msfs::GaugeExecutor {
                handle: remap,
                tx: None,
                future: None,
            };
            unsafe {
                executor.handle(ctx, service_id)
            }
        }
    };

    TokenStream::from(output)
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
/// sim.add_data_definition::<ControlSurfaces>(definition_id);
/// ```
#[proc_macro_attribute]
pub fn sim_connect_data_definition(_args: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);
    let name = input.ident.clone();

    let mut data = Vec::new();

    for field in &mut input.fields {
        let mut meta = HashMap::new();

        let ty = match &field.ty {
            Type::Path(p) => p.path.get_ident().unwrap().to_string(),
            _ => panic!("Unsupported type"),
        };

        meta.insert(
            "type".to_string(),
            match ty.as_str() {
                "i32" => "INT32",
                "i64" => "INT64",
                "f32" => "FLOAT32",
                "f64" => "FLOAT64",
                _ => panic!("Unsupported type {}", ty),
            }
            .to_string(),
        );

        let mut attrs = Vec::new();
        for a in &field.attrs {
            let simish = if let Some(i) = a.path.get_ident() {
                i == "name" || i == "unit"
            } else {
                false
            };
            if simish {
                let (name, value) = match a.parse_meta().unwrap() {
                    Meta::NameValue(mnv) => {
                        let name = mnv.path.get_ident().unwrap().to_string();
                        let value = match mnv.lit {
                            Lit::Str(s) => s.value(),
                            _ => panic!("argument must be a string"),
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

    let mut array = String::from("&[\n");
    for meta in data {
        let name = meta["name"].clone();
        let unit = meta["unit"].clone();
        let ty = meta["type"].clone();
        array += &format!(
            "  ({:?}, {:?}, ::msfs::sys::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_{}),\n",
            name, unit, ty
        );
    }
    array += "]";
    let array = syn::parse_str::<Expr>(&array).unwrap();

    let output = quote! {
        #[repr(C)]
        #input

        impl ::msfs::sim_connect::DataDefinition for #name {
            const DEFINITIONS: &'static [(&'static str, &'static str, ::msfs::sys::SIMCONNECT_DATATYPE)] = #array;
        }
    };

    TokenStream::from(output)
}
