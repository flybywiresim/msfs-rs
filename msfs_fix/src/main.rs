use std::io::Write;

fn main() {
    let mut m = walrus::Module::from_file(
        std::env::args()
            .nth(1)
            .expect("must provide the input wasm file as the first argument"),
    )
    .expect("invalid wasm");

    let mut fix = |name| {
        m.exports.add(name, m.funcs.by_name(name).unwrap());
    };

    fix("malloc");
    fix("free");

    std::io::stdout().write_all(&m.emit_wasm()).unwrap();
}
