// Omni Launcher - Frontend Engine
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

  // State
  let currentResults = [];
  let selectedIndex = 0;
  let debounceTimer = null;
  let searchEpoch = 0;
  let currentMode = 'Universal';

  // DOM Elements
  const searchInput = document.getElementById('search-input');
  const resultsList = document.getElementById('results-list');
  const emptyState = document.getElementById('empty-state');
  const badgeMode = document.getElementById('badge-mode');
  const btnSettings = document.getElementById('btn-settings');
  const settingsModal = document.getElementById('settings-modal');
  const btnCloseSettings = document.getElementById('btn-close-settings');
  const btnSaveSettings = document.getElementById('btn-save-settings');

  // Monitor elements
  const metricCpu = document.getElementById('metric-cpu-val');
  const metricRam = document.getElementById('metric-ram-val');
  const metricGpu = document.getElementById('metric-gpu-val');

  // SVG Icons helper
  const ICONS = {
    app: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"></rect><rect x="14" y="3" width="7" height="7"></rect><rect x="14" y="14" width="7" height="7"></rect><rect x="3" y="14" width="7" height="7"></rect></svg>`,
    file: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"></path><polyline points="13 2 13 9 20 9"></polyline></svg>`,
    calc: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="4" y="2" width="16" height="20" rx="2"></rect><line x1="8" y1="6" x2="16" y2="6"></line><line x1="16" y1="14" x2="16" y2="18"></line><path d="M16 10h.01"></path><path d="M12 10h.01"></path><path d="M8 10h.01"></path><path d="M12 14h.01"></path><path d="M8 14h.01"></path><path d="M12 18h.01"></path><path d="M8 18h.01"></path></svg>`,
    ai: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z"></path></svg>`,
    cmd: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="4 17 10 11 4 5"></polyline><line x1="12" y1="19" x2="20" y2="19"></line></svg>`,
    note: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><line x1="10" y1="9" x2="8" y2="9"></line></svg>`,
    web: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>`
  };

  // Perform search query
  async function performSearch(query) {
    const trimmed = query.trim();
    searchEpoch++;
    const thisEpoch = searchEpoch;

    // Detect prefix mode
    if (trimmed.startsWith('/ai ')) {
      currentMode = 'AI Assistant';
    } else if (trimmed.startsWith('/note ')) {
      currentMode = 'Scratchpad';
    } else if (trimmed.startsWith('>')) {
      currentMode = 'Terminal';
    } else {
      currentMode = 'Universal';
    }
    badgeMode.textContent = currentMode;

    try {
      const results = await invoke('search_query', { query: trimmed, epoch: thisEpoch });
      if (thisEpoch !== searchEpoch) return; // Stale query result cancelled

      currentResults = results || [];
      selectedIndex = 0;
      renderResults();
    } catch (err) {
      console.error('Search failed:', err);
    }
  }

  // Render search results
  function renderResults() {
    resultsList.innerHTML = '';

    if (currentResults.length === 0) {
      emptyState.classList.remove('hidden');
      return;
    }
    emptyState.classList.add('hidden');

    let currentCategory = null;

    currentResults.forEach((item, index) => {
      // Add section divider when category changes
      if (item.category && item.category !== currentCategory) {
        currentCategory = item.category;
        const sectionHeader = document.createElement('div');
        sectionHeader.className = 'section-header';
        sectionHeader.textContent = currentCategory;
        resultsList.appendChild(sectionHeader);
      }

      // Check if item is a special instant card (Math or AI response)
      if (item.card_type === 'math') {
        const card = document.createElement('div');
        card.className = 'instant-card math';
        card.innerHTML = `
          <div class="calc-formula">${escapeHtml(item.subtitle || '')}</div>
          <div class="calc-value">${escapeHtml(item.title)}</div>
        `;
        card.addEventListener('click', () => executeItem(index));
        resultsList.appendChild(card);
        return;
      }

      if (item.card_type === 'ai_response') {
        const card = document.createElement('div');
        card.className = 'instant-card ai-response';
        card.innerHTML = `
          <div class="ai-header">
            ${ICONS.ai}
            <span>Ollama AI Answer</span>
          </div>
          <div class="ai-body">${escapeHtml(item.title)}</div>
        `;
        resultsList.appendChild(card);
        return;
      }

      // Standard result item
      const el = document.createElement('div');
      el.className = `result-item ${index === selectedIndex ? 'selected' : ''}`;
      el.dataset.index = index;

      const iconType = item.item_type || 'app';
      const iconSvg = ICONS[iconType] || ICONS.app;

      el.innerHTML = `
        <div class="item-icon ${iconType}">
          ${iconSvg}
        </div>
        <div class="item-content">
          <div class="item-title">${escapeHtml(item.title)}</div>
          ${item.subtitle ? `<div class="item-subtitle" title="${escapeHtml(item.subtitle)}">${escapeHtml(item.subtitle)}</div>` : ''}
        </div>
        <div class="item-action-hint">
          ${item.path ? `<button class="item-copy-btn" title="Copy Path (Tab)">Copy Path</button>` : ''}
          <span>${item.action_hint || '↵ Open'}</span>
        </div>
      `;

      el.addEventListener('click', () => executeItem(index));

      const copyBtn = el.querySelector('.item-copy-btn');
      if (copyBtn) {
        copyBtn.addEventListener('click', (e) => {
          e.stopPropagation();
          copyPath(item.path);
        });
      }

      resultsList.appendChild(el);
    });

    scrollToSelected();
  }

  function scrollToSelected() {
    const selectedEl = resultsList.querySelector(`.result-item[data-index="${selectedIndex}"]`);
    if (selectedEl) {
      selectedEl.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    }
  }

  function updateSelection(newIndex) {
    if (currentResults.length === 0) return;
    if (newIndex < 0) newIndex = 0;
    if (newIndex >= currentResults.length) newIndex = currentResults.length - 1;

    selectedIndex = newIndex;

    const items = resultsList.querySelectorAll('.result-item');
    items.forEach(el => {
      const idx = parseInt(el.dataset.index, 10);
      el.classList.toggle('selected', idx === selectedIndex);
    });

    scrollToSelected();
  }

  async function executeItem(index) {
    const item = currentResults[index];
    if (!item) return;

    try {
      await invoke('execute_action', { item });
      // Clear input and reload initial state
      searchInput.value = '';
      performSearch('');
    } catch (err) {
      console.error('Action failed:', err);
    }
  }

  async function copyPath(path) {
    if (!path) return;
    try {
      await navigator.clipboard.writeText(path);
      const prevMode = badgeMode.textContent;
      badgeMode.textContent = 'Copied!';
      setTimeout(() => { badgeMode.textContent = prevMode; }, 1200);
    } catch (err) {
      console.error('Clipboard copy failed:', err);
    }
  }

  // Keyboard navigation
  searchInput.addEventListener('keydown', (e) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      updateSelection(selectedIndex + 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      updateSelection(selectedIndex - 1);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      executeItem(selectedIndex);
    } else if (e.key === 'Tab') {
      e.preventDefault();
      const item = currentResults[selectedIndex];
      if (item && item.path) {
        copyPath(item.path);
      }
    } else if (e.key === 'Escape') {
      e.preventDefault();
      invoke('hide_launcher');
    }
  });

  // Input debouncing
  searchInput.addEventListener('input', (e) => {
    clearTimeout(debounceTimer);
    const query = e.target.value;
    // Immediate for empty or single char commands, 150ms debounce for full queries
    const delay = (query.length < 2 || query.startsWith('>')) ? 0 : 150;
    debounceTimer = setTimeout(() => {
      performSearch(query);
    }, delay);
  });

  // Settings Modal elements & events
  const settingTheme = document.getElementById('setting-theme');
  if (settingTheme) {
    settingTheme.addEventListener('change', (e) => {
      applyTheme(e.target.value);
    });
  }

  btnSettings.addEventListener('click', openSettings);
  btnCloseSettings.addEventListener('click', closeSettings);
  btnSaveSettings.addEventListener('click', saveSettings);

  // Shortcut for settings (Ctrl+,)
  window.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === ',') {
      e.preventDefault();
      if (settingsModal.classList.contains('hidden')) {
        openSettings();
      } else {
        closeSettings();
      }
    }
  });

  async function openSettings() {
    try {
      const cfg = await invoke('get_config');
      if (cfg) {
        document.getElementById('setting-hotkey').value = cfg.hotkey || 'RightAlt';
        document.getElementById('setting-bottom-offset').value = cfg.bottom_offset ?? 80;
        document.getElementById('setting-theme').value = cfg.theme || 'oled';
        document.getElementById('setting-hide-blur').checked = cfg.hide_on_blur === true;
        document.getElementById('setting-ollama-model').value = cfg.ollama_model || 'auto';
      }
    } catch (err) {
      console.error('Get config failed:', err);
    }
    settingsModal.classList.remove('hidden');
  }

  function closeSettings() {
    settingsModal.classList.add('hidden');
    searchInput.focus();
  }

  async function saveSettings() {
    btnSaveSettings.textContent = 'Saving...';
    btnSaveSettings.disabled = true;

    try {
      const cur = await invoke('get_config').catch(() => ({}));
      const themeVal = document.getElementById('setting-theme').value || 'oled';
      const hotkeyVal = document.getElementById('setting-hotkey').value.trim() || 'RightAlt';
      const offsetVal = parseInt(document.getElementById('setting-bottom-offset').value, 10) || 80;
      const hideBlurVal = document.getElementById('setting-hide-blur').checked;
      const ollamaVal = document.getElementById('setting-ollama-model').value.trim() || 'auto';

      const newConfig = {
        ...cur,
        hotkey: hotkeyVal,
        bottom_offset: offsetVal,
        bottomOffset: offsetVal,
        theme: themeVal,
        hide_on_blur: hideBlurVal,
        hideOnBlur: hideBlurVal,
        ollama_model: ollamaVal,
        ollamaModel: ollamaVal,
      };

      await invoke('save_config', { newConfig, new_config: newConfig });
      applyTheme(themeVal);
      btnSaveSettings.textContent = 'Saved!';
      setTimeout(() => {
        btnSaveSettings.textContent = 'Save Configuration';
        btnSaveSettings.disabled = false;
        closeSettings();
      }, 400);
    } catch (err) {
      console.error('Failed to save settings:', err);
      btnSaveSettings.textContent = 'Error!';
      setTimeout(() => {
        btnSaveSettings.textContent = 'Save Configuration';
        btnSaveSettings.disabled = false;
      }, 1500);
    }
  }

  function applyTheme(theme) {
    const t = theme || 'oled';
    document.body.className = `theme-${t}`;
    try {
      localStorage.setItem('finder_theme', t);
    } catch (_) {}
  }

  function escapeHtml(str) {
    if (!str) return '';
    return str
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  }

  // Setup Tauri Event Listeners
  listen('launcher-shown', () => {
    searchInput.focus();
    searchInput.select();
    performSearch(searchInput.value);
  });

  listen('resource-update', (event) => {
    const data = event.payload;
    if (!data) return;

    if (metricCpu && data.cpu_usage !== undefined) {
      metricCpu.textContent = `${data.cpu_usage.toFixed(0)}% ${data.cpu_freq ? (data.cpu_freq / 1000).toFixed(1) + 'G' : ''}`;
    }
    if (metricRam && data.ram_used_gb !== undefined) {
      metricRam.textContent = `${data.ram_used_gb.toFixed(1)}/${data.ram_total_gb.toFixed(1)} GB`;
    }
    if (metricGpu) {
      if (data.gpu_usage !== null && data.gpu_usage !== undefined) {
        metricGpu.textContent = `${data.gpu_usage.toFixed(0)}%`;
      } else {
        metricGpu.textContent = '--';
      }
    }
  });

  // Load cached theme immediately
  try {
    const cachedTheme = localStorage.getItem('finder_theme');
    if (cachedTheme) {
      applyTheme(cachedTheme);
    }
  } catch (_) {}

  // Initial load
  (async function init() {
    try {
      const cfg = await invoke('get_config');
      if (cfg && cfg.theme) {
        applyTheme(cfg.theme);
      }
    } catch (err) {
      console.warn('Initial config load:', err);
    }
    performSearch('');
  })();
})();
