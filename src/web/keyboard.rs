//! v1.96.0: Keyboard Shortcuts System
//!
//! Provides keyboard shortcut handling for the web terminal:
//! - Standard shortcuts (Ctrl+L, Ctrl+C, Ctrl+K, etc.)
//! - Customizable shortcut registry
//! - Help overlay (Ctrl+? or F1)
//! - Vim-style navigation support

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Get keyboard shortcuts CSS
pub fn get_keyboard_css() -> &'static str {
    KEYBOARD_CSS
}

/// Get keyboard shortcuts JavaScript
pub fn get_keyboard_js() -> &'static str {
    KEYBOARD_JS
}

/// Shortcut definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shortcut {
    /// Unique identifier
    pub id: String,
    /// Key combination (e.g., "Ctrl+L")
    pub keys: String,
    /// Description
    pub description: String,
    /// Category for grouping
    pub category: ShortcutCategory,
    /// Whether shortcut is enabled
    pub enabled: bool,
}

/// Shortcut categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShortcutCategory {
    /// Navigation shortcuts
    Navigation,
    /// Editing shortcuts
    Editing,
    /// View shortcuts
    View,
    /// Session shortcuts
    Session,
    /// Help shortcuts
    Help,
}

impl ShortcutCategory {
    pub fn name(&self) -> &'static str {
        match self {
            ShortcutCategory::Navigation => "Navigation",
            ShortcutCategory::Editing => "Editing",
            ShortcutCategory::View => "View",
            ShortcutCategory::Session => "Session",
            ShortcutCategory::Help => "Help",
        }
    }

    pub fn all() -> Vec<ShortcutCategory> {
        vec![
            ShortcutCategory::Navigation,
            ShortcutCategory::Editing,
            ShortcutCategory::View,
            ShortcutCategory::Session,
            ShortcutCategory::Help,
        ]
    }
}

/// Default shortcuts configuration
pub fn default_shortcuts() -> Vec<Shortcut> {
    vec![
        // Navigation
        Shortcut {
            id: "history_prev".to_string(),
            keys: "Up".to_string(),
            description: "Previous command in history".to_string(),
            category: ShortcutCategory::Navigation,
            enabled: true,
        },
        Shortcut {
            id: "history_next".to_string(),
            keys: "Down".to_string(),
            description: "Next command in history".to_string(),
            category: ShortcutCategory::Navigation,
            enabled: true,
        },
        Shortcut {
            id: "scroll_top".to_string(),
            keys: "Ctrl+Home".to_string(),
            description: "Scroll to top".to_string(),
            category: ShortcutCategory::Navigation,
            enabled: true,
        },
        Shortcut {
            id: "scroll_bottom".to_string(),
            keys: "Ctrl+End".to_string(),
            description: "Scroll to bottom".to_string(),
            category: ShortcutCategory::Navigation,
            enabled: true,
        },
        // Editing
        Shortcut {
            id: "clear_input".to_string(),
            keys: "Ctrl+U".to_string(),
            description: "Clear input line".to_string(),
            category: ShortcutCategory::Editing,
            enabled: true,
        },
        Shortcut {
            id: "clear_word".to_string(),
            keys: "Ctrl+W".to_string(),
            description: "Delete word before cursor".to_string(),
            category: ShortcutCategory::Editing,
            enabled: true,
        },
        Shortcut {
            id: "cancel".to_string(),
            keys: "Ctrl+C".to_string(),
            description: "Cancel current operation".to_string(),
            category: ShortcutCategory::Editing,
            enabled: true,
        },
        Shortcut {
            id: "submit".to_string(),
            keys: "Enter".to_string(),
            description: "Submit command".to_string(),
            category: ShortcutCategory::Editing,
            enabled: true,
        },
        // View
        Shortcut {
            id: "clear_screen".to_string(),
            keys: "Ctrl+L".to_string(),
            description: "Clear screen".to_string(),
            category: ShortcutCategory::View,
            enabled: true,
        },
        Shortcut {
            id: "toggle_view".to_string(),
            keys: "Ctrl+Shift+V".to_string(),
            description: "Toggle view mode (rounds/stream)".to_string(),
            category: ShortcutCategory::View,
            enabled: true,
        },
        Shortcut {
            id: "toggle_theme".to_string(),
            keys: "Ctrl+Shift+T".to_string(),
            description: "Toggle dark/light theme".to_string(),
            category: ShortcutCategory::View,
            enabled: true,
        },
        Shortcut {
            id: "focus_input".to_string(),
            keys: "Escape".to_string(),
            description: "Focus input field".to_string(),
            category: ShortcutCategory::View,
            enabled: true,
        },
        // Session
        Shortcut {
            id: "save_session".to_string(),
            keys: "Ctrl+S".to_string(),
            description: "Save current session".to_string(),
            category: ShortcutCategory::Session,
            enabled: true,
        },
        Shortcut {
            id: "open_sessions".to_string(),
            keys: "Ctrl+O".to_string(),
            description: "Open session manager".to_string(),
            category: ShortcutCategory::Session,
            enabled: true,
        },
        Shortcut {
            id: "new_session".to_string(),
            keys: "Ctrl+N".to_string(),
            description: "Start new session".to_string(),
            category: ShortcutCategory::Session,
            enabled: true,
        },
        // Help
        Shortcut {
            id: "show_help".to_string(),
            keys: "F1".to_string(),
            description: "Show keyboard shortcuts".to_string(),
            category: ShortcutCategory::Help,
            enabled: true,
        },
        Shortcut {
            id: "show_help_alt".to_string(),
            keys: "Ctrl+/".to_string(),
            description: "Show keyboard shortcuts".to_string(),
            category: ShortcutCategory::Help,
            enabled: true,
        },
        Shortcut {
            id: "command_palette".to_string(),
            keys: "Ctrl+K".to_string(),
            description: "Open command palette".to_string(),
            category: ShortcutCategory::Help,
            enabled: true,
        },
    ]
}

/// Shortcut registry for customization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutRegistry {
    shortcuts: HashMap<String, Shortcut>,
}

impl Default for ShortcutRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ShortcutRegistry {
    pub fn new() -> Self {
        let mut shortcuts = HashMap::new();
        for shortcut in default_shortcuts() {
            shortcuts.insert(shortcut.id.clone(), shortcut);
        }
        Self { shortcuts }
    }

    pub fn get(&self, id: &str) -> Option<&Shortcut> {
        self.shortcuts.get(id)
    }

    pub fn set_keys(&mut self, id: &str, keys: String) -> bool {
        if let Some(shortcut) = self.shortcuts.get_mut(id) {
            shortcut.keys = keys;
            true
        } else {
            false
        }
    }

    pub fn set_enabled(&mut self, id: &str, enabled: bool) -> bool {
        if let Some(shortcut) = self.shortcuts.get_mut(id) {
            shortcut.enabled = enabled;
            true
        } else {
            false
        }
    }

    pub fn by_category(&self, category: ShortcutCategory) -> Vec<&Shortcut> {
        self.shortcuts
            .values()
            .filter(|s| s.category == category && s.enabled)
            .collect()
    }

    pub fn all(&self) -> Vec<&Shortcut> {
        self.shortcuts.values().collect()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.shortcuts).unwrap_or_default()
    }
}

/// Keyboard CSS for help overlay
const KEYBOARD_CSS: &str = r#"
/* ============================================================================
   v1.96.0: Keyboard Shortcuts System - CSS
   ============================================================================ */

/* ===== Help Overlay ===== */
.keyboard-help-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10000;
    opacity: 0;
    visibility: hidden;
    transition: opacity 0.2s ease, visibility 0.2s ease;
}

.keyboard-help-overlay.visible {
    opacity: 1;
    visibility: visible;
}

.keyboard-help-dialog {
    background: var(--bg-secondary, #161B22);
    border: 1px solid var(--border-color, #30363D);
    border-radius: 12px;
    max-width: 700px;
    width: 90%;
    max-height: 80vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
}

.keyboard-help-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color, #30363D);
    background: var(--bg-tertiary, #21262D);
}

.keyboard-help-header h2 {
    margin: 0;
    font-size: 1.2em;
    color: var(--text-primary, #E6EDF3);
}

.keyboard-help-close {
    background: none;
    border: none;
    color: var(--text-secondary, #8B949E);
    font-size: 24px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
    transition: background 0.2s ease;
}

.keyboard-help-close:hover {
    background: var(--bg-hover, #30363D);
    color: var(--text-primary, #E6EDF3);
}

.keyboard-help-content {
    padding: 20px;
    overflow-y: auto;
    flex: 1;
}

.keyboard-help-category {
    margin-bottom: 24px;
}

.keyboard-help-category:last-child {
    margin-bottom: 0;
}

.keyboard-help-category-title {
    font-size: 0.85em;
    font-weight: 600;
    color: var(--text-accent, #A371F7);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 12px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border-color, #30363D);
}

.keyboard-shortcut-list {
    display: grid;
    gap: 8px;
}

.keyboard-shortcut-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background: var(--bg-tertiary, #21262D);
    border-radius: 6px;
}

.keyboard-shortcut-description {
    color: var(--text-primary, #E6EDF3);
    font-size: 0.9em;
}

.keyboard-shortcut-keys {
    display: flex;
    gap: 4px;
}

.keyboard-key {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 28px;
    height: 28px;
    padding: 0 8px;
    background: var(--bg-primary, #0D1117);
    border: 1px solid var(--border-color, #30363D);
    border-radius: 4px;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
    font-size: 0.8em;
    color: var(--text-secondary, #8B949E);
    box-shadow: 0 2px 0 var(--border-color, #30363D);
}

.keyboard-key-separator {
    color: var(--text-tertiary, #6E7681);
    font-size: 0.8em;
    align-self: center;
}

/* ===== Command Palette ===== */
.command-palette-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 15vh;
    z-index: 10001;
    opacity: 0;
    visibility: hidden;
    transition: opacity 0.15s ease, visibility 0.15s ease;
}

.command-palette-overlay.visible {
    opacity: 1;
    visibility: visible;
}

.command-palette {
    background: var(--bg-secondary, #161B22);
    border: 1px solid var(--border-color, #30363D);
    border-radius: 8px;
    width: 500px;
    max-width: 90%;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    overflow: hidden;
}

.command-palette-input-container {
    display: flex;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-color, #30363D);
}

.command-palette-icon {
    color: var(--text-secondary, #8B949E);
    margin-right: 12px;
    font-size: 1.1em;
}

.command-palette-input {
    flex: 1;
    background: none;
    border: none;
    color: var(--text-primary, #E6EDF3);
    font-size: 1em;
    outline: none;
}

.command-palette-input::placeholder {
    color: var(--text-tertiary, #6E7681);
}

.command-palette-results {
    max-height: 300px;
    overflow-y: auto;
}

.command-palette-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 16px;
    cursor: pointer;
    transition: background 0.1s ease;
}

.command-palette-item:hover,
.command-palette-item.selected {
    background: var(--bg-hover, #21262D);
}

.command-palette-item-label {
    color: var(--text-primary, #E6EDF3);
    font-size: 0.9em;
}

.command-palette-item-shortcut {
    font-size: 0.8em;
}

.command-palette-empty {
    padding: 20px;
    text-align: center;
    color: var(--text-tertiary, #6E7681);
}

/* ===== Shortcut Indicator Toast ===== */
.shortcut-toast {
    position: fixed;
    bottom: 80px;
    left: 50%;
    transform: translateX(-50%);
    background: var(--bg-secondary, #161B22);
    border: 1px solid var(--border-color, #30363D);
    border-radius: 8px;
    padding: 8px 16px;
    display: flex;
    align-items: center;
    gap: 8px;
    z-index: 9999;
    opacity: 0;
    visibility: hidden;
    transition: opacity 0.2s ease, visibility 0.2s ease, transform 0.2s ease;
}

.shortcut-toast.visible {
    opacity: 1;
    visibility: visible;
}

.shortcut-toast-keys {
    display: flex;
    gap: 4px;
}

.shortcut-toast-action {
    color: var(--text-primary, #E6EDF3);
    font-size: 0.9em;
}

/* ===== Mobile Adjustments ===== */
@media (max-width: 600px) {
    .keyboard-help-dialog {
        width: 95%;
        max-height: 90vh;
    }

    .keyboard-shortcut-item {
        flex-direction: column;
        align-items: flex-start;
        gap: 8px;
    }

    .command-palette {
        width: 95%;
        margin-top: 20px;
    }
}
"#;

/// Keyboard JavaScript for shortcut handling
const KEYBOARD_JS: &str = r#"
/* ============================================================================
   v1.96.0: Keyboard Shortcuts System - JavaScript
   ============================================================================ */

(function() {
    'use strict';

    // ===== Shortcut Registry =====
    const shortcuts = {
        // Navigation
        'history_prev': { keys: ['ArrowUp'], description: 'Previous command', category: 'Navigation' },
        'history_next': { keys: ['ArrowDown'], description: 'Next command', category: 'Navigation' },
        'scroll_top': { keys: ['Ctrl', 'Home'], description: 'Scroll to top', category: 'Navigation' },
        'scroll_bottom': { keys: ['Ctrl', 'End'], description: 'Scroll to bottom', category: 'Navigation' },

        // Editing
        'clear_input': { keys: ['Ctrl', 'u'], description: 'Clear input line', category: 'Editing' },
        'clear_word': { keys: ['Ctrl', 'w'], description: 'Delete word', category: 'Editing' },
        'cancel': { keys: ['Ctrl', 'c'], description: 'Cancel operation', category: 'Editing' },

        // View
        'clear_screen': { keys: ['Ctrl', 'l'], description: 'Clear screen', category: 'View' },
        'toggle_view': { keys: ['Ctrl', 'Shift', 'v'], description: 'Toggle view mode', category: 'View' },
        'toggle_theme': { keys: ['Ctrl', 'Shift', 't'], description: 'Toggle theme', category: 'View' },
        'focus_input': { keys: ['Escape'], description: 'Focus input', category: 'View' },

        // Session
        'save_session': { keys: ['Ctrl', 's'], description: 'Save session', category: 'Session' },
        'open_sessions': { keys: ['Ctrl', 'o'], description: 'Open sessions', category: 'Session' },
        'new_session': { keys: ['Ctrl', 'n'], description: 'New session', category: 'Session' },

        // Help
        'show_help': { keys: ['F1'], description: 'Show shortcuts', category: 'Help' },
        'show_help_alt': { keys: ['Ctrl', '/'], description: 'Show shortcuts', category: 'Help' },
        'command_palette': { keys: ['Ctrl', 'k'], description: 'Command palette', category: 'Help' }
    };

    // ===== State =====
    const state = {
        helpVisible: false,
        paletteVisible: false,
        paletteSelectedIndex: 0,
        historyIndex: -1
    };

    // ===== Create Help Overlay =====
    function createHelpOverlay() {
        const overlay = document.createElement('div');
        overlay.className = 'keyboard-help-overlay';
        overlay.id = 'keyboard-help-overlay';

        const categories = {};
        Object.entries(shortcuts).forEach(([id, shortcut]) => {
            if (!categories[shortcut.category]) {
                categories[shortcut.category] = [];
            }
            categories[shortcut.category].push({ id, ...shortcut });
        });

        let categoriesHtml = '';
        Object.entries(categories).forEach(([category, items]) => {
            const itemsHtml = items.map(item => `
                <div class="keyboard-shortcut-item">
                    <span class="keyboard-shortcut-description">${item.description}</span>
                    <span class="keyboard-shortcut-keys">
                        ${item.keys.map(k => `<span class="keyboard-key">${formatKey(k)}</span>`).join('<span class="keyboard-key-separator">+</span>')}
                    </span>
                </div>
            `).join('');

            categoriesHtml += `
                <div class="keyboard-help-category">
                    <div class="keyboard-help-category-title">${category}</div>
                    <div class="keyboard-shortcut-list">${itemsHtml}</div>
                </div>
            `;
        });

        overlay.innerHTML = `
            <div class="keyboard-help-dialog">
                <div class="keyboard-help-header">
                    <h2>⌨️ Keyboard Shortcuts</h2>
                    <button class="keyboard-help-close" title="Close (Esc)">×</button>
                </div>
                <div class="keyboard-help-content">
                    ${categoriesHtml}
                </div>
            </div>
        `;

        document.body.appendChild(overlay);

        // Close handlers
        overlay.querySelector('.keyboard-help-close').addEventListener('click', hideHelp);
        overlay.addEventListener('click', function(e) {
            if (e.target === overlay) hideHelp();
        });

        return overlay;
    }

    // ===== Create Command Palette =====
    function createCommandPalette() {
        const overlay = document.createElement('div');
        overlay.className = 'command-palette-overlay';
        overlay.id = 'command-palette-overlay';

        overlay.innerHTML = `
            <div class="command-palette">
                <div class="command-palette-input-container">
                    <span class="command-palette-icon">🔍</span>
                    <input type="text" class="command-palette-input" placeholder="Type a command..." autocomplete="off">
                </div>
                <div class="command-palette-results"></div>
            </div>
        `;

        document.body.appendChild(overlay);

        const input = overlay.querySelector('.command-palette-input');
        const results = overlay.querySelector('.command-palette-results');

        input.addEventListener('input', function() {
            updatePaletteResults(input.value, results);
        });

        input.addEventListener('keydown', function(e) {
            handlePaletteNavigation(e, results);
        });

        overlay.addEventListener('click', function(e) {
            if (e.target === overlay) hidePalette();
        });

        return overlay;
    }

    // ===== Format Key Display =====
    function formatKey(key) {
        const keyMap = {
            'Ctrl': '⌃',
            'Shift': '⇧',
            'Alt': '⌥',
            'Meta': '⌘',
            'ArrowUp': '↑',
            'ArrowDown': '↓',
            'ArrowLeft': '←',
            'ArrowRight': '→',
            'Escape': 'Esc',
            'Enter': '↵',
            'Backspace': '⌫',
            'Delete': 'Del',
            'Home': 'Home',
            'End': 'End'
        };
        return keyMap[key] || key.toUpperCase();
    }

    // ===== Show/Hide Help =====
    function showHelp() {
        let overlay = document.getElementById('keyboard-help-overlay');
        if (!overlay) overlay = createHelpOverlay();
        overlay.classList.add('visible');
        state.helpVisible = true;
    }

    function hideHelp() {
        const overlay = document.getElementById('keyboard-help-overlay');
        if (overlay) overlay.classList.remove('visible');
        state.helpVisible = false;
        focusInput();
    }

    // ===== Show/Hide Command Palette =====
    function showPalette() {
        let overlay = document.getElementById('command-palette-overlay');
        if (!overlay) overlay = createCommandPalette();
        overlay.classList.add('visible');
        state.paletteVisible = true;
        state.paletteSelectedIndex = 0;

        const input = overlay.querySelector('.command-palette-input');
        input.value = '';
        input.focus();
        updatePaletteResults('', overlay.querySelector('.command-palette-results'));
    }

    function hidePalette() {
        const overlay = document.getElementById('command-palette-overlay');
        if (overlay) overlay.classList.remove('visible');
        state.paletteVisible = false;
        focusInput();
    }

    // ===== Update Palette Results =====
    function updatePaletteResults(query, resultsContainer) {
        const commands = [
            { id: 'clear', label: 'Clear Screen', action: clearScreen },
            { id: 'save', label: 'Save Session', action: () => triggerButton('save-session-btn') },
            { id: 'sessions', label: 'Open Session Manager', action: () => triggerButton('session-menu-btn') },
            { id: 'theme', label: 'Toggle Theme', action: () => triggerButton('theme-toggle-btn') },
            { id: 'view', label: 'Toggle View Mode', action: () => triggerButton('view-mode-toggle') },
            { id: 'help', label: 'Show Keyboard Shortcuts', action: () => { hidePalette(); showHelp(); } },
            { id: 'new', label: 'New Session', action: newSession }
        ];

        const filtered = query
            ? commands.filter(c => c.label.toLowerCase().includes(query.toLowerCase()))
            : commands;

        if (filtered.length === 0) {
            resultsContainer.innerHTML = '<div class="command-palette-empty">No commands found</div>';
            return;
        }

        resultsContainer.innerHTML = filtered.map((cmd, i) => `
            <div class="command-palette-item${i === state.paletteSelectedIndex ? ' selected' : ''}" data-index="${i}">
                <span class="command-palette-item-label">${cmd.label}</span>
            </div>
        `).join('');

        // Add click handlers
        resultsContainer.querySelectorAll('.command-palette-item').forEach((item, i) => {
            item.addEventListener('click', () => {
                filtered[i].action();
                hidePalette();
            });
        });

        // Store filtered commands for keyboard navigation
        resultsContainer.dataset.commands = JSON.stringify(filtered.map(c => c.id));
    }

    // ===== Handle Palette Navigation =====
    function handlePaletteNavigation(e, resultsContainer) {
        const items = resultsContainer.querySelectorAll('.command-palette-item');
        if (items.length === 0) return;

        if (e.key === 'ArrowDown') {
            e.preventDefault();
            state.paletteSelectedIndex = Math.min(state.paletteSelectedIndex + 1, items.length - 1);
            updatePaletteSelection(items);
        } else if (e.key === 'ArrowUp') {
            e.preventDefault();
            state.paletteSelectedIndex = Math.max(state.paletteSelectedIndex - 1, 0);
            updatePaletteSelection(items);
        } else if (e.key === 'Enter') {
            e.preventDefault();
            items[state.paletteSelectedIndex]?.click();
        } else if (e.key === 'Escape') {
            e.preventDefault();
            hidePalette();
        }
    }

    function updatePaletteSelection(items) {
        items.forEach((item, i) => {
            item.classList.toggle('selected', i === state.paletteSelectedIndex);
        });
    }

    // ===== Show Shortcut Toast =====
    function showShortcutToast(keys, action) {
        let toast = document.getElementById('shortcut-toast');
        if (!toast) {
            toast = document.createElement('div');
            toast.id = 'shortcut-toast';
            toast.className = 'shortcut-toast';
            document.body.appendChild(toast);
        }

        toast.innerHTML = `
            <span class="shortcut-toast-keys">
                ${keys.map(k => `<span class="keyboard-key">${formatKey(k)}</span>`).join('')}
            </span>
            <span class="shortcut-toast-action">${action}</span>
        `;

        toast.classList.add('visible');
        setTimeout(() => toast.classList.remove('visible'), 1500);
    }

    // ===== Action Handlers =====
    function focusInput() {
        const input = document.querySelector('#input-container input');
        if (input) input.focus();
    }

    function clearScreen() {
        const clearBtn = document.getElementById('clear-screen-btn');
        if (clearBtn) clearBtn.click();
    }

    function triggerButton(id) {
        const btn = document.getElementById(id);
        if (btn) btn.click();
    }

    function newSession() {
        if (confirm('Start a new session? Current session will be cleared.')) {
            location.reload();
        }
    }

    function clearInput() {
        const input = document.querySelector('#input-container input');
        if (input) input.value = '';
    }

    function deleteWordBefore() {
        const input = document.querySelector('#input-container input');
        if (!input) return;

        const pos = input.selectionStart;
        const value = input.value;
        const beforeCursor = value.substring(0, pos);
        const afterCursor = value.substring(pos);

        // Find word boundary
        const match = beforeCursor.match(/\s*\S*$/);
        const deleteCount = match ? match[0].length : 0;

        input.value = beforeCursor.substring(0, pos - deleteCount) + afterCursor;
        input.setSelectionRange(pos - deleteCount, pos - deleteCount);
    }

    function scrollToTop() {
        const output = document.getElementById('output');
        if (output) output.scrollTop = 0;
    }

    function scrollToBottom() {
        const output = document.getElementById('output');
        if (output) output.scrollTop = output.scrollHeight;
    }

    // ===== Main Keyboard Handler =====
    function handleKeydown(e) {
        // Skip if in editable element (except for specific shortcuts)
        const isEditing = e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA';

        // Help overlay
        if (state.helpVisible) {
            if (e.key === 'Escape') {
                e.preventDefault();
                hideHelp();
            }
            return;
        }

        // Command palette
        if (state.paletteVisible) {
            return; // Handled by palette's own handler
        }

        // Check shortcuts
        const ctrl = e.ctrlKey || e.metaKey;
        const shift = e.shiftKey;
        const key = e.key.toLowerCase();

        // F1 - Show help
        if (e.key === 'F1') {
            e.preventDefault();
            showHelp();
            return;
        }

        // Ctrl+/ - Show help
        if (ctrl && key === '/') {
            e.preventDefault();
            showHelp();
            return;
        }

        // Ctrl+K - Command palette
        if (ctrl && key === 'k') {
            e.preventDefault();
            showPalette();
            return;
        }

        // Ctrl+L - Clear screen
        if (ctrl && key === 'l') {
            e.preventDefault();
            clearScreen();
            showShortcutToast(['Ctrl', 'L'], 'Clear Screen');
            return;
        }

        // Ctrl+S - Save session
        if (ctrl && key === 's') {
            e.preventDefault();
            triggerButton('save-session-btn');
            showShortcutToast(['Ctrl', 'S'], 'Save Session');
            return;
        }

        // Ctrl+O - Open sessions
        if (ctrl && key === 'o') {
            e.preventDefault();
            triggerButton('session-menu-btn');
            return;
        }

        // Ctrl+N - New session
        if (ctrl && key === 'n') {
            e.preventDefault();
            newSession();
            return;
        }

        // Ctrl+Shift+V - Toggle view
        if (ctrl && shift && key === 'v') {
            e.preventDefault();
            triggerButton('view-mode-toggle');
            return;
        }

        // Ctrl+Shift+T - Toggle theme
        if (ctrl && shift && key === 't') {
            e.preventDefault();
            triggerButton('theme-toggle-btn');
            return;
        }

        // Escape - Focus input
        if (e.key === 'Escape' && !isEditing) {
            e.preventDefault();
            focusInput();
            return;
        }

        // Only if editing
        if (isEditing) {
            // Ctrl+U - Clear input
            if (ctrl && key === 'u') {
                e.preventDefault();
                clearInput();
                return;
            }

            // Ctrl+W - Delete word
            if (ctrl && key === 'w') {
                e.preventDefault();
                deleteWordBefore();
                return;
            }
        }

        // Ctrl+Home/End - Scroll
        if (ctrl && e.key === 'Home') {
            e.preventDefault();
            scrollToTop();
            return;
        }

        if (ctrl && e.key === 'End') {
            e.preventDefault();
            scrollToBottom();
            return;
        }
    }

    // ===== Initialize =====
    function init() {
        console.log('[Keyboard] Initializing keyboard shortcuts');

        document.addEventListener('keydown', handleKeydown);

        // Create help overlay on demand
        console.log('[Keyboard] Keyboard shortcuts initialized (F1 or Ctrl+/ for help)');
    }

    // Run on DOM ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

    // Export for external use
    window.RealConsoleKeyboard = {
        showHelp,
        hideHelp,
        showPalette,
        hidePalette,
        shortcuts
    };
})();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_css_not_empty() {
        let css = get_keyboard_css();
        assert!(!css.is_empty());
        assert!(css.contains("keyboard-help"));
    }

    #[test]
    fn test_keyboard_js_not_empty() {
        let js = get_keyboard_js();
        assert!(!js.is_empty());
        assert!(js.contains("keydown"));
    }

    #[test]
    fn test_default_shortcuts() {
        let shortcuts = default_shortcuts();
        assert!(!shortcuts.is_empty());

        // Check that essential shortcuts exist
        let ids: Vec<_> = shortcuts.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"clear_screen"));
        assert!(ids.contains(&"show_help"));
        assert!(ids.contains(&"command_palette"));
    }

    #[test]
    fn test_shortcut_registry() {
        let registry = ShortcutRegistry::new();

        // Check get
        let clear = registry.get("clear_screen");
        assert!(clear.is_some());
        assert_eq!(clear.unwrap().keys, "Ctrl+L");
    }

    #[test]
    fn test_shortcut_registry_modify() {
        let mut registry = ShortcutRegistry::new();

        // Modify keys
        assert!(registry.set_keys("clear_screen", "Ctrl+Shift+L".to_string()));
        assert_eq!(registry.get("clear_screen").unwrap().keys, "Ctrl+Shift+L");

        // Disable
        assert!(registry.set_enabled("clear_screen", false));
        assert!(!registry.get("clear_screen").unwrap().enabled);
    }

    #[test]
    fn test_shortcut_category() {
        let categories = ShortcutCategory::all();
        assert_eq!(categories.len(), 5);
        assert_eq!(ShortcutCategory::Navigation.name(), "Navigation");
    }

    #[test]
    fn test_registry_by_category() {
        let registry = ShortcutRegistry::new();

        let nav = registry.by_category(ShortcutCategory::Navigation);
        assert!(!nav.is_empty());

        let help = registry.by_category(ShortcutCategory::Help);
        assert!(!help.is_empty());
    }

    #[test]
    fn test_registry_to_json() {
        let registry = ShortcutRegistry::new();
        let json = registry.to_json();
        assert!(!json.is_empty());
        assert!(json.contains("clear_screen"));
    }

    #[test]
    fn test_css_has_command_palette() {
        let css = get_keyboard_css();
        assert!(css.contains("command-palette"));
    }

    #[test]
    fn test_js_has_command_palette() {
        let js = get_keyboard_js();
        assert!(js.contains("showPalette"));
        assert!(js.contains("command_palette"));
    }
}
