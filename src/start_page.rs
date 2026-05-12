use anyhow::Result;
use serde_json::Value;

const START_PAGE_HTML: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>New Tab</title>
    <style>
      :root {
        color-scheme: light dark;
        --bg:     #ffffff;
        --bg-2:   #f4f4f4;
        --bg-3:   #ececec;
        --text:   #111111;
        --muted:  #666666;
        --border: rgba(0, 0, 0, 0.09);
        --shadow: 0 1px 4px rgba(0,0,0,0.08);
        --accent: #0066cc;
      }

      /* system auto-dark */
      @media (prefers-color-scheme: dark) {
        :root:not(.theme-warm):not(.theme-light) {
          --bg:     #141414;
          --bg-2:   #1e1e1e;
          --bg-3:   #282828;
          --text:   #f0f0f0;
          --muted:  #888888;
          --border: rgba(255, 255, 255, 0.09);
          --shadow: 0 1px 4px rgba(0,0,0,0.3);
          --accent: #4ea3ff;
        }
      }

      /* force dark */
      :root.theme-dark {
        color-scheme: dark;
        --bg:     #141414;
        --bg-2:   #1e1e1e;
        --bg-3:   #282828;
        --text:   #f0f0f0;
        --muted:  #888888;
        --border: rgba(255, 255, 255, 0.09);
        --shadow: 0 1px 4px rgba(0,0,0,0.3);
        --accent: #4ea3ff;
      }

      /* force light */
      :root.theme-light {
        color-scheme: light;
        --bg:     #ffffff;
        --bg-2:   #f4f4f4;
        --bg-3:   #ececec;
        --text:   #111111;
        --muted:  #666666;
        --border: rgba(0, 0, 0, 0.09);
        --shadow: 0 1px 4px rgba(0,0,0,0.08);
        --accent: #0066cc;
      }

      /* warm theme */
      :root.theme-warm {
        color-scheme: light;
        --bg:     #f6efe6;
        --bg-2:   #ede0cf;
        --bg-3:   #e3d0bc;
        --text:   #27180c;
        --muted:  #7b5d45;
        --border: rgba(75, 48, 22, 0.12);
        --shadow: 0 1px 4px rgba(93,61,34,0.1);
        --accent: #bc5f33;
      }

      * { box-sizing: border-box; margin: 0; padding: 0; }

      html, body {
        min-height: 100%;
        font-family: "SF Pro Text", "Segoe UI", system-ui, sans-serif;
        font-size: 14px;
        color: var(--text);
        background: var(--bg);
      }

      body { padding: 40px 24px; }

      .page {
        max-width: 720px;
        margin: 0 auto;
        display: grid;
        gap: 28px;
      }

      .hero {
        display: grid;
        gap: 8px;
        padding-top: 36px;
      }

      .wordmark {
        font-size: 13px;
        font-weight: 600;
        letter-spacing: 0.06em;
        text-transform: uppercase;
        color: var(--muted);
      }

      .hero-title {
        font-size: 30px;
        line-height: 1.1;
        font-weight: 650;
        color: var(--text);
      }

      .hero-copy {
        font-size: 14px;
        line-height: 1.5;
        color: var(--muted);
        max-width: 520px;
      }

      /* links area */
      .links-area {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 16px;
      }

      .section {
        display: grid;
        gap: 8px;
      }

      .section-title {
        font-size: 11px;
        font-weight: 600;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: var(--muted);
        padding-bottom: 2px;
      }

      .item-list { display: grid; gap: 4px; }

      .item {
        display: grid;
        gap: 1px;
        padding: 9px 12px;
        border-radius: 8px;
        border: 1px solid var(--border);
        background: var(--bg-2);
        cursor: pointer;
        text-align: left;
        transition: background 100ms;
      }
      .item:hover { background: var(--bg-3); }

      .item-title {
        font-size: 13px;
        font-weight: 500;
        color: var(--text);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
      }
      .item-host {
        font-size: 11px;
        color: var(--muted);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
      }

      .empty {
        font-size: 12px;
        color: var(--muted);
        padding: 8px 0;
      }

      @media (max-width: 560px) {
        .links-area { grid-template-columns: 1fr; }
        .hero {
          padding-top: 20px;
          gap: 6px;
        }
        .hero-title { font-size: 24px; }
      }
    </style>
  </head>
  <body>
    <main class="page">
      <div class="hero">
        <div class="wordmark">Tartanos</div>
        <div class="hero-title">New tab, ready to type.</div>
        <div class="hero-copy">Gunakan address bar di atas untuk cari atau buka alamat. Bookmark dan halaman terakhir tetap tersedia di bawah.</div>
      </div>

      <div class="links-area">
        <section class="section">
          <div class="section-title">Bookmarks</div>
          <div id="bookmarks" class="item-list"></div>
        </section>

        <section class="section">
          <div class="section-title">Recent</div>
          <div id="history" class="item-list"></div>
        </section>
      </div>
    </main>

    <script>
      const initialState = __INITIAL_STATE__;
      const state = { bookmarks: [], history: [], theme: "system" };

      const bookmarksEl = document.getElementById("bookmarks");
      const historyEl   = document.getElementById("history");

      function esc(v) {
        return String(v)
          .replaceAll("&","&amp;").replaceAll("<","&lt;")
          .replaceAll(">","&gt;").replaceAll('"',"&quot;");
      }
      function host(url) {
        try { return new URL(url).host; } catch(_) { return url; }
      }
      function post(p) { window.ipc.postMessage(JSON.stringify(p)); }

      function renderList(el, items, emptyText, kind) {
        if (!items.length) {
          el.innerHTML = '<div class="empty">' + esc(emptyText) + '</div>';
          return;
        }
        el.innerHTML = items.map(function(item) {
          return '<button class="item" type="button" data-kind="' + kind + '" data-url="' + esc(item.url) + '">'
            + '<span class="item-title">' + esc(item.title) + '</span>'
            + '<span class="item-host">'  + esc(host(item.url)) + '</span>'
            + '</button>';
        }).join("");
      }

      function applyTheme(theme) {
        document.documentElement.classList.remove("theme-warm", "theme-light", "theme-dark");
        if (theme === "warm")  document.documentElement.classList.add("theme-warm");
        if (theme === "light") document.documentElement.classList.add("theme-light");
        if (theme === "dark")  document.documentElement.classList.add("theme-dark");
      }

      function render(next) {
        Object.assign(state, next || {});
        applyTheme(state.theme || "system");
        renderList(bookmarksEl, state.bookmarks || [], "No bookmarks yet.", "open-bookmark");
        renderList(historyEl,   state.history   || [], "Nothing visited yet.", "open-history");
      }

      window.__syncStartPage = function(next) { render(next); };

      [bookmarksEl, historyEl].forEach(function(el) {
        el.addEventListener("click", function(e) {
          var t = e.target.closest("[data-kind]");
          if (t) post({ kind: t.dataset.kind, value: t.dataset.url });
        });
      });

      render(initialState);
    </script>
  </body>
</html>
"#;

pub fn html(initial_state: &Value) -> Result<String> {
    Ok(START_PAGE_HTML.replace("__INITIAL_STATE__", &initial_state.to_string()))
}

pub fn sync_script(state: &Value) -> Result<String> {
    Ok(format!("window.__syncStartPage({state});"))
}
