.PHONY: check install wiki wiki-build build-web build-web-release e2e dev dev-kill dev-clean rust-clean clean-rust clean

# Run build, test, clippy, and format check
check:
	cargo build
	cargo test
	cargo clippy -- -D warnings
	cargo fmt --check

# Install the jig CLI, with the web view built into it
install: build-web-release
	cargo install --path crates/jig-cli

# Build the web view's Yew app (crates/jig-cli/web) into web/dist, which jig
# embeds. Needs trunk + the wasm32 target. It can't run from build.rs: trunk
# runs a nested cargo that deadlocks on the outer build's lock.
build-web:
	@command -v trunk >/dev/null 2>&1 || { echo "trunk not installed: cargo install trunk && rustup target add wasm32-unknown-unknown" >&2; exit 1; }
	cd crates/jig-cli/web && trunk build $(ARGS)

build-web-release:
	$(MAKE) build-web ARGS=--release

# Preview wiki locally at http://localhost:4000
wiki:
	cd wiki && bundle install && bundle exec jekyll serve --livereload

# Build wiki without serving
wiki-build:
	cd wiki && bundle exec jekyll build

# Hermetic e2e: real daemons in temp sandboxes, free ports, PASS/FAIL exit
e2e:
	cargo test -p jig-cli --test daemon --test cli

# Local dev fleet (bin/dev): isolated daemons with web views, ports per
# worktree from confit.toml (needs confit). `./bin/dev ports` shows them.
dev:
	./bin/dev up

# Stop this worktree's fleet: its daemons and their tmux servers
dev-kill:
	./bin/dev kill

# dev-kill, then delete the fleet's data (.dev/)
dev-clean:
	./bin/dev clean

# Remove cargo build artifacts, the web app's included
rust-clean:
	cargo clean
	rm -rf crates/jig-cli/web/dist crates/jig-cli/web/target

clean-rust: rust-clean

# Everything: dev-clean + rust-clean
clean: dev-clean rust-clean
