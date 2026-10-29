# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.15.0 (2026-10-29)

### Refactor

 - <csr-id-e1aa97c222f2099de18c1e6648c8b89c323459af/> adopt zim-api's ApiRequest pattern for the web view API
   jig-api was a bag of serde types with path constants. It now follows
   zim-api: an ApiRequest trait whose implementors own their method, path
   and body; a generic Client (`call`, `with_token` for the command token);
   and a module per subject with a file per request:
   
     session/get.rs  status/get.rs
     workers/{kill,nudge,prune,retry_setup,model,setup_log}.rs
   
   Each request file exports its route's PATH, and the daemon mounts its
   handler on it, so a route can't move on one side only. The server's
   daemon/web/api/ mirrors the layout, a handler file per request, with
   an ApiJson extractor so bad bodies are refused with an ErrorBody.
   
   - the single POST /command taking a verb is now one route per verb
   - the setup log is a typed JSON request; the failed-setups panel loads
     it in place instead of opening a plain-text tab
   - the Yew app and tests/daemon/web.rs both drive the daemon through
     jig_api::Client (reqwest, no TLS: the web view is loopback only)
 - <csr-id-095f9a0b4d8ba3f8adef06c6e4efe77cc0f16542/> serve the web view with axum and a typed JSON API
   Replaces the hand-rolled HTTP server (httparse, HTML built in format
   strings, app.js swapping fragments) with axum:
   
   - jig-api: the /api/v0 paths and wire types, serde only so a wasm app
     can compile them too
   - axum runs on a current-thread tokio runtime owned by the jig-web
     thread; the rest of the daemon stays std threads, and handlers make
     their blocking daemon calls on tokio's blocking pool
   - Host check and security headers are middleware; the token and origin
     check is an Authorized extractor
   - the app is embedded from web/dist with rust-embed, and the CSP allows
     its inline loader by hash rather than 'unsafe-inline'
   - the router is tested in-process with tower's oneshot, and
     tests/daemon/web.rs drives a real daemon over the JSON API

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge pull request #178 from krondor-corp/feature/kro-250-fleet-local-web-view-served-by-the-daemon-on-127001 ([`9cf4bf6`](https://github.com/krondor-corp/jig/commit/9cf4bf678ef4cbd235587105afa79c6cb3664a47))
    - Adopt zim-api's ApiRequest pattern for the web view API ([`e1aa97c`](https://github.com/krondor-corp/jig/commit/e1aa97c222f2099de18c1e6648c8b89c323459af))
    - Serve the web view with axum and a typed JSON API ([`095f9a0`](https://github.com/krondor-corp/jig/commit/095f9a0b4d8ba3f8adef06c6e4efe77cc0f16542))
</details>

