.PHONY: check install wiki wiki-build e2e fleet tui dev dev-kill dev-clean rust-clean clean-rust clean

# Run build, test, clippy, and format check
check:
	cargo build
	cargo test
	cargo clippy -- -D warnings
	cargo fmt --check

# Install the jig CLI
install:
	cargo install --path crates/jig-cli

# Preview wiki locally at http://localhost:4000
wiki:
	cd wiki && bundle install && bundle exec jekyll serve --livereload

# Build wiki without serving
wiki-build:
	cd wiki && bundle exec jekyll build

# Hermetic e2e: real daemons in temp sandboxes, free ports, PASS/FAIL exit
e2e:
	cargo test -p jig-cli --test daemon --test cli

# Local dev fleet (bin/dev): isolated daemons, ports per
# worktree from confit.toml (needs confit). `./bin/dev ports` shows them.
dev:
	./bin/dev up

# A local two-machine fleet, set up from nothing: loopback aliases (asks for
# sudo only when they're missing), build, alice and bob up and meshed.
fleet:
	./bin/dev fleet

# A dev node's TUI: `make tui`, or `make tui NODE=bob`.
NODE ?= alice
tui:
	./bin/dev jig $(NODE)

# Stop this worktree's fleet: its daemons and their tmux servers
dev-kill:
	./bin/dev kill

# dev-kill, then delete the fleet's data (.dev/)
dev-clean:
	./bin/dev clean

# Remove cargo build artifacts
rust-clean:
	cargo clean

clean-rust: rust-clean

# Everything: dev-clean + rust-clean
clean: dev-clean rust-clean
