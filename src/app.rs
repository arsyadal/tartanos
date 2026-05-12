use std::{
    collections::HashMap,
    env, fs,
    net::IpAddr,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tao::{
    dpi::{LogicalPosition, LogicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    window::{Theme, Window, WindowBuilder},
};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;
use url::{Url, form_urlencoded::byte_serialize};
use wry::{NewWindowResponse, PageLoadEvent, Rect, WebContext, WebView, WebViewBuilder};

use crate::{history_page, settings_page, start_page, toolbar};

const TOOLBAR_HEIGHT: u32 = 88;
const HISTORY_LIMIT: usize = 80;
const BOOKMARK_LIMIT: usize = 40;
const SHELF_LIMIT: usize = 6;
const DOWNLOAD_LIMIT: usize = 20;
const START_PAGE_TITLE: &str = "New Tab";

const KEYBOARD_SHORTCUTS_SCRIPT: &str = r#"
(function() {
  document.addEventListener('keydown', function(e) {
    var meta = /Mac|iPhone|iPad/.test(navigator.platform) ? e.metaKey : e.ctrlKey;
    if (!meta) return;
    var key = e.key.toLowerCase();
    var cmd = null;
    if (key === 't' && e.shiftKey) cmd = {kind:'reopen-closed-tab'};
    else if (key === 't') cmd = {kind:'new-tab'};
    else if (key === 'd' && e.shiftKey) cmd = {kind:'duplicate-tab'};
    else if (key === 'p' && e.shiftKey) cmd = {kind:'toggle-pin-tab'};
    else if (key === 'w') cmd = {kind:'close-tab'};
    else if (key === 'r') cmd = {kind:'reload'};
    else if (key === 'y') cmd = {kind:'open-history-page'};
    else if (key === 'l' || key === 'k') cmd = {kind:'focus-address'};
    else if (key === 'arrowleft') cmd = {kind:'select-prev-tab'};
    else if (key === 'arrowright') cmd = {kind:'select-next-tab'};
    else if (key === '[') cmd = {kind:'back'};
    else if (key === ']') cmd = {kind:'forward'};
    else if (key === ',') cmd = {kind:'open-settings'};
    if (cmd && window.ipc) {
      e.preventDefault();
      window.ipc.postMessage(JSON.stringify(cmd));
    }
  }, true);

  function tartanosReportAudible() {
    if (!window.ipc) return;
    var media = Array.from(document.querySelectorAll('audio,video'));
    var audible = media.some(function(node) {
      return !node.paused && !node.ended && !node.muted && node.volume > 0;
    });
    window.ipc.postMessage(JSON.stringify({ kind: 'tab-audible-state', value: String(audible) }));
  }

  function tartanosWatchMedia(node) {
    if (!node || node.__tartanosAudioHooked) return;
    node.__tartanosAudioHooked = true;
    ['play', 'pause', 'volumechange', 'ended', 'emptied'].forEach(function(eventName) {
      node.addEventListener(eventName, tartanosReportAudible, true);
    });
  }

  function tartanosInitMediaTracking() {
    document.querySelectorAll('audio,video').forEach(tartanosWatchMedia);
    tartanosReportAudible();
  }

  var tartanosObserver = new MutationObserver(function() {
    tartanosInitMediaTracking();
  });

  if (document.documentElement) {
    tartanosObserver.observe(document.documentElement, { childList: true, subtree: true });
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', tartanosInitMediaTracking, { once: true });
  } else {
    tartanosInitMediaTracking();
  }
})();
"#;

pub fn run() -> Result<()> {
    init_tracing();

    let app = BrowserApp::bootstrap()?;
    app.run()
}

struct BrowserApp {
    data_dir: PathBuf,
    config_dir: PathBuf,
}

#[derive(Debug, Clone)]
enum UserEvent {
    Toolbar(ToolbarCommand),
    TabCommand {
        tab_id: u64,
        command: ToolbarCommand,
    },
    Navigation {
        tab_id: u64,
        url: String,
    },
    PageLoad {
        tab_id: u64,
        url: String,
        is_loading: bool,
    },
    TitleChanged {
        tab_id: u64,
        title: String,
    },
    OpenInTab {
        tab_id: u64,
        url: String,
    },
    DownloadStarted {
        url: String,
        filename: String,
    },
    DownloadCompleted {
        url: String,
        path: Option<PathBuf>,
        success: bool,
    },
}

#[derive(Debug, Clone)]
enum ToolbarCommand {
    Navigate(String),
    Back,
    Forward,
    Reload,
    NewTab,
    ActivateTab(u64),
    ReorderTab { tab_id: u64, target_id: u64 },
    TogglePinTab(Option<u64>),
    ToggleMuteTab(Option<u64>),
    TabAudibleState(bool),
    DuplicateTab(Option<u64>),
    ReopenClosedTab,
    SelectPrevTab,
    SelectNextTab,
    CloseTab(u64),
    CloseCurrentTab,
    ToggleBookmark,
    OpenBookmark(String),
    OpenHistory(String),
    OpenHistoryPage,
    DeleteHistory(u64),
    ClearHistory,
    CopyAddressSelection(String),
    CutAddressSelection(String),
    PasteIntoAddress,
    OpenSettings,
    FocusAddress,
    SettingsUpdate { key: String, value: String },
    SetHeight(u32),
}

#[derive(Debug, Deserialize)]
struct ToolbarMessage {
    kind: String,
    value: Option<String>,
    id: Option<u64>,
    target_id: Option<u64>,
    key: Option<String>,
}

impl ToolbarCommand {
    fn parse(message: &str) -> Option<Self> {
        let command: ToolbarMessage = serde_json::from_str(message).ok()?;

        match command.kind.as_str() {
            "navigate" => Some(Self::Navigate(command.value?)),
            "back" => Some(Self::Back),
            "forward" => Some(Self::Forward),
            "reload" => Some(Self::Reload),
            "new-tab" => Some(Self::NewTab),
            "activate-tab" => Some(Self::ActivateTab(command.id?)),
            "reorder-tab" => Some(Self::ReorderTab {
                tab_id: command.id?,
                target_id: command.target_id?,
            }),
            "toggle-pin-tab" => Some(Self::TogglePinTab(command.id)),
            "toggle-mute-tab" => Some(Self::ToggleMuteTab(command.id)),
            "tab-audible-state" => Some(Self::TabAudibleState(command.value?.parse::<bool>().ok()?)),
            "duplicate-tab" => Some(Self::DuplicateTab(command.id)),
            "reopen-closed-tab" => Some(Self::ReopenClosedTab),
            "select-prev-tab" => Some(Self::SelectPrevTab),
            "select-next-tab" => Some(Self::SelectNextTab),
            "close-tab" => Some(
                command
                    .id
                    .map(Self::CloseTab)
                    .unwrap_or(Self::CloseCurrentTab),
            ),
            "toggle-bookmark" => Some(Self::ToggleBookmark),
            "open-bookmark" => Some(Self::OpenBookmark(command.value?)),
            "open-history" => Some(Self::OpenHistory(command.value?)),
            "open-history-page" => Some(Self::OpenHistoryPage),
            "delete-history" => Some(Self::DeleteHistory(command.id?)),
            "clear-history" => Some(Self::ClearHistory),
            "copy-address-selection" => Some(Self::CopyAddressSelection(command.value.unwrap_or_default())),
            "cut-address-selection" => Some(Self::CutAddressSelection(command.value.unwrap_or_default())),
            "paste-into-address" => Some(Self::PasteIntoAddress),
            "open-settings" => Some(Self::OpenSettings),
            "focus-address" => Some(Self::FocusAddress),
            "set-height" => {
                let h = command.value?.parse::<u32>().ok()?;
                Some(Self::SetHeight(h))
            }
            "settings-update" => Some(Self::SettingsUpdate {
                key: command.key?,
                value: command.value?,
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
enum TabContent {
    StartPage,
    Settings,
    History,
    Page { url: String },
}

#[derive(Debug, Clone)]
struct TabSession {
    id: u64,
    title: String,
    content: TabContent,
    pinned: bool,
    muted: bool,
    audible: bool,
    is_loading: bool,
}

impl TabSession {
    fn new_start_page(id: u64) -> Self {
        Self {
            id,
            title: START_PAGE_TITLE.to_string(),
            content: TabContent::StartPage,
            pinned: false,
            muted: false,
            audible: false,
            is_loading: false,
        }
    }

    fn new_page(id: u64, url: String) -> Self {
        Self {
            id,
            title: title_from_url(&url),
            content: TabContent::Page { url },
            pinned: false,
            muted: false,
            audible: false,
            is_loading: false,
        }
    }

    fn restored(saved_tab: PersistedTab) -> Self {
        match saved_tab.state {
            PersistedTabState::StartPage => Self {
                id: saved_tab.id,
                title: START_PAGE_TITLE.to_string(),
                content: TabContent::StartPage,
                pinned: saved_tab.pinned,
                muted: saved_tab.muted,
                audible: false,
                is_loading: false,
            },
            PersistedTabState::Settings => Self {
                id: saved_tab.id,
                title: "Settings".to_string(),
                content: TabContent::Settings,
                pinned: saved_tab.pinned,
                muted: saved_tab.muted,
                audible: false,
                is_loading: false,
            },
            PersistedTabState::History => Self {
                id: saved_tab.id,
                title: "History".to_string(),
                content: TabContent::History,
                pinned: saved_tab.pinned,
                muted: saved_tab.muted,
                audible: false,
                is_loading: false,
            },
            PersistedTabState::Page { url } => {
                let mut tab = Self::new_page(saved_tab.id, url);
                tab.pinned = saved_tab.pinned;
                tab.muted = saved_tab.muted;
                tab.audible = false;
                tab.title = if saved_tab.title.trim().is_empty() {
                    title_from_url(tab.display_url())
                } else {
                    saved_tab.title
                };
                tab
            }
        }
    }

    fn new_settings(id: u64) -> Self {
        Self {
            id,
            title: "Settings".to_string(),
            content: TabContent::Settings,
            pinned: false,
            muted: false,
            audible: false,
            is_loading: false,
        }
    }

    fn new_history(id: u64) -> Self {
        Self {
            id,
            title: "History".to_string(),
            content: TabContent::History,
            pinned: false,
            muted: false,
            audible: false,
            is_loading: false,
        }
    }

    fn display_url(&self) -> &str {
        match &self.content {
            TabContent::StartPage => "",
            TabContent::Settings => "tartanos://settings",
            TabContent::History => "tartanos://history",
            TabContent::Page { url } => url.as_str(),
        }
    }

    fn is_start_page(&self) -> bool {
        matches!(self.content, TabContent::StartPage)
    }

    fn is_settings_page(&self) -> bool {
        matches!(self.content, TabContent::Settings)
    }

    fn is_history_page(&self) -> bool {
        matches!(self.content, TabContent::History)
    }

    fn is_internal_page(&self) -> bool {
        matches!(self.content, TabContent::StartPage | TabContent::Settings | TabContent::History)
    }

    fn begin_navigation(&mut self, url: String) {
        let next_title = if self.display_url() == url && !self.title.trim().is_empty() {
            self.title.clone()
        } else {
            title_from_url(&url)
        };

        self.title = next_title;
        self.content = TabContent::Page { url };
        self.is_loading = true;
    }

    fn finish_navigation(&mut self, url: String) {
        self.content = TabContent::Page { url };
        self.is_loading = false;
    }

    fn reset_to_start_page(&mut self) {
        self.title = START_PAGE_TITLE.to_string();
        self.content = TabContent::StartPage;
        self.is_loading = false;
    }

    fn persisted(&self) -> PersistedTab {
        let state = match &self.content {
            TabContent::StartPage => PersistedTabState::StartPage,
            TabContent::Settings => PersistedTabState::Settings,
            TabContent::History => PersistedTabState::History,
            TabContent::Page { url } => PersistedTabState::Page { url: url.clone() },
        };

        PersistedTab {
            id: self.id,
            title: self.title.clone(),
            pinned: self.pinned,
            muted: self.muted,
            state,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BookmarkItem {
    id: u64,
    title: String,
    url: String,
    saved_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HistoryEntry {
    id: u64,
    title: String,
    url: String,
    visited_at: u64,
}

#[derive(Debug, Clone, PartialEq)]
enum DownloadStatus {
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
struct DownloadItem {
    id: u64,
    url: String,
    filename: String,
    status: DownloadStatus,
    path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Preferences {
    #[serde(default = "default_search_engine")]
    search_engine: String,
    #[serde(default = "default_theme")]
    theme: String,
}

fn default_search_engine() -> String {
    "google".to_string()
}

fn default_theme() -> String {
    "system".to_string()
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            search_engine: default_search_engine(),
            theme: default_theme(),
        }
    }
}

impl Preferences {
    fn search_base_url(&self) -> &'static str {
        match self.search_engine.as_str() {
            "google" => "https://www.google.com/search?q=",
            "bing" => "https://www.bing.com/search?q=",
            _ => "https://duckduckgo.com/?q=",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedSession {
    active_tab_id: u64,
    tabs: Vec<PersistedTab>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedTab {
    id: u64,
    title: String,
    #[serde(default)]
    pinned: bool,
    #[serde(default)]
    muted: bool,
    #[serde(flatten)]
    state: PersistedTabState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum PersistedTabState {
    StartPage,
    Settings,
    History,
    Page { url: String },
}

struct BrowserStore {
    bookmarks_path: PathBuf,
    history_path: PathBuf,
    session_path: PathBuf,
    preferences_path: PathBuf,
}

impl BrowserStore {
    fn new(data_dir: impl AsRef<Path>) -> Self {
        let data_dir = data_dir.as_ref();
        Self {
            bookmarks_path: data_dir.join("bookmarks.json"),
            history_path: data_dir.join("history.json"),
            session_path: data_dir.join("session.json"),
            preferences_path: data_dir.join("preferences.json"),
        }
    }

    fn load_bookmarks(&self) -> Vec<BookmarkItem> {
        load_json_collection(&self.bookmarks_path, "bookmarks")
    }

    fn load_history(&self) -> Vec<HistoryEntry> {
        load_json_collection(&self.history_path, "history")
    }

    fn save_bookmarks(&self, bookmarks: &[BookmarkItem]) -> Result<()> {
        save_json_collection(&self.bookmarks_path, bookmarks, "bookmarks")
    }

    fn save_history(&self, history: &[HistoryEntry]) -> Result<()> {
        save_json_collection(&self.history_path, history, "history")
    }

    fn load_preferences(&self) -> Preferences {
        load_json_value(&self.preferences_path, "preferences").unwrap_or_default()
    }

    fn save_preferences(&self, prefs: &Preferences) -> Result<()> {
        save_json_value(&self.preferences_path, prefs, "preferences")
    }

    fn load_session(&self) -> Option<PersistedSession> {
        load_json_value(&self.session_path, "session")
    }

    fn save_session(&self, session: &PersistedSession) -> Result<()> {
        save_json_value(&self.session_path, session, "session")
    }
}

struct BrowserState {
    store: BrowserStore,
    tabs: Vec<TabSession>,
    active_tab_id: u64,
    next_tab_id: u64,
    bookmarks: Vec<BookmarkItem>,
    history: Vec<HistoryEntry>,
    next_saved_id: u64,
    downloads: Vec<DownloadItem>,
    next_download_id: u64,
    closed_tabs: Vec<ClosedTabSnapshot>,
    preferences: Preferences,
    download_dir: PathBuf,
    status_text: String,
    is_loading: bool,
}

struct BrowserViews {
    web_context: WebContext,
    content_bounds: Rect,
    webviews: HashMap<u64, WebView>,
    download_dir: PathBuf,
}

struct CloseTabResult {
    removed_tab_id: Option<u64>,
    active_tab_id: u64,
    recreate_active_tab: bool,
}

#[derive(Debug, Clone)]
struct ClosedTabSnapshot {
    tab: TabSession,
    index: usize,
}

impl BrowserState {
    fn new(store: BrowserStore, download_dir: PathBuf) -> Self {
        let bookmarks = store.load_bookmarks();
        let history = store.load_history();
        let preferences = store.load_preferences();
        let restored_session = store.load_session();
        let next_saved_id = next_saved_id(&bookmarks, &history);
        let (tabs, active_tab_id, next_tab_id, status_text) =
            restore_tabs(restored_session.as_ref());

        Self {
            store,
            tabs,
            active_tab_id,
            next_tab_id,
            bookmarks,
            history,
            next_saved_id,
            downloads: Vec::new(),
            next_download_id: 1,
            closed_tabs: Vec::new(),
            preferences,
            download_dir,
            status_text,
            is_loading: false,
        }
    }

    fn tab_index(&self, tab_id: u64) -> Option<usize> {
        self.tabs.iter().position(|tab| tab.id == tab_id)
    }

    fn active_tab_id(&self) -> u64 {
        self.active_tab_id
    }

    fn active_tab(&self) -> &TabSession {
        self.tabs
            .iter()
            .find(|tab| tab.id == self.active_tab_id)
            .expect("active tab must exist")
    }

    fn active_tab_mut(&mut self) -> &mut TabSession {
        self.tabs
            .iter_mut()
            .find(|tab| tab.id == self.active_tab_id)
            .expect("active tab must exist")
    }

    fn tab(&self, tab_id: u64) -> Option<&TabSession> {
        self.tabs.iter().find(|tab| tab.id == tab_id)
    }

    fn tab_mut(&mut self, tab_id: u64) -> Option<&mut TabSession> {
        self.tabs.iter_mut().find(|tab| tab.id == tab_id)
    }

    fn current_url(&self) -> &str {
        self.active_tab().display_url()
    }

    fn page_title(&self) -> &str {
        self.active_tab().title.as_str()
    }

    fn window_title(&self) -> String {
        format!("{} - Tartanos", self.page_title())
    }

    fn set_loading_for(&mut self, tab_id: u64, url: String) {
        let should_ignore_runtime_url = self
            .tab(tab_id)
            .map(|tab| tab.is_internal_page() && is_runtime_start_page_url(&url))
            .unwrap_or(false);

        if let Some(tab) = self.tab_mut(tab_id) {
            if should_ignore_runtime_url {
                tab.is_loading = false;
            } else {
                tab.begin_navigation(url);
            }
        }

        if self.active_tab_id == tab_id {
            if should_ignore_runtime_url {
                self.status_text = "Ready".to_string();
                self.is_loading = false;
            } else {
                self.status_text = "Loading...".to_string();
                self.is_loading = true;
            }
        }
    }

    fn set_ready_for(&mut self, tab_id: u64, url: String) {
        let should_ignore_runtime_url = self
            .tab(tab_id)
            .map(|tab| tab.is_internal_page() && is_runtime_start_page_url(&url))
            .unwrap_or(false);

        if let Some(tab) = self.tab_mut(tab_id) {
            if should_ignore_runtime_url {
                tab.is_loading = false;
                if tab.is_start_page() {
                    tab.title = START_PAGE_TITLE.to_string();
                }
            } else {
                tab.finish_navigation(url);
            }
        }

        if self.active_tab_id == tab_id {
            self.status_text = "Ready".to_string();
            self.is_loading = false;
        }
    }

    fn set_error(&mut self, message: impl Into<String>) {
        self.status_text = message.into();
        self.is_loading = false;

        if let Some(tab) = self.tab_mut(self.active_tab_id) {
            tab.is_loading = false;
        }
    }

    fn update_title_for(&mut self, tab_id: u64, title: String) {
        let fallback = self
            .tab(tab_id)
            .map(|tab| {
                if tab.is_start_page() {
                    START_PAGE_TITLE.to_string()
                } else {
                    title_from_url(tab.display_url())
                }
            })
            .unwrap_or_else(|| START_PAGE_TITLE.to_string());

        if let Some(tab) = self.tab_mut(tab_id) {
            let next_title = if title.trim().is_empty() {
                fallback
            } else {
                title
            };

            tab.title = next_title;
        }
    }

    fn new_settings_tab(&mut self) -> TabSession {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;
        let tab = TabSession::new_settings(tab_id);
        self.tabs.push(tab.clone());
        self.active_tab_id = tab_id;
        self.status_text = "Settings".to_string();
        self.is_loading = false;
        tab
    }

    fn new_history_tab(&mut self) -> TabSession {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;
        let tab = TabSession::new_history(tab_id);
        self.tabs.push(tab.clone());
        self.active_tab_id = tab_id;
        self.status_text = "History".to_string();
        self.is_loading = false;
        tab
    }

    fn new_tab(&mut self) -> TabSession {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;

        let tab = TabSession::new_start_page(tab_id);
        self.tabs.push(tab.clone());
        self.active_tab_id = tab_id;
        self.status_text = "Ready".to_string();
        self.is_loading = false;

        tab
    }

    fn discard_tab(&mut self, tab_id: u64) {
        if let Some(index) = self.tab_index(tab_id) {
            self.tabs.remove(index);
        }

        if !self.tabs.iter().any(|tab| tab.id == self.active_tab_id) {
            self.active_tab_id = self.tabs.last().map(|tab| tab.id).unwrap_or(1);
        }

        if let Some(active_tab) = self.tabs.iter().find(|tab| tab.id == self.active_tab_id) {
            self.is_loading = active_tab.is_loading;
            self.status_text = if active_tab.is_loading {
                "Loading...".to_string()
            } else {
                "Ready".to_string()
            };
        }
    }

    fn activate_tab(&mut self, tab_id: u64) -> bool {
        if let Some(is_loading) = self.tab(tab_id).map(|tab| tab.is_loading) {
            self.active_tab_id = tab_id;
            self.is_loading = is_loading;
            self.status_text = if is_loading {
                "Loading...".to_string()
            } else {
                "Ready".to_string()
            };
            return true;
        }

        false
    }

    fn activate_adjacent_tab(&mut self, current_tab_id: u64, delta: isize) -> Option<u64> {
        let current_index = self.tab_index(current_tab_id)?;
        let len = self.tabs.len();
        if len <= 1 {
            return Some(current_tab_id);
        }

        let next_index = if delta < 0 {
            current_index.checked_sub(delta.unsigned_abs()).unwrap_or(len - 1)
        } else {
            (current_index + delta as usize) % len
        };
        let next_tab_id = self.tabs[next_index].id;

        if self.activate_tab(next_tab_id) {
            Some(next_tab_id)
        } else {
            None
        }
    }

    fn pinned_count(&self) -> usize {
        self.tabs.iter().take_while(|tab| tab.pinned).count()
    }

    fn toggle_pin_tab(&mut self, tab_id: u64) -> Option<u64> {
        let index = self.tab_index(tab_id)?;
        let mut tab = self.tabs.remove(index);
        tab.pinned = !tab.pinned;
        let next_index = if tab.pinned {
            self.pinned_count()
        } else {
            self.pinned_count()
        };
        let insert_index = next_index.min(self.tabs.len());
        let status = if tab.pinned { "Pinned tab" } else { "Unpinned tab" };
        self.tabs.insert(insert_index, tab.clone());
        self.active_tab_id = tab.id;
        self.status_text = status.to_string();
        Some(tab.id)
    }

    fn toggle_mute_tab(&mut self, tab_id: u64) -> Option<bool> {
        let muted = {
            let tab = self.tab_mut(tab_id)?;
            tab.muted = !tab.muted;
            tab.muted
        };
        self.status_text = if muted {
            "Muted tab".to_string()
        } else {
            "Unmuted tab".to_string()
        };
        Some(muted)
    }

    fn set_tab_audible(&mut self, tab_id: u64, audible: bool) -> bool {
        let Some(tab) = self.tab_mut(tab_id) else {
            return false;
        };
        if tab.audible == audible {
            return false;
        }
        tab.audible = audible;
        true
    }

    fn reorder_tab(&mut self, tab_id: u64, target_id: u64) -> Option<u64> {
        if tab_id == target_id {
            return Some(tab_id);
        }
        let from_index = self.tab_index(tab_id)?;
        let moving = self.tabs.remove(from_index);
        let mut insert_index = self.tab_index(target_id)?;
        let pinned_count = self.pinned_count();
        insert_index = if moving.pinned {
            insert_index.min(pinned_count)
        } else {
            insert_index.max(pinned_count)
        };
        self.tabs.insert(insert_index, moving.clone());
        self.status_text = "Moved tab".to_string();
        Some(moving.id)
    }

    fn duplicate_tab(&mut self, tab_id: u64) -> Option<TabSession> {
        let original_index = self.tab_index(tab_id)?;
        let original = self.tab(tab_id)?.clone();
        let new_id = self.next_tab_id;
        self.next_tab_id += 1;
        let mut duplicate = original.clone();
        duplicate.id = new_id;
        duplicate.is_loading = false;
        let insert_index = (original_index + 1).min(self.tabs.len());
        self.tabs.insert(insert_index, duplicate.clone());
        self.active_tab_id = duplicate.id;
        self.status_text = "Duplicated tab".to_string();
        self.is_loading = false;
        Some(duplicate)
    }

    fn reopen_closed_tab(&mut self) -> Option<TabSession> {
        let snapshot = self.closed_tabs.pop()?;
        let insert_index = if snapshot.tab.pinned {
            snapshot.index.min(self.pinned_count())
        } else {
            snapshot.index.max(self.pinned_count()).min(self.tabs.len())
        };
        let mut tab = snapshot.tab;
        if tab.id >= self.next_tab_id {
            self.next_tab_id = tab.id + 1;
        }
        tab.is_loading = false;
        self.tabs.insert(insert_index, tab.clone());
        self.active_tab_id = tab.id;
        self.status_text = "Reopened closed tab".to_string();
        self.is_loading = false;
        Some(tab)
    }

    fn close_tab(&mut self, tab_id: u64) -> CloseTabResult {
        if self.tabs.len() == 1 {
            let active_tab_id = self.active_tab_id;
            let closed = self.active_tab().clone();
            self.closed_tabs.push(ClosedTabSnapshot {
                tab: closed,
                index: 0,
            });
            {
                let active = self.active_tab_mut();
                active.reset_to_start_page();
            }
            self.status_text = "Ready".to_string();
            self.is_loading = false;

            return CloseTabResult {
                removed_tab_id: Some(active_tab_id),
                active_tab_id,
                recreate_active_tab: true,
            };
        }

        let Some(closing_index) = self.tab_index(tab_id) else {
            return CloseTabResult {
                removed_tab_id: None,
                active_tab_id: self.active_tab_id,
                recreate_active_tab: false,
            };
        };

        let was_active = self.active_tab_id == tab_id;
        let closed = self.tabs[closing_index].clone();
        self.tabs.remove(closing_index);
        self.closed_tabs.push(ClosedTabSnapshot {
            tab: closed,
            index: closing_index,
        });
        self.closed_tabs.truncate(20);

        if was_active {
            let next_index = closing_index.saturating_sub(1).min(self.tabs.len() - 1);
            self.active_tab_id = self.tabs[next_index].id;
        }

        let active_is_loading = self.active_tab().is_loading;
        self.status_text = "Closed tab".to_string();
        self.is_loading = active_is_loading;

        CloseTabResult {
            removed_tab_id: Some(tab_id),
            active_tab_id: self.active_tab_id,
            recreate_active_tab: false,
        }
    }

    fn toggle_bookmark(&mut self) {
        let current_url = self.current_url().to_string();
        if current_url.is_empty() {
            self.set_error("Nothing to bookmark yet");
            return;
        }

        if let Some(index) = self
            .bookmarks
            .iter()
            .position(|bookmark| bookmark.url == current_url)
        {
            self.bookmarks.remove(index);
            self.status_text = "Removed bookmark".to_string();
        } else {
            let bookmark = BookmarkItem {
                id: self.next_saved_id,
                title: self.page_title().to_string(),
                url: current_url,
                saved_at: unix_timestamp(),
            };
            self.next_saved_id += 1;
            self.bookmarks.insert(0, bookmark);
            self.bookmarks.truncate(BOOKMARK_LIMIT);
            self.status_text = "Saved bookmark".to_string();
        }

        if let Err(error) = self.store.save_bookmarks(&self.bookmarks) {
            warn!(%error, "failed to persist bookmarks");
            self.status_text = "Bookmark changed, but save failed".to_string();
        }
    }

    fn record_history_for(&mut self, tab_id: u64) {
        let Some((title, url)) = self
            .tab(tab_id)
            .map(|tab| (tab.title.clone(), tab.display_url().to_string()))
        else {
            return;
        };

        if url.is_empty() {
            return;
        }

        if let Some(index) = self.history.iter().position(|entry| entry.url == url) {
            self.history.remove(index);
        }

        let entry = HistoryEntry {
            id: self.next_saved_id,
            title,
            url,
            visited_at: unix_timestamp(),
        };
        self.next_saved_id += 1;

        self.history.insert(0, entry);
        self.history.truncate(HISTORY_LIMIT);

        if let Err(error) = self.store.save_history(&self.history) {
            warn!(%error, "failed to persist history");
        }
    }

    fn history_page_payload(&self) -> serde_json::Value {
        let history = self.history.iter().map(|entry| {
            json!({
                "id": entry.id,
                "title": tab_title(entry.title.as_str(), entry.url.as_str()),
                "url": entry.url,
            })
        });

        json!({
            "history": history.collect::<Vec<_>>(),
            "theme": self.preferences.theme,
        })
    }

    fn delete_history_entry(&mut self, id: u64) {
        let original_len = self.history.len();
        self.history.retain(|entry| entry.id != id);
        if self.history.len() == original_len {
            return;
        }

        self.status_text = "History entry deleted".to_string();
        if let Err(error) = self.store.save_history(&self.history) {
            warn!(%error, "failed to persist history");
            self.status_text = "History changed, but save failed".to_string();
        }
    }

    fn clear_history(&mut self) {
        if self.history.is_empty() {
            self.status_text = "History already empty".to_string();
            return;
        }

        self.history.clear();
        self.status_text = "History cleared".to_string();
        if let Err(error) = self.store.save_history(&self.history) {
            warn!(%error, "failed to persist history");
            self.status_text = "History cleared, but save failed".to_string();
        }
    }

    fn update_preferences(&mut self, key: &str, value: &str) {
        match key {
            "search_engine" if matches!(value, "duckduckgo" | "google" | "bing") => {
                self.preferences.search_engine = value.to_string();
                self.status_text = format!("Search engine: {value}");
            }
            "theme" if matches!(value, "system" | "light" | "dark" | "warm") => {
                self.preferences.theme = value.to_string();
                self.status_text = format!("Theme: {value}");
            }
            _ => {
                warn!(key, value, "unknown preferences key");
            }
        }

        if let Err(error) = self.store.save_preferences(&self.preferences) {
            warn!(%error, "failed to persist preferences");
        }
    }

    fn settings_payload(&self) -> serde_json::Value {
        serde_json::json!({
            "search_engine": self.preferences.search_engine,
            "theme": self.preferences.theme,
            "download_dir": self.download_dir.to_string_lossy(),
        })
    }

    fn add_download(&mut self, url: String, filename: String) {
        let id = self.next_download_id;
        self.next_download_id += 1;
        self.downloads.insert(
            0,
            DownloadItem {
                id,
                url,
                filename: filename.clone(),
                status: DownloadStatus::InProgress,
                path: None,
            },
        );
        self.downloads.truncate(DOWNLOAD_LIMIT);
        self.status_text = format!("Downloading: {filename}");
    }

    fn complete_download(&mut self, url: &str, path: Option<PathBuf>, success: bool) {
        if let Some(item) = self.downloads.iter_mut().find(|d| d.url == url) {
            item.status = if success {
                DownloadStatus::Completed
            } else {
                DownloadStatus::Failed
            };
            item.path = path;
            self.status_text = if success {
                format!("Downloaded: {}", item.filename)
            } else {
                format!("Download failed: {}", item.filename)
            };
        }
    }

    fn is_bookmarked(&self) -> bool {
        self.bookmarks
            .iter()
            .any(|bookmark| bookmark.url == self.current_url())
    }

    fn toolbar_payload(&self) -> serde_json::Value {
        let tabs = self.tabs.iter().map(|tab| {
            json!({
                "id": tab.id,
                "title": tab_title(tab.title.as_str(), tab.display_url()),
                "url": tab.display_url(),
                "pinned": tab.pinned,
                "muted": tab.muted,
                "audible": tab.audible,
                "favicon": tab_favicon_url(tab),
                "icon": tab_internal_icon(tab),
                "active": tab.id == self.active_tab_id,
            })
        });

        let bookmarks = self.bookmarks.iter().take(SHELF_LIMIT).map(|bookmark| {
            json!({
                "id": bookmark.id,
                "title": tab_title(bookmark.title.as_str(), bookmark.url.as_str()),
                "url": bookmark.url,
            })
        });

        let history = self.history.iter().take(SHELF_LIMIT).map(|entry| {
            json!({
                "id": entry.id,
                "title": tab_title(entry.title.as_str(), entry.url.as_str()),
                "url": entry.url,
            })
        });

        let downloads = self.downloads.iter().take(SHELF_LIMIT).map(|d| {
            json!({
                "id": d.id,
                "filename": d.filename,
                "url": d.url,
                "status": match d.status {
                    DownloadStatus::InProgress => "in_progress",
                    DownloadStatus::Completed => "completed",
                    DownloadStatus::Failed => "failed",
                },
            })
        });

        json!({
            "url": self.current_url(),
            "title": self.page_title(),
            "status": self.status_text,
            "loading": self.is_loading,
            "bookmarked": self.is_bookmarked(),
            "theme": self.preferences.theme,
            "search_engine": self.preferences.search_engine,
            "tabs": tabs.collect::<Vec<_>>(),
            "bookmarks": bookmarks.collect::<Vec<_>>(),
            "history": history.collect::<Vec<_>>(),
            "downloads": downloads.collect::<Vec<_>>(),
        })
    }

    fn start_page_payload(&self) -> serde_json::Value {
        let bookmarks = self.bookmarks.iter().take(8).map(|bookmark| {
            json!({
                "id": bookmark.id,
                "title": tab_title(bookmark.title.as_str(), bookmark.url.as_str()),
                "url": bookmark.url,
            })
        });

        let history = self.history.iter().take(8).map(|entry| {
            json!({
                "id": entry.id,
                "title": tab_title(entry.title.as_str(), entry.url.as_str()),
                "url": entry.url,
            })
        });

        json!({
            "bookmarks": bookmarks.collect::<Vec<_>>(),
            "history": history.collect::<Vec<_>>(),
            "theme": self.preferences.theme,
        })
    }

    fn persist_session(&self) {
        let session = PersistedSession {
            active_tab_id: self.active_tab_id,
            tabs: self.tabs.iter().map(TabSession::persisted).collect(),
        };

        if let Err(error) = self.store.save_session(&session) {
            warn!(%error, "failed to persist browser session");
        }
    }
}

impl BrowserViews {
    fn new(data_dir: &Path, content_bounds: Rect, download_dir: PathBuf) -> Result<Self> {
        let web_context_dir = data_dir.join("web-context");
        fs::create_dir_all(&web_context_dir).with_context(|| {
            format!(
                "could not create web context dir at {}",
                web_context_dir.display()
            )
        })?;

        fs::create_dir_all(&download_dir).with_context(|| {
            format!("could not create download dir at {}", download_dir.display())
        })?;

        Ok(Self {
            web_context: WebContext::new(Some(web_context_dir)),
            content_bounds,
            webviews: HashMap::new(),
            download_dir,
        })
    }

    fn create_tab(
        &mut self,
        window: &Window,
        platform_container: &PlatformContainer,
        proxy: &EventLoopProxy<UserEvent>,
        tab: &TabSession,
        start_page_payload: &serde_json::Value,
        visible: bool,
    ) -> Result<()> {
        let tab_id = tab.id;
        let navigation_proxy = proxy.clone();
        let page_load_proxy = proxy.clone();
        let title_proxy = proxy.clone();
        let new_window_proxy = proxy.clone();
        let ipc_proxy = proxy.clone();
        let download_started_proxy = proxy.clone();
        let download_completed_proxy = proxy.clone();
        let download_dir = self.download_dir.clone();

        let mut builder = WebViewBuilder::new_with_web_context(&mut self.web_context)
            .with_bounds(self.content_bounds)
            .with_back_forward_navigation_gestures(true)
            .with_initialization_script(KEYBOARD_SHORTCUTS_SCRIPT)
            .with_visible(visible)
            .with_navigation_handler(move |url| {
                let _ = navigation_proxy.send_event(UserEvent::Navigation { tab_id, url });
                true
            })
            .with_on_page_load_handler(move |event, url| {
                let is_loading = matches!(event, PageLoadEvent::Started);
                let _ = page_load_proxy.send_event(UserEvent::PageLoad {
                    tab_id,
                    url,
                    is_loading,
                });
            })
            .with_document_title_changed_handler(move |title| {
                let _ = title_proxy.send_event(UserEvent::TitleChanged { tab_id, title });
            })
            .with_new_window_req_handler(move |url, _features| {
                let _ = new_window_proxy.send_event(UserEvent::OpenInTab { tab_id, url });
                NewWindowResponse::Deny
            })
            .with_ipc_handler(move |request| {
                if let Some(command) = ToolbarCommand::parse(request.body()) {
                    let _ = ipc_proxy.send_event(UserEvent::TabCommand { tab_id, command });
                }
            })
            .with_accept_first_mouse(true)
            .with_download_started_handler(move |url, path| {
                let filename = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .filter(|n| !n.is_empty())
                    .or_else(|| {
                        Url::parse(&url).ok().and_then(|u| {
                            u.path_segments()
                                .and_then(|s| s.last().map(str::to_string))
                                .filter(|s| !s.is_empty())
                        })
                    })
                    .unwrap_or_else(|| format!("download_{}", unix_timestamp()));

                *path = download_dir.join(&filename);
                let _ = download_started_proxy.send_event(UserEvent::DownloadStarted {
                    url,
                    filename,
                });
                true
            })
            .with_download_completed_handler(move |url, path, success| {
                let _ = download_completed_proxy.send_event(UserEvent::DownloadCompleted {
                    url,
                    path,
                    success,
                });
            });

        builder = if tab.is_start_page() {
            builder.with_html(start_page::html(start_page_payload)?)
        } else {
            builder.with_url(tab.display_url())
        };

        let webview = build_child_webview(builder, window, platform_container)
            .with_context(|| format!("failed to build webview for tab {tab_id}"))?;

        self.webviews.insert(tab_id, webview);

        Ok(())
    }

    fn webview(&self, tab_id: u64) -> Option<&WebView> {
        self.webviews.get(&tab_id)
    }

    fn create_settings_tab(
        &mut self,
        window: &Window,
        platform_container: &PlatformContainer,
        proxy: &EventLoopProxy<UserEvent>,
        tab: &TabSession,
        settings_payload: &serde_json::Value,
        visible: bool,
    ) -> Result<()> {
        let tab_id = tab.id;
        let page_load_proxy = proxy.clone();
        let title_proxy = proxy.clone();
        let ipc_proxy = proxy.clone();
        let download_started_proxy = proxy.clone();
        let download_completed_proxy = proxy.clone();
        let download_dir = self.download_dir.clone();

        let builder = WebViewBuilder::new_with_web_context(&mut self.web_context)
            .with_bounds(self.content_bounds)
            .with_visible(visible)
            .with_initialization_script(KEYBOARD_SHORTCUTS_SCRIPT)
            .with_on_page_load_handler(move |event, url| {
                let is_loading = matches!(event, PageLoadEvent::Started);
                let _ = page_load_proxy.send_event(UserEvent::PageLoad { tab_id, url, is_loading });
            })
            .with_document_title_changed_handler(move |title| {
                let _ = title_proxy.send_event(UserEvent::TitleChanged { tab_id, title });
            })
            .with_ipc_handler(move |request| {
                if let Some(command) = ToolbarCommand::parse(request.body()) {
                    let _ = ipc_proxy.send_event(UserEvent::TabCommand { tab_id, command });
                }
            })
            .with_download_started_handler(move |url, path| {
                let filename = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .filter(|n| !n.is_empty())
                    .unwrap_or_else(|| format!("download_{}", unix_timestamp()));
                *path = download_dir.join(&filename);
                let _ = download_started_proxy.send_event(UserEvent::DownloadStarted { url, filename });
                true
            })
            .with_download_completed_handler(move |url, path, success| {
                let _ = download_completed_proxy.send_event(UserEvent::DownloadCompleted { url, path, success });
            })
            .with_accept_first_mouse(true)
            .with_html(settings_page::html(settings_payload)?);

        let webview = build_child_webview(builder, window, platform_container)
            .with_context(|| format!("failed to build settings webview for tab {tab_id}"))?;

        self.webviews.insert(tab_id, webview);
        Ok(())
    }

    fn create_history_tab(
        &mut self,
        window: &Window,
        platform_container: &PlatformContainer,
        proxy: &EventLoopProxy<UserEvent>,
        tab: &TabSession,
        history_payload: &serde_json::Value,
        visible: bool,
    ) -> Result<()> {
        let tab_id = tab.id;
        let page_load_proxy = proxy.clone();
        let title_proxy = proxy.clone();
        let ipc_proxy = proxy.clone();
        let builder = WebViewBuilder::new_with_web_context(&mut self.web_context)
            .with_bounds(self.content_bounds)
            .with_visible(visible)
            .with_initialization_script(KEYBOARD_SHORTCUTS_SCRIPT)
            .with_on_page_load_handler(move |event, url| {
                let is_loading = matches!(event, PageLoadEvent::Started);
                let _ = page_load_proxy.send_event(UserEvent::PageLoad { tab_id, url, is_loading });
            })
            .with_document_title_changed_handler(move |title| {
                let _ = title_proxy.send_event(UserEvent::TitleChanged { tab_id, title });
            })
            .with_ipc_handler(move |request| {
                if let Some(command) = ToolbarCommand::parse(request.body()) {
                    let _ = ipc_proxy.send_event(UserEvent::TabCommand { tab_id, command });
                }
            })
            .with_accept_first_mouse(true)
            .with_html(history_page::html(history_payload)?);

        let webview = build_child_webview(builder, window, platform_container)
            .with_context(|| format!("failed to build history webview for tab {tab_id}"))?;

        self.webviews.insert(tab_id, webview);
        Ok(())
    }

    fn remove_tab(&mut self, tab_id: u64) {
        self.webviews.remove(&tab_id);
    }

    fn show_only(&self, active_tab_id: u64) {
        for (tab_id, webview) in &self.webviews {
            if let Err(error) = webview.set_visible(*tab_id == active_tab_id) {
                warn!(%error, tab_id, "failed to update tab visibility");
            }
        }

        if let Some(active_webview) = self.webviews.get(&active_tab_id) {
            if let Err(error) = active_webview.focus() {
                warn!(%error, active_tab_id, "failed to focus active tab");
            }
        }
    }

    fn resize_all(&mut self, bounds: Rect) {
        self.content_bounds = bounds;

        for (tab_id, webview) in &self.webviews {
            if let Err(error) = webview.set_bounds(bounds) {
                warn!(%error, tab_id, "failed to resize tab webview");
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ViewportLayout {
    toolbar_height: u32,
}

impl Default for ViewportLayout {
    fn default() -> Self {
        Self {
            toolbar_height: TOOLBAR_HEIGHT,
        }
    }
}

impl ViewportLayout {
    fn toolbar_bounds(&self, size: LogicalSize<u32>) -> Rect {
        Rect {
            position: LogicalPosition::new(0, 0).into(),
            size: LogicalSize::new(size.width.max(1), self.toolbar_height).into(),
        }
    }

    fn content_bounds(&self, size: LogicalSize<u32>) -> Rect {
        let content_height = size.height.saturating_sub(self.toolbar_height).max(1);

        Rect {
            position: LogicalPosition::new(0, self.toolbar_height).into(),
            size: LogicalSize::new(size.width.max(1), content_height).into(),
        }
    }
}

impl BrowserApp {
    fn bootstrap() -> Result<Self> {
        let (data_dir, config_dir) = resolve_app_dirs()?;

        info!(
            data_dir = %data_dir.display(),
            config_dir = %config_dir.display(),
            "bootstrapped browser shell"
        );

        Ok(Self {
            data_dir,
            config_dir,
        })
    }

    fn run(self) -> Result<()> {
        let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
        let proxy = event_loop.create_proxy();
        let window = WindowBuilder::new()
            .with_title("Tartanos")
            .with_inner_size(LogicalSize::new(1280.0, 840.0))
            .with_min_inner_size(LogicalSize::new(1024.0, 700.0))
            .build(&event_loop)
            .context("failed to create the main browser window")?;

        let platform_container = create_platform_container(&window)?;
        let layout = ViewportLayout::default();
        let viewport_size = logical_window_size(&window);
        let download_dir = resolve_download_dir(&self.data_dir);
        let mut browser_state =
            BrowserState::new(BrowserStore::new(self.data_dir.clone()), download_dir.clone());
        apply_window_theme(&window, &browser_state.preferences.theme);
        let mut browser_views =
            BrowserViews::new(&self.data_dir, layout.content_bounds(viewport_size), download_dir)?;

        info!(
            data_dir = %self.data_dir.display(),
            config_dir = %self.config_dir.display(),
            "created native browser window"
        );

        let chrome = build_toolbar_webview(
            &window,
            &platform_container,
            proxy.clone(),
            layout.toolbar_bounds(viewport_size),
            browser_state.toolbar_payload(),
        )?;

        for tab in browser_state.tabs.clone() {
            materialize_tab(
                &window,
                &platform_container,
                &proxy,
                &mut browser_views,
                &browser_state,
                &tab,
                tab.id == browser_state.active_tab_id(),
            )?;
        }
        browser_views.show_only(browser_state.active_tab_id());

        window.set_title(&browser_state.window_title());
        sync_toolbar(&chrome, browser_state.toolbar_payload());
        sync_start_pages(&browser_views, &browser_state);
        sync_settings_pages(&browser_views, &browser_state);
        sync_history_pages(&browser_views, &browser_state);
        browser_state.persist_session();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::UserEvent(user_event) => {
                    match user_event {
                        UserEvent::Toolbar(command) => {
                            handle_browser_command(
                                command,
                                None,
                                &window,
                                &chrome,
                                &platform_container,
                                &proxy,
                                &mut browser_views,
                                &mut browser_state,
                            );
                        }
                        UserEvent::TabCommand { tab_id, command } => {
                            handle_browser_command(
                                command,
                                Some(tab_id),
                                &window,
                                &chrome,
                                &platform_container,
                                &proxy,
                                &mut browser_views,
                                &mut browser_state,
                            );
                        }
                        UserEvent::Navigation { tab_id, url } => {
                            browser_state.set_loading_for(tab_id, url);
                        }
                        UserEvent::PageLoad {
                            tab_id,
                            url,
                            is_loading,
                        } => {
                            if is_loading {
                                browser_state.set_loading_for(tab_id, url);
                                browser_state.set_tab_audible(tab_id, false);
                            } else {
                                browser_state.set_ready_for(tab_id, url);
                                browser_state.record_history_for(tab_id);
                                if let Some(tab) = browser_state.tab(tab_id) {
                                    if tab.muted {
                                        if let Some(webview) = browser_views.webview(tab_id) {
                                            apply_tab_mute_state(webview, true);
                                        }
                                    }
                                }
                            }
                        }
                        UserEvent::TitleChanged { tab_id, title } => {
                            browser_state.update_title_for(tab_id, title);
                        }
                        UserEvent::OpenInTab { tab_id, url } => {
                            navigate_tab(&browser_views, &mut browser_state, tab_id, &url);
                        }
                        UserEvent::DownloadStarted { url, filename } => {
                            browser_state.add_download(url, filename);
                        }
                        UserEvent::DownloadCompleted { url, path, success } => {
                            browser_state.complete_download(&url, path, success);
                        }
                    }

                    browser_state.persist_session();
                    window.set_title(&browser_state.window_title());
                    sync_toolbar(&chrome, browser_state.toolbar_payload());
                    sync_start_pages(&browser_views, &browser_state);
                    sync_settings_pages(&browser_views, &browser_state);
                    sync_history_pages(&browser_views, &browser_state);
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    let logical_size = size.to_logical::<u32>(window.scale_factor());

                    if let Err(error) = chrome.set_bounds(layout.toolbar_bounds(logical_size)) {
                        warn!(%error, "failed to resize toolbar webview");
                    }

                    browser_views.resize_all(layout.content_bounds(logical_size));
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    browser_state.persist_session();
                    info!("close requested, shutting down");
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
        });
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}

fn resolve_app_dirs() -> Result<(PathBuf, PathBuf)> {
    if let Some(project_dirs) = ProjectDirs::from("dev", "tartanos", "Tartanos") {
        let data_dir = project_dirs.data_local_dir().to_path_buf();
        let config_dir = project_dirs.config_dir().to_path_buf();

        match ensure_app_dirs(&data_dir, &config_dir) {
            Ok(()) => return Ok((data_dir, config_dir)),
            Err(error) => warn!(
                %error,
                data_dir = %data_dir.display(),
                config_dir = %config_dir.display(),
                "falling back to workspace-local app directories"
            ),
        }
    } else {
        warn!(
            "could not resolve platform-specific app directories, using workspace-local fallback"
        );
    }

    let fallback_root = env::current_dir()
        .context("could not resolve the current working directory for app data fallback")?
        .join(".tartanos");
    let data_dir = fallback_root.join("data");
    let config_dir = fallback_root.join("config");

    ensure_app_dirs(&data_dir, &config_dir)?;

    Ok((data_dir, config_dir))
}

fn ensure_app_dirs(data_dir: &PathBuf, config_dir: &PathBuf) -> Result<()> {
    fs::create_dir_all(data_dir)
        .with_context(|| format!("could not create data dir at {}", data_dir.display()))?;
    fs::create_dir_all(config_dir)
        .with_context(|| format!("could not create config dir at {}", config_dir.display()))?;
    Ok(())
}

fn handle_browser_command(
    command: ToolbarCommand,
    source_tab_id: Option<u64>,
    window: &Window,
    chrome: &WebView,
    platform_container: &PlatformContainer,
    proxy: &EventLoopProxy<UserEvent>,
    browser_views: &mut BrowserViews,
    browser_state: &mut BrowserState,
) {
    let navigation_target_tab = source_tab_id.unwrap_or_else(|| browser_state.active_tab_id());

    match command {
        ToolbarCommand::Navigate(input) => {
            navigate_tab(browser_views, browser_state, navigation_target_tab, &input);
        }
        ToolbarCommand::Back => {
            if let Some(active_webview) = browser_views.webview(browser_state.active_tab_id()) {
                if let Err(error) = active_webview.evaluate_script("window.history.back();") {
                    warn!(%error, "failed to navigate backward");
                    browser_state.set_error("Back navigation failed");
                }
            }
        }
        ToolbarCommand::Forward => {
            if let Some(active_webview) = browser_views.webview(browser_state.active_tab_id()) {
                if let Err(error) = active_webview.evaluate_script("window.history.forward();") {
                    warn!(%error, "failed to navigate forward");
                    browser_state.set_error("Forward navigation failed");
                }
            }
        }
        ToolbarCommand::Reload => {
            if let Some(active_webview) = browser_views.webview(browser_state.active_tab_id()) {
                browser_state.set_loading_for(
                    browser_state.active_tab_id(),
                    browser_state.current_url().to_string(),
                );

                if let Err(error) = active_webview.reload() {
                    warn!(%error, "failed to reload page");
                    browser_state.set_error("Reload failed");
                }
            }
        }
        ToolbarCommand::NewTab => {
            let tab = browser_state.new_tab();
            let start_page_payload = browser_state.start_page_payload();

            match browser_views.create_tab(
                window,
                platform_container,
                proxy,
                &tab,
                &start_page_payload,
                true,
            ) {
                Ok(()) => {
                    browser_views.show_only(tab.id);
                    focus_address_bar(chrome);
                }
                Err(error) => {
                    warn!(%error, tab_id = tab.id, "failed to create new tab webview");
                    browser_state.discard_tab(tab.id);
                    browser_state.set_error("New tab failed");
                }
            }
        }
        ToolbarCommand::ActivateTab(tab_id) => {
            if browser_state.activate_tab(tab_id) {
                browser_views.show_only(tab_id);
            }
        }
        ToolbarCommand::ReorderTab { tab_id, target_id } => {
            if let Some(active_tab_id) = browser_state.reorder_tab(tab_id, target_id) {
                browser_views.show_only(active_tab_id);
            }
        }
        ToolbarCommand::TogglePinTab(tab_id) => {
            let tab_id = tab_id.unwrap_or(navigation_target_tab);
            if let Some(active_tab_id) = browser_state.toggle_pin_tab(tab_id) {
                browser_views.show_only(active_tab_id);
            }
        }
        ToolbarCommand::ToggleMuteTab(tab_id) => {
            let tab_id = tab_id.unwrap_or(navigation_target_tab);
            if let Some(muted) = browser_state.toggle_mute_tab(tab_id) {
                if let Some(webview) = browser_views.webview(tab_id) {
                    apply_tab_mute_state(webview, muted);
                }
            }
        }
        ToolbarCommand::TabAudibleState(audible) => {
            browser_state.set_tab_audible(navigation_target_tab, audible);
        }
        ToolbarCommand::DuplicateTab(tab_id) => {
            let tab_id = tab_id.unwrap_or(navigation_target_tab);
            if let Some(tab) = browser_state.duplicate_tab(tab_id) {
                match materialize_tab(
                    window,
                    platform_container,
                    proxy,
                    browser_views,
                    browser_state,
                    &tab,
                    true,
                ) {
                    Ok(()) => browser_views.show_only(tab.id),
                    Err(error) => {
                        warn!(%error, tab_id = tab.id, "failed to duplicate tab webview");
                        browser_state.discard_tab(tab.id);
                        browser_state.set_error("Duplicate tab failed");
                    }
                }
            }
        }
        ToolbarCommand::ReopenClosedTab => {
            if let Some(tab) = browser_state.reopen_closed_tab() {
                match materialize_tab(
                    window,
                    platform_container,
                    proxy,
                    browser_views,
                    browser_state,
                    &tab,
                    true,
                ) {
                    Ok(()) => browser_views.show_only(tab.id),
                    Err(error) => {
                        warn!(%error, tab_id = tab.id, "failed to reopen closed tab webview");
                        browser_state.discard_tab(tab.id);
                        browser_state.set_error("Reopen closed tab failed");
                    }
                }
            }
        }
        ToolbarCommand::SelectPrevTab => {
            if let Some(tab_id) = browser_state.activate_adjacent_tab(navigation_target_tab, -1) {
                browser_views.show_only(tab_id);
            }
        }
        ToolbarCommand::SelectNextTab => {
            if let Some(tab_id) = browser_state.activate_adjacent_tab(navigation_target_tab, 1) {
                browser_views.show_only(tab_id);
            }
        }
        ToolbarCommand::CloseCurrentTab => {
            let result = browser_state.close_tab(navigation_target_tab);

            if let Some(removed_tab_id) = result.removed_tab_id {
                browser_views.remove_tab(removed_tab_id);
            }

            if result.recreate_active_tab {
                if let Some(active_tab) = browser_state.tab(result.active_tab_id).cloned() {
                    let start_page_payload = browser_state.start_page_payload();
                    match browser_views.create_tab(
                        window,
                        platform_container,
                        proxy,
                        &active_tab,
                        &start_page_payload,
                        true,
                    ) {
                        Ok(()) => {
                            browser_views.show_only(result.active_tab_id);
                            focus_address_bar(chrome);
                        }
                        Err(error) => {
                            warn!(
                                %error,
                                tab_id = result.active_tab_id,
                                "failed to recreate reset active tab"
                            );
                            browser_state.set_error("Could not reset the last tab");
                        }
                    }
                }
            } else {
                browser_views.show_only(result.active_tab_id);
            }
        }
        ToolbarCommand::CloseTab(tab_id) => {
            let result = browser_state.close_tab(tab_id);

            if let Some(removed_tab_id) = result.removed_tab_id {
                browser_views.remove_tab(removed_tab_id);
            }

            if result.recreate_active_tab {
                if let Some(active_tab) = browser_state.tab(result.active_tab_id).cloned() {
                    let start_page_payload = browser_state.start_page_payload();
                    match browser_views.create_tab(
                        window,
                        platform_container,
                        proxy,
                        &active_tab,
                        &start_page_payload,
                        true,
                    ) {
                        Ok(()) => browser_views.show_only(result.active_tab_id),
                        Err(error) => {
                            warn!(
                                %error,
                                tab_id = result.active_tab_id,
                                "failed to recreate reset active tab"
                            );
                            browser_state.set_error("Could not reset the last tab");
                        }
                    }
                }
            } else {
                browser_views.show_only(result.active_tab_id);
            }
        }
        ToolbarCommand::ToggleBookmark => browser_state.toggle_bookmark(),
        ToolbarCommand::OpenBookmark(url) | ToolbarCommand::OpenHistory(url) => {
            navigate_tab(browser_views, browser_state, navigation_target_tab, &url);
        }
        ToolbarCommand::OpenHistoryPage => {
            let tab = browser_state.new_history_tab();
            let history_payload = browser_state.history_page_payload();
            match browser_views.create_history_tab(
                window,
                platform_container,
                proxy,
                &tab,
                &history_payload,
                true,
            ) {
                Ok(()) => browser_views.show_only(tab.id),
                Err(error) => {
                    warn!(%error, tab_id = tab.id, "failed to create history tab");
                    browser_state.discard_tab(tab.id);
                    browser_state.set_error("Could not open history");
                }
            }
        }
        ToolbarCommand::DeleteHistory(id) => browser_state.delete_history_entry(id),
        ToolbarCommand::ClearHistory => browser_state.clear_history(),
        ToolbarCommand::CopyAddressSelection(text) => {
            if let Err(error) = write_native_clipboard(&text) {
                warn!(%error, "failed to copy address selection");
            }
        }
        ToolbarCommand::CutAddressSelection(text) => {
            if let Err(error) = write_native_clipboard(&text) {
                warn!(%error, "failed to cut address selection");
            }
        }
        ToolbarCommand::PasteIntoAddress => match read_native_clipboard() {
            Ok(text) => {
                if let Ok(encoded) = serde_json::to_string(&text) {
                    if let Err(error) = chrome.evaluate_script(
                        &format!("window.__pasteAddressFromNative && window.__pasteAddressFromNative({encoded});"),
                    ) {
                        warn!(%error, "failed to paste clipboard text into address bar");
                    }
                }
            }
            Err(error) => warn!(%error, "failed to read native clipboard"),
        },
        ToolbarCommand::OpenSettings => {
            let tab = browser_state.new_settings_tab();
            let settings_payload = browser_state.settings_payload();
            match browser_views.create_settings_tab(window, platform_container, proxy, &tab, &settings_payload, true) {
                Ok(()) => browser_views.show_only(tab.id),
                Err(error) => {
                    warn!(%error, tab_id = tab.id, "failed to create settings tab");
                    browser_state.discard_tab(tab.id);
                    browser_state.set_error("Could not open settings");
                }
            }
        }
        ToolbarCommand::FocusAddress => {
            focus_address_bar(chrome);
        }
        ToolbarCommand::SettingsUpdate { key, value } => {
            browser_state.update_preferences(&key, &value);
            if key == "theme" {
                apply_window_theme(window, &value);
            }
        }
        ToolbarCommand::SetHeight(height) => {
            let viewport = logical_window_size(window);
            let h = height.max(TOOLBAR_HEIGHT).min(viewport.height.saturating_sub(100));
            // content first — eliminate overlap before toolbar grows
            let content_rect = Rect {
                position: LogicalPosition::new(0, h).into(),
                size: LogicalSize::new(viewport.width.max(1), viewport.height.saturating_sub(h).max(1)).into(),
            };
            browser_views.resize_all(content_rect);
            // then expand toolbar
            let toolbar_rect = Rect {
                position: LogicalPosition::new(0, 0).into(),
                size: LogicalSize::new(viewport.width.max(1), h).into(),
            };
            if let Err(e) = chrome.set_bounds(toolbar_rect) {
                warn!(%e, "failed to resize toolbar");
            }
        }
    }
}

fn navigate_tab(
    browser_views: &BrowserViews,
    browser_state: &mut BrowserState,
    tab_id: u64,
    raw_input: &str,
) {
    let search_base = browser_state.preferences.search_base_url();
    let Some(target_url) = normalize_input(raw_input, search_base) else {
        browser_state.set_error("Type a URL or search query");
        return;
    };

    browser_state.set_loading_for(tab_id, target_url.clone());

    let Some(webview) = browser_views.webview(tab_id) else {
        browser_state.set_error("Tab view is missing");
        return;
    };

    if let Err(error) = webview.load_url(&target_url) {
        warn!(%error, %target_url, tab_id, "failed to load requested URL");
        browser_state.set_error("Could not open that address");
        return;
    }

    if browser_state.active_tab_id() == tab_id {
        if let Err(error) = webview.focus() {
            warn!(%error, tab_id, "failed to focus content webview after navigation");
        }
    }
}

fn normalize_input(raw_input: &str, search_base_url: &str) -> Option<String> {
    let input = raw_input.trim();
    if input.is_empty() {
        return None;
    }

    if let Some(keyword_url) = keyword_search_url(input) {
        return Some(keyword_url);
    }

    if let Ok(url) = Url::parse(input) {
        return Some(url.to_string());
    }

    let compact = compact_pasted_url(input);
    if compact != input {
        if let Ok(url) = Url::parse(&compact) {
            return Some(url.to_string());
        }
    }

    let looks_like_host = !compact.contains(' ')
        && (input.contains('.')
            || compact.contains('/')
            || compact.eq_ignore_ascii_case("localhost")
            || compact.starts_with("localhost:")
            || compact.parse::<IpAddr>().is_ok());

    if looks_like_host {
        let candidate = format!("https://{compact}");
        if Url::parse(&candidate).is_ok() {
            return Some(candidate);
        }
    }

    let query = byte_serialize(input.as_bytes()).collect::<String>();
    Some(format!("{search_base_url}{query}"))
}

fn compact_pasted_url(input: &str) -> String {
    input.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn keyword_search_url(input: &str) -> Option<String> {
    let mut parts = input.split_whitespace();
    let keyword = parts.next()?.to_ascii_lowercase();
    let query = parts.collect::<Vec<_>>().join(" ");
    if query.trim().is_empty() {
        return None;
    }

    let base = match keyword.as_str() {
        "g" | "google" => "https://www.google.com/search?q=",
        "b" | "bing" => "https://www.bing.com/search?q=",
        "d" | "ddg" | "duck" => "https://duckduckgo.com/?q=",
        "yt" => "https://www.youtube.com/results?search_query=",
        "gh" => "https://github.com/search?q=",
        "w" => "https://en.wikipedia.org/w/index.php?search=",
        _ => return None,
    };

    let encoded = byte_serialize(query.trim().as_bytes()).collect::<String>();
    Some(format!("{base}{encoded}"))
}

fn write_native_clipboard(text: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let mut child = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .context("failed to spawn pbcopy")?;
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin
                .write_all(text.as_bytes())
                .context("failed to write to pbcopy stdin")?;
        }
        let status = child.wait().context("failed to wait for pbcopy")?;
        if status.success() {
            return Ok(());
        }
        anyhow::bail!("pbcopy exited with status {status}");
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = text;
        anyhow::bail!("native clipboard write is not implemented on this platform")
    }
}

fn read_native_clipboard() -> Result<String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("pbpaste")
            .output()
            .context("failed to run pbpaste")?;
        if output.status.success() {
            return String::from_utf8(output.stdout).context("clipboard was not valid UTF-8");
        }
        anyhow::bail!("pbpaste exited with status {}", output.status);
    }

    #[cfg(not(target_os = "macos"))]
    {
        anyhow::bail!("native clipboard read is not implemented on this platform")
    }
}

fn build_toolbar_webview(
    window: &Window,
    platform_container: &PlatformContainer,
    proxy: EventLoopProxy<UserEvent>,
    bounds: Rect,
    initial_state: serde_json::Value,
) -> Result<WebView> {
    let html = toolbar::html(&initial_state)?;
    let builder = WebViewBuilder::new()
        .with_bounds(bounds)
        .with_html(html)
        .with_ipc_handler(move |request| {
            if let Some(command) = ToolbarCommand::parse(request.body()) {
                let _ = proxy.send_event(UserEvent::Toolbar(command));
            }
        })
        .with_accept_first_mouse(true);

    build_child_webview(builder, window, platform_container)
        .context("failed to build the browser toolbar webview")
}

fn sync_toolbar(chrome: &WebView, state: serde_json::Value) {
    match toolbar::sync_script(&state) {
        Ok(script) => {
            if let Err(error) = chrome.evaluate_script(&script) {
                warn!(%error, "failed to sync toolbar state");
            }
        }
        Err(error) => warn!(%error, "failed to build toolbar sync script"),
    }
}

fn focus_address_bar(chrome: &WebView) {
    if let Err(error) = chrome.focus() {
        warn!(%error, "failed to focus toolbar webview");
    }

    if let Err(error) = chrome.evaluate_script(
        "window.__focusAddress && window.__focusAddress();",
    ) {
        warn!(%error, "failed to focus address bar");
    }
}

fn tab_internal_icon(tab: &TabSession) -> &'static str {
    match tab.content {
        TabContent::StartPage => "➕",
        TabContent::Settings => "⚙",
        TabContent::History => "🕘",
        TabContent::Page { .. } => "",
    }
}

fn tab_favicon_url(tab: &TabSession) -> String {
    match &tab.content {
        TabContent::Page { url } => Url::parse(url)
            .ok()
            .and_then(|parsed| parsed.host_str().map(|_| {
                let scheme = parsed.scheme();
                let host = parsed.host_str().unwrap_or_default();
                let mut favicon = format!("{scheme}://{host}");
                if let Some(port) = parsed.port() {
                    favicon.push(':');
                    favicon.push_str(&port.to_string());
                }
                favicon.push_str("/favicon.ico");
                favicon
            }))
            .unwrap_or_default(),
        _ => String::new(),
    }
}

fn apply_tab_mute_state(webview: &WebView, muted: bool) {
    let script = if muted {
        r#"
        (function() {
          window.__tartanosMuted = true;
          const apply = function() {
            document.querySelectorAll('audio,video').forEach(function(node) {
              node.muted = true;
              node.defaultMuted = true;
              if (typeof node.volume === 'number') node.volume = 0;
            });
          };
          apply();
          if (!window.__tartanosMuteObserver) {
            window.__tartanosMuteObserver = new MutationObserver(apply);
            window.__tartanosMuteObserver.observe(document.documentElement || document.body, { childList: true, subtree: true });
          }
        })();
        "#
    } else {
        r#"
        (function() {
          window.__tartanosMuted = false;
          document.querySelectorAll('audio,video').forEach(function(node) {
            node.muted = false;
            node.defaultMuted = false;
          });
          if (window.__tartanosMuteObserver) {
            window.__tartanosMuteObserver.disconnect();
            window.__tartanosMuteObserver = null;
          }
        })();
        "#
    };

    if let Err(error) = webview.evaluate_script(script) {
        warn!(%error, muted, "failed to apply mute state to tab");
    }
}

fn materialize_tab(
    window: &Window,
    platform_container: &PlatformContainer,
    proxy: &EventLoopProxy<UserEvent>,
    browser_views: &mut BrowserViews,
    browser_state: &BrowserState,
    tab: &TabSession,
    visible: bool,
) -> Result<()> {
    if tab.is_settings_page() {
        let settings_payload = browser_state.settings_payload();
        browser_views.create_settings_tab(
            window,
            platform_container,
            proxy,
            tab,
            &settings_payload,
            visible,
        )
    } else if tab.is_history_page() {
        let history_payload = browser_state.history_page_payload();
        browser_views.create_history_tab(
            window,
            platform_container,
            proxy,
            tab,
            &history_payload,
            visible,
        )
    } else {
        let start_page_payload = browser_state.start_page_payload();
        browser_views.create_tab(
            window,
            platform_container,
            proxy,
            tab,
            &start_page_payload,
            visible,
        )
    }
}

fn sync_start_pages(browser_views: &BrowserViews, browser_state: &BrowserState) {
    let payload = browser_state.start_page_payload();

    match start_page::sync_script(&payload) {
        Ok(script) => {
            for tab in browser_state.tabs.iter().filter(|tab| tab.is_start_page()) {
                if let Some(webview) = browser_views.webview(tab.id) {
                    if let Err(error) = webview.evaluate_script(&script) {
                        warn!(%error, tab_id = tab.id, "failed to sync start page state");
                    }
                }
            }
        }
        Err(error) => warn!(%error, "failed to build start page sync script"),
    }
}

fn sync_settings_pages(browser_views: &BrowserViews, browser_state: &BrowserState) {
    let payload = browser_state.settings_payload();

    match settings_page::sync_script(&payload) {
        Ok(script) => {
            for tab in browser_state.tabs.iter().filter(|tab| tab.is_settings_page()) {
                if let Some(webview) = browser_views.webview(tab.id) {
                    if let Err(error) = webview.evaluate_script(&script) {
                        warn!(%error, tab_id = tab.id, "failed to sync settings page state");
                    }
                }
            }
        }
        Err(error) => warn!(%error, "failed to build settings page sync script"),
    }
}

fn sync_history_pages(browser_views: &BrowserViews, browser_state: &BrowserState) {
    let payload = browser_state.history_page_payload();

    match history_page::sync_script(&payload) {
        Ok(script) => {
            for tab in browser_state.tabs.iter().filter(|tab| tab.is_history_page()) {
                if let Some(webview) = browser_views.webview(tab.id) {
                    if let Err(error) = webview.evaluate_script(&script) {
                        warn!(%error, tab_id = tab.id, "failed to sync history page state");
                    }
                }
            }
        }
        Err(error) => warn!(%error, "failed to build history page sync script"),
    }
}

fn logical_window_size(window: &Window) -> LogicalSize<u32> {
    window.inner_size().to_logical::<u32>(window.scale_factor())
}

fn load_json_collection<T>(path: &Path, label: &str) -> Vec<T>
where
    T: for<'de> Deserialize<'de>,
{
    match fs::read_to_string(path) {
        Ok(contents) => match serde_json::from_str::<Vec<T>>(&contents) {
            Ok(entries) => entries,
            Err(error) => {
                warn!(%error, path = %path.display(), "failed to parse {label} store");
                Vec::new()
            }
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => {
            warn!(%error, path = %path.display(), "failed to read {label} store");
            Vec::new()
        }
    }
}

fn load_json_value<T>(path: &Path, label: &str) -> Option<T>
where
    T: for<'de> Deserialize<'de>,
{
    match fs::read_to_string(path) {
        Ok(contents) => match serde_json::from_str::<T>(&contents) {
            Ok(value) => Some(value),
            Err(error) => {
                warn!(%error, path = %path.display(), "failed to parse {label} store");
                None
            }
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            warn!(%error, path = %path.display(), "failed to read {label} store");
            None
        }
    }
}

fn save_json_collection<T>(path: &Path, values: &[T], label: &str) -> Result<()>
where
    T: Serialize,
{
    let json = serde_json::to_string_pretty(values)
        .with_context(|| format!("could not serialize {label} store"))?;
    fs::write(path, json)
        .with_context(|| format!("could not write {label} store to {}", path.display()))
}

fn save_json_value<T>(path: &Path, value: &T, label: &str) -> Result<()>
where
    T: Serialize,
{
    let json = serde_json::to_string_pretty(value)
        .with_context(|| format!("could not serialize {label} store"))?;
    fs::write(path, json)
        .with_context(|| format!("could not write {label} store to {}", path.display()))
}

fn next_saved_id(bookmarks: &[BookmarkItem], history: &[HistoryEntry]) -> u64 {
    let bookmark_max = bookmarks.iter().map(|item| item.id).max().unwrap_or(0);
    let history_max = history.iter().map(|item| item.id).max().unwrap_or(0);
    bookmark_max.max(history_max) + 1
}

fn restore_tabs(session: Option<&PersistedSession>) -> (Vec<TabSession>, u64, u64, String) {
    if let Some(session) = session {
        let tabs: Vec<_> = session
            .tabs
            .iter()
            .cloned()
            .map(TabSession::restored)
            .collect();

        if !tabs.is_empty() {
            let active_tab_id = if tabs.iter().any(|tab| tab.id == session.active_tab_id) {
                session.active_tab_id
            } else {
                tabs[0].id
            };
            let next_tab_id = tabs.iter().map(|tab| tab.id).max().unwrap_or(0) + 1;
            let label = if tabs.len() == 1 { "tab" } else { "tabs" };

            return (
                tabs,
                active_tab_id,
                next_tab_id,
                format!("Restored {} {label}", session.tabs.len()),
            );
        }
    }

    (
        vec![TabSession::new_start_page(1)],
        1,
        2,
        "Ready".to_string(),
    )
}

fn apply_window_theme(window: &Window, theme: &str) {
    let tao_theme = match theme {
        "dark" => Some(Theme::Dark),
        "light" | "warm" => Some(Theme::Light),
        _ => None, // system — ikut OS
    };
    window.set_theme(tao_theme);
}

fn resolve_download_dir(data_dir: &Path) -> PathBuf {
    directories::UserDirs::new()
        .and_then(|u| u.download_dir().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| data_dir.join("downloads"))
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn is_runtime_start_page_url(url: &str) -> bool {
    url.is_empty() || url == "about:blank" || url.starts_with("data:text/html")
}

fn title_from_url(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(str::to_string))
        .unwrap_or_else(|| START_PAGE_TITLE.to_string())
}

fn tab_title(title: &str, url: &str) -> String {
    let title = title.trim();
    if title.is_empty() {
        title_from_url(url)
    } else {
        title.to_string()
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
))]
type PlatformContainer = gtk::Fixed;

#[cfg(not(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
)))]
type PlatformContainer = ();

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
))]
fn create_platform_container(window: &Window) -> Result<PlatformContainer> {
    use gtk::prelude::*;
    use tao::platform::unix::WindowExtUnix;

    let fixed = gtk::Fixed::new();
    let vbox = window
        .default_vbox()
        .context("failed to access the linux window container")?;

    vbox.pack_start(&fixed, true, true, 0);
    fixed.show_all();

    Ok(fixed)
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
)))]
fn create_platform_container(_: &Window) -> Result<PlatformContainer> {
    Ok(())
}

fn build_child_webview(
    builder: WebViewBuilder<'_>,
    window: &Window,
    platform_container: &PlatformContainer,
) -> Result<WebView> {
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
    ))]
    {
        use wry::WebViewBuilderExtUnix;

        return builder
            .build_gtk(platform_container)
            .context("failed to build child webview on linux");
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
    )))]
    {
        let _ = platform_container;
        builder
            .build_as_child(window)
            .context("failed to build child webview")
    }
}
