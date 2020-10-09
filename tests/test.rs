extern crate msfs;

fn callback(ctx: &msfs::FsContext) -> bool {
    false
}
msfs::gauge!(callback);
