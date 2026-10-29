//! Make sure the web app's `web/dist/` exists, so `daemon::web::spa` can
//! embed it.
//!
//! This doesn't build the app. Trunk runs a nested `cargo` that would
//! deadlock on this build's lock, so `make build-web` builds it instead, and
//! CI runs that before building jig. Without it jig still builds, and the
//! web view says it isn't built.

use std::path::PathBuf;

fn main() {
    let dist = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"))
        .join("web/dist");
    std::fs::create_dir_all(&dist).ok();
    println!("cargo:rerun-if-changed=web/dist");
    if !dist.join("index.html").exists() {
        println!("cargo:warning=web/dist is empty: the web view won't be in this build — run `make build-web`");
    }
}
