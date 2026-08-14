/* soothfast default theme: theme toggle, copy buttons, TOC scroll-spy,
   client-side search over search_index.json. No dependencies. */
(function () {
  "use strict";
  var base = document.body.dataset.base || "";

  // ----- theme toggle (persisted; explicit choice overrides OS) -----
  var toggle = document.getElementById("themetoggle");
  if (toggle) {
    toggle.addEventListener("click", function () {
      var mediaDark = matchMedia("(prefers-color-scheme: dark)").matches;
      var current = document.documentElement.dataset.theme || (mediaDark ? "dark" : "light");
      var next = current === "dark" ? "light" : "dark";
      document.documentElement.dataset.theme = next;
      try { localStorage.setItem("soothfast-theme", next); } catch (e) {}
    });
  }

  // ----- mobile nav drawer -----
  var navToggle = document.getElementById("navtoggle");
  var sidenav = document.getElementById("sidenav");
  var backdrop = document.getElementById("navbackdrop");
  if (navToggle && sidenav && backdrop) {
    var closeNav = function () {
      sidenav.classList.remove("open");
      backdrop.hidden = true;
      navToggle.setAttribute("aria-expanded", "false");
    };
    navToggle.addEventListener("click", function () {
      var open = sidenav.classList.toggle("open");
      backdrop.hidden = !open;
      navToggle.setAttribute("aria-expanded", String(open));
    });
    backdrop.addEventListener("click", closeNav);
    sidenav.addEventListener("click", function (e) {
      if (e.target.closest("a")) closeNav();
    });
    document.addEventListener("keydown", function (e) {
      if (e.key === "Escape") closeNav();
    });
  }

  // ----- mobile search overlay -----
  var searchToggle = document.getElementById("searchtoggle");
  var searchBox = document.getElementById("search");
  if (searchToggle && searchBox) {
    searchToggle.addEventListener("click", function () {
      var open = searchBox.classList.toggle("open");
      searchToggle.setAttribute("aria-expanded", String(open));
      if (open) {
        var input = document.getElementById("searchbox");
        if (input) input.focus();
      }
    });
  }

  // ----- copy buttons -----
  document.addEventListener("click", function (e) {
    var btn = e.target.closest("[data-copy]");
    if (!btn) return;
    var fig = btn.closest("figure");
    var code = fig && fig.querySelector("pre code, pre");
    if (!code) return;
    navigator.clipboard.writeText(code.textContent).then(function () {
      var old = btn.textContent;
      btn.textContent = "Copied";
      setTimeout(function () { btn.textContent = old; }, 1200);
    });
  });

  // ----- TOC scroll-spy -----
  var tocLinks = Array.prototype.slice.call(document.querySelectorAll("#toc a"));
  if (tocLinks.length && "IntersectionObserver" in window) {
    var spy = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (!entry.isIntersecting) return;
        tocLinks.forEach(function (a) {
          a.classList.toggle("current", a.hash === "#" + entry.target.id);
        });
      });
    }, { rootMargin: "-10% 0px -75% 0px" });
    document.querySelectorAll("h2[id], h3[id]").forEach(function (h) { spy.observe(h); });
  }

  // ----- search -----
  var box = document.getElementById("searchbox");
  var results = document.getElementById("searchresults");
  if (!box || !results) return;
  var index = null;
  var sel = -1;

  function load() {
    if (index) return Promise.resolve(index);
    return fetch(base + "search_index.json")
      .then(function (r) { return r.json(); })
      .then(function (data) {
        // Lowercase once at load, not per record per keystroke.
        data.forEach(function (rec) {
          rec.hay = (rec.page + " " + rec.section + " " + rec.text).toLowerCase();
          rec.head = (rec.section || rec.page).toLowerCase();
        });
        index = data;
        return index;
      })
      .catch(function () { index = []; return index; });
  }

  function score(rec, terms) {
    var s = 0;
    for (var i = 0; i < terms.length; i++) {
      var t = terms[i];
      if (!t) continue;
      if (rec.head.indexOf(t) !== -1) s += 4;
      else if (rec.hay.indexOf(t) !== -1) s += 1;
      else return 0; // every term must match somewhere
    }
    return s;
  }

  // Index fields hold page text with entities decoded, so a docs page that
  // shows an HTML snippet would otherwise re-enter the DOM as live markup.
  function esc(s) {
    return String(s == null ? "" : s)
      .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
  }

  function excerpt(text, term) {
    var at = text.toLowerCase().indexOf(term);
    if (at < 0) return text.slice(0, 90);
    var start = Math.max(0, at - 30);
    return (start ? "…" : "") + text.slice(start, start + 110) + "…";
  }

  function render(hits, terms) {
    sel = -1;
    if (!hits.length) {
      results.innerHTML = '<div class="search-empty">No matches</div>';
      results.hidden = false;
      return;
    }
    results.innerHTML = hits.slice(0, 8).map(function (rec) {
      var href = base + rec.route + (rec.anchor ? "#" + rec.anchor : "");
      var title = rec.section
        ? esc(rec.section) + ' <small>· ' + esc(rec.page) + "</small>"
        : esc(rec.page);
      return '<a class="search-hit" href="' + esc(href || "./") + '">' +
        '<div class="search-hit-title">' + title + "</div>" +
        '<div class="search-hit-text">' + esc(excerpt(rec.text, terms[0])) + "</div></a>";
    }).join("");
    results.hidden = false;
  }

  box.addEventListener("input", function () {
    var q = box.value.trim().toLowerCase();
    if (q.length < 2) { results.hidden = true; return; }
    load().then(function (idx) {
      var terms = q.split(/\s+/);
      var hits = idx
        .map(function (rec) { return { rec: rec, s: score(rec, terms) }; })
        .filter(function (h) { return h.s > 0; })
        .sort(function (a, b) { return b.s - a.s; })
        .map(function (h) { return h.rec; });
      render(hits, terms);
    });
  });

  box.addEventListener("keydown", function (e) {
    var hits = results.querySelectorAll(".search-hit");
    if (e.key === "Escape") { results.hidden = true; box.blur(); return; }
    if (!hits.length || results.hidden) return;
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      sel = e.key === "ArrowDown" ? Math.min(sel + 1, hits.length - 1) : Math.max(sel - 1, 0);
      hits.forEach(function (h, i) { h.classList.toggle("sel", i === sel); });
    } else if (e.key === "Enter" && sel >= 0) {
      e.preventDefault();
      hits[sel].click();
    }
  });

  document.addEventListener("click", function (e) {
    if (!e.target.closest("#search")) results.hidden = true;
  });
  document.addEventListener("keydown", function (e) {
    if ((e.metaKey || e.ctrlKey) && e.key === "k") {
      e.preventDefault();
      box.focus();
      box.select();
    }
  });

  // ----- "Try it out" (generated OpenAPI reference, `spec html`) -----
  // Everything is driven by data-* attributes on the enclosing `.spec-row`
  // (method, path template) and on each value field (data-name) — this
  // file has no per-page knowledge, just a generic validator + sender. The
  // parameter table's value column and the request-body editor are always
  // visible (built by the generator, not toggled), so the reference
  // (available values, defaults, descriptions) sits right next to the
  // field that fills it in. Execute starts disabled on any row with a
  // required text field, and stays disabled until every such field in that
  // row actually has a value — a `<select>` is never "empty" (the browser
  // always has some option chosen), so only text inputs are watched.
  document.addEventListener("click", function (e) {
    var send = e.target.closest(".spec-try-send");
    if (send) sendTryIt(send.closest(".spec-row"));
  });

  document.addEventListener("input", function (e) {
    var root = e.target.closest(".spec-row");
    if (root) updateTryItValidity(root);
  });

  function updateTryItValidity(root) {
    var send = root.querySelector(".spec-try-send");
    if (!send) return;
    var unmet = Array.prototype.some.call(
      root.querySelectorAll(".spec-try-col input[required]"),
      function (field) { return field.value.trim() === ""; }
    );
    send.disabled = unmet;
  }

  document.querySelectorAll(".spec-row .spec-try-send").forEach(function (send) {
    updateTryItValidity(send.closest(".spec-row"));
  });

  function sendTryIt(root) {
    if (!root) return;
    var method = root.dataset.method || "GET";
    var path = root.dataset.path || "/";
    var serverSel = root.querySelector(".spec-try-server");
    var base = serverSel ? serverSel.value : "";
    var query = [];
    root.querySelectorAll('.spec-try-col[data-param="path"]').forEach(function (cell) {
      var field = cell.querySelector("input, select");
      if (!field) return;
      path = path.split("{" + field.dataset.name + "}").join(encodeURIComponent(field.value));
    });
    root.querySelectorAll('.spec-try-col[data-param="query"]').forEach(function (cell) {
      var field = cell.querySelector("input, select");
      if (!field || field.value === "") return;
      query.push(encodeURIComponent(field.dataset.name) + "=" + encodeURIComponent(field.value));
    });
    var bodyEl = root.querySelector(".spec-try-bodytext");
    var body = bodyEl && bodyEl.value.trim() !== "" ? bodyEl.value : null;
    var url = base + path + (query.length ? "?" + query.join("&") : "");
    var opts = { method: method, headers: {} };
    if (body !== null) {
      opts.headers["Content-Type"] = "application/json";
      opts.body = body;
    }

    var result = root.querySelector(".spec-try-result");
    var status = root.querySelector(".spec-try-status");
    var out = root.querySelector(".spec-try-out");
    if (!result || !status || !out) return;
    result.hidden = false;
    status.textContent = url;
    status.className = "spec-try-status";
    out.textContent = "Sending…";

    fetch(url, opts)
      .then(function (res) {
        status.textContent = res.status + " " + res.statusText + " · " + url;
        status.className = "spec-try-status " + (res.ok ? "spec-try-ok" : "spec-try-err");
        return res.text();
      })
      .then(function (text) {
        try {
          out.textContent = JSON.stringify(JSON.parse(text), null, 2);
        } catch (e) {
          out.textContent = text;
        }
      })
      .catch(function (err) {
        status.className = "spec-try-status spec-try-err";
        status.textContent = "Request failed · " + url;
        out.textContent = String(err) + "\n\nA network/CORS error usually means the selected " +
          "server isn't reachable from here, or doesn't allow cross-origin requests from this page's origin.";
      });
  }
})();
