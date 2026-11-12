// Finder — Advanced System Overlay Launcher (Rust + Tauri v2)
(function() {
  'use strict';

  // Tauri v2 API wrappers
  const invoke = window.__TAURI__ ? window.__TAURI__.core.invoke : async (cmd, args) => {
    console.log(`[Mock Invoke] ${cmd}`, args);
    return [];
  };

  const listen = window.__TAURI__ ? window.__TAURI__.event.listen : async (event, handler) => {
    console.log(`[Mock Listen] ${event}`);
  };

  // State Management
  let currentResults = [];
  let selectedIndex = 0;
  let debounceTimer = null;
  let searchEpoch = 0;
  let activeCategoryFilter = 'all';
  let activeCmdCategory = 'all';
  let currentConfig = {};
  let notesList = [];
  let editingNoteId = null;
  let customBindsMap = {};
  let isRecordingHotkey = false;
  let speechRecognizer = null;
  let currentAccentColor = '#6366f1';

  // DOM Elements
  const searchInput = document.getElementById('search-input');
  const searchClearBtn = document.getElementById('search-clear-btn');
  const searchModeTag = document.getElementById('search-mode-tag');
  const resultsList = document.getElementById('results-list');
  const resultsScroll = document.getElementById('results-scroll');
  const emptyState = document.getElementById('empty-state');
  const headerClock = document.getElementById('header-clock');
  const headerWeather = document.getElementById('header-weather');
  const weatherText = document.getElementById('weather-text');
  const toastNotify = document.getElementById('toast-notify');
  const toastMessage = document.getElementById('toast-message');

  // Header Buttons
  const btnVoice = document.getElementById('btn-voice');
  const btnNotes = document.getElementById('btn-notes');
  const btnBinds = document.getElementById('btn-binds');
  const btnSettings = document.getElementById('btn-settings');

  // Category Pills
  const catPills = document.querySelectorAll('.cat-pill');
  const cmdTabs = document.querySelectorAll('.cmd-tab');
  const commandsList = document.getElementById('commands-list');

  // Hardware Monitor Elements
  const metricCpuText = document.getElementById('metric-cpu-text');
  const barCpu = document.getElementById('bar-cpu');
  const metricCpuSub = document.getElementById('metric-cpu-sub');

  const metricRamText = document.getElementById('metric-ram-text');
  const barRam = document.getElementById('bar-ram');
  const metricRamSub = document.getElementById('metric-ram-sub');

  const metricGpuText = document.getElementById('metric-gpu-text');
  const barGpu = document.getElementById('bar-gpu');
  const metricGpuSub = document.getElementById('metric-gpu-sub');

  const sysmonBattery = document.getElementById('sysmon-battery');
  const hardwareDetail = document.getElementById('hardware-detail');

  // Notes Modal Elements
  const notesModal = document.getElementById('notes-modal');
  const btnCloseNotes = document.getElementById('btn-close-notes');
  const notesCountBadge = document.getElementById('notes-count-badge');
  const notesSearchInput = document.getElementById('notes-search-input');
  const btnToggleAddNote = document.getElementById('btn-toggle-add-note');
  const notesEditorCard = document.getElementById('notes-editor-card');
  const noteEditorText = document.getElementById('note-editor-text');
  const btnSaveNote = document.getElementById('btn-save-note');
  const btnCancelNote = document.getElementById('btn-cancel-note');
  const notesItemsList = document.getElementById('notes-items-list');
  const btnExportNotes = document.getElementById('btn-export-notes');
  const btnDelAllNotes = document.getElementById('btn-del-all-notes');

  // Settings Modal Elements
  const settingsModal = document.getElementById('settings-modal');
  const btnCloseSettings = document.getElementById('btn-close-settings');
  const settingHotkey = document.getElementById('setting-hotkey');
  const btnRecordHotkey = document.getElementById('btn-record-hotkey');
  const settingTheme = document.getElementById('setting-theme');
  const settingScale = document.getElementById('setting-scale');
  const scaleValueLabel = document.getElementById('scale-value-label');
  const settingBottomOffset = document.getElementById('setting-bottom-offset');
  const settingAutostart = document.getElementById('setting-autostart');
  const settingHideBlur = document.getElementById('setting-hide-blur');
  const settingOllamaModel = document.getElementById('setting-ollama-model');
  const colorDots = document.querySelectorAll('.color-dot');
  const customAccentPicker = document.getElementById('custom-accent-picker');
  const customAccentHex = document.getElementById('custom-accent-hex');
  const btnSaveSettings = document.getElementById('btn-save-settings');
  const btnExitApp = document.getElementById('btn-exit-app');

  // Binds Modal Elements
  const bindsModal = document.getElementById('binds-modal');
  const btnCloseBinds = document.getElementById('btn-close-binds');
  const bindInputCombo = document.getElementById('bind-input-combo');
  const bindInputAction = document.getElementById('bind-input-action');
  const btnAddBind = document.getElementById('btn-add-bind');
  const bindsItemsList = document.getElementById('binds-items-list');

  // Weather Modal Elements
  const weatherModal = document.getElementById('weather-modal');
  const btnCloseWeather = document.getElementById('btn-close-weather');
  const weatherCityInput = document.getElementById('weather-city-input');
  const btnFetchWeather = document.getElementById('btn-fetch-weather');
  const btnQuickCities = document.querySelectorAll('.btn-quick-city');
  const weatherCardCity = document.getElementById('weather-card-city');
  const weatherCardTemp = document.getElementById('weather-card-temp');
  const weatherCardDesc = document.getElementById('weather-card-desc');
  const wxWind = document.getElementById('wx-wind');
  const wxHumidity = document.getElementById('wx-humidity');
  const wxCode = document.getElementById('wx-code');

  // SVG Icons Map
  const ICONS = {
    app: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"></rect><rect x="14" y="3" width="7" height="7"></rect><rect x="14" y="14" width="7" height="7"></rect><rect x="3" y="14" width="7" height="7"></rect></svg>`,
    file: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"></path><polyline points="13 2 13 9 20 9"></polyline></svg>`,
    window: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect><line x1="8" y1="21" x2="16" y2="21"></line><line x1="12" y1="17" x2="12" y2="21"></line></svg>`,
    calc: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="4" y="2" width="16" height="20" rx="2"></rect><line x1="8" y1="6" x2="16" y2="6"></line><line x1="16" y1="14" x2="16" y2="18"></line><path d="M16 10h.01M12 10h.01M8 10h.01M12 14h.01M8 14h.01M12 18h.01M8 18h.01"></path></svg>`,
    ai: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z"></path></svg>`,
    cmd: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="4 17 10 11 4 5"></polyline><line x1="12" y1="19" x2="20" y2="19"></line></svg>`,
    note: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"></path><rect x="8" y="2" width="8" height="4" rx="1" ry="1"></rect><line x1="9" y1="11" x2="15" y2="11"></line><line x1="9" y1="15" x2="13" y2="15"></line></svg>`,
    web: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>`,
    clip: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"></path><rect x="8" y="2" width="8" height="4" rx="1" ry="1"></rect></svg>`,
    folder: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"></path></svg>`,
    time: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><polyline points="12 6 12 12 16 14"></polyline></svg>`
  };

  // Complete List of Quick Commands with direct actions & URLs
  const ALL_COMMANDS = [
    // System
    { name: "Lock PC", key: "lock", cat: "System", sub: "Lock screen", type: "cmd", path: "cmd://lock" },
    { name: "Sleep", key: "sleep", cat: "System", sub: "Suspend state", type: "cmd", path: "cmd://sleep" },
    { name: "Reboot (10s)", key: "reboot", cat: "System", sub: "Restart with timer", type: "cmd", path: "cmd://reboot" },
    { name: "Shutdown (10s)", key: "shutdown", cat: "System", sub: "Power off with timer", type: "cmd", path: "cmd://shutdown" },
    { name: "Cancel Shutdown", key: "cancel", cat: "System", sub: "Abort shutdown", type: "cmd", path: "cmd://cancel_shutdown" },
    { name: "Task Manager", key: "taskmgr", cat: "System", sub: "taskmgr.exe", type: "cmd", path: "taskmgr.exe" },
    { name: "Control Panel", key: "panel", cat: "System", sub: "control.exe", type: "cmd", path: "control.exe" },
    { name: "Windows Settings", key: "settings", cat: "System", sub: "ms-settings:", type: "cmd", path: "ms-settings:" },
    { name: "Snipping Tool", key: "snip", cat: "System", sub: "ms-screenclip:", type: "cmd", path: "ms-screenclip:" },
    { name: "PowerShell", key: "ps", cat: "System", sub: "powershell.exe", type: "cmd", path: "powershell.exe" },
    { name: "Terminal", key: "wt", cat: "System", sub: "Windows Terminal", type: "cmd", path: "wt.exe" },
    { name: "Registry Editor", key: "regedit", cat: "System", sub: "regedit.exe", type: "cmd", path: "regedit.exe" },
    { name: "Services", key: "services", cat: "System", sub: "services.msc", type: "cmd", path: "services.msc" },
    { name: "Disk Management", key: "disk", cat: "System", sub: "diskmgmt.msc", type: "cmd", path: "diskmgmt.msc" },
    { name: "Device Manager", key: "devmgmt", cat: "System", sub: "devmgmt.msc", type: "cmd", path: "devmgmt.msc" },
    { name: "Event Viewer", key: "eventvwr", cat: "System", sub: "eventvwr.msc", type: "cmd", path: "eventvwr.msc" },
    { name: "System Info", key: "msinfo", cat: "System", sub: "msinfo32.exe", type: "cmd", path: "msinfo32.exe" },
    { name: "Paint", key: "paint", cat: "System", sub: "mspaint.exe", type: "cmd", path: "mspaint.exe" },

    // Folders
    { name: "Home Profile", key: "home", cat: "Folders", sub: "User directory", type: "cmd", path: "cmd://home" },
    { name: "Downloads", key: "downloads", cat: "Folders", sub: "~/Downloads", type: "cmd", path: "cmd://downloads" },
    { name: "Documents", key: "docs", cat: "Folders", sub: "~/Documents", type: "cmd", path: "cmd://docs" },
    { name: "Pictures", key: "pics", cat: "Folders", sub: "~/Pictures", type: "cmd", path: "cmd://pics" },
    { name: "Videos", key: "videos", cat: "Folders", sub: "~/Videos", type: "cmd", path: "cmd://videos" },
    { name: "Desktop", key: "desktop", cat: "Folders", sub: "~/Desktop", type: "cmd", path: "cmd://desktop" },

    // Web
    { name: "My Public IP", key: "ip", cat: "Web", sub: "2ip.ru / ipify", type: "web", path: "https://2ip.ru" },
    { name: "Google Search", key: "g", cat: "Web", sub: "google.com", type: "web", path: "https://www.google.com" },
    { name: "YouTube", key: "yt", cat: "Web", sub: "youtube.com", type: "web", path: "https://www.youtube.com" },
    { name: "Wikipedia", key: "wiki", cat: "Web", sub: "wikipedia.org", type: "web", path: "https://www.wikipedia.org" },
    { name: "Gmail", key: "gmail", cat: "Web", sub: "mail.google.com", type: "web", path: "https://mail.google.com" },
    { name: "Telegram Web", key: "tg", cat: "Web", sub: "web.telegram.org", type: "web", path: "https://web.telegram.org" },
    { name: "GitHub", key: "gh", cat: "Web", sub: "github.com", type: "web", path: "https://github.com" },
    { name: "Steam", key: "steam", cat: "Web", sub: "steam://open/main", type: "web", path: "steam://open/main" },
    { name: "Spotify", key: "spotify", cat: "Web", sub: "spotify:", type: "web", path: "spotify:" },

    // AI
    { name: "ChatGPT", key: "chatgpt", cat: "AI", sub: "chatgpt.com", type: "web", path: "https://chatgpt.com" },
    { name: "Gemini", key: "gemini", cat: "AI", sub: "gemini.google.com", type: "web", path: "https://gemini.google.com" },
    { name: "Claude AI", key: "claude", cat: "AI", sub: "claude.ai", type: "web", path: "https://claude.ai" },
    { name: "DALL-E 3", key: "dalle", cat: "AI", sub: "chatgpt.com/?model=gpt-4o", type: "web", path: "https://chatgpt.com/?model=gpt-4o" },

    // Utils
    { name: "Clipboard History", key: "clip", cat: "Utils", sub: ">clip", type: "input", query: ">clip" },
    { name: "Password 16", key: "pw", cat: "Utils", sub: "16 char random", type: "action", action: "generate_password_16" },
    { name: "Password 24", key: "pw24", cat: "Utils", sub: "24 char random", type: "action", action: "generate_password_24" },
    { name: "Password 32", key: "pw32", cat: "Utils", sub: "32 char random", type: "action", action: "generate_password_32" },
    { name: "Random 1-100", key: "rand", cat: "Utils", sub: "random number", type: "action", action: "random_1_100" },
    { name: "Dice Roll (1-6)", key: "dice", cat: "Utils", sub: "d6 roll", type: "action", action: "dice_roll" },
    { name: "Coin Flip", key: "coin", cat: "Utils", sub: "Heads or Tails", type: "action", action: "coin_flip" },
    { name: "My Key Binds", key: "binds", cat: "Utils", sub: "Open Key Binds", type: "modal", modal: "binds" },
    { name: "Quick Notes", key: "notes", cat: "Utils", sub: "Open Notes", type: "modal", modal: "notes" },
    { name: "Preferences", key: "settings", cat: "Utils", sub: "Open Preferences", type: "modal", modal: "settings" },
    { name: "Usage Statistics", key: "stats", cat: "Utils", sub: ">stats", type: "input", query: ">stats" },

    // Time & Weather
    { name: "Weather Forecast", key: "weather", cat: "Time", sub: "Check any city", type: "modal", modal: "weather" },
    { name: "Timer 5m", key: "timer 5m", cat: "Time", sub: "5 minutes", type: "input", query: ">timer 5m" },
    { name: "Timer 15m", key: "timer 15m", cat: "Time", sub: "15 minutes", type: "input", query: ">timer 15m" },
    { name: "Timer 25m (Pomodoro)", key: "timer 25m", cat: "Time", sub: "Pomodoro", type: "input", query: ">timer 25m" },
    { name: "Timer 1h", key: "timer 1h", cat: "Time", sub: "1 hour", type: "input", query: ">timer 1h" },
    { name: "Remind 19:00", key: "remind 19:00", cat: "Time", sub: "Evening reminder", type: "input", query: ">remind 19:00" },
  ];

  // Initialize Clock
  function startClock() {
    function tick() {
      const now = new Date();
      const hh = String(now.getHours()).padStart(2, '0');
      const mm = String(now.getMinutes()).padStart(2, '0');
      const ss = String(now.getSeconds()).padStart(2, '0');
      if (headerClock) headerClock.textContent = `${hh}:${mm}:${ss}`;
    }
    tick();
    setInterval(tick, 1000);
  }

  // Toast Helper
  function showToast(msg, duration = 2200) {
    if (!toastNotify || !toastMessage) return;
    toastMessage.textContent = msg;
    toastNotify.classList.remove('hidden');
    clearTimeout(toastNotify._timer);
    toastNotify._timer = setTimeout(() => {
      toastNotify.classList.add('hidden');
    }, duration);
  }

  // Render Commands to Right Panel
  function renderCommandsList() {
    if (!commandsList) return;
    commandsList.innerHTML = '';

    const filtered = ALL_COMMANDS.filter(cmd => {
      if (activeCmdCategory === 'all') return true;
      return cmd.cat.toLowerCase() === activeCmdCategory.toLowerCase();
    });

    filtered.forEach(cmd => {
      const btn = document.createElement('div');
      btn.className = 'cmd-card-btn';
      btn.innerHTML = `
        <span class="cmd-name">${escapeHtml(cmd.name)}</span>
        <span class="cmd-sub">${escapeHtml(cmd.sub)}</span>
      `;
      btn.addEventListener('click', () => {
        executeQuickCommand(cmd);
      });
      commandsList.appendChild(btn);
    });
  }

  // Direct Execution of Quick Commands
  async function executeQuickCommand(cmd) {
    if (!cmd) return;

    if (cmd.type === 'modal') {
      if (cmd.modal === 'weather') openWeatherModal();
      else if (cmd.modal === 'notes') openNotesModal();
      else if (cmd.modal === 'binds') openBindsModal();
      else if (cmd.modal === 'settings') openSettingsModal();
      return;
    }

    if (cmd.type === 'web' || cmd.type === 'ai') {
      try {
        await invoke('execute_action', {
          item: {
            id: `cmd_${cmd.key}`,
            title: cmd.name,
            category: cmd.cat,
            item_type: 'web',
            path: cmd.path,
            score: 1000
          }
        });
        showToast(`Opened ${cmd.name}`);
      } catch (err) {
        console.error('Failed to open web action:', err);
        showToast(`Error: ${err}`);
      }
      return;
    }

    if (cmd.type === 'cmd') {
      try {
        await invoke('execute_action', {
          item: {
            id: `cmd_${cmd.key}`,
            title: cmd.name,
            category: cmd.cat,
            item_type: 'cmd',
            path: cmd.path,
            score: 1000
          }
        });
        showToast(`Executed: ${cmd.name}`);
      } catch (err) {
        console.error('Failed to execute command:', err);
        showToast(`Error: ${err}`);
      }
      return;
    }

    if (cmd.type === 'action') {
      handleUtilityAction(cmd.action);
      return;
    }

    if (cmd.type === 'input') {
      searchInput.value = cmd.query;
      performSearch(cmd.query);
      searchInput.focus();
      return;
    }

    // Fallback
    searchInput.value = `>${cmd.key}`;
    performSearch(searchInput.value);
    searchInput.focus();
  }

  // Utility actions (Password, Random, Dice, Coin)
  async function handleUtilityAction(action) {
    if (action.startsWith('generate_password_')) {
      const len = parseInt(action.replace('generate_password_', ''), 10) || 16;
      const chars = 'abcdefghjkmnpqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789!@#$%^&*()_+-=';
      let pw = '';
      const array = new Uint32Array(len);
      window.crypto.getRandomValues(array);
      for (let i = 0; i < len; i++) {
        pw += chars[array[i] % chars.length];
      }
      try {
        await navigator.clipboard.writeText(pw);
        showToast(`Generated & copied ${len}-char password!`);
      } catch (_) {}
    } else if (action === 'random_1_100') {
      const num = Math.floor(Math.random() * 100) + 1;
      try {
        await navigator.clipboard.writeText(String(num));
        showToast(`🎲 Random 1-100: ${num} (Copied!)`);
      } catch (_) {}
    } else if (action === 'dice_roll') {
      const roll = Math.floor(Math.random() * 6) + 1;
      try {
        await navigator.clipboard.writeText(String(roll));
        showToast(`🎲 Dice roll: ${roll} (Copied!)`);
      } catch (_) {}
    } else if (action === 'coin_flip') {
      const res = Math.random() < 0.5 ? 'Heads' : 'Tails';
      try {
        await navigator.clipboard.writeText(res);
        showToast(`🪙 Coin flip: ${res} (Copied!)`);
      } catch (_) {}
    }
  }

  // Category filter pills click
  catPills.forEach(pill => {
    pill.addEventListener('click', () => {
      catPills.forEach(p => p.classList.remove('active'));
      pill.classList.add('active');
      activeCategoryFilter = pill.dataset.filter || 'all';
      renderResults();
    });
  });

  // Command category tabs click
  cmdTabs.forEach(tab => {
    tab.addEventListener('click', () => {
      cmdTabs.forEach(t => t.classList.remove('active'));
      tab.classList.add('active');
      activeCmdCategory = tab.dataset.cmdCat || 'all';
      renderCommandsList();
    });
  });

  // Search Mode Tag Helper
  function updateSearchModeTag(query) {
    const trimmed = query.trim();
    if (trimmed.startsWith('/ai ') || trimmed.startsWith('?')) {
      searchModeTag.textContent = 'AI';
      searchModeTag.style.background = 'var(--metric-warning)';
    } else if (trimmed.startsWith('!note')) {
      searchModeTag.textContent = 'Note';
      searchModeTag.style.background = 'var(--metric-success)';
    } else if (trimmed.startsWith('>')) {
      searchModeTag.textContent = 'Command';
      searchModeTag.style.background = 'var(--accent)';
    } else if (trimmed.startsWith('=')) {
      searchModeTag.textContent = 'Calc';
      searchModeTag.style.background = 'var(--metric-ram)';
    } else if (trimmed.startsWith('#')) {
      searchModeTag.textContent = 'Files';
      searchModeTag.style.background = 'var(--metric-cpu)';
    } else if (trimmed.startsWith('@')) {
      searchModeTag.textContent = 'Apps';
      searchModeTag.style.background = 'var(--accent)';
    } else {
      searchModeTag.textContent = 'All';
      searchModeTag.style.background = 'var(--accent)';
    }
  }

  // Perform Search Query
  async function performSearch(query) {
    const trimmed = query.trim();
    searchEpoch++;
    const thisEpoch = searchEpoch;

    updateSearchModeTag(trimmed);
    searchClearBtn.classList.toggle('hidden', query.length === 0);

    // Fast handle !notes command directly
    if (trimmed === '!notes') {
      openNotesModal();
      return;
    }

    try {
      const results = await invoke('search_query', { query: trimmed, epoch: thisEpoch });
      if (thisEpoch !== searchEpoch) return; // Stale query cancelled

      currentResults = results || [];
      selectedIndex = 0;
      renderResults();
    } catch (err) {
      console.error('Search failed:', err);
    }
  }

  // Render Search Results
  function renderResults() {
    resultsList.innerHTML = '';

    // Filter results by category if pill selected
    let filtered = currentResults;
    if (activeCategoryFilter !== 'all') {
      filtered = currentResults.filter(item => {
        if (activeCategoryFilter === 'app') return item.item_type === 'app' || item.item_type === 'window';
        if (activeCategoryFilter === 'file') return item.item_type === 'file';
        if (activeCategoryFilter === 'content') return item.item_type === 'content' || item.category === 'CONTENT';
        if (activeCategoryFilter === 'web') return item.item_type === 'web' || item.item_type === 'ai' || item.item_type === 'bookmark';
        if (activeCategoryFilter === 'clip') return item.item_type === 'clip' || item.category === 'CLIPBOARD';
        return true;
      });
    }

    if (filtered.length === 0) {
      emptyState.classList.remove('hidden');
      return;
    }
    emptyState.classList.add('hidden');

    let currentCategory = null;
    let renderedCount = 0;

    filtered.forEach((item, index) => {
      // Add section divider when category changes
      if (item.category && item.category !== currentCategory) {
        currentCategory = item.category;
        const sectionHeader = document.createElement('div');
        sectionHeader.className = 'section-header';
        sectionHeader.textContent = currentCategory;
        resultsList.appendChild(sectionHeader);
      }

      // 1. Math / Instant Calculation Card
      if (item.card_type === 'math') {
        const card = document.createElement('div');
        card.className = `instant-card math ${index === selectedIndex ? 'selected' : ''}`;
        card.style.setProperty('--item-idx', renderedCount++);
        card.innerHTML = `
          <div class="calc-formula">${escapeHtml(item.subtitle || '')}</div>
          <div class="calc-value">${escapeHtml(item.title)}</div>
          <div class="card-copy-hint">↵ Copy result to clipboard</div>
        `;
        card.addEventListener('click', () => executeItem(index));
        resultsList.appendChild(card);
        return;
      }

      // 2. Standard or Specialized Result Row
      const el = document.createElement('div');
      el.className = `result-item ${index === selectedIndex ? 'selected' : ''}`;
      el.dataset.index = index;
      el.style.setProperty('--item-idx', renderedCount++);

      const iconType = item.item_type || 'app';
      const iconSvg = ICONS[iconType] || ICONS.app;
      const keyHint = index < 9 ? `Ctrl+${index + 1}` : '';

      el.innerHTML = `
        <div class="item-main-row">
          <div class="item-icon-wrap ${iconType}">
            ${iconSvg}
          </div>
          <div class="item-info">
            <div class="item-title-row">
              <span class="item-title" title="${escapeHtml(item.title)}">${escapeHtml(item.title)}</span>
              ${item.admin_path ? `<span class="item-badge-admin">Admin</span>` : ''}
            </div>
            ${item.subtitle ? `<div class="item-subtitle" title="${escapeHtml(item.subtitle)}">${escapeHtml(item.subtitle)}</div>` : ''}
          </div>
          <div class="item-actions">
            ${item.item_type === 'file' && item.path ? `
              <button class="btn-file-action btn-path" title="Toggle Full Path">Path</button>
              <button class="btn-file-action btn-folder" title="Open containing folder">Folder</button>
            ` : ''}
            ${item.item_type === 'content' && item.path ? `
              <button class="btn-file-action btn-folder" title="Open containing folder">Folder</button>
            ` : ''}
            ${keyHint ? `<span class="key-hint">${keyHint}</span>` : ''}
          </div>
        </div>
        ${item.item_type === 'file' && item.path ? `<div class="item-expanded-path">${escapeHtml(item.path)}</div>` : ''}
        ${item.snippet ? `
          <div class="item-snippet-box">
            <span class="snippet-line-badge">snippet</span>${escapeHtml(item.snippet)}
          </div>
        ` : ''}
      `;

      // Event handlers
      el.addEventListener('click', (e) => {
        if (e.target.closest('.btn-file-action')) return;
        executeItem(index, e.shiftKey);
      });

      // Path expander button
      const btnPath = el.querySelector('.btn-path');
      if (btnPath) {
        btnPath.addEventListener('click', (e) => {
          e.stopPropagation();
          const pathBox = el.querySelector('.item-expanded-path');
          if (pathBox) {
            const isVis = pathBox.classList.toggle('visible');
            btnPath.textContent = isVis ? 'Hide' : 'Path';
          }
        });
      }

      // Open folder button (reveal_file)
      const btnFolder = el.querySelector('.btn-folder');
      if (btnFolder && item.path) {
        btnFolder.addEventListener('click', async (e) => {
          e.stopPropagation();
          try {
            await invoke('reveal_file', { path: item.path });
            showToast('Opened containing folder');
          } catch (err) {
            console.error('reveal_file error:', err);
          }
        });
      }

      resultsList.appendChild(el);
    });

    scrollToSelected();
  }

  function scrollToSelected() {
    const selectedEl = resultsList.querySelector(`.result-item[data-index="${selectedIndex}"], .instant-card.selected`);
    if (selectedEl) {
      selectedEl.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    }
  }

  function updateSelection(newIndex) {
    if (currentResults.length === 0) return;
    if (newIndex < 0) newIndex = 0;
    if (newIndex >= currentResults.length) newIndex = currentResults.length - 1;

    selectedIndex = newIndex;

    const items = resultsList.querySelectorAll('.result-item, .instant-card');
    items.forEach(el => {
      const idx = parseInt(el.dataset.index, 10);
      el.classList.toggle('selected', idx === selectedIndex);
    });

    scrollToSelected();
  }

  // Execute Search Item
  async function executeItem(index, asAdmin = false) {
    const item = currentResults[index];
    if (!item) return;

    // Handle math copy
    if (item.card_type === 'math') {
      try {
        await navigator.clipboard.writeText(item.title);
        showToast(`Copied ${item.title} to clipboard!`);
      } catch (_) {}
      return;
    }

    // Handle local note additions (!note text)
    if (item.id === 'note_add' && item.path) {
      const text = item.path.replace('note://add/', '');
      try {
        await invoke('add_note', { text });
        showToast('Note saved successfully!');
        searchInput.value = '';
        performSearch('');
      } catch (e) {
        showToast(`Error saving note: ${e}`);
      }
      return;
    }

    try {
      await invoke('execute_action', { item, asAdmin, as_admin: asAdmin });
      searchInput.value = '';
      performSearch('');
    } catch (err) {
      console.error('execute_action error:', err);
      showToast(`Execution error: ${err}`);
    }
  }

  // Copy Path or Result to Clipboard
  async function copySelectedItem() {
    const item = currentResults[selectedIndex];
    if (!item) return;
    const toCopy = item.path || item.title;
    if (!toCopy) return;

    try {
      await navigator.clipboard.writeText(toCopy);
      showToast(`Copied to clipboard: ${toCopy}`);
    } catch (err) {
      console.error('Clipboard copy failed:', err);
    }
  }

  // =========================================================================
  // Keyboard Navigation & Shortcuts
  // =========================================================================

  window.addEventListener('keydown', (e) => {
    // If user is actively recording hotkey in Settings, don't intercept!
    if (isRecordingHotkey) return;

    // 1. Check if any modal is currently visible
    const isModalOpen = (notesModal && !notesModal.classList.contains('hidden')) ||
                        (settingsModal && !settingsModal.classList.contains('hidden')) ||
                        (bindsModal && !bindsModal.classList.contains('hidden')) ||
                        (weatherModal && !weatherModal.classList.contains('hidden'));

    if (e.key === 'Escape') {
      e.preventDefault();
      if (isModalOpen) {
        closeAllModals();
        searchInput.focus();
        return;
      }
      if (searchInput.value.length > 0) {
        searchInput.value = '';
        performSearch('');
        return;
      }
      invoke('hide_launcher');
      return;
    }

    // 2. Open Settings (Ctrl+,)
    if ((e.ctrlKey || e.metaKey) && e.key === ',') {
      e.preventDefault();
      if (settingsModal.classList.contains('hidden')) {
        openSettingsModal();
      } else {
        closeAllModals();
      }
      return;
    }

    // Don't intercept list navigation if inside a modal input
    if (isModalOpen) return;

    // 3. Arrow Down / Up
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      updateSelection(selectedIndex + 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      updateSelection(selectedIndex - 1);
    }
    // 4. Enter / Shift+Enter
    else if (e.key === 'Enter') {
      e.preventDefault();
      executeItem(selectedIndex, e.shiftKey);
    }
    // 5. Tab / Ctrl+C to copy
    else if (e.key === 'Tab' || ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'c' && document.activeElement !== searchInput)) {
      e.preventDefault();
      copySelectedItem();
    }
    // 6. Ctrl + 1..9 quick select
    else if ((e.ctrlKey || e.metaKey) && e.key >= '1' && e.key <= '9') {
      e.preventDefault();
      const targetIndex = parseInt(e.key, 10) - 1;
      if (targetIndex < currentResults.length) {
        executeItem(targetIndex, e.shiftKey);
      }
    }
  });

  // Search input typing listener
  searchInput.addEventListener('input', (e) => {
    clearTimeout(debounceTimer);
    const query = e.target.value;
    const delay = (query.length < 2 || query.startsWith('>') || query.startsWith('=')) ? 0 : 120;
    debounceTimer = setTimeout(() => {
      performSearch(query);
    }, delay);
  });

  // Clear button click
  searchClearBtn.addEventListener('click', () => {
    searchInput.value = '';
    performSearch('');
    searchInput.focus();
  });

  // Close modals helper
  function closeAllModals() {
    if (notesModal) notesModal.classList.add('hidden');
    if (settingsModal) settingsModal.classList.add('hidden');
    if (bindsModal) bindsModal.classList.add('hidden');
    if (weatherModal) weatherModal.classList.add('hidden');
    searchInput.focus();
  }

  // =========================================================================
  // Live Weather Forecast Modal
  // =========================================================================

  const WMO_CODES = {
    0: "☀️ Clear sky",
    1: "🌤️ Mainly clear",
    2: "⛅ Partly cloudy",
    3: "☁️ Overcast",
    45: "🌫️ Fog",
    48: "🌫️ Depositing rime fog",
    51: "🌦️ Light drizzle",
    53: "🌦️ Moderate drizzle",
    55: "🌧️ Dense drizzle",
    61: "🌧️ Slight rain",
    63: "🌧️ Moderate rain",
    65: "🌧️ Heavy rain",
    71: "🌨️ Slight snow fall",
    73: "🌨️ Moderate snow fall",
    75: "❄️ Heavy snow fall",
    77: "❄️ Snow grains",
    80: "🌦️ Slight rain showers",
    81: "🌧️ Moderate rain showers",
    82: "⛈️ Violent rain showers",
    85: "🌨️ Slight snow showers",
    86: "❄️ Heavy snow showers",
    95: "⛈️ Thunderstorm",
    96: "⛈️ Thunderstorm with hail",
    99: "⛈️ Violent thunderstorm"
  };

  function openWeatherModal() {
    closeAllModals();
    if (!weatherModal) return;
    weatherModal.classList.remove('hidden');
    if (weatherCityInput) {
      weatherCityInput.value = '';
      weatherCityInput.focus();
    }
  }

  async function fetchCityWeather(cityName) {
    const city = (cityName || '').trim();
    if (!city) return;

    if (weatherCardCity) weatherCardCity.textContent = `Searching "${city}"...`;
    if (weatherCardTemp) weatherCardTemp.textContent = '--°C';
    if (weatherCardDesc) weatherCardDesc.textContent = 'Contacting Open-Meteo satellite network...';

    try {
      // 1. Geocode city name to lat/lon
      const geoUrl = `https://geocoding-api.open-meteo.com/v1/search?name=${encodeURIComponent(city)}&count=1&language=en&format=json`;
      const geoResp = await fetch(geoUrl);
      if (!geoResp.ok) throw new Error('Geocoding request failed');
      const geoData = await geoResp.json();

      if (!geoData.results || geoData.results.length === 0) {
        if (weatherCardCity) weatherCardCity.textContent = `City "${city}" not found`;
        if (weatherCardDesc) weatherCardDesc.textContent = 'Please check the spelling or try another city name';
        return;
      }

      const match = geoData.results[0];
      const displayName = `${match.name}${match.country ? ', ' + match.country : ''}`;
      const lat = match.latitude;
      const lon = match.longitude;

      // 2. Query forecast
      const wxUrl = `https://api.open-meteo.com/v1/forecast?latitude=${lat}&longitude=${lon}&current=temperature_2m,relative_humidity_2m,weather_code,wind_speed_10m`;
      const wxResp = await fetch(wxUrl);
      if (!wxResp.ok) throw new Error('Forecast request failed');
      const wxData = await wxResp.json();

      const curr = wxData.current || {};
      const temp = curr.temperature_2m !== undefined ? Math.round(curr.temperature_2m) : '--';
      const wind = curr.wind_speed_10m !== undefined ? `${curr.wind_speed_10m} km/h` : '--';
      const humidity = curr.relative_humidity_2m !== undefined ? `${curr.relative_humidity_2m}%` : '--';
      const code = curr.weather_code !== undefined ? curr.weather_code : -1;
      const desc = WMO_CODES[code] || "⛅ Fair conditions";

      if (weatherCardCity) weatherCardCity.textContent = displayName;
      if (weatherCardTemp) weatherCardTemp.textContent = `${temp}°C`;
      if (weatherCardDesc) weatherCardDesc.textContent = desc;
      if (wxWind) wxWind.textContent = wind;
      if (wxHumidity) wxHumidity.textContent = humidity;
      if (wxCode) wxCode.textContent = desc;

      // Update header pill
      if (weatherText) {
        weatherText.textContent = `${match.name}: ${temp}°C`;
      }
    } catch (err) {
      console.error('fetchCityWeather error:', err);
      if (weatherCardCity) weatherCardCity.textContent = 'Weather lookup error';
      if (weatherCardDesc) weatherCardDesc.textContent = String(err.message || err);
    }
  }

  if (headerWeather) {
    headerWeather.addEventListener('click', openWeatherModal);
  }

  if (btnCloseWeather) {
    btnCloseWeather.addEventListener('click', closeAllModals);
  }

  if (btnFetchWeather) {
    btnFetchWeather.addEventListener('click', () => {
      fetchCityWeather(weatherCityInput.value);
    });
  }

  if (weatherCityInput) {
    weatherCityInput.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        fetchCityWeather(weatherCityInput.value);
      }
    });
  }

  btnQuickCities.forEach(btn => {
    btn.addEventListener('click', () => {
      const city = btn.dataset.city;
      if (weatherCityInput) weatherCityInput.value = city;
      fetchCityWeather(city);
    });
  });

  // Background Auto Weather Detection
  async function fetchLiveWeather() {
    try {
      const geoReq = await fetch('https://ipapi.co/json/');
      if (!geoReq.ok) return;
      const geo = await geoReq.json();
      const city = geo.city || 'Local';
      const lat = geo.latitude;
      const lon = geo.longitude;
      if (!lat || !lon) return;

      const wxReq = await fetch(`https://api.open-meteo.com/v1/forecast?latitude=${lat}&longitude=${lon}&current=temperature_2m,weather_code`);
      if (!wxReq.ok) return;
      const wx = await wxReq.json();
      const temp = wx.current?.temperature_2m;
      if (temp !== undefined && weatherText) {
        weatherText.textContent = `${city}: ${Math.round(temp)}°C`;
      }
    } catch (_) {
      if (weatherText) weatherText.textContent = 'Weather';
    }
  }

  // =========================================================================
  // Notes Modal Implementation
  // =========================================================================

  async function openNotesModal() {
    closeAllModals();
    notesModal.classList.remove('hidden');
    notesSearchInput.value = '';
    notesEditorCard.classList.add('hidden');
    editingNoteId = null;
    await refreshNotesList();
    notesSearchInput.focus();
  }

  async function refreshNotesList() {
    try {
      notesList = await invoke('get_notes');
      renderNotes();
    } catch (err) {
      console.error('get_notes failed:', err);
    }
  }

  function renderNotes() {
    notesItemsList.innerHTML = '';
    const q = (notesSearchInput.value || '').trim().toLowerCase();
    const filtered = notesList.filter(n => !q || (n.text && n.text.toLowerCase().includes(q)));

    notesCountBadge.textContent = String(filtered.length);

    if (filtered.length === 0) {
      notesItemsList.innerHTML = `<div style="text-align:center;padding:24px;color:var(--text-dim);font-size:12px;">No notes found</div>`;
      return;
    }

    filtered.forEach(note => {
      const card = document.createElement('div');
      card.className = 'note-card';
      const tsFormatted = note.timestamp ? note.timestamp.substring(0, 16).replace('T', ' ') : '';

      card.innerHTML = `
        <div class="note-text">${escapeHtml(note.text)}</div>
        <div class="note-footer">
          <span class="note-ts">${escapeHtml(tsFormatted)}</span>
          <div class="note-actions">
            <button class="note-btn btn-copy" title="Copy text">Copy</button>
            <button class="note-btn btn-edit" title="Edit note">Edit</button>
            <button class="note-btn del btn-del" title="Delete note">Delete</button>
          </div>
        </div>
      `;

      card.querySelector('.btn-copy').addEventListener('click', async () => {
        try {
          await navigator.clipboard.writeText(note.text);
          showToast('Note copied to clipboard');
        } catch (_) {}
      });

      card.querySelector('.btn-edit').addEventListener('click', () => {
        editingNoteId = note.id;
        noteEditorText.value = note.text;
        notesEditorCard.classList.remove('hidden');
        noteEditorText.focus();
      });

      card.querySelector('.btn-del').addEventListener('click', async () => {
        try {
          await invoke('delete_note', { id: note.id });
          await refreshNotesList();
          showToast('Note deleted');
        } catch (err) {
          showToast(`Error: ${err}`);
        }
      });

      notesItemsList.appendChild(card);
    });
  }

  btnNotes.addEventListener('click', openNotesModal);
  btnCloseNotes.addEventListener('click', closeAllModals);

  notesSearchInput.addEventListener('input', renderNotes);

  btnToggleAddNote.addEventListener('click', () => {
    editingNoteId = null;
    noteEditorText.value = '';
    notesEditorCard.classList.remove('hidden');
    noteEditorText.focus();
  });

  btnCancelNote.addEventListener('click', () => {
    notesEditorCard.classList.add('hidden');
    editingNoteId = null;
  });

  btnSaveNote.addEventListener('click', async () => {
    const text = noteEditorText.value.trim();
    if (!text) return;

    try {
      if (editingNoteId) {
        await invoke('edit_note', { id: editingNoteId, text });
        showToast('Note updated');
      } else {
        await invoke('add_note', { text });
        showToast('Note created');
      }
      notesEditorCard.classList.add('hidden');
      editingNoteId = null;
      await refreshNotesList();
    } catch (err) {
      showToast(`Error: ${err}`);
    }
  });

  btnExportNotes.addEventListener('click', async () => {
    try {
      const path = await invoke('export_notes');
      await navigator.clipboard.writeText(path);
      showToast(`Exported to Desktop! Copied path: ${path}`);
    } catch (err) {
      showToast(`Export failed: ${err}`);
    }
  });

  btnDelAllNotes.addEventListener('click', async () => {
    if (!confirm('Are you sure you want to delete ALL notes?')) return;
    try {
      await invoke('delete_all_notes');
      await refreshNotesList();
      showToast('All notes deleted');
    } catch (err) {
      showToast(`Error: ${err}`);
    }
  });

  // =========================================================================
  // Settings Modal Implementation (With Color Palette & Fix for Save Error)
  // =========================================================================

  function hexToRgba(hex, alpha = 0.35) {
    let c = hex.replace('#', '');
    if (c.length === 3) c = c.split('').map(x => x + x).join('');
    const num = parseInt(c, 16);
    const r = (num >> 16) & 255;
    const g = (num >> 8) & 255;
    const b = num & 255;
    return `rgba(${r}, ${g}, ${b}, ${alpha})`;
  }

  function adjustHexBrightness(hex, percent) {
    let c = hex.replace('#', '');
    if (c.length === 3) c = c.split('').map(x => x + x).join('');
    let num = parseInt(c, 16);
    let r = Math.min(255, Math.max(0, ((num >> 16) & 255) + Math.round(255 * (percent / 100))));
    let g = Math.min(255, Math.max(0, ((num >> 8) & 255) + Math.round(255 * (percent / 100))));
    let b = Math.min(255, Math.max(0, (num & 255) + Math.round(255 * (percent / 100))));
    return '#' + ((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1);
  }

  function applyAccentHex(hex) {
    if (!hex || !hex.startsWith('#')) return;
    currentAccentColor = hex;
    document.documentElement.style.setProperty('--accent', hex);
    document.documentElement.style.setProperty('--accent-hover', adjustHexBrightness(hex, 18));
    document.documentElement.style.setProperty('--accent-glow', hexToRgba(hex, 0.35));

    if (customAccentPicker) customAccentPicker.value = hex;
    if (customAccentHex) customAccentHex.value = hex;

    colorDots.forEach(dot => {
      dot.classList.toggle('active', dot.dataset.hex && dot.dataset.hex.toLowerCase() === hex.toLowerCase());
    });

    try {
      localStorage.setItem('finder_accent_hex', hex);
    } catch (_) {}
  }

  const NAMED_COLOR_MAP = {
    blue: '#3b82f6',
    green: '#10b981',
    cyan: '#06b6d4',
    purple: '#a855f7',
    amber: '#f59e0b',
    rose: '#f43f5e'
  };

  async function openSettingsModal() {
    closeAllModals();
    settingsModal.classList.remove('hidden');

    try {
      currentConfig = await invoke('get_config');
      settingHotkey.value = currentConfig.hotkey || 'RightAlt';
      settingTheme.value = currentConfig.theme || 'oled';
      settingBottomOffset.value = currentConfig.bottom_offset ?? 80;
      settingHideBlur.checked = currentConfig.hide_on_blur === true;
      settingOllamaModel.value = currentConfig.ollama_model || 'auto';
      settingScale.value = currentConfig.scale || 1.0;
      scaleValueLabel.textContent = `${Number(settingScale.value).toFixed(2)}x`;

      const autostartActive = await invoke('is_autostart').catch(() => false);
      settingAutostart.checked = autostartActive;

      // Restore Accent Color
      const acc = currentConfig.accent_color || '#6366f1';
      const hex = acc.startsWith('#') ? acc : (NAMED_COLOR_MAP[acc] || '#6366f1');
      applyAccentHex(hex);
    } catch (err) {
      console.error('get_config error:', err);
    }
  }

  btnSettings.addEventListener('click', openSettingsModal);
  btnCloseSettings.addEventListener('click', closeAllModals);

  // Live Scale Zoom Slider
  settingScale.addEventListener('input', (e) => {
    const val = parseFloat(e.target.value);
    scaleValueLabel.textContent = `${val.toFixed(2)}x`;
    applyScale(val);
  });

  // Live Theme Selector
  settingTheme.addEventListener('change', (e) => {
    applyTheme(e.target.value);
  });

  // Live Palette Dot Clicks
  colorDots.forEach(dot => {
    dot.addEventListener('click', () => {
      const hex = dot.dataset.hex;
      if (hex) {
        applyAccentHex(hex);
      }
    });
  });

  // Live Custom Color Picker Input
  if (customAccentPicker) {
    customAccentPicker.addEventListener('input', (e) => {
      applyAccentHex(e.target.value);
    });
  }

  // Live Custom Hex Text Input
  if (customAccentHex) {
    customAccentHex.addEventListener('input', (e) => {
      let val = e.target.value.trim();
      if (!val.startsWith('#')) val = '#' + val;
      if (/^#[0-9A-Fa-f]{6}$/.test(val)) {
        applyAccentHex(val);
      }
    });
  }

  // Hotkey Recorder
  btnRecordHotkey.addEventListener('click', () => {
    isRecordingHotkey = true;
    btnRecordHotkey.textContent = 'Press key...';
    btnRecordHotkey.style.color = 'var(--metric-warning)';
    settingHotkey.value = 'Listening...';

    const onKey = (e) => {
      e.preventDefault();
      e.stopPropagation();

      const parts = [];
      if (e.ctrlKey) parts.push('Ctrl');
      if (e.altKey) parts.push('Alt');
      if (e.shiftKey) parts.push('Shift');
      if (e.metaKey) parts.push('Super');

      let keyName = e.key;
      if (keyName === 'Control' || keyName === 'Alt' || keyName === 'Shift' || keyName === 'Meta') {
        return;
      }
      if (keyName === ' ') keyName = 'Space';
      if (keyName.length === 1) keyName = keyName.toUpperCase();

      parts.push(keyName);
      const combo = parts.join('+');

      settingHotkey.value = combo;
      btnRecordHotkey.textContent = 'Record Key';
      btnRecordHotkey.style.color = '';
      isRecordingHotkey = false;
      window.removeEventListener('keydown', onKey, true);
    };

    window.addEventListener('keydown', onKey, true);
  });

  // Save Settings (Strictly clean fields, NO duplicate aliases!)
  btnSaveSettings.addEventListener('click', async () => {
    btnSaveSettings.textContent = 'Saving...';
    btnSaveSettings.disabled = true;

    try {
      const chosenHex = currentAccentColor || '#6366f1';
      const cleanConfig = {
        hotkey: settingHotkey.value.trim() || 'RightAlt',
        bottom_offset: parseInt(settingBottomOffset.value, 10) || 80,
        theme: settingTheme.value || 'oled',
        accent_color: chosenHex,
        scale: parseFloat(settingScale.value) || 1.0,
        hide_on_blur: settingHideBlur.checked,
        ollama_model: settingOllamaModel.value.trim() || 'auto',
        autostart: settingAutostart.checked,
        file_search_dirs: currentConfig.file_search_dirs || [],
        file_exclude_patterns: currentConfig.file_exclude_patterns || [],
        snippets: currentConfig.snippets || {},
      };

      try {
        localStorage.setItem('finder_config', JSON.stringify(cleanConfig));
      } catch (_) {}

      await invoke('save_config', { newConfig: cleanConfig, new_config: cleanConfig });
      await invoke('set_autostart', { enabled: settingAutostart.checked });

      currentConfig = cleanConfig;
      applyTheme(cleanConfig.theme);
      applyAccentHex(cleanConfig.accent_color);
      applyScale(cleanConfig.scale);

      showToast('Settings saved successfully!');
      setTimeout(() => {
        btnSaveSettings.textContent = 'Save Settings';
        btnSaveSettings.disabled = false;
        closeAllModals();
      }, 400);
    } catch (err) {
      console.error('Failed to save settings:', err);
      showToast(`Save error: ${err}`);
      btnSaveSettings.textContent = 'Save Settings';
      btnSaveSettings.disabled = false;
    }
  });

  btnExitApp.addEventListener('click', async () => {
    if (confirm('Exit Finder?')) {
      await invoke('execute_action', {
        item: {
          id: "exit",
          title: "Exit Finder",
          category: "System",
          item_type: "cmd",
          path: "cmd://exit",
          score: 1000
        }
      }).catch(() => {});
    }
  });

  // =========================================================================
  // Custom Binds Modal Implementation
  // =========================================================================

  async function openBindsModal() {
    closeAllModals();
    bindsModal.classList.remove('hidden');
    bindInputCombo.value = '';
    bindInputAction.value = '';
    await refreshBindsList();
  }

  async function refreshBindsList() {
    try {
      customBindsMap = await invoke('get_binds');
      renderBinds();
    } catch (err) {
      console.error('get_binds error:', err);
    }
  }

  function renderBinds() {
    bindsItemsList.innerHTML = '';
    const entries = Object.entries(customBindsMap);

    if (entries.length === 0) {
      bindsItemsList.innerHTML = `<div style="text-align:center;padding:24px;color:var(--text-dim);font-size:12px;">No custom key binds defined</div>`;
      return;
    }

    entries.forEach(([combo, action]) => {
      const row = document.createElement('div');
      row.className = 'bind-row';
      row.innerHTML = `
        <span class="bind-combo">${escapeHtml(combo)}</span>
        <span class="bind-action" title="${escapeHtml(action)}">${escapeHtml(action)}</span>
        <button class="btn-ghost-sm btn-del-bind" style="color:var(--metric-danger);">Remove</button>
      `;

      row.querySelector('.btn-del-bind').addEventListener('click', async () => {
        try {
          customBindsMap = await invoke('remove_bind', { combo });
          renderBinds();
          showToast(`Removed bind: ${combo}`);
        } catch (err) {
          showToast(`Error: ${err}`);
        }
      });

      bindsItemsList.appendChild(row);
    });
  }

  btnBinds.addEventListener('click', openBindsModal);
  btnCloseBinds.addEventListener('click', closeAllModals);

  btnAddBind.addEventListener('click', async () => {
    const combo = bindInputCombo.value.trim();
    const action = bindInputAction.value.trim();
    if (!combo || !action) {
      showToast('Please specify both hotkey combo and action');
      return;
    }

    try {
      customBindsMap = await invoke('save_bind', { combo, action });
      bindInputCombo.value = '';
      bindInputAction.value = '';
      renderBinds();
      showToast(`Added bind: ${combo} -> ${action}`);
    } catch (err) {
      showToast(`Error adding bind: ${err}`);
    }
  });

  // =========================================================================
  // Voice Input (Windows Native Voice Typing Dictation)
  // =========================================================================

  btnVoice.addEventListener('click', async () => {
    searchInput.focus();
    try {
      await invoke('start_voice_typing');
      btnVoice.classList.add('listening');
      showToast('🎙️ Voice Typing (Win+H) activated — speak now!', 4500);
      setTimeout(() => {
        btnVoice.classList.remove('listening');
      }, 5000);
    } catch (err) {
      console.warn('Voice typing error:', err);
      showToast('Press Win+H to start Voice Typing');
    }
  });

  // =========================================================================
  // Theme, Accent & Scale Application Helpers
  // =========================================================================

  function applyTheme(theme) {
    const t = theme || 'oled';
    document.body.classList.remove('theme-oled', 'theme-monochrome', 'theme-glass', 'theme-dark', 'theme-light');
    document.body.classList.add(`theme-${t}`);
    
    if (t === 'monochrome') {
      applyAccentHex('#ffffff');
    }

    try {
      localStorage.setItem('finder_theme', t);
    } catch (_) {}
  }

  function applyScale(scale) {
    const s = scale || 1.0;
    document.documentElement.style.zoom = s;
  }

  function escapeHtml(str) {
    if (!str) return '';
    return String(str)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  }

  // =========================================================================
  // Tauri Event Listeners
  // =========================================================================

  listen('launcher-shown', () => {
    closeAllModals();
    searchInput.focus();
    searchInput.select();
    performSearch(searchInput.value);
  });

  listen('open-settings', () => {
    openSettingsModal();
  });

  listen('open-notes', () => {
    openNotesModal();
  });

  listen('resource-update', (event) => {
    const data = event.payload;
    if (!data) return;

    // CPU
    if (data.cpu_usage !== undefined && metricCpuText && barCpu) {
      const pct = Math.min(100, Math.max(0, data.cpu_usage));
      metricCpuText.textContent = `${pct.toFixed(0)}%`;
      barCpu.style.width = `${pct}%`;
      if (data.cpu_freq && metricCpuSub) {
        metricCpuSub.textContent = `${(data.cpu_freq / 1000).toFixed(1)} GHz`;
      }
    }

    // RAM
    if (data.ram_used_gb !== undefined && data.ram_total_gb !== undefined && metricRamText && barRam) {
      const pct = Math.min(100, Math.max(0, data.ram_percent || (data.ram_used_gb / data.ram_total_gb * 100)));
      metricRamText.textContent = `${pct.toFixed(0)}%`;
      barRam.style.width = `${pct}%`;
      if (metricRamSub) {
        metricRamSub.textContent = `${data.ram_used_gb.toFixed(1)} / ${data.ram_total_gb.toFixed(1)} GB`;
      }
    }

    // GPU
    if (metricGpuText && barGpu) {
      if (data.gpu_usage !== null && data.gpu_usage !== undefined) {
        const pct = Math.min(100, Math.max(0, data.gpu_usage));
        metricGpuText.textContent = `${pct.toFixed(0)}%`;
        barGpu.style.width = `${pct}%`;
        if (data.gpu_temp !== null && data.gpu_temp !== undefined && metricGpuSub) {
          metricGpuSub.textContent = `${data.gpu_temp.toFixed(0)} °C`;
        }
      } else {
        metricGpuText.textContent = '--';
        barGpu.style.width = '0%';
        if (metricGpuSub) metricGpuSub.textContent = '-- °C';
      }
    }

    // Battery / Power
    if (sysmonBattery && data.battery) {
      sysmonBattery.textContent = data.battery;
    }

    // CPU Brand Detail
    if (hardwareDetail && data.cpu_brand) {
      hardwareDetail.textContent = data.cpu_brand;
      hardwareDetail.title = data.cpu_brand;
    }
  });

  // Listen for background timer triggers
  listen('timer-alarm', (event) => {
    const payload = event.payload;
    const label = payload?.label || 'Timer Completed!';
    showToast(`⏰ ALARM: ${label}`, 6000);
  });

  // Initial Startup
  (async function init() {
    startClock();
    renderCommandsList();
    fetchLiveWeather();
    setInterval(fetchLiveWeather, 30 * 60 * 1000);

    // Load cached preferences immediately
    try {
      const cachedTheme = localStorage.getItem('finder_theme');
      if (cachedTheme) applyTheme(cachedTheme);
      const cachedHex = localStorage.getItem('finder_accent_hex');
      if (cachedHex) applyAccentHex(cachedHex);
    } catch (_) {}

    // Load persisted configuration from backend
    try {
      const cfg = await invoke('get_config');
      if (cfg) {
        currentConfig = cfg;
        if (cfg.theme) applyTheme(cfg.theme);
        if (cfg.accent_color) {
          const hex = cfg.accent_color.startsWith('#') ? cfg.accent_color : (NAMED_COLOR_MAP[cfg.accent_color] || '#6366f1');
          applyAccentHex(hex);
        }
        if (cfg.scale) applyScale(cfg.scale);
      }
    } catch (err) {
      console.warn('Initial config load:', err);
    }

    // Initial search
    performSearch('');
  })();

})();
