//! v1.95.0: Mobile Experience Enhancement
//!
//! Provides mobile-specific CSS and JavaScript for:
//! - Touch-friendly controls (larger tap targets)
//! - Virtual keyboard optimization
//! - Swipe gestures (history navigation)
//! - Pinch-to-zoom for charts

/// Get mobile-specific CSS
pub fn get_mobile_css() -> &'static str {
    MOBILE_CSS
}

/// Get mobile-specific JavaScript
pub fn get_mobile_js() -> &'static str {
    MOBILE_JS
}

/// Mobile CSS for touch-friendly UI
const MOBILE_CSS: &str = r#"
/* ============================================================================
   v1.95.0: Mobile Experience Enhancement - CSS
   ============================================================================ */

/* Mobile Detection */
@media (max-width: 768px), (pointer: coarse) {
    /* ===== Touch-Friendly Controls ===== */

    /* Larger tap targets (minimum 44x44px per Apple HIG) */
    button,
    .btn,
    .session-btn,
    .clear-btn,
    .theme-toggle-btn,
    .view-mode-btn,
    .session-action-btn {
        min-height: 44px;
        min-width: 44px;
        padding: 12px 16px;
        font-size: 16px; /* Prevent iOS zoom on focus */
    }

    /* Larger input fields */
    input[type="text"],
    input[type="search"],
    textarea,
    select {
        min-height: 44px;
        font-size: 16px; /* Prevent iOS zoom on focus */
        padding: 12px;
    }

    /* Terminal input optimization */
    #input-container input {
        font-size: 16px;
        padding: 14px;
    }

    /* ===== Header Optimization ===== */
    #header {
        flex-wrap: wrap;
        padding: 8px;
        gap: 8px;
    }

    #header-content h1 {
        font-size: 1.2em;
    }

    #header-content p {
        font-size: 0.85em;
        display: none; /* Hide tagline on mobile */
    }

    #header-left-controls,
    #header-right-controls {
        gap: 4px;
    }

    /* Compact header buttons */
    #header button {
        padding: 8px 10px;
        font-size: 14px;
    }

    /* Hide text labels, show only icons on small screens */
    @media (max-width: 480px) {
        #session-menu-btn,
        #view-mode-toggle,
        #clear-screen-btn,
        #theme-toggle-btn {
            padding: 10px;
            min-width: 44px;
        }

        #session-menu-btn::after { content: ''; }
        #view-mode-toggle::after { content: ''; }
        #clear-screen-btn::after { content: ''; }
    }

    /* ===== Session Panel Mobile ===== */
    .session-panel-dialog {
        width: 95%;
        max-width: none;
        margin: 10px;
        max-height: 90vh;
    }

    .session-item {
        padding: 16px;
        margin-bottom: 12px;
    }

    .session-item-actions button {
        min-height: 40px;
        padding: 10px 14px;
    }

    /* ===== Output Area Optimization ===== */
    #output {
        padding: 12px;
        font-size: 14px;
    }

    .round-container {
        margin-bottom: 16px;
        padding: 12px;
    }

    /* ===== Chart Container Mobile ===== */
    .chart-container {
        min-height: 300px;
        touch-action: pan-x pan-y pinch-zoom;
    }

    /* Enable pinch-to-zoom on charts */
    .chart-container canvas {
        touch-action: pinch-zoom;
    }

    /* ===== Virtual Keyboard Optimization ===== */

    /* When keyboard is open, adjust layout */
    body.keyboard-open {
        height: 100vh;
        overflow: hidden;
    }

    body.keyboard-open #output {
        max-height: calc(100vh - 200px);
    }

    body.keyboard-open #input-container {
        position: fixed;
        bottom: 0;
        left: 0;
        right: 0;
        background: var(--bg-secondary, #161B22);
        padding: 8px;
        border-top: 1px solid var(--border-color, #30363D);
        z-index: 1000;
    }

    /* ===== Swipe Gesture Indicators ===== */
    .swipe-indicator {
        position: fixed;
        top: 50%;
        transform: translateY(-50%);
        background: rgba(255, 255, 255, 0.2);
        color: white;
        padding: 20px 10px;
        border-radius: 4px;
        font-size: 24px;
        pointer-events: none;
        opacity: 0;
        transition: opacity 0.2s ease;
        z-index: 9999;
    }

    .swipe-indicator.left {
        left: 10px;
    }

    .swipe-indicator.right {
        right: 10px;
    }

    .swipe-indicator.visible {
        opacity: 1;
    }

    /* ===== Touch Feedback ===== */
    button:active,
    .btn:active,
    .session-item:active {
        transform: scale(0.98);
        opacity: 0.8;
    }

    /* Disable hover effects on touch devices */
    @media (hover: none) {
        button:hover,
        .btn:hover,
        .session-item:hover {
            transform: none;
            opacity: 1;
        }
    }

    /* ===== Scroll Optimization ===== */
    #output {
        -webkit-overflow-scrolling: touch;
        scroll-behavior: smooth;
    }

    /* Momentum scrolling */
    .session-list {
        -webkit-overflow-scrolling: touch;
    }

    /* ===== Safe Area Insets (Notch devices) ===== */
    @supports (padding: max(0px)) {
        #header {
            padding-top: max(8px, env(safe-area-inset-top));
            padding-left: max(8px, env(safe-area-inset-left));
            padding-right: max(8px, env(safe-area-inset-right));
        }

        #input-container {
            padding-bottom: max(8px, env(safe-area-inset-bottom));
            padding-left: max(12px, env(safe-area-inset-left));
            padding-right: max(12px, env(safe-area-inset-right));
        }
    }
}

/* ===== Landscape Mode Optimization ===== */
@media (max-height: 500px) and (orientation: landscape) {
    #header {
        padding: 4px 8px;
    }

    #header-content {
        display: none;
    }

    #output {
        max-height: calc(100vh - 100px);
    }
}

/* ===== High DPI Display Optimization ===== */
@media (-webkit-min-device-pixel-ratio: 2), (min-resolution: 192dpi) {
    .round-container,
    .session-panel-dialog,
    button {
        border-width: 0.5px;
    }
}
"#;

/// Mobile JavaScript for gestures and touch handling
const MOBILE_JS: &str = r#"
/* ============================================================================
   v1.95.0: Mobile Experience Enhancement - JavaScript
   ============================================================================ */

(function() {
    'use strict';

    // ===== Mobile Detection =====
    const isMobile = /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent);
    const isTouch = 'ontouchstart' in window || navigator.maxTouchPoints > 0;

    if (!isMobile && !isTouch) {
        console.log('[Mobile] Desktop detected, skipping mobile enhancements');
        return;
    }

    console.log('[Mobile] Mobile/touch device detected, enabling enhancements');

    // ===== State =====
    const state = {
        historyIndex: -1,
        commandHistory: [],
        touchStartX: 0,
        touchStartY: 0,
        touchStartTime: 0,
        isSwipe: false,
        keyboardVisible: false
    };

    // ===== Virtual Keyboard Detection =====
    function setupKeyboardDetection() {
        const viewportHeight = window.innerHeight;

        window.addEventListener('resize', function() {
            const currentHeight = window.innerHeight;
            const heightDiff = viewportHeight - currentHeight;

            // If viewport shrinks significantly, keyboard is likely open
            if (heightDiff > 150) {
                if (!state.keyboardVisible) {
                    state.keyboardVisible = true;
                    document.body.classList.add('keyboard-open');
                    console.log('[Mobile] Keyboard opened');

                    // Scroll to input
                    setTimeout(() => {
                        const input = document.querySelector('#input-container input');
                        if (input) {
                            input.scrollIntoView({ behavior: 'smooth', block: 'center' });
                        }
                    }, 100);
                }
            } else {
                if (state.keyboardVisible) {
                    state.keyboardVisible = false;
                    document.body.classList.remove('keyboard-open');
                    console.log('[Mobile] Keyboard closed');
                }
            }
        });

        // Also detect via focus events
        document.addEventListener('focusin', function(e) {
            if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') {
                setTimeout(() => {
                    document.body.classList.add('keyboard-open');
                }, 300);
            }
        });

        document.addEventListener('focusout', function(e) {
            if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') {
                setTimeout(() => {
                    document.body.classList.remove('keyboard-open');
                }, 100);
            }
        });
    }

    // ===== Swipe Gesture Detection =====
    function setupSwipeGestures() {
        const output = document.getElementById('output');
        if (!output) return;

        // Create swipe indicators
        const leftIndicator = document.createElement('div');
        leftIndicator.className = 'swipe-indicator left';
        leftIndicator.textContent = '◀';
        document.body.appendChild(leftIndicator);

        const rightIndicator = document.createElement('div');
        rightIndicator.className = 'swipe-indicator right';
        rightIndicator.textContent = '▶';
        document.body.appendChild(rightIndicator);

        const SWIPE_THRESHOLD = 80;
        const SWIPE_TIME_LIMIT = 300;

        output.addEventListener('touchstart', function(e) {
            if (e.touches.length !== 1) return;

            state.touchStartX = e.touches[0].clientX;
            state.touchStartY = e.touches[0].clientY;
            state.touchStartTime = Date.now();
            state.isSwipe = false;
        }, { passive: true });

        output.addEventListener('touchmove', function(e) {
            if (e.touches.length !== 1) return;

            const deltaX = e.touches[0].clientX - state.touchStartX;
            const deltaY = Math.abs(e.touches[0].clientY - state.touchStartY);

            // Only consider horizontal swipes
            if (Math.abs(deltaX) > 30 && deltaY < 50) {
                state.isSwipe = true;

                // Show indicator
                if (deltaX > SWIPE_THRESHOLD / 2) {
                    leftIndicator.classList.add('visible');
                    rightIndicator.classList.remove('visible');
                } else if (deltaX < -SWIPE_THRESHOLD / 2) {
                    rightIndicator.classList.add('visible');
                    leftIndicator.classList.remove('visible');
                } else {
                    leftIndicator.classList.remove('visible');
                    rightIndicator.classList.remove('visible');
                }
            }
        }, { passive: true });

        output.addEventListener('touchend', function(e) {
            // Hide indicators
            leftIndicator.classList.remove('visible');
            rightIndicator.classList.remove('visible');

            if (!state.isSwipe) return;

            const touchEndX = e.changedTouches[0].clientX;
            const deltaX = touchEndX - state.touchStartX;
            const deltaTime = Date.now() - state.touchStartTime;

            if (deltaTime > SWIPE_TIME_LIMIT) return;

            if (Math.abs(deltaX) >= SWIPE_THRESHOLD) {
                if (deltaX > 0) {
                    // Swipe right: Previous command
                    navigateHistory(-1);
                } else {
                    // Swipe left: Next command
                    navigateHistory(1);
                }
            }
        }, { passive: true });
    }

    // ===== History Navigation =====
    function navigateHistory(direction) {
        const input = document.querySelector('#input-container input');
        if (!input) return;

        // Get history from localStorage or global
        let history = [];
        try {
            const stored = localStorage.getItem('realconsole_history');
            if (stored) {
                history = JSON.parse(stored);
            }
        } catch (e) {
            console.warn('[Mobile] Failed to load history:', e);
        }

        if (history.length === 0) {
            showToast('No command history');
            return;
        }

        // Navigate history
        const newIndex = state.historyIndex + direction;

        if (newIndex < 0) {
            state.historyIndex = -1;
            input.value = '';
            showToast('End of history');
        } else if (newIndex >= history.length) {
            showToast('Start of history');
        } else {
            state.historyIndex = newIndex;
            input.value = history[history.length - 1 - newIndex];
            input.setSelectionRange(input.value.length, input.value.length);
            showToast(`History: ${state.historyIndex + 1}/${history.length}`);
        }
    }

    // ===== Toast Notification =====
    function showToast(message) {
        let toast = document.getElementById('mobile-toast');
        if (!toast) {
            toast = document.createElement('div');
            toast.id = 'mobile-toast';
            toast.style.cssText = `
                position: fixed;
                bottom: 80px;
                left: 50%;
                transform: translateX(-50%);
                background: rgba(0, 0, 0, 0.8);
                color: white;
                padding: 10px 20px;
                border-radius: 20px;
                font-size: 14px;
                z-index: 10000;
                pointer-events: none;
                opacity: 0;
                transition: opacity 0.3s ease;
            `;
            document.body.appendChild(toast);
        }

        toast.textContent = message;
        toast.style.opacity = '1';

        setTimeout(() => {
            toast.style.opacity = '0';
        }, 1500);
    }

    // ===== Pinch-to-Zoom for Charts =====
    function setupChartZoom() {
        // Monitor for chart containers
        const observer = new MutationObserver(function(mutations) {
            mutations.forEach(function(mutation) {
                mutation.addedNodes.forEach(function(node) {
                    if (node.classList && node.classList.contains('chart-container')) {
                        enableChartZoom(node);
                    }
                    // Also check children
                    if (node.querySelectorAll) {
                        node.querySelectorAll('.chart-container').forEach(enableChartZoom);
                    }
                });
            });
        });

        observer.observe(document.body, {
            childList: true,
            subtree: true
        });

        // Enable zoom on existing charts
        document.querySelectorAll('.chart-container').forEach(enableChartZoom);
    }

    function enableChartZoom(container) {
        if (container.dataset.zoomEnabled) return;
        container.dataset.zoomEnabled = 'true';

        let scale = 1;
        let lastDistance = 0;

        container.addEventListener('touchstart', function(e) {
            if (e.touches.length === 2) {
                lastDistance = getDistance(e.touches[0], e.touches[1]);
            }
        }, { passive: true });

        container.addEventListener('touchmove', function(e) {
            if (e.touches.length === 2) {
                e.preventDefault();

                const currentDistance = getDistance(e.touches[0], e.touches[1]);
                const delta = currentDistance / lastDistance;

                scale *= delta;
                scale = Math.min(Math.max(0.5, scale), 3); // Limit scale

                container.style.transform = `scale(${scale})`;
                lastDistance = currentDistance;
            }
        }, { passive: false });

        container.addEventListener('touchend', function(e) {
            if (e.touches.length < 2) {
                // Optionally reset scale
                // scale = 1;
                // container.style.transform = '';
            }
        }, { passive: true });

        // Double-tap to reset
        let lastTap = 0;
        container.addEventListener('touchend', function(e) {
            const currentTime = new Date().getTime();
            const tapLength = currentTime - lastTap;
            if (tapLength < 300 && tapLength > 0) {
                scale = 1;
                container.style.transform = '';
                showToast('Zoom reset');
            }
            lastTap = currentTime;
        });
    }

    function getDistance(touch1, touch2) {
        const dx = touch1.clientX - touch2.clientX;
        const dy = touch1.clientY - touch2.clientY;
        return Math.sqrt(dx * dx + dy * dy);
    }

    // ===== Touch Feedback =====
    function setupTouchFeedback() {
        document.addEventListener('touchstart', function(e) {
            if (e.target.tagName === 'BUTTON' || e.target.classList.contains('btn')) {
                e.target.style.transform = 'scale(0.95)';
            }
        }, { passive: true });

        document.addEventListener('touchend', function(e) {
            if (e.target.tagName === 'BUTTON' || e.target.classList.contains('btn')) {
                e.target.style.transform = '';
            }
        }, { passive: true });
    }

    // ===== Prevent Double-Tap Zoom =====
    function preventDoubleTapZoom() {
        let lastTouchEnd = 0;
        document.addEventListener('touchend', function(e) {
            const now = Date.now();
            if (now - lastTouchEnd <= 300) {
                // Only prevent on buttons and inputs
                if (e.target.tagName === 'BUTTON' || e.target.tagName === 'INPUT') {
                    e.preventDefault();
                }
            }
            lastTouchEnd = now;
        }, { passive: false });
    }

    // ===== Save Command to History =====
    function setupHistorySaving() {
        const input = document.querySelector('#input-container input');
        if (!input) return;

        input.addEventListener('keydown', function(e) {
            if (e.key === 'Enter' && input.value.trim()) {
                try {
                    let history = JSON.parse(localStorage.getItem('realconsole_history') || '[]');
                    // Avoid duplicates
                    if (history[history.length - 1] !== input.value.trim()) {
                        history.push(input.value.trim());
                        // Limit history size
                        if (history.length > 100) {
                            history = history.slice(-100);
                        }
                        localStorage.setItem('realconsole_history', JSON.stringify(history));
                    }
                    state.historyIndex = -1;
                } catch (e) {
                    console.warn('[Mobile] Failed to save history:', e);
                }
            }
        });
    }

    // ===== Initialize =====
    function init() {
        console.log('[Mobile] Initializing mobile enhancements');

        setupKeyboardDetection();
        setupSwipeGestures();
        setupChartZoom();
        setupTouchFeedback();
        preventDoubleTapZoom();
        setupHistorySaving();

        // Add mobile class to body
        document.body.classList.add('mobile-device');

        console.log('[Mobile] Mobile enhancements initialized');
    }

    // Run on DOM ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mobile_css_not_empty() {
        let css = get_mobile_css();
        assert!(!css.is_empty());
        assert!(css.contains("@media"));
        assert!(css.contains("touch"));
    }

    #[test]
    fn test_mobile_js_not_empty() {
        let js = get_mobile_js();
        assert!(!js.is_empty());
        assert!(js.contains("touchstart"));
        assert!(js.contains("swipe"));
    }

    #[test]
    fn test_mobile_css_has_touch_targets() {
        let css = get_mobile_css();
        assert!(css.contains("min-height: 44px"));
        assert!(css.contains("min-width: 44px"));
    }

    #[test]
    fn test_mobile_css_has_safe_area() {
        let css = get_mobile_css();
        assert!(css.contains("safe-area-inset"));
    }

    #[test]
    fn test_mobile_js_has_gesture_support() {
        let js = get_mobile_js();
        assert!(js.contains("setupSwipeGestures"));
        assert!(js.contains("setupChartZoom"));
        assert!(js.contains("getDistance")); // For pinch-zoom calculation
    }

    #[test]
    fn test_mobile_js_has_keyboard_detection() {
        let js = get_mobile_js();
        assert!(js.contains("setupKeyboardDetection"));
        assert!(js.contains("keyboard-open"));
    }

    #[test]
    fn test_mobile_js_has_history_navigation() {
        let js = get_mobile_js();
        assert!(js.contains("navigateHistory"));
        assert!(js.contains("realconsole_history"));
    }
}
