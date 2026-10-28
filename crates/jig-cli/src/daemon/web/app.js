// jig web view: command buttons and a periodic refresh of #view.
//
// Commands POST an IPC request to /command with the session token from the
// page's <meta name="jig-token">; GETs only ever read.
(function () {
  "use strict";

  var REFRESH_MS = 5000;
  var token = document.querySelector('meta[name="jig-token"]').content;
  var view = document.getElementById("view");
  var result = document.getElementById("result");
  var pending = 0;

  function show(ok, message) {
    result.hidden = false;
    result.className = ok ? "ok" : "err";
    result.textContent = message;
  }

  // Build the request a verb sends, asking for its argument where it has
  // one. Returns null when the user backs out.
  function requestFor(verb, worker) {
    switch (verb) {
      case "nudge": {
        var text = window.prompt("Nudge " + worker + " with:");
        return text && text.trim() ? { request: "nudge", worker: worker, text: text } : null;
      }
      case "model": {
        var model = window.prompt("Switch " + worker + " to model:");
        return model && model.trim() ? { request: "model", worker: worker, model: model.trim() } : null;
      }
      case "kill":
        return window.confirm("Kill " + worker + "? Its window closes; the worktree stays.")
          ? { request: "kill", worker: worker } : null;
      case "prune":
        return window.confirm("Prune " + worker + "? Its worktree is removed.")
          ? { request: "prune", worker: worker } : null;
      case "retry-setup":
        return { request: "retry_setup", worker: worker };
      default:
        return null;
    }
  }

  function run(button) {
    var body = requestFor(button.dataset.verb, button.dataset.worker);
    if (!body) return;
    button.disabled = true;
    pending++;
    show(true, button.dataset.verb + " " + button.dataset.worker + " …");
    fetch("/command", {
      method: "POST",
      headers: { "Content-Type": "application/json", "X-Jig-Token": token },
      body: JSON.stringify(body),
    })
      .then(function (r) { return r.json(); })
      .then(function (r) { show(r.ok, r.ok ? r.message : r.error); })
      .catch(function (e) { show(false, String(e)); })
      .then(function () {
        pending--;
        button.disabled = false;
        refresh();
      });
  }

  function refresh() {
    if (pending > 0) return;
    fetch("/view")
      .then(function (r) { if (r.ok) return r.text(); throw new Error(r.status); })
      .then(function (html) { if (pending === 0) view.innerHTML = html; })
      .catch(function () { /* daemon restarting; try again next round */ });
  }

  document.addEventListener("click", function (e) {
    var button = e.target.closest("button[data-verb]");
    if (button) run(button);
  });
  setInterval(refresh, REFRESH_MS);
})();
