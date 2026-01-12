//! Theme System - v1.97.0
//!
//! Provides comprehensive theming support:
//! - Light/Dark/System themes
//! - Custom theme definitions
//! - System preference detection
//! - Per-session theme synchronization

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Theme mode options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ThemeMode {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// Follow system preference
    #[default]
    System,
}


impl ThemeMode {
    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
            ThemeMode::System => "System",
        }
    }

    /// Get icon for theme mode
    pub fn icon(&self) -> &'static str {
        match self {
            ThemeMode::Light => "sun",
            ThemeMode::Dark => "moon",
            ThemeMode::System => "monitor",
        }
    }
}

/// Color palette for a theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePalette {
    /// Primary background color
    pub bg_primary: String,
    /// Secondary background color
    pub bg_secondary: String,
    /// Tertiary background color
    pub bg_tertiary: String,
    /// Primary text color
    pub text_primary: String,
    /// Secondary text color
    pub text_secondary: String,
    /// Accent color (brand color)
    pub accent: String,
    /// Success color
    pub success: String,
    /// Warning color
    pub warning: String,
    /// Error color
    pub error: String,
    /// Border color
    pub border: String,
}

impl ThemePalette {
    /// Create dark theme palette (default)
    pub fn dark() -> Self {
        Self {
            bg_primary: "#0a0e27".to_string(),
            bg_secondary: "#0d1117".to_string(),
            bg_tertiary: "#1a0b2e".to_string(),
            text_primary: "#e6edf3".to_string(),
            text_secondary: "#8b949e".to_string(),
            accent: "#a371f7".to_string(),
            success: "#0ecb81".to_string(),
            warning: "#f0b90b".to_string(),
            error: "#f85149".to_string(),
            border: "#30363d".to_string(),
        }
    }

    /// Create light theme palette
    pub fn light() -> Self {
        Self {
            bg_primary: "#ffffff".to_string(),
            bg_secondary: "#f6f8fa".to_string(),
            bg_tertiary: "#f0f0f0".to_string(),
            text_primary: "#1c1c1c".to_string(),
            text_secondary: "#7c7c7c".to_string(),
            accent: "#8b5cf6".to_string(),
            success: "#0ecb81".to_string(),
            warning: "#f0b90b".to_string(),
            error: "#cf222e".to_string(),
            border: "#d0d7de".to_string(),
        }
    }

    /// Generate CSS variables from palette
    pub fn to_css_variables(&self) -> String {
        format!(
            r#"--bg-primary: {};
    --bg-secondary: {};
    --bg-tertiary: {};
    --text-primary: {};
    --text-secondary: {};
    --accent: {};
    --success: {};
    --warning: {};
    --error: {};
    --border: {};"#,
            self.bg_primary,
            self.bg_secondary,
            self.bg_tertiary,
            self.text_primary,
            self.text_secondary,
            self.accent,
            self.success,
            self.warning,
            self.error,
            self.border
        )
    }
}

/// Custom theme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTheme {
    /// Theme identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Base theme (light or dark)
    pub base: ThemeMode,
    /// Color palette
    pub palette: ThemePalette,
    /// Whether this is a built-in theme
    #[serde(default)]
    pub builtin: bool,
}

impl CustomTheme {
    /// Create a new custom theme
    pub fn new(id: &str, name: &str, base: ThemeMode) -> Self {
        let palette = match base {
            ThemeMode::Light | ThemeMode::System => ThemePalette::light(),
            ThemeMode::Dark => ThemePalette::dark(),
        };
        Self {
            id: id.to_string(),
            name: name.to_string(),
            base,
            palette,
            builtin: false,
        }
    }

    /// Create built-in dark theme
    pub fn builtin_dark() -> Self {
        Self {
            id: "dark".to_string(),
            name: "Dark".to_string(),
            base: ThemeMode::Dark,
            palette: ThemePalette::dark(),
            builtin: true,
        }
    }

    /// Create built-in light theme
    pub fn builtin_light() -> Self {
        Self {
            id: "light".to_string(),
            name: "Light".to_string(),
            base: ThemeMode::Light,
            palette: ThemePalette::light(),
            builtin: true,
        }
    }
}

/// Theme registry for managing themes
#[derive(Debug, Clone)]
pub struct ThemeRegistry {
    themes: HashMap<String, CustomTheme>,
    current_mode: ThemeMode,
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeRegistry {
    /// Create new registry with built-in themes
    pub fn new() -> Self {
        let mut themes = HashMap::new();
        themes.insert("dark".to_string(), CustomTheme::builtin_dark());
        themes.insert("light".to_string(), CustomTheme::builtin_light());

        Self {
            themes,
            current_mode: ThemeMode::System,
        }
    }

    /// Get current theme mode
    pub fn current_mode(&self) -> ThemeMode {
        self.current_mode
    }

    /// Set theme mode
    pub fn set_mode(&mut self, mode: ThemeMode) {
        self.current_mode = mode;
    }

    /// Get theme by ID
    pub fn get(&self, id: &str) -> Option<&CustomTheme> {
        self.themes.get(id)
    }

    /// Register a custom theme
    pub fn register(&mut self, theme: CustomTheme) {
        self.themes.insert(theme.id.clone(), theme);
    }

    /// Remove a custom theme (cannot remove built-in)
    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(theme) = self.themes.get(id) {
            if !theme.builtin {
                self.themes.remove(id);
                return true;
            }
        }
        false
    }

    /// List all themes
    pub fn list(&self) -> Vec<&CustomTheme> {
        let mut themes: Vec<_> = self.themes.values().collect();
        themes.sort_by(|a, b| {
            // Built-in themes first, then by name
            match (a.builtin, b.builtin) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
        themes
    }

    /// List custom themes only
    pub fn list_custom(&self) -> Vec<&CustomTheme> {
        self.themes.values().filter(|t| !t.builtin).collect()
    }

    /// Convert to JSON for frontend
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "currentMode": self.current_mode,
            "themes": self.list().iter().map(|t| {
                serde_json::json!({
                    "id": t.id,
                    "name": t.name,
                    "base": t.base,
                    "builtin": t.builtin
                })
            }).collect::<Vec<_>>()
        })
    }
}

/// Theme CSS for system preference detection and enhanced theming
pub const THEME_CSS: &str = r#"
/* ============================================
   Theme System v1.97.0 - System Detection
   ============================================ */

/* System theme detection media queries */
@media (prefers-color-scheme: light) {
    :root:not([data-theme]) {
        --theme-detected: light;
    }
}

@media (prefers-color-scheme: dark) {
    :root:not([data-theme]) {
        --theme-detected: dark;
    }
}

/* Theme toggle dropdown */
.theme-dropdown {
    position: relative;
    display: inline-block;
}

.theme-dropdown-content {
    display: none;
    position: absolute;
    right: 0;
    top: 100%;
    min-width: 160px;
    background: var(--bg-secondary, #0d1117);
    border: 1px solid var(--border, #30363d);
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
    z-index: 1000;
    padding: 8px 0;
    margin-top: 4px;
}

.theme-dropdown:hover .theme-dropdown-content,
.theme-dropdown.open .theme-dropdown-content {
    display: block;
}

.theme-option {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    cursor: pointer;
    color: var(--text-secondary, #8b949e);
    transition: all 0.15s ease;
}

.theme-option:hover {
    background: var(--bg-tertiary, #1a0b2e);
    color: var(--text-primary, #e6edf3);
}

.theme-option.active {
    color: var(--accent, #a371f7);
}

.theme-option-icon {
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
}

.theme-option-label {
    flex: 1;
}

.theme-option-check {
    opacity: 0;
}

.theme-option.active .theme-option-check {
    opacity: 1;
    color: var(--accent, #a371f7);
}

/* Theme transition animation */
:root {
    transition: background-color 0.3s ease, color 0.3s ease;
}

[data-theme-transition] * {
    transition: background-color 0.3s ease,
                color 0.3s ease,
                border-color 0.3s ease,
                box-shadow 0.3s ease !important;
}

/* Light theme overrides */
[data-theme="light"] .theme-dropdown-content {
    background: #ffffff;
    border-color: #d0d7de;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
}

[data-theme="light"] .theme-option:hover {
    background: #f6f8fa;
}
"#;

/// Theme JavaScript for enhanced functionality
pub const THEME_JS: &str = r#"
// ============================================
// Theme System v1.97.0 - Enhanced Detection
// ============================================

(function() {
    'use strict';

    // Theme modes
    const THEME_MODES = {
        LIGHT: 'light',
        DARK: 'dark',
        SYSTEM: 'system'
    };

    // Storage key
    const STORAGE_KEY = 'realconsole-theme-mode';

    // Current state
    let currentMode = THEME_MODES.SYSTEM;
    let systemPreference = 'dark';

    // Detect system preference
    function detectSystemPreference() {
        if (window.matchMedia) {
            if (window.matchMedia('(prefers-color-scheme: light)').matches) {
                return 'light';
            }
        }
        return 'dark';
    }

    // Get effective theme (resolve 'system' to actual theme)
    function getEffectiveTheme() {
        if (currentMode === THEME_MODES.SYSTEM) {
            return systemPreference;
        }
        return currentMode;
    }

    // Apply theme to document
    function applyTheme(theme) {
        const root = document.getElementById('html-root') || document.documentElement;

        // Add transition class
        root.setAttribute('data-theme-transition', '');

        if (theme === 'light') {
            root.setAttribute('data-theme', 'light');
        } else {
            root.removeAttribute('data-theme');
        }

        // Remove transition class after animation
        setTimeout(() => {
            root.removeAttribute('data-theme-transition');
        }, 300);

        // Update theme toggle button
        updateThemeButton();

        // Dispatch custom event
        window.dispatchEvent(new CustomEvent('themechange', {
            detail: { theme, mode: currentMode }
        }));
    }

    // Update theme toggle button text
    function updateThemeButton() {
        const btn = document.getElementById('theme-toggle-btn');
        if (!btn) return;

        const effectiveTheme = getEffectiveTheme();
        const icons = {
            light: { icon: 'sun', text: 'Light' },
            dark: { icon: 'moon', text: 'Dark' },
            system: { icon: 'monitor', text: 'System' }
        };

        const modeInfo = icons[currentMode] || icons.system;
        const themeInfo = icons[effectiveTheme];

        if (currentMode === THEME_MODES.SYSTEM) {
            btn.innerHTML = `<span class="theme-icon">💻</span> System (${themeInfo.text})`;
            btn.title = 'Using system theme preference';
        } else if (effectiveTheme === 'light') {
            btn.innerHTML = '<span class="theme-icon">☀️</span> Light';
            btn.title = 'Switch to dark theme';
        } else {
            btn.innerHTML = '<span class="theme-icon">🌙</span> Dark';
            btn.title = 'Switch to light theme';
        }
    }

    // Set theme mode
    function setThemeMode(mode) {
        currentMode = mode;
        localStorage.setItem(STORAGE_KEY, mode);
        applyTheme(getEffectiveTheme());
    }

    // Cycle through theme modes
    function cycleTheme() {
        const modes = [THEME_MODES.LIGHT, THEME_MODES.DARK, THEME_MODES.SYSTEM];
        const currentIndex = modes.indexOf(currentMode);
        const nextIndex = (currentIndex + 1) % modes.length;
        setThemeMode(modes[nextIndex]);
    }

    // Load saved preference
    function loadPreference() {
        const saved = localStorage.getItem(STORAGE_KEY);
        if (saved && Object.values(THEME_MODES).includes(saved)) {
            currentMode = saved;
        } else {
            currentMode = THEME_MODES.SYSTEM;
        }
    }

    // Listen for system preference changes
    function watchSystemPreference() {
        if (window.matchMedia) {
            const mediaQuery = window.matchMedia('(prefers-color-scheme: light)');

            // Modern API
            if (mediaQuery.addEventListener) {
                mediaQuery.addEventListener('change', (e) => {
                    systemPreference = e.matches ? 'light' : 'dark';
                    if (currentMode === THEME_MODES.SYSTEM) {
                        applyTheme(systemPreference);
                    }
                });
            }
            // Legacy API
            else if (mediaQuery.addListener) {
                mediaQuery.addListener((e) => {
                    systemPreference = e.matches ? 'light' : 'dark';
                    if (currentMode === THEME_MODES.SYSTEM) {
                        applyTheme(systemPreference);
                    }
                });
            }
        }
    }

    // Initialize theme system
    function initTheme() {
        // Detect system preference first
        systemPreference = detectSystemPreference();

        // Load user preference
        loadPreference();

        // Apply theme
        applyTheme(getEffectiveTheme());

        // Watch for system changes
        watchSystemPreference();

        // Bind theme toggle button
        const themeBtn = document.getElementById('theme-toggle-btn');
        if (themeBtn) {
            themeBtn.addEventListener('click', cycleTheme);
        }

        // Expose API
        window.RealConsoleTheme = {
            getMode: () => currentMode,
            getEffectiveTheme,
            setMode: setThemeMode,
            cycle: cycleTheme,
            MODES: THEME_MODES
        };
    }

    // Initialize on DOM ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initTheme);
    } else {
        initTheme();
    }
})();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_mode_default() {
        assert_eq!(ThemeMode::default(), ThemeMode::System);
    }

    #[test]
    fn test_theme_mode_display() {
        assert_eq!(ThemeMode::Light.display_name(), "Light");
        assert_eq!(ThemeMode::Dark.display_name(), "Dark");
        assert_eq!(ThemeMode::System.display_name(), "System");
    }

    #[test]
    fn test_theme_mode_icons() {
        assert_eq!(ThemeMode::Light.icon(), "sun");
        assert_eq!(ThemeMode::Dark.icon(), "moon");
        assert_eq!(ThemeMode::System.icon(), "monitor");
    }

    #[test]
    fn test_palette_dark() {
        let palette = ThemePalette::dark();
        assert_eq!(palette.bg_primary, "#0a0e27");
        assert!(palette.text_primary.starts_with('#'));
    }

    #[test]
    fn test_palette_light() {
        let palette = ThemePalette::light();
        assert_eq!(palette.bg_primary, "#ffffff");
        assert!(palette.text_primary.starts_with('#'));
    }

    #[test]
    fn test_palette_to_css() {
        let palette = ThemePalette::dark();
        let css = palette.to_css_variables();
        assert!(css.contains("--bg-primary"));
        assert!(css.contains("--accent"));
    }

    #[test]
    fn test_custom_theme_new() {
        let theme = CustomTheme::new("midnight", "Midnight Blue", ThemeMode::Dark);
        assert_eq!(theme.id, "midnight");
        assert_eq!(theme.name, "Midnight Blue");
        assert!(!theme.builtin);
    }

    #[test]
    fn test_builtin_themes() {
        let dark = CustomTheme::builtin_dark();
        let light = CustomTheme::builtin_light();

        assert!(dark.builtin);
        assert!(light.builtin);
        assert_eq!(dark.id, "dark");
        assert_eq!(light.id, "light");
    }

    #[test]
    fn test_theme_registry() {
        let registry = ThemeRegistry::new();
        assert!(registry.get("dark").is_some());
        assert!(registry.get("light").is_some());
        assert_eq!(registry.current_mode(), ThemeMode::System);
    }

    #[test]
    fn test_registry_set_mode() {
        let mut registry = ThemeRegistry::new();
        registry.set_mode(ThemeMode::Dark);
        assert_eq!(registry.current_mode(), ThemeMode::Dark);
    }

    #[test]
    fn test_registry_register_custom() {
        let mut registry = ThemeRegistry::new();
        let custom = CustomTheme::new("ocean", "Ocean Blue", ThemeMode::Dark);
        registry.register(custom);

        assert!(registry.get("ocean").is_some());
        assert_eq!(registry.list().len(), 3);
    }

    #[test]
    fn test_registry_remove_custom() {
        let mut registry = ThemeRegistry::new();
        let custom = CustomTheme::new("ocean", "Ocean Blue", ThemeMode::Dark);
        registry.register(custom);

        assert!(registry.remove("ocean"));
        assert!(registry.get("ocean").is_none());
    }

    #[test]
    fn test_registry_cannot_remove_builtin() {
        let mut registry = ThemeRegistry::new();
        assert!(!registry.remove("dark"));
        assert!(!registry.remove("light"));
        assert!(registry.get("dark").is_some());
    }

    #[test]
    fn test_registry_list_sorted() {
        let mut registry = ThemeRegistry::new();
        registry.register(CustomTheme::new("zeta", "Zeta", ThemeMode::Dark));
        registry.register(CustomTheme::new("alpha", "Alpha", ThemeMode::Light));

        let list = registry.list();
        // Built-in first (dark, light), then custom alphabetically
        assert!(list[0].builtin);
        assert!(list[1].builtin);
    }

    #[test]
    fn test_registry_to_json() {
        let registry = ThemeRegistry::new();
        let json = registry.to_json();

        assert!(json.get("currentMode").is_some());
        assert!(json.get("themes").is_some());
    }

    #[test]
    fn test_theme_css_not_empty() {
        assert!(!THEME_CSS.is_empty());
        assert!(THEME_CSS.contains("prefers-color-scheme"));
    }

    #[test]
    fn test_theme_js_not_empty() {
        assert!(!THEME_JS.is_empty());
        assert!(THEME_JS.contains("detectSystemPreference"));
    }

    #[test]
    fn test_js_has_system_detection() {
        assert!(THEME_JS.contains("matchMedia"));
        assert!(THEME_JS.contains("prefers-color-scheme"));
    }

    #[test]
    fn test_js_has_theme_cycle() {
        assert!(THEME_JS.contains("cycleTheme"));
        assert!(THEME_JS.contains("THEME_MODES"));
    }
}
