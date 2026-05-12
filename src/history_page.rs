use anyhow::Result;
use serde_json::Value;

const HISTORY_PAGE_HTML: &str = r##"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>History</title>
    <style>
      :root {
        color-scheme: light dark;
        --bg: #ffffff;
        --bg-2: #f4f4f4;
        --panel: rgba(255,255,255,0.88);
        --panel-soft: rgba(248,248,248,0.82);
        --line: rgba(0,0,0,0.09);
        --line-strong: rgba(0,0,0,0.14);
        --text: #111111;
        --muted: #666666;
        --accent: #0066cc;
        --accent-soft: rgba(0,102,204,0.12);
        --danger: #cc3b2f;
        --danger-soft: rgba(204,59,47,0.12);
        --shadow: 0 4px 20px rgba(0,0,0,0.08);
      }
      @media (prefers-color-scheme: dark) {
        :root:not(.theme-warm):not(.theme-light) {
          --bg: #141414;
          --bg-2: #1e1e1e;
          --panel: rgba(30,30,30,0.92);
          --panel-soft: rgba(35,35,35,0.82);
          --line: rgba(255,255,255,0.09);
          --line-strong: rgba(255,255,255,0.14);
          --text: #f0f0f0;
          --muted: #8d8d8d;
          --accent: #4ea3ff;
          --accent-soft: rgba(78,163,255,0.15);
          --danger: #ff6b5e;
          --danger-soft: rgba(255,107,94,0.16);
          --shadow: 0 4px 20px rgba(0,0,0,0.35);
        }
      }
      :root.theme-dark {
        color-scheme: dark;
        --bg: #141414;
        --bg-2: #1e1e1e;
        --panel: rgba(30,30,30,0.92);
        --panel-soft: rgba(35,35,35,0.82);
        --line: rgba(255,255,255,0.09);
        --line-strong: rgba(255,255,255,0.14);
        --text: #f0f0f0;
        --muted: #8d8d8d;
        --accent: #4ea3ff;
        --accent-soft: rgba(78,163,255,0.15);
        --danger: #ff6b5e;
        --danger-soft: rgba(255,107,94,0.16);
        --shadow: 0 4px 20px rgba(0,0,0,0.35);
      }
      :root.theme-light {
        color-scheme: light;
      }
      :root.theme-warm {
        color-scheme: light;
        --bg: #f6efe6;
        --bg-2: #ede0cf;
        --panel: rgba(255,252,247,0.9);
        --panel-soft: rgba(255,251,246,0.74);
        --line: rgba(75,48,22,0.1);
        --line-strong: rgba(75,48,22,0.16);
        --text: #27180c;
        --muted: #7b5d45;
        --accent: #bc5f33;
        --accent-soft: rgba(188,95,51,0.14);
        --danger: #b94c2d;
        --danger-soft: rgba(185,76,45,0.14);
        --shadow: 0 4px 20px rgba(93,61,34,0.12);
      }

      * { box-sizing: border-box; }
      html, body {
        margin: 0;
        min-height: 100%;
        font-family: "SF Pro Display", "Segoe UI", sans-serif;
        color: var(--text);
        background: var(--bg);
        padding: 34px;
      }
      .page {
        display: grid;
        gap: 20px;
        max-width: 840px;
        margin: 0 auto;
      }
      .eyebrow {
        display: inline-flex;
        align-items: center;
        gap: 10px;
        width: fit-content;
        padding: 10px 14px;
        border-radius: 999px;
        border: 1px solid var(--line);
        background: var(--panel-soft);
        color: var(--muted);
        font-size: 11px;
        font-weight: 700;
        letter-spacing: 0.12em;
        text-transform: uppercase;
      }
      .eyebrow-dot {
        width: 10px;
        height: 10px;
        border-radius: 999px;
        background: var(--accent);
        box-shadow: 0 0 0 4px var(--accent-soft);
      }
      .hero {
        display: flex;
        justify-content: space-between;
        align-items: end;
        gap: 16px;
      }
      h1 {
        margin: 0 0 4px;
        font-size: 32px;
        letter-spacing: -0.03em;
      }
      .subtitle {
        margin: 0;
        font-size: 14px;
        color: var(--muted);
      }
      .ghost-btn {
        border: 1px solid var(--line-strong);
        background: var(--panel-soft);
        color: var(--danger);
        border-radius: 14px;
        padding: 10px 14px;
        font: inherit;
        font-size: 13px;
        font-weight: 600;
        cursor: pointer;
      }
      .ghost-btn:hover {
        background: var(--danger-soft);
      }
      .card {
        padding: 20px;
        border-radius: 24px;
        border: 1px solid var(--line-strong);
        background: var(--panel);
        box-shadow: var(--shadow);
      }
      .history-list {
        display: grid;
        gap: 10px;
      }
      .entry {
        display: grid;
        grid-template-columns: minmax(0, 1fr) auto;
        gap: 12px;
        align-items: center;
        padding: 14px 16px;
        border-radius: 16px;
        border: 1px solid var(--line);
        background: var(--panel-soft);
      }
      .entry-main {
        min-width: 0;
        display: grid;
        gap: 4px;
      }
      .entry-title {
        font-size: 15px;
        font-weight: 600;
        color: var(--text);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
      }
      .entry-url {
        font-size: 12px;
        color: var(--muted);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
      }
      .entry-actions {
        display: flex;
        gap: 8px;
      }
      .action-btn {
        border: 1px solid var(--line);
        border-radius: 12px;
        background: transparent;
        color: var(--text);
        padding: 8px 12px;
        font: inherit;
        font-size: 12px;
        font-weight: 600;
        cursor: pointer;
      }
      .action-btn:hover {
        background: var(--bg-2);
      }
      .action-btn.danger {
        color: var(--danger);
      }
      .action-btn.danger:hover {
        background: var(--danger-soft);
      }
      .empty {
        padding: 20px 4px;
        color: var(--muted);
        font-size: 14px;
      }
      @media (max-width: 720px) {
        html, body { padding: 18px; }
        .hero { display: grid; align-items: start; }
        .entry { grid-template-columns: 1fr; }
        .entry-actions { justify-content: start; }
      }
    </style>
  </head>
  <body>
    <main class="page">
      <div class="eyebrow">
        <span class="eyebrow-dot"></span>
        <span>Tartanos History</span>
      </div>

      <div class="hero">
        <div>
          <h1>History</h1>
          <p class="subtitle">Riwayat halaman yang pernah dibuka. Bisa dibuka ulang atau dihapus satu per satu.</p>
        </div>
        <button id="clear-history" class="ghost-btn" type="button">Clear all history</button>
      </div>

      <div class="card">
        <div id="history-list" class="history-list"></div>
      </div>
    </main>

    <script>
      const initialState = __INITIAL_STATE__;
      const state = { history: [], theme: "system" };
      const listEl = document.getElementById("history-list");
      const clearBtn = document.getElementById("clear-history");

      function post(payload) {
        window.ipc.postMessage(JSON.stringify(payload));
      }

      function esc(v) {
        return String(v)
          .replaceAll("&","&amp;").replaceAll("<","&lt;")
          .replaceAll(">","&gt;").replaceAll('"',"&quot;");
      }

      function applyTheme(theme) {
        document.documentElement.classList.remove("theme-warm", "theme-light", "theme-dark");
        if (theme === "warm") document.documentElement.classList.add("theme-warm");
        if (theme === "light") document.documentElement.classList.add("theme-light");
        if (theme === "dark") document.documentElement.classList.add("theme-dark");
      }

      function render(nextState) {
        Object.assign(state, nextState || {});
        applyTheme(state.theme || "system");
        clearBtn.disabled = !(state.history || []).length;
        if (!(state.history || []).length) {
          listEl.innerHTML = '<div class="empty">History kosong.</div>';
          return;
        }

        listEl.innerHTML = state.history.map(function(item) {
          return '<div class="entry">'
            + '<div class="entry-main">'
            + '<div class="entry-title">' + esc(item.title) + '</div>'
            + '<div class="entry-url">' + esc(item.url) + '</div>'
            + '</div>'
            + '<div class="entry-actions">'
            + '<button class="action-btn" type="button" data-open-url="' + esc(item.url) + '">Open</button>'
            + '<button class="action-btn danger" type="button" data-delete-id="' + item.id + '">Delete</button>'
            + '</div>'
            + '</div>';
        }).join("");
      }

      window.__syncHistoryPage = function(nextState) { render(nextState); };

      listEl.addEventListener("click", function(e) {
        const openBtn = e.target.closest("[data-open-url]");
        if (openBtn) {
          post({ kind: "open-history", value: openBtn.dataset.openUrl });
          return;
        }
        const deleteBtn = e.target.closest("[data-delete-id]");
        if (deleteBtn) {
          post({ kind: "delete-history", id: Number(deleteBtn.dataset.deleteId) });
        }
      });

      clearBtn.addEventListener("click", function() {
        post({ kind: "clear-history" });
      });

      render(initialState);
    </script>
  </body>
</html>
"##;

pub fn html(initial_state: &Value) -> Result<String> {
    Ok(HISTORY_PAGE_HTML.replace("__INITIAL_STATE__", &initial_state.to_string()))
}

pub fn sync_script(state: &Value) -> Result<String> {
    Ok(format!("window.__syncHistoryPage({state});"))
}
