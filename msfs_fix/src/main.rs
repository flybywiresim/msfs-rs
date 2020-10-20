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

    std::fs::write(
        std::env::args()
            .nth(2)
            .expect("must provide output dir as second argument"),
        &m.emit_wasm(),
    )
    .unwrap();
}
