#!/usr/bin/env python3
"""Generate self-contained MCP tools reference HTML from a tools/list JSON-RPC response on stdin."""
import json
import sys

GROUPS = [
    ("Quotes & Charts",  ["get_quote", "get_quotes", "get_chart", "get_charts", "get_spark", "get_recommendations", "get_splits"]),
    ("Financials",       ["get_financials", "get_batch_financials"]),
    ("Indicators",       ["get_indicators", "get_batch_indicators"]),
    ("Dividends",        ["get_dividends", "get_batch_dividends"]),
    ("Options",          ["get_options"]),
    ("Market",           ["get_market_summary", "get_fear_and_greed", "get_trending", "get_indices", "get_market_hours", "get_sector", "get_industry"]),
    ("Discovery",        ["search", "lookup", "screener"]),
    ("News & Feeds",     ["get_news", "get_feeds"]),
    ("Analysis",         ["get_holders", "get_analysis"]),
    ("Risk",             ["get_risk"]),
    ("Crypto",           ["get_crypto_coins"]),
    ("FRED & Treasury",  ["get_fred_series", "get_treasury_yields"]),
    ("EDGAR",            ["get_edgar_facts", "get_edgar_submissions", "get_edgar_search"]),
    ("Transcripts",      ["get_transcripts"]),
]

CSS = """
* { box-sizing: border-box; margin: 0; padding: 0; }
body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  font-size: 14px;
  line-height: 1.6;
  background: #0d1117;
  color: #e6edf3;
  display: flex;
  height: 100vh;
  overflow: hidden;
}
a { color: #58a6ff; text-decoration: none; }
a:hover { text-decoration: underline; }
code {
  font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
  font-size: 0.85em;
}
#sidebar {
  width: 240px;
  min-width: 240px;
  background: #161b22;
  border-right: 1px solid #30363d;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.sidebar-header {
  padding: 18px 16px 12px;
  border-bottom: 1px solid #30363d;
  flex-shrink: 0;
}
.sidebar-logo {
  font-size: 1em;
  font-weight: 700;
  color: #e6edf3;
  letter-spacing: -0.01em;
}
.sidebar-meta {
  font-size: 0.75em;
  color: #8b949e;
  margin-top: 2px;
}
nav {
  overflow-y: auto;
  flex: 1;
  padding: 8px 0 20px;
}
.nav-group { margin-bottom: 2px; }
.nav-group-title {
  padding: 8px 16px 3px;
  font-size: 0.68em;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: #8b949e;
}
.nav-item {
  display: block;
  padding: 3px 16px 3px 22px;
  color: #c9d1d9;
  font-size: 0.8em;
  font-family: "SFMono-Regular", Consolas, monospace;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  border-left: 2px solid transparent;
  margin-left: 0;
}
.nav-item:hover { color: #58a6ff; text-decoration: none; background: rgba(88,166,255,0.06); }
.nav-item.active { color: #58a6ff; border-left-color: #58a6ff; background: rgba(88,166,255,0.08); }
#main {
  flex: 1;
  overflow-y: auto;
  padding: 28px 40px 60px;
}
.section-label {
  font-size: 0.68em;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: #8b949e;
  border-bottom: 1px solid #21262d;
  padding-bottom: 6px;
  margin: 28px 0 12px;
}
.section-label:first-child { margin-top: 0; }
.tool-card {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 18px 22px;
  margin-bottom: 10px;
  scroll-margin-top: 16px;
  transition: border-color 0.15s;
}
.tool-card:target { border-color: #388bfd; }
.tool-name {
  font-family: "SFMono-Regular", Consolas, monospace;
  font-size: 1em;
  font-weight: 600;
  color: #79c0ff;
  margin-bottom: 6px;
}
.tool-desc {
  color: #c9d1d9;
  font-size: 0.88em;
  margin-bottom: 14px;
  line-height: 1.55;
}
table.params {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.83em;
}
table.params th {
  text-align: left;
  padding: 5px 12px;
  font-size: 0.72em;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: #8b949e;
  border-bottom: 1px solid #30363d;
}
table.params td {
  padding: 7px 12px;
  border-bottom: 1px solid #21262d;
  vertical-align: top;
}
table.params tr:last-child td { border-bottom: none; }
.p-name { color: #e6edf3; font-weight: 500; }
.p-type { color: #d2a8ff; }
.p-desc { color: #c9d1d9; line-height: 1.5; }
.badge {
  display: inline-block;
  font-size: 0.65em;
  font-weight: 600;
  padding: 1px 5px;
  border-radius: 3px;
  margin-left: 5px;
  vertical-align: middle;
  font-family: -apple-system, sans-serif;
  letter-spacing: 0.02em;
}
.req { background: rgba(248,81,73,0.15); color: #f85149; }
.opt { background: rgba(63,185,80,0.1); color: #56d364; }
.no-params { color: #8b949e; font-size: 0.83em; font-style: italic; }
::-webkit-scrollbar { width: 5px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: #30363d; border-radius: 3px; }
::-webkit-scrollbar-thumb:hover { background: #484f58; }
"""

JS = """
const navItems = document.querySelectorAll('.nav-item');
const cards = document.querySelectorAll('.tool-card');
const main = document.getElementById('main');

const io = new IntersectionObserver(entries => {
  entries.forEach(e => {
    if (e.isIntersecting) {
      navItems.forEach(a => a.classList.remove('active'));
      const link = document.querySelector(`.nav-item[href="#${e.target.id}"]`);
      if (link) {
        link.classList.add('active');
        link.scrollIntoView({ block: 'nearest' });
      }
    }
  });
}, { root: main, threshold: 0.25 });

cards.forEach(c => io.observe(c));
"""


def resolve_type(info):
    if "anyOf" in info:
        types = [x.get("type", "") for x in info["anyOf"] if x.get("type") and x.get("type") != "null"]
        return " | ".join(types) if types else "any"
    return info.get("type", "any")


def render_params(schema):
    props = schema.get("properties", {})
    required = set(schema.get("required", []))
    if not props:
        return '<p class="no-params">No parameters required.</p>'
    rows = []
    for name, info in props.items():
        typ = resolve_type(info)
        desc = info.get("description", "")
        badge = f'<span class="badge req">required</span>' if name in required else f'<span class="badge opt">optional</span>'
        rows.append(
            f"<tr>"
            f'<td><code class="p-name">{name}</code>{badge}</td>'
            f'<td><code class="p-type">{typ}</code></td>'
            f'<td class="p-desc">{desc}</td>'
            f"</tr>"
        )
    header = "<tr><th>Parameter</th><th>Type</th><th>Description</th></tr>"
    return f'<table class="params"><thead>{header}</thead><tbody>{"".join(rows)}</tbody></table>'


def main():
    data = json.load(sys.stdin)
    tools = data.get("result", {}).get("tools", [])
    by_name = {t["name"]: t for t in tools}

    # Sidebar nav
    nav_parts = []
    for group, names in GROUPS:
        items = [n for n in names if n in by_name]
        if not items:
            continue
        links = "".join(f'<a href="#{n}" class="nav-item">{n}</a>' for n in items)
        nav_parts.append(f'<div class="nav-group"><div class="nav-group-title">{group}</div>{links}</div>')
    nav_html = "".join(nav_parts)

    # Main content
    content_parts = []
    for group, names in GROUPS:
        group_tools = [by_name[n] for n in names if n in by_name]
        if not group_tools:
            continue
        content_parts.append(f'<div class="section-label">{group}</div>')
        for tool in group_tools:
            params_html = render_params(tool.get("inputSchema", {}))
            content_parts.append(
                f'<div id="{tool["name"]}" class="tool-card">'
                f'<div class="tool-name">{tool["name"]}</div>'
                f'<div class="tool-desc">{tool.get("description", "")}</div>'
                f"{params_html}"
                f"</div>"
            )
    content_html = "".join(content_parts)

    tool_count = len(tools)

    out = [
        "<!DOCTYPE html>",
        '<html lang="en">',
        "<head>",
        '  <meta charset="utf-8">',
        '  <meta name="viewport" content="width=device-width, initial-scale=1">',
        "  <title>Finance Query \u2014 MCP Tools Reference</title>",
        f"  <style>{CSS}</style>",
        "</head>",
        "<body>",
        f'<div id="sidebar">',
        f'  <div class="sidebar-header">',
        f'    <div class="sidebar-logo">Finance Query MCP</div>',
        f'    <div class="sidebar-meta">{tool_count} tools</div>',
        f"  </div>",
        f"  <nav>{nav_html}</nav>",
        f"</div>",
        f'<div id="main">{content_html}</div>',
        f"<script>{JS}</script>",
        "</body>",
        "</html>",
    ]
    print("\n".join(out))


if __name__ == "__main__":
    main()
