use anyhow::Result;
use serde_json::Value;

const SETTINGS_PAGE_HTML: &str = r##"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Settings</title>
    <style>
      :root {
        color-scheme: light dark;
        --bg:          #ffffff;
        --bg-2:        #f4f4f4;
        --panel:       rgba(255,255,255,0.85);
        --panel-soft:  rgba(248,248,248,0.8);
        --line:        rgba(0,0,0,0.09);
        --line-strong: rgba(0,0,0,0.14);
        --text:        #111111;
        --muted:       #666666;
        --accent:      #0066cc;
        --accent-soft: rgba(0,102,204,0.12);
        --shadow:      0 4px 20px rgba(0,0,0,0.08);
      }
      @media (prefers-color-scheme: dark) {
        :root:not(.theme-warm):not(.theme-light) {
          --bg:          #141414;
          --bg-2:        #1e1e1e;
          --panel:       rgba(30,30,30,0.9);
          --panel-soft:  rgba(35,35,35,0.8);
          --line:        rgba(255,255,255,0.09);
          --line-strong: rgba(255,255,255,0.14);
          --text:        #f0f0f0;
          --muted:       #888888;
          --accent:      #4ea3ff;
          --accent-soft: rgba(78,163,255,0.15);
          --shadow:      0 4px 20px rgba(0,0,0,0.35);
        }
      }
      :root.theme-dark {
        color-scheme: dark;
        --bg:          #141414;
        --bg-2:        #1e1e1e;
        --panel:       rgba(30,30,30,0.9);
        --panel-soft:  rgba(35,35,35,0.8);
        --line:        rgba(255,255,255,0.09);
        --line-strong: rgba(255,255,255,0.14);
        --text:        #f0f0f0;
        --muted:       #888888;
        --accent:      #4ea3ff;
        --accent-soft: rgba(78,163,255,0.15);
        --shadow:      0 4px 20px rgba(0,0,0,0.35);
      }
      :root.theme-light {
        color-scheme: light;
        --bg: #ffffff; --bg-2: #f4f4f4; --text: #111111; --muted: #666666;
        --panel: rgba(255,255,255,0.85); --panel-soft: rgba(248,248,248,0.8);
        --line: rgba(0,0,0,0.09); --line-strong: rgba(0,0,0,0.14);
        --accent: #0066cc; --accent-soft: rgba(0,102,204,0.12);
        --shadow: 0 4px 20px rgba(0,0,0,0.08);
      }
      :root.theme-warm {
        color-scheme: light;
        --bg:          #f6efe6;
        --bg-2:        #ede0cf;
        --panel:       rgba(255,252,247,0.9);
        --panel-soft:  rgba(255,251,246,0.72);
        --line:        rgba(75,48,22,0.1);
        --line-strong: rgba(75,48,22,0.16);
        --text:        #27180c;
        --muted:       #7b5d45;
        --accent:      #bc5f33;
        --accent-soft: rgba(188,95,51,0.14);
        --shadow:      0 4px 20px rgba(93,61,34,0.12);
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
        max-width: 680px;
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

      .card {
        padding: 24px;
        border-radius: 24px;
        border: 1px solid var(--line-strong);
        background: var(--panel);
        box-shadow: var(--shadow);
        display: grid;
        gap: 20px;
      }

      .section-label {
        font-size: 11px;
        font-weight: 700;
        letter-spacing: 0.12em;
        text-transform: uppercase;
        color: var(--muted);
        margin-bottom: 12px;
      }

      .option-grid {
        display: grid;
        gap: 10px;
      }

      .option {
        display: flex;
        align-items: center;
        gap: 14px;
        padding: 14px 16px;
        border-radius: 16px;
        border: 1.5px solid var(--line);
        background: var(--panel-soft);
        cursor: pointer;
        transition: border-color 120ms, background 120ms;
      }

      .option:hover {
        background: var(--bg-2);
        border-color: var(--accent);
      }

      .option.selected {
        border-color: var(--accent);
        background: var(--accent-soft);
      }

      .option input[type="radio"] {
        display: none;
      }

      .radio-dot {
        width: 18px;
        height: 18px;
        border-radius: 999px;
        border: 2px solid var(--line-strong);
        background: var(--bg);
        flex-shrink: 0;
        display: grid;
        place-items: center;
        transition: border-color 120ms;
      }

      .option.selected .radio-dot {
        border-color: var(--accent);
        background: var(--accent);
      }

      .radio-dot::after {
        content: "";
        width: 6px;
        height: 6px;
        border-radius: 999px;
        background: white;
        opacity: 0;
        transition: opacity 120ms;
      }

      .option.selected .radio-dot::after {
        opacity: 1;
      }

      .option-text {
        display: grid;
        gap: 2px;
      }

      .option-name {
        font-size: 15px;
        font-weight: 600;
      }

      .option-desc {
        font-size: 12px;
        color: var(--muted);
      }

      .info-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 12px 16px;
        border-radius: 14px;
        background: var(--panel-soft);
        border: 1px solid var(--line);
        gap: 16px;
      }

      .info-label {
        font-size: 13px;
        font-weight: 600;
      }

      .info-value {
        font-size: 12px;
        color: var(--muted);
        word-break: break-all;
        text-align: right;
      }

      .shortcuts-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 8px;
      }

      .shortcut-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 8px 12px;
        border-radius: 10px;
        background: var(--panel-soft);
        border: 1px solid var(--line);
        font-size: 13px;
        gap: 12px;
      }

      .shortcut-action {
        color: var(--muted);
      }

      kbd {
        font-family: "SF Mono", "Menlo", monospace;
        font-size: 11px;
        padding: 2px 6px;
        border-radius: 6px;
        background: var(--accent-soft);
        border: 1px solid var(--line);
        color: var(--accent);
        white-space: nowrap;
      }

      @media (max-width: 680px) {
        html, body { padding: 18px; }
        .shortcuts-grid { grid-template-columns: 1fr; }
      }
    </style>
  </head>
  <body>
    <main class="page">
      <div class="eyebrow">
        <span class="eyebrow-dot"></span>
        <span>Tartanos Settings</span>
      </div>

      <div>
        <h1>Settings</h1>
        <p class="subtitle">Konfigurasi dasar browser.</p>
      </div>

      <div class="card">
        <div>
          <div class="section-label">Search Engine</div>
          <div class="option-grid" id="search-options">
            <label class="option" data-value="duckduckgo">
              <input type="radio" name="search_engine" value="duckduckgo" />
              <div class="radio-dot"></div>
              <div class="option-text">
                <span class="option-name">DuckDuckGo</span>
                <span class="option-desc">Privacy-focused, no tracking</span>
              </div>
            </label>
            <label class="option" data-value="google">
              <input type="radio" name="search_engine" value="google" />
              <div class="radio-dot"></div>
              <div class="option-text">
                <span class="option-name">Google</span>
                <span class="option-desc">Most comprehensive results</span>
              </div>
            </label>
            <label class="option" data-value="bing">
              <input type="radio" name="search_engine" value="bing" />
              <div class="radio-dot"></div>
              <div class="option-text">
                <span class="option-name">Bing</span>
                <span class="option-desc">Microsoft search engine</span>
              </div>
            </label>
          </div>
        </div>
      </div>

      <div class="card">
        <div>
          <div class="section-label">Theme</div>
          <div class="option-grid" id="theme-options">
            <label class="option" data-value="system">
              <input type="radio" name="theme" value="system" />
              <div class="radio-dot"></div>
              <div class="option-text">
                <span class="option-name">System</span>
                <span class="option-desc">Follow macOS dark/light mode</span>
              </div>
            </label>
            <label class="option" data-value="light">
              <input type="radio" name="theme" value="light" />
              <div class="radio-dot"></div>
              <div class="option-text">
                <span class="option-name">Light</span>
                <span class="option-desc">Always light</span>
              </div>
            </label>
            <label class="option" data-value="dark">
              <input type="radio" name="theme" value="dark" />
              <div class="radio-dot"></div>
              <div class="option-text">
                <span class="option-name">Dark</span>
                <span class="option-desc">Always dark</span>
              </div>
            </label>
            <label class="option" data-value="warm">
              <input type="radio" name="theme" value="warm" />
              <div class="radio-dot"></div>
              <div class="option-text">
                <span class="option-name">Warm</span>
                <span class="option-desc">Cream / sand color palette</span>
              </div>
            </label>
          </div>
        </div>
      </div>

      <div class="card">
        <div>
          <div class="section-label">Downloads</div>
          <div class="info-row">
            <span class="info-label">Download folder</span>
            <span class="info-value" id="download-dir">—</span>
          </div>
        </div>
      </div>

      <div class="card">
        <div>
          <div class="section-label">Keyboard Shortcuts</div>
          <div class="shortcuts-grid">
            <div class="shortcut-row"><span class="shortcut-action">New tab</span><kbd>⌘ T</kbd></div>
            <div class="shortcut-row"><span class="shortcut-action">Close tab</span><kbd>⌘ W</kbd></div>
            <div class="shortcut-row"><span class="shortcut-action">Reopen closed tab</span><kbd>⌘ ⇧ T</kbd></div>
            <div class="shortcut-row"><span class="shortcut-action">Duplicate tab</span><kbd>⌘ ⇧ D</kbd></div>
            <div class="shortcut-row"><span class="shortcut-action">Pin tab</span><kbd>⌘ ⇧ P</kbd></div>
            <div class="shortcut-row"><span class="shortcut-action">History</span><kbd>⌘ Y</kbd></div>
            <div class="shortcut-row"><span class="shortcut-action">Focus address</span><kbd>⌘ L</kbd></div>
            <div class="shortcut-row"><span class="shortcut-action">Reload</span><kbd>⌘ R</kbd></div>
            <div class="shortcut-row"><span class="shortcut-action">Back</span><kbd>⌘ [</kbd></div>
            <div class="shortcut-row"><span class="shortcut-action">Forward</span><kbd>⌘ ]</kbd></div>
            <div class="shortcut-row"><span class="shortcut-action">Settings</span><kbd>⌘ ,</kbd></div>
          </div>
        </div>
      </div>
    </main>

    <script>
      const initialState = __INITIAL_STATE__;
      const state = {
        search_engine: "google",
        theme: "system",
        download_dir: "",
      };

      function post(payload) {
        window.ipc.postMessage(JSON.stringify(payload));
      }

      function updateGroup(groupId, value) {
        document.querySelectorAll("#" + groupId + " .option").forEach(function(el) {
          el.classList.toggle("selected", el.dataset.value === value);
        });
      }

      function applyTheme(theme) {
        document.documentElement.classList.remove("theme-warm","theme-light","theme-dark");
        if (theme === "warm")  document.documentElement.classList.add("theme-warm");
        if (theme === "light") document.documentElement.classList.add("theme-light");
        if (theme === "dark")  document.documentElement.classList.add("theme-dark");
      }

      function render(nextState) {
        Object.assign(state, nextState ?? {});
        applyTheme(state.theme || "system");
        updateGroup("search-options", state.search_engine);
        updateGroup("theme-options",  state.theme || "system");
        const el = document.getElementById("download-dir");
        el.textContent = state.download_dir || "Default system folder";
      }

      window.__syncSettingsPage = function(nextState) {
        render(nextState);
      };

      document.getElementById("search-options").addEventListener("click", function(e) {
        const option = e.target.closest(".option");
        if (!option) return;
        post({ kind: "settings-update", key: "search_engine", value: option.dataset.value });
      });

      document.getElementById("theme-options").addEventListener("click", function(e) {
        const option = e.target.closest(".option");
        if (!option) return;
        post({ kind: "settings-update", key: "theme", value: option.dataset.value });
      });

      render(initialState);
    </script>
  </body>
</html>
"##;

pub fn html(initial_state: &Value) -> Result<String> {
    Ok(SETTINGS_PAGE_HTML.replace("__INITIAL_STATE__", &initial_state.to_string()))
}

pub fn sync_script(state: &Value) -> Result<String> {
    Ok(format!("window.__syncSettingsPage({state});"))
}
