extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream, Result as SynResult},
    parse_macro_input, Ident, ItemFn, Token,
};

struct Args {
    name: Option<String>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> SynResult<Self> {
        match input.parse::<Ident>() {
            Ok(i) if i == "name" => {
                input.parse::<Token![=]>()?;
                Ok(Args {
                    name: Some(input.parse::<Ident>()?.to_string()),
                })
            }
            _ => Ok(Args { name: None })
        }
    }
}

/// Declare a gauge callback. It will be automatically exported with the name
/// `NAME_gauge_callback`, where `NAME` is the name of the decorated function.
/// ```rs
/// // Declare and export `FOO_gauge_callback`
/// #[msfs::gauge]
/// fn FOO(ctx: &msfs::FsContext, service_id: msfs::PanelServiceID) -> msfs::GaugeCallbackResult {
///   // ...
/// }
/// ```
///
/// The macro can also be given a parameter, `name`, to rename the exported function.
/// ```rs
/// // Declare and export `FOO_gauge_callback`
/// #[msfs::gauge(name=FOO)]
/// fn xyz(...) {}
#[proc_macro_attribute]
pub fn gauge(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);
    let input = parse_macro_input!(item as ItemFn);

    let rusty_name = format_ident!("{}", input.sig.ident);
    let extern_name = format_ident!(
        "{}_gauge_callback",
        args.name.unwrap_or_else(|| input.sig.ident.to_string())
    );

    let output = quote! {
        #input

        #[no_mangle]
        pub extern "C" fn #extern_name(ctx: ::msfs::sys::FsContext, service_id: i32, _: *mut u8) -> bool {
            let rusty: ::msfs::msfs::GaugeCallback = #rusty_name;
            let ctx = ::msfs::msfs::FsContext::from(ctx);
            let service_id = unsafe { std::mem::transmute(service_id) };
            match rusty(&ctx, service_id) {
                Ok(()) => true,
                Err(()) => false,
            }
        }
    };

    TokenStream::from(output)
}
