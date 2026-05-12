use anyhow::Result;
use serde_json::Value;

const TOOLBAR_HTML: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <style>
      :root {
        color-scheme: light dark;
        --bg:         #d7d9de;
        --panel:      rgba(255,255,255,0.78);
        --line:       rgba(0,0,0,0.1);
        --text:       #111111;
        --muted:      #5b616d;
        --control-text: #20242b;
        --placeholder: rgba(32, 36, 43, 0.46);
        --accent:     #0066cc;
        --accent-soft: rgba(0,102,204,0.12);
        --tab-active: rgba(255,255,255,0.98);
        --tab-idle:   rgba(86, 92, 104, 0.14);
        --toolbar-top: rgba(255,255,255,0.24);
        --toolbar-bottom: rgba(255,255,255,0.72);
        --control-bg: rgba(255,255,255,0.36);
        --control-hover: rgba(255,255,255,0.62);
        --address-bg: rgba(255,255,255,0.72);
        --address-focus-bg: rgba(255,255,255,0.96);
        --shadow-sm:  0 1px 4px rgba(0,0,0,0.1);
      }

      /* system auto-dark */
      @media (prefers-color-scheme: dark) {
        :root:not(.theme-warm):not(.theme-light) {
          --bg:         #2a2a2a;
          --panel:      rgba(50,50,50,0.9);
          --line:       rgba(255,255,255,0.1);
          --text:       #f0f0f0;
          --muted:      #aab0bb;
          --control-text: #f3f6fb;
          --placeholder: rgba(243, 246, 251, 0.34);
          --accent:     #4ea3ff;
          --accent-soft: rgba(78,163,255,0.15);
          --tab-active: rgba(73,76,84,0.98);
          --tab-idle:   rgba(255,255,255,0.08);
          --toolbar-top: rgba(255,255,255,0.08);
          --toolbar-bottom: rgba(0,0,0,0.18);
          --control-bg: rgba(255,255,255,0.06);
          --control-hover: rgba(255,255,255,0.12);
          --address-bg: rgba(255,255,255,0.08);
          --address-focus-bg: rgba(47,51,58,0.98);
          --shadow-sm:  0 1px 4px rgba(0,0,0,0.4);
        }
      }

      /* force dark */
      :root.theme-dark {
        color-scheme: dark;
        --bg:         #2a2a2a;
        --panel:      rgba(50,50,50,0.9);
        --line:       rgba(255,255,255,0.1);
        --text:       #f0f0f0;
        --muted:      #aab0bb;
        --control-text: #f3f6fb;
        --placeholder: rgba(243, 246, 251, 0.34);
        --accent:     #4ea3ff;
        --accent-soft: rgba(78,163,255,0.15);
        --tab-active: rgba(73,76,84,0.98);
        --tab-idle:   rgba(255,255,255,0.08);
        --toolbar-top: rgba(255,255,255,0.08);
        --toolbar-bottom: rgba(0,0,0,0.18);
        --control-bg: rgba(255,255,255,0.06);
        --control-hover: rgba(255,255,255,0.12);
        --address-bg: rgba(255,255,255,0.08);
        --address-focus-bg: rgba(47,51,58,0.98);
        --shadow-sm:  0 1px 4px rgba(0,0,0,0.4);
      }

      /* force light */
      :root.theme-light {
        color-scheme: light;
        --bg:         #d7d9de;
        --panel:      rgba(255,255,255,0.78);
        --line:       rgba(0,0,0,0.1);
        --text:       #111111;
        --muted:      #5b616d;
        --control-text: #20242b;
        --placeholder: rgba(32, 36, 43, 0.46);
        --accent:     #0066cc;
        --accent-soft: rgba(0,102,204,0.12);
        --tab-active: rgba(255,255,255,0.98);
        --tab-idle:   rgba(86, 92, 104, 0.14);
        --toolbar-top: rgba(255,255,255,0.24);
        --toolbar-bottom: rgba(255,255,255,0.72);
        --control-bg: rgba(255,255,255,0.36);
        --control-hover: rgba(255,255,255,0.62);
        --address-bg: rgba(255,255,255,0.72);
        --address-focus-bg: rgba(255,255,255,0.96);
        --shadow-sm:  0 1px 4px rgba(0,0,0,0.1);
      }

      /* warm theme */
      :root.theme-warm {
        color-scheme: light;
        --bg:         #f5ede0;
        --panel:      rgba(255,251,245,0.92);
        --line:       rgba(60,40,19,0.13);
        --text:       #23170d;
        --muted:      #8a6a52;
        --control-text: #2f1f13;
        --placeholder: rgba(47, 31, 19, 0.42);
        --accent:     #be5b2d;
        --accent-soft: rgba(190,91,45,0.14);
        --tab-active: rgba(255,255,255,0.9);
        --tab-idle:   rgba(116, 74, 34, 0.12);
        --toolbar-top: rgba(255,255,255,0.3);
        --toolbar-bottom: rgba(255,250,242,0.85);
        --control-bg: rgba(255,255,255,0.34);
        --control-hover: rgba(255,255,255,0.58);
        --address-bg: rgba(255,251,245,0.82);
        --address-focus-bg: rgba(255,255,255,0.96);
        --shadow-sm:  0 2px 8px rgba(93,61,34,0.1);
      }

      * { box-sizing: border-box; margin: 0; padding: 0; }

      html, body {
        height: 100%;
        font-family: "SF Pro Text", "Segoe UI", system-ui, sans-serif;
        color: var(--text);
        background:
          linear-gradient(180deg, var(--toolbar-top) 0%, var(--toolbar-bottom) 100%),
          var(--bg);
        overflow: hidden;
        user-select: none;
      }

      /* loading bar */
      #loading-bar {
        position: fixed;
        top: 0; left: 0;
        height: 2px;
        width: 0%;
        background: var(--accent);
        transition: width 200ms ease, opacity 300ms ease;
        opacity: 0;
        z-index: 100;
      }
      body[data-loading="true"] #loading-bar {
        opacity: 1;
        animation: load-pulse 1.4s ease-in-out infinite;
      }
      @keyframes load-pulse {
        0%   { width: 0%; }
        50%  { width: 70%; }
        100% { width: 90%; }
      }

      .chrome {
        display: grid;
        grid-template-rows: 34px 36px;
        gap: 8px;
        height: 100%;
        padding: 8px 10px 10px;
      }

      .top-row,
      .bottom-row {
        min-width: 0;
        display: grid;
        align-items: center;
        gap: 8px;
      }

      .top-row {
        grid-template-columns: minmax(0, 1fr) auto auto;
        padding-bottom: 2px;
        border-bottom: 1px solid color-mix(in srgb, var(--line) 85%, transparent);
      }

      .bottom-row {
        grid-template-columns: auto minmax(0, 1fr) auto;
      }

      /* tabs */
      .tabs-area {
        display: flex;
        align-items: center;
        min-width: 0;
        overflow: hidden;
        align-self: stretch;
      }

      .tabs {
        display: flex;
        gap: 6px;
        overflow-x: auto;
        scrollbar-width: none;
        min-width: 0;
        align-items: center;
        width: 100%;
        height: 100%;
        padding-top: 1px;
        padding-left: 2px;
        padding-right: 6px;
      }
      .tabs::-webkit-scrollbar { display: none; }

      .tab {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        justify-content: flex-start;
        height: 34px;
        width: clamp(92px, calc((100% - (var(--tab-count, 1) - 1) * 6px) / var(--tab-count, 1)), 224px);
        flex: 0 1 clamp(92px, calc((100% - (var(--tab-count, 1) - 1) * 6px) / var(--tab-count, 1)), 224px);
        min-width: 0;
        max-width: 224px;
        padding: 0 16px 0 18px;
        border-radius: 14px 14px 0 0;
        border: 1px solid color-mix(in srgb, var(--line) 72%, transparent);
        border-bottom-color: transparent;
        background: var(--tab-idle);
        color: color-mix(in srgb, var(--control-text) 72%, var(--muted));
        font: inherit;
        font-size: 12px;
        cursor: pointer;
        white-space: nowrap;
        overflow: hidden;
        transition: background 120ms ease, color 120ms ease, border-color 120ms ease, transform 120ms ease;
      }
      .tab.pinned {
        width: 76px;
        flex-basis: 76px;
        flex-grow: 0;
        padding-left: 12px;
        padding-right: 12px;
      }
      .tab.dragging {
        opacity: 0.55;
      }
      .tab:hover {
        background: color-mix(in srgb, var(--control-hover) 70%, transparent);
        color: var(--control-text);
        border-color: color-mix(in srgb, var(--line) 90%, transparent);
      }
      .tab.active {
        background: var(--tab-active);
        border-color: var(--line);
        border-bottom-color: transparent;
        color: var(--control-text);
        box-shadow: 0 1px 0 rgba(255,255,255,0.6), var(--shadow-sm);
      }
      .tab-label {
        overflow: hidden;
        text-overflow: ellipsis;
        flex: 1;
        text-align: left;
        font-weight: 500;
      }
      .tab-favicon {
        width: 16px;
        height: 16px;
        border-radius: 4px;
        flex: 0 0 16px;
        object-fit: cover;
        background: color-mix(in srgb, var(--panel) 70%, transparent);
      }
      .tab-favicon-fallback {
        display: inline-grid;
        place-items: center;
        width: 16px;
        height: 16px;
        border-radius: 4px;
        flex: 0 0 16px;
        font-size: 11px;
        background: color-mix(in srgb, var(--panel) 70%, transparent);
        color: var(--muted);
      }
      .tab-pin {
        display: inline-grid;
        place-items: center;
        width: 12px;
        flex: 0 0 12px;
        color: var(--accent);
        font-size: 10px;
      }
      .tab-audio {
        display: inline-grid;
        place-items: center;
        width: 16px;
        height: 16px;
        border: 0;
        border-radius: 999px;
        padding: 0;
        background: transparent;
        color: var(--muted);
        opacity: 0.72;
        flex: 0 0 16px;
      }
      .tab-audio:hover {
        color: var(--control-text);
        opacity: 1;
      }
      .tab-audio svg {
        width: 13px;
        height: 13px;
        display: block;
        stroke: currentColor;
        fill: none;
        stroke-width: 1.7;
        stroke-linecap: round;
        stroke-linejoin: round;
      }
      .tab.muted .tab-audio {
        color: var(--accent);
        opacity: 0.92;
      }

      /* buttons */
      button {
        border: 1px solid var(--line);
        background: var(--control-bg);
        color: var(--control-text);
        border-radius: 9px;
        height: 32px;
        padding: 0 10px;
        font: inherit;
        font-size: 13px;
        cursor: pointer;
        transition: background 100ms, border-color 100ms, transform 80ms;
        white-space: nowrap;
      }
      button:hover {
        background: var(--control-hover);
        border-color: color-mix(in srgb, var(--accent) 20%, var(--line));
        transform: translateY(-1px);
      }
      button:active { transform: translateY(0); }

      .top-row-btn {
        min-width: 32px;
        width: 32px;
        padding: 0;
        font-size: 18px;
        font-weight: 400;
        flex-shrink: 0;
        border-radius: 999px;
        background: var(--control-bg);
      }
      .close-tab-btn {
        font-size: 16px;
      }

      /* nav group */
      .nav-group {
        display: flex;
        gap: 4px;
        flex-shrink: 0;
      }
      .nav-btn {
        min-width: 32px;
        width: 32px;
        padding: 0;
        font-size: 15px;
      }

      /* address bar */
      .address-wrap {
        min-width: 0;
        position: relative;
      }
      .address-shell {
        display: flex;
        align-items: center;
        height: 32px;
        padding: 0 12px;
        border: 1px solid var(--line);
        border-radius: 999px;
        background: var(--address-bg);
        gap: 8px;
        box-shadow: inset 0 1px 0 rgba(255,255,255,0.06);
        transition: background 120ms ease, border-color 120ms ease, box-shadow 120ms ease;
      }
      .address-shell:focus-within {
        background: var(--address-focus-bg);
        border-color: var(--accent);
        box-shadow: 0 0 0 3px var(--accent-soft), inset 0 1px 0 rgba(255,255,255,0.06);
      }
      input {
        flex: 1;
        min-width: 0;
        border: 0;
        outline: none;
        background: transparent;
        color: var(--control-text);
        font: inherit;
        font-size: 13px;
        font-weight: 500;
        user-select: text;
        -webkit-user-select: text;
      }
      input::placeholder { color: var(--placeholder); }

      /* suggestions */
      .suggestions {
        display: none;
        position: fixed;
        left: 0; right: 0;
        top: 88px;
        background: color-mix(in srgb, var(--address-focus-bg) 88%, var(--bg));
        border: 1px solid var(--line);
        border-top: none;
        border-radius: 0 0 12px 12px;
        box-shadow: 0 8px 24px rgba(0,0,0,0.15);
        overflow: hidden;
        z-index: 999;
      }
      .sug-item {
        display: grid;
        grid-template-columns: 20px 1fr;
        align-items: center;
        gap: 10px;
        padding: 9px 14px;
        cursor: pointer;
        border-top: 1px solid var(--line);
        transition: background 80ms;
      }
      .sug-item:first-child { border-top: none; }
      .sug-item:hover, .sug-item.selected { background: var(--accent-soft); }
      .sug-icon { font-size: 12px; color: var(--muted); text-align: center; }
      .sug-text { min-width: 0; }
      .sug-main { font-size: 13px; color: var(--text); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
      .sug-sub  { font-size: 11px; color: var(--muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
      .sug-badge {
        display: inline-flex;
        align-items: center;
        height: 18px;
        margin-right: 8px;
        padding: 0 6px;
        border-radius: 999px;
        border: 1px solid var(--line);
        background: color-mix(in srgb, var(--panel) 75%, transparent);
        color: var(--muted);
        font-size: 10px;
        font-weight: 700;
        letter-spacing: 0.02em;
        vertical-align: middle;
      }
      .sug-match {
        color: var(--text);
        font-weight: 700;
      }

      /* action group */
      .action-group {
        display: flex;
        gap: 4px;
        flex-shrink: 0;
      }
      .action-btn {
        min-width: 32px;
        width: 32px;
        padding: 0;
        font-size: 15px;
      }
      body[data-bookmarked="true"] #bookmark {
        color: var(--accent);
        border-color: rgba(190, 91, 45, 0.28);
        background: var(--accent-soft);
      }

      @media (max-width: 720px) {
        .chrome {
          grid-template-rows: 34px 36px;
          gap: 6px;
          padding-top: 6px;
        }
        .bottom-row {
          grid-template-columns: auto minmax(0, 1fr) auto;
          gap: 6px;
        }
      }
    </style>
  </head>
  <body data-loading="false" data-bookmarked="false">
    <div id="loading-bar"></div>
    <div id="suggestions" class="suggestions"></div>

    <div class="chrome">
      <div class="top-row">
        <div class="tabs-area">
          <div id="tabs" class="tabs"></div>
        </div>

        <button id="new-tab" class="top-row-btn" type="button" aria-label="New tab" title="New tab (⌘T)">+</button>
        <button id="close-tab" class="top-row-btn close-tab-btn" type="button" aria-label="Close tab" title="Close tab (⌘W)">×</button>
      </div>

      <div class="bottom-row">
        <div class="nav-group">
          <button id="back"    class="nav-btn" type="button" aria-label="Back"    title="Back (⌘[)">&#8249;</button>
          <button id="forward" class="nav-btn" type="button" aria-label="Forward" title="Forward (⌘])">&#8250;</button>
          <button id="reload"  class="nav-btn" type="button" aria-label="Reload"  title="Reload (⌘R)">↺</button>
        </div>

        <div class="address-wrap">
          <form id="nav-form">
            <div class="address-shell">
              <input
                id="address"
                type="text"
                autocomplete="off"
                autocapitalize="off"
                spellcheck="false"
                placeholder="Search or enter address"
              />
            </div>
          </form>
        </div>

        <div class="action-group">
          <button id="bookmark" class="action-btn" type="button" aria-label="Bookmark" title="Bookmark">★</button>
          <button id="settings" class="action-btn" type="button" aria-label="Settings" title="Settings (⌘,)" style="font-size:14px;">⚙</button>
        </div>
      </div>
    </div>

    <script>
      const initialState = __INITIAL_STATE__;
      const state = {
        url: "",
        title: "Tartanos",
        status: "Ready",
        loading: false,
        bookmarked: false,
        tabs: [],
        bookmarks: [],
        history: [],
        theme: "system",
        search_engine: "google",
      };

      const address     = document.getElementById("address");
      const navForm     = document.getElementById("nav-form");
      const backBtn     = document.getElementById("back");
      const forwardBtn  = document.getElementById("forward");
      const reloadBtn   = document.getElementById("reload");
      const bookmarkBtn = document.getElementById("bookmark");
      const newTabBtn   = document.getElementById("new-tab");
      const closeTabBtn = document.getElementById("close-tab");
      const settingsBtn  = document.getElementById("settings");
      const tabsEl       = document.getElementById("tabs");
      const suggestEl    = document.getElementById("suggestions");

      const BASE_H     = 88;
      const ITEM_H     = 40;
      const MAX_SUG    = 7;
      let sugTimer     = null;
      let suggestions  = [];
      let selIndex     = -1;
      let lastQuery    = "";
      let dragTabId    = null;

      const SUGGEST_URLS = {
        google:     "https://suggestqueries.google.com/complete/search?client=firefox&q=",
        bing:       "https://api.bing.com/osjson.aspx?query=",
        duckduckgo: "https://duckduckgo.com/ac/?q=",
      };

      const SEARCH_KEYWORDS = {
        g: { label: "Google", base: "https://www.google.com/search?q=" },
        google: { label: "Google", base: "https://www.google.com/search?q=" },
        b: { label: "Bing", base: "https://www.bing.com/search?q=" },
        bing: { label: "Bing", base: "https://www.bing.com/search?q=" },
        d: { label: "DuckDuckGo", base: "https://duckduckgo.com/?q=" },
        ddg: { label: "DuckDuckGo", base: "https://duckduckgo.com/?q=" },
        duck: { label: "DuckDuckGo", base: "https://duckduckgo.com/?q=" },
        duckduckgo: { label: "DuckDuckGo", base: "https://duckduckgo.com/?q=" },
        yt: { label: "YouTube", base: "https://www.youtube.com/results?search_query=" },
        gh: { label: "GitHub", base: "https://github.com/search?q=" },
        w: { label: "Wikipedia", base: "https://en.wikipedia.org/w/index.php?search=" },
      };

      function showSug(items) {
        suggestions = items;
        selIndex = -1;
        if (!items.length) { hideSug(); return; }
        const h = BASE_H + Math.min(items.length, MAX_SUG) * ITEM_H + 4;
        post({ kind: "set-height", value: String(h) });
        suggestEl.style.display = "none"; // hide until WebView finishes resizing
        suggestEl.innerHTML = items.map(function(it, i) {
          const q = lastQuery.trim();
          return '<div class="sug-item" data-i="' + i + '" data-val="' + esc(it.val) + '">'
            + '<span class="sug-icon">' + esc(it.icon || (it.local ? "\u{1F550}" : "\u{1F50D}")) + '</span>'
            + '<span class="sug-text">'
            + '<div class="sug-main">' + highlightMatch(it.text, q) + '</div>'
            + renderSuggestionSub(it, q)
            + '</span></div>';
        }).join("");
        setTimeout(function() { suggestEl.style.display = "block"; }, 60);
      }

      function hideSug() {
        suggestions = [];
        selIndex = -1;
        suggestEl.style.display = "none";
        suggestEl.innerHTML = "";
        post({ kind: "set-height", value: String(BASE_H) });
      }

      function setSelection(i) {
        selIndex = i;
        suggestEl.querySelectorAll(".sug-item").forEach(function(el, idx) {
          el.classList.toggle("selected", idx === selIndex);
        });
        if (selIndex >= 0 && suggestions[selIndex]) {
          address.value = suggestions[selIndex].fill ?? suggestions[selIndex].val ?? lastQuery;
        }
      }

      function parseMaybeUrl(input) {
        const trimmed = input.trim();
        if (!trimmed) return null;
        try { return new URL(trimmed).toString(); } catch (_) {}

        const compact = trimmed.replace(/\s+/g, "");
        const looksLikeHost = compact &&
          (
            compact.includes(".") ||
            compact.includes("/") ||
            compact.startsWith("localhost") ||
            /^\d{1,3}(\.\d{1,3}){3}/.test(compact)
          );
        if (!looksLikeHost) return null;
        try { return new URL("https://" + compact).toString(); } catch (_) {}
        return null;
      }

      function extractHost(url) {
        try { return new URL(url).host; } catch (_) { return ""; }
      }

      function highlightMatch(text, query) {
        const raw = String(text || "");
        const q = String(query || "").trim();
        if (!q) return esc(raw);
        const lower = raw.toLowerCase();
        const ql = q.toLowerCase();
        const index = lower.indexOf(ql);
        if (index < 0) return esc(raw);
        const before = raw.slice(0, index);
        const match = raw.slice(index, index + q.length);
        const after = raw.slice(index + q.length);
        return esc(before) + '<span class="sug-match">' + esc(match) + '</span>' + esc(after);
      }

      function renderSuggestionSub(item, query) {
        if (!item.sub && !item.badge) return "";
        const badge = item.badge
          ? '<span class="sug-badge">' + esc(item.badge) + '</span>'
          : "";
        const sub = item.sub
          ? highlightMatch(item.sub, query)
          : "";
        return '<div class="sug-sub">' + badge + sub + '</div>';
      }

      function scoreLocalSuggestion(item, ql, index, sourceKind) {
        const title = (item.title || "").toLowerCase();
        const url = (item.url || "").toLowerCase();
        let host = url;
        try { host = new URL(item.url).host.toLowerCase(); } catch (_) {}
        let score = sourceKind === "bookmark" ? 35 : 20;
        if (url === ql || title === ql || host === ql) score += 120;
        if (host.startsWith(ql)) score += 80;
        if (url.startsWith(ql)) score += 70;
        if (title.startsWith(ql)) score += 55;
        if (host.includes(ql)) score += 35;
        if (title.includes(ql)) score += 24;
        if (url.includes(ql)) score += 18;
        score += Math.max(0, 12 - index);
        return score;
      }

      function localSug(q) {
        const ql = q.trim().toLowerCase();
        const sources = []
          .concat((state.bookmarks || []).map(function(item, index) { return { item: item, index: index, kind: "bookmark" }; }))
          .concat((state.history || []).map(function(item, index) { return { item: item, index: index, kind: "history" }; }));

        return sources
          .map(function(source) {
            const title = (source.item.title || "").toLowerCase();
            const url = (source.item.url || "").toLowerCase();
            if (!title.includes(ql) && !url.includes(ql)) return null;
            return {
              text: source.item.title || source.item.url,
              sub: source.item.url,
              badge: extractHost(source.item.url) || (source.kind === "bookmark" ? "bookmark" : "history"),
              val: source.item.url,
              fill: source.item.url,
              icon: source.kind === "bookmark" ? "★" : "🕘",
              local: true,
              score: scoreLocalSuggestion(source.item, ql, source.index, source.kind),
              command: { kind: source.kind === "bookmark" ? "open-bookmark" : "open-history", value: source.item.url },
            };
          })
          .filter(Boolean)
          .sort(function(a, b) { return b.score - a.score; })
          .filter(function(item, index, arr) {
            return arr.findIndex(function(other) { return other.val === item.val; }) === index;
          })
          .slice(0, 4);
      }

      function actionSug(q) {
        const ql = q.trim().toLowerCase();
        if (!ql) return [];
        const actions = [
          { match: ["settings", "prefs", "preferences"], text: "Open Settings", sub: "Browser preferences", fill: "settings", icon: "⚙", command: { kind: "open-settings" } },
          { match: ["history", "recent"], text: "Open History", sub: "Visited pages", fill: "history", icon: "🕘", command: { kind: "open-history-page" } },
          { match: ["new", "new tab", "tab"], text: "Open New Tab", sub: "Create a fresh tab", fill: "new tab", icon: "+", command: { kind: "new-tab" } },
          { match: ["pin", "pin tab"], text: "Pin or Unpin Tab", sub: "Toggle pinned state for active tab", fill: "pin tab", icon: "📌", command: { kind: "toggle-pin-tab" } },
          { match: ["duplicate", "clone tab"], text: "Duplicate Tab", sub: "Copy current tab", fill: "duplicate tab", icon: "⧉", command: { kind: "duplicate-tab" } },
          { match: ["reopen", "restore tab"], text: "Reopen Closed Tab", sub: "Restore last closed tab", fill: "reopen closed tab", icon: "↶", command: { kind: "reopen-closed-tab" } },
          { match: ["bookmark", "save"], text: "Toggle Bookmark", sub: "Save or remove current page", fill: "bookmark", icon: "★", command: { kind: "toggle-bookmark" } },
          { match: ["clear history", "wipe history"], text: "Clear All History", sub: "Delete every visited page entry", fill: "clear history", icon: "⌫", command: { kind: "clear-history" } },
          { match: ["reload", "refresh"], text: "Reload Page", sub: "Reload current tab", fill: "reload", icon: "↺", command: { kind: "reload" } },
        ];
        return actions
          .filter(function(action) {
            return action.match.some(function(term) { return term.includes(ql) || ql.includes(term); });
          })
          .map(function(action, index) {
            return {
              text: action.text,
              sub: action.sub,
              val: action.fill,
              fill: action.fill,
              icon: action.icon,
              local: true,
              score: 100 - index,
              command: action.command,
            };
          })
          .slice(0, 2);
      }

      function getKeywordSuggestion(q) {
        const trimmed = q.trim();
        const parts = trimmed.split(/\s+/);
        const keyword = parts[0] ? parts[0].toLowerCase() : "";
        const engine = SEARCH_KEYWORDS[keyword];
        if (!engine) return null;
        const remainder = trimmed.slice(parts[0].length).trim();
        if (!remainder) return null;
        return {
          text: remainder,
          sub: "Search with " + engine.label,
          badge: parts[0].toUpperCase(),
          val: trimmed,
          fill: trimmed,
          icon: "⌘",
          local: true,
          score: 140,
          command: { kind: "navigate", value: trimmed },
        };
      }

      function getDirectNavigationSuggestion(q) {
        const target = parseMaybeUrl(q);
        if (!target) return null;
        return {
          text: target,
          sub: "Open address",
          badge: extractHost(target),
          val: target,
          fill: q.trim().replace(/\s+/g, ""),
          icon: "↗",
          local: true,
          score: 160,
          command: { kind: "navigate", value: target },
        };
      }

      function getSearchSuggestion(q) {
        const trimmed = q.trim();
        if (!trimmed) return null;
        const engine = SEARCH_KEYWORDS[state.search_engine] || SEARCH_KEYWORDS.google;
        return {
          text: trimmed,
          sub: "Search with " + engine.label,
          badge: engine.label,
          val: trimmed,
          fill: trimmed,
          icon: "⌕",
          local: false,
          score: 60,
          command: { kind: "navigate", value: trimmed },
        };
      }

      function parseRemoteResults(data) {
        if (Array.isArray(data) && Array.isArray(data[1])) {
          return data[1];
        }
        if (Array.isArray(data) && data.length && typeof data[0] === "object") {
          return data.map(function(entry) { return entry.phrase || entry.query || ""; }).filter(Boolean);
        }
        return [];
      }

      async function remoteSug(q) {
        try {
          const base = SUGGEST_URLS[state.search_engine] || SUGGEST_URLS.google;
          const ctrl = new AbortController();
          const tid  = setTimeout(function() { ctrl.abort(); }, 2000);
          const res  = await fetch(base + encodeURIComponent(q), { signal: ctrl.signal });
          clearTimeout(tid);
          const data = await res.json();
          const list = parseRemoteResults(data);
          return list.slice(0, 5).map(function(s) {
            return {
              text: s,
              sub: "Suggested search",
              badge: "suggested",
              val: s,
              fill: s,
              icon: "⌕",
              local: false,
              score: 40,
              command: { kind: "navigate", value: s },
            };
          });
        } catch(_) { return []; }
      }

      function dedupeSuggestions(items) {
        const seen = new Set();
        return items.filter(function(item) {
          const key = JSON.stringify(item.command || { kind: "navigate", value: item.val }) + "::" + (item.text || "");
          if (seen.has(key)) return false;
          seen.add(key);
          return true;
        });
      }

      function executeSuggestion(item) {
        if (!item) return;
        if (item.fill != null) {
          address.value = item.fill;
        }
        hideSug();
        if (item.command) {
          post(item.command);
        } else if (item.val) {
          post({ kind: "navigate", value: item.val });
        }
      }

      async function doSuggest(q) {
        if (!q.trim()) { hideSug(); return; }
        lastQuery = q;
        const initial = dedupeSuggestions(
          []
            .concat(getDirectNavigationSuggestion(q) ? [getDirectNavigationSuggestion(q)] : [])
            .concat(getKeywordSuggestion(q) ? [getKeywordSuggestion(q)] : [])
            .concat(actionSug(q))
            .concat(localSug(q))
            .concat(getSearchSuggestion(q) ? [getSearchSuggestion(q)] : [])
        )
          .sort(function(a, b) { return (b.score || 0) - (a.score || 0); })
          .slice(0, MAX_SUG);
        if (initial.length) showSug(initial);
        const remote = await remoteSug(q);
        if (q !== lastQuery) return; // stale
        const combined = dedupeSuggestions(initial.concat(remote))
          .sort(function(a, b) { return (b.score || 0) - (a.score || 0); })
          .slice(0, MAX_SUG);
        showSug(combined);
      }

      function esc(v) {
        return String(v)
          .replaceAll("&","&amp;").replaceAll("<","&lt;")
          .replaceAll(">","&gt;").replaceAll('"',"&quot;");
      }

      function post(payload) {
        window.ipc.postMessage(JSON.stringify(payload));
      }

      function selectedAddressText() {
        const start = address.selectionStart ?? 0;
        const end = address.selectionEnd ?? 0;
        return address.value.slice(start, end);
      }

      function replaceAddressSelection(text) {
        const start = address.selectionStart ?? 0;
        const end = address.selectionEnd ?? 0;
        const next = address.value.slice(0, start) + text + address.value.slice(end);
        address.value = next;
        const caret = start + text.length;
        address.setSelectionRange(caret, caret);
        address.dispatchEvent(new Event("input", { bubbles: true }));
      }

      function tabAudioIcon(muted) {
        if (muted) {
          return '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M2.5 6.25h2.3l2.7-2.5v8.5l-2.7-2.5H2.5z"></path><path d="M10.4 5.2l3.1 5.6"></path><path d="M13.5 5.2l-3.1 5.6"></path></svg>';
        }
        return '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M2.5 6.25h2.3l2.7-2.5v8.5l-2.7-2.5H2.5z"></path><path d="M10.2 6.1c.85.55 1.35 1.18 1.35 1.9s-.5 1.35-1.35 1.9"></path><path d="M11.7 4.6c1.45.95 2.3 2.1 2.3 3.4s-.85 2.45-2.3 3.4"></path></svg>';
      }

      function renderTabs(items) {
        tabsEl.style.setProperty("--tab-count", String(Math.max(items.length, 1)));
        if (!items.length) { tabsEl.innerHTML = ""; return; }
        tabsEl.innerHTML = items.map(function(tab) {
          const favicon = tab.favicon
            ? '<img class="tab-favicon" src="' + esc(tab.favicon) + '" alt="" onerror="this.style.display=\'none\'; this.nextElementSibling.style.display=\'inline-grid\';" />'
            : '<img class="tab-favicon" src="" alt="" style="display:none;" />';
          const fallback = '<span class="tab-favicon-fallback"' + (tab.favicon ? ' style="display:none;"' : '') + '>' + esc(tab.icon || '•') + '</span>';
          const audio = (tab.audible || tab.muted)
            ? '<button class="tab-audio" type="button" data-mute-tab="' + tab.id + '" title="' + esc(tab.muted ? "Unmute tab" : "Mute tab") + '" aria-label="' + esc(tab.muted ? "Unmute tab" : "Mute tab") + '">' + tabAudioIcon(tab.muted) + '</button>'
            : '';
          return '<button class="tab'
            + (tab.active ? ' active' : '')
            + (tab.pinned ? ' pinned' : '')
            + (tab.muted ? ' muted' : '')
            + '" type="button" draggable="true" data-tab-id="' + tab.id + '" data-pinned="' + String(!!tab.pinned) + '" title="' + esc(tab.pinned ? "Pinned tab" : "Tab") + '">'
            + favicon
            + fallback
            + (tab.pinned ? '<span class="tab-pin">📌</span>' : '')
            + '<span class="tab-label">' + esc(tab.title) + '</span>'
            + audio
            + '</button>';
        }).join("");
      }

      function applyTheme(theme) {
        document.documentElement.classList.remove("theme-warm", "theme-light", "theme-dark");
        if (theme === "warm")  document.documentElement.classList.add("theme-warm");
        if (theme === "light") document.documentElement.classList.add("theme-light");
        if (theme === "dark")  document.documentElement.classList.add("theme-dark");
      }

      function render(nextState) {
        Object.assign(state, nextState || {});
        applyTheme(state.theme || "system");
        document.body.dataset.loading    = String(!!state.loading);
        document.body.dataset.bookmarked = String(!!state.bookmarked);
        if (document.activeElement !== address) {
          address.value = state.url || "";
        }
        renderTabs(state.tabs || []);
      }

      window.__syncToolbar = function(nextState) { render(nextState); };

      window.__focusAddress = function() { address.focus(); address.select(); };
      window.__pasteAddressFromNative = function(text) {
        address.focus();
        replaceAddressSelection(String(text || ""));
      };

      navForm.addEventListener("submit", function(e) {
        e.preventDefault();
        if (selIndex >= 0 && suggestions[selIndex]) {
          executeSuggestion(suggestions[selIndex]);
          return;
        }
        hideSug();
        post({ kind: "navigate", value: address.value });
      });

      backBtn.addEventListener("click",     function() { post({ kind: "back" }); });
      forwardBtn.addEventListener("click",  function() { post({ kind: "forward" }); });
      reloadBtn.addEventListener("click",   function() { post({ kind: "reload" }); });
      bookmarkBtn.addEventListener("click", function() { post({ kind: "toggle-bookmark" }); });
      newTabBtn.addEventListener("click",   function() { post({ kind: "new-tab" }); });
      closeTabBtn.addEventListener("click", function() { post({ kind: "close-tab" }); });
      settingsBtn.addEventListener("click", function() { post({ kind: "open-settings" }); });

      tabsEl.addEventListener("click", function(e) {
        var muteTarget = e.target.closest("[data-mute-tab]");
        if (muteTarget) {
          e.stopPropagation();
          post({ kind: "toggle-mute-tab", id: Number(muteTarget.dataset.muteTab) });
          return;
        }
        var tabTarget = e.target.closest("[data-tab-id]");
        if (tabTarget) post({ kind: "activate-tab", id: Number(tabTarget.dataset.tabId) });
      });

      tabsEl.addEventListener("dblclick", function(e) {
        var tabTarget = e.target.closest("[data-tab-id]");
        if (!tabTarget) return;
        post({ kind: "toggle-pin-tab", id: Number(tabTarget.dataset.tabId) });
      });

      tabsEl.addEventListener("dragstart", function(e) {
        var tabTarget = e.target.closest("[data-tab-id]");
        if (!tabTarget) return;
        dragTabId = Number(tabTarget.dataset.tabId);
        tabTarget.classList.add("dragging");
        if (e.dataTransfer) {
          e.dataTransfer.setData("text/plain", String(dragTabId));
          e.dataTransfer.effectAllowed = "move";
        }
      });

      tabsEl.addEventListener("dragend", function() {
        dragTabId = null;
        tabsEl.querySelectorAll(".tab.dragging").forEach(function(el) {
          el.classList.remove("dragging");
        });
      });

      tabsEl.addEventListener("dragover", function(e) {
        if (dragTabId == null) return;
        var tabTarget = e.target.closest("[data-tab-id]");
        if (!tabTarget) return;
        if (Number(tabTarget.dataset.tabId) === dragTabId) return;
        e.preventDefault();
        if (e.dataTransfer) {
          e.dataTransfer.dropEffect = "move";
        }
      });

      tabsEl.addEventListener("drop", function(e) {
        if (dragTabId == null) return;
        var tabTarget = e.target.closest("[data-tab-id]");
        if (!tabTarget) return;
        var targetId = Number(tabTarget.dataset.tabId);
        if (targetId === dragTabId) return;
        e.preventDefault();
        post({ kind: "reorder-tab", id: dragTabId, target_id: targetId });
      });

      address.addEventListener("focus", function() {
        if (address.value) doSuggest(address.value);
      });

      address.addEventListener("input", function() {
        clearTimeout(sugTimer);
        sugTimer = setTimeout(function() { doSuggest(address.value); }, 150);
      });

      address.addEventListener("keydown", function(e) {
        var meta = /Mac|iPhone|iPad/.test(navigator.platform) ? e.metaKey : e.ctrlKey;
        var key = e.key.toLowerCase();
        if (meta && key === "a") {
          e.preventDefault();
          e.stopPropagation();
          address.focus();
          address.select();
          return;
        }
        if (meta && key === "c") {
          e.preventDefault();
          e.stopPropagation();
          post({ kind: "copy-address-selection", value: selectedAddressText() });
          return;
        }
        if (meta && key === "x") {
          e.preventDefault();
          e.stopPropagation();
          post({ kind: "cut-address-selection", value: selectedAddressText() });
          if (selectedAddressText()) replaceAddressSelection("");
          return;
        }
        if (meta && key === "v") {
          e.preventDefault();
          e.stopPropagation();
          post({ kind: "paste-into-address" });
          return;
        }
        if (e.key === "Escape") { hideSug(); render(state); address.blur(); return; }
        if (e.key === "Enter" && selIndex >= 0 && suggestions[selIndex]) {
          e.preventDefault();
          executeSuggestion(suggestions[selIndex]);
          return;
        }
        if (e.key === "ArrowDown") {
          e.preventDefault();
          setSelection(Math.min(selIndex + 1, suggestions.length - 1));
          return;
        }
        if (e.key === "ArrowUp") {
          e.preventDefault();
          setSelection(Math.max(selIndex - 1, -1));
          if (selIndex < 0) address.value = lastQuery;
          return;
        }
      });

      address.addEventListener("blur", function() {
        setTimeout(hideSug, 200);
      });

      suggestEl.addEventListener("mousedown", function(e) {
        e.preventDefault(); // prevent blur before click
      });

      suggestEl.addEventListener("click", function(e) {
        const item = e.target.closest(".sug-item");
        if (!item) return;
        executeSuggestion(suggestions[Number(item.dataset.i)]);
      });

      document.addEventListener("keydown", function(e) {
        var meta = /Mac|iPhone|iPad/.test(navigator.platform) ? e.metaKey : e.ctrlKey;
        if (!meta) return;
        var key = e.key.toLowerCase();
        if      (key === "t" && e.shiftKey) { e.preventDefault(); post({kind:"reopen-closed-tab"}); }
        else if (key === "t")              { e.preventDefault(); post({kind:"new-tab"}); }
        else if (key === "d" && e.shiftKey) { e.preventDefault(); post({kind:"duplicate-tab"}); }
        else if (key === "p" && e.shiftKey) { e.preventDefault(); post({kind:"toggle-pin-tab"}); }
        else if (key === "w")              { e.preventDefault(); post({kind:"close-tab"}); }
        else if (key === "y")              { e.preventDefault(); post({kind:"open-history-page"}); }
        else if (key === "r")              { e.preventDefault(); post({kind:"reload"}); }
        else if (key === "l" || key === "k") { e.preventDefault(); window.__focusAddress(); }
        else if (key === "arrowleft")      { e.preventDefault(); post({kind:"select-prev-tab"}); }
        else if (key === "arrowright")     { e.preventDefault(); post({kind:"select-next-tab"}); }
        else if (key === "[")              { e.preventDefault(); post({kind:"back"}); }
        else if (key === "]")              { e.preventDefault(); post({kind:"forward"}); }
        else if (key === ",")              { e.preventDefault(); post({kind:"open-settings"}); }
      });

      render(initialState);
    </script>
  </body>
</html>
"#;

pub fn html(initial_state: &Value) -> Result<String> {
    Ok(TOOLBAR_HTML.replace("__INITIAL_STATE__", &initial_state.to_string()))
}

pub fn sync_script(state: &Value) -> Result<String> {
    Ok(format!("window.__syncToolbar({state});"))
}
