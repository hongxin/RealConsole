//! Virtual Scrolling System - v1.98.0
//!
//! Provides efficient rendering for large outputs:
//! - Only renders visible items in viewport
//! - DOM element recycling for memory efficiency
//! - Lazy loading of off-screen content
//! - Smooth scrolling with buffer zones

use serde::{Deserialize, Serialize};

/// Configuration for virtual scrolling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualScrollConfig {
    /// Estimated height of each item in pixels
    pub item_height: u32,
    /// Number of items to render above/below viewport
    pub buffer_size: u32,
    /// Maximum items to keep in DOM
    pub max_dom_nodes: u32,
    /// Enable smooth scrolling
    pub smooth_scroll: bool,
    /// Threshold for triggering lazy load (pixels from edge)
    pub lazy_load_threshold: u32,
}

impl Default for VirtualScrollConfig {
    fn default() -> Self {
        Self {
            item_height: 24,
            buffer_size: 10,
            max_dom_nodes: 200,
            smooth_scroll: true,
            lazy_load_threshold: 100,
        }
    }
}

/// Viewport state for virtual scrolling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportState {
    /// First visible item index
    pub start_index: usize,
    /// Last visible item index
    pub end_index: usize,
    /// Scroll position in pixels
    pub scroll_top: f64,
    /// Viewport height in pixels
    pub viewport_height: f64,
    /// Total content height in pixels
    pub total_height: f64,
    /// Total number of items
    pub total_items: usize,
}

impl ViewportState {
    /// Create new viewport state
    pub fn new(total_items: usize, item_height: u32, viewport_height: f64) -> Self {
        let total_height = total_items as f64 * item_height as f64;
        let visible_count = (viewport_height / item_height as f64).ceil() as usize;

        Self {
            start_index: 0,
            end_index: visible_count.min(total_items),
            scroll_top: 0.0,
            viewport_height,
            total_height,
            total_items,
        }
    }

    /// Update viewport based on scroll position
    pub fn update_scroll(&mut self, scroll_top: f64, item_height: u32, buffer_size: u32) {
        self.scroll_top = scroll_top;

        let item_height_f = item_height as f64;
        let buffer = buffer_size as usize;

        // Calculate visible range
        let first_visible = (scroll_top / item_height_f).floor() as usize;
        let visible_count = (self.viewport_height / item_height_f).ceil() as usize;

        // Apply buffer
        self.start_index = first_visible.saturating_sub(buffer);
        self.end_index = (first_visible + visible_count + buffer).min(self.total_items);
    }

    /// Get the range of items to render
    pub fn render_range(&self) -> std::ops::Range<usize> {
        self.start_index..self.end_index
    }

    /// Calculate offset for first rendered item
    pub fn offset_top(&self, item_height: u32) -> f64 {
        self.start_index as f64 * item_height as f64
    }

    /// Check if scrolled to bottom
    pub fn is_at_bottom(&self, threshold: f64) -> bool {
        self.scroll_top + self.viewport_height >= self.total_height - threshold
    }

    /// Check if scrolled to top
    pub fn is_at_top(&self, threshold: f64) -> bool {
        self.scroll_top <= threshold
    }
}

/// DOM pool for recycling elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomPoolConfig {
    /// Pool name/identifier
    pub name: String,
    /// Maximum elements in pool
    pub max_size: usize,
    /// Element tag name
    pub tag_name: String,
    /// CSS class for pooled elements
    pub css_class: String,
}

impl Default for DomPoolConfig {
    fn default() -> Self {
        Self {
            name: "output-lines".to_string(),
            max_size: 100,
            tag_name: "div".to_string(),
            css_class: "virtual-item".to_string(),
        }
    }
}

/// Statistics for virtual scroll performance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VirtualScrollStats {
    /// Total items managed
    pub total_items: usize,
    /// Items currently in DOM
    pub rendered_items: usize,
    /// Items recycled from pool
    pub recycled_count: u64,
    /// Items newly created
    pub created_count: u64,
    /// Scroll events processed
    pub scroll_events: u64,
    /// Average render time in ms
    pub avg_render_time_ms: f64,
}

impl VirtualScrollStats {
    /// Calculate memory savings percentage
    pub fn memory_savings(&self) -> f64 {
        if self.total_items == 0 {
            return 0.0;
        }
        let saved = self.total_items.saturating_sub(self.rendered_items);
        (saved as f64 / self.total_items as f64) * 100.0
    }
}

/// Virtual scroll CSS styles
pub const VIRTUAL_SCROLL_CSS: &str = r#"
/* ============================================
   Virtual Scrolling v1.98.0
   Efficient large output rendering
   ============================================ */

/* Virtual scroll container */
.virtual-scroll-container {
    position: relative;
    overflow-y: auto;
    overflow-x: hidden;
    will-change: transform;
    contain: strict;
}

/* Spacer to maintain scroll height */
.virtual-scroll-spacer {
    position: absolute;
    top: 0;
    left: 0;
    width: 1px;
    pointer-events: none;
    visibility: hidden;
}

/* Content wrapper with transform for positioning */
.virtual-scroll-content {
    position: relative;
    will-change: transform;
}

/* Individual virtual items */
.virtual-item {
    position: absolute;
    left: 0;
    right: 0;
    contain: layout style;
    will-change: transform;
}

/* Recycled item (ready for reuse) */
.virtual-item.pooled {
    display: none;
    visibility: hidden;
}

/* Loading placeholder */
.virtual-item.loading {
    opacity: 0.5;
    background: var(--bg-tertiary, #1a0b2e);
    animation: virtualItemPulse 1.5s ease-in-out infinite;
}

@keyframes virtualItemPulse {
    0%, 100% { opacity: 0.3; }
    50% { opacity: 0.6; }
}

/* Smooth scroll behavior */
.virtual-scroll-container.smooth-scroll {
    scroll-behavior: smooth;
}

/* Performance optimizations */
.virtual-scroll-container * {
    backface-visibility: hidden;
}

/* Scroll indicators */
.virtual-scroll-indicator {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    padding: 4px 12px;
    background: var(--bg-secondary, #0d1117);
    border: 1px solid var(--border, #30363d);
    border-radius: 12px;
    font-size: 12px;
    color: var(--text-secondary, #8b949e);
    opacity: 0;
    transition: opacity 0.2s ease;
    pointer-events: none;
    z-index: 10;
}

.virtual-scroll-indicator.top {
    top: 8px;
}

.virtual-scroll-indicator.bottom {
    bottom: 8px;
}

.virtual-scroll-container.scrolling .virtual-scroll-indicator {
    opacity: 1;
}

/* Light theme */
[data-theme="light"] .virtual-item.loading {
    background: #f0f0f0;
}

[data-theme="light"] .virtual-scroll-indicator {
    background: #ffffff;
    border-color: #d0d7de;
}
"#;

/// Virtual scroll JavaScript implementation
pub const VIRTUAL_SCROLL_JS: &str = r#"
// ============================================
// Virtual Scrolling v1.98.0
// Efficient large output rendering
// ============================================

(function() {
    'use strict';

    // Configuration
    const DEFAULT_CONFIG = {
        itemHeight: 24,
        bufferSize: 10,
        maxDomNodes: 200,
        smoothScroll: true,
        lazyLoadThreshold: 100,
        scrollDebounce: 16 // ~60fps
    };

    /**
     * Virtual Scroll Manager
     */
    class VirtualScrollManager {
        constructor(container, config = {}) {
            this.container = container;
            this.config = { ...DEFAULT_CONFIG, ...config };
            this.items = [];
            this.renderedItems = new Map(); // index -> element
            this.pool = []; // Recycled elements
            this.viewport = {
                startIndex: 0,
                endIndex: 0,
                scrollTop: 0
            };
            this.stats = {
                totalItems: 0,
                renderedItems: 0,
                recycledCount: 0,
                createdCount: 0,
                scrollEvents: 0
            };
            this.scrollTimeout = null;
            this.isScrolling = false;

            this.init();
        }

        init() {
            // Create structure
            this.spacer = document.createElement('div');
            this.spacer.className = 'virtual-scroll-spacer';

            this.content = document.createElement('div');
            this.content.className = 'virtual-scroll-content';

            this.container.classList.add('virtual-scroll-container');
            if (this.config.smoothScroll) {
                this.container.classList.add('smooth-scroll');
            }

            this.container.appendChild(this.spacer);
            this.container.appendChild(this.content);

            // Bind scroll handler
            this.handleScroll = this.handleScroll.bind(this);
            this.container.addEventListener('scroll', this.handleScroll, { passive: true });

            // Observe resize
            if (window.ResizeObserver) {
                this.resizeObserver = new ResizeObserver(() => this.updateViewport());
                this.resizeObserver.observe(this.container);
            }
        }

        handleScroll() {
            this.stats.scrollEvents++;
            this.viewport.scrollTop = this.container.scrollTop;

            // Debounce scroll updates
            if (this.scrollTimeout) {
                return;
            }

            this.scrollTimeout = setTimeout(() => {
                this.scrollTimeout = null;
                this.updateViewport();
            }, this.config.scrollDebounce);

            // Show scrolling state
            if (!this.isScrolling) {
                this.isScrolling = true;
                this.container.classList.add('scrolling');
            }

            clearTimeout(this.scrollEndTimeout);
            this.scrollEndTimeout = setTimeout(() => {
                this.isScrolling = false;
                this.container.classList.remove('scrolling');
            }, 150);
        }

        updateViewport() {
            const containerHeight = this.container.clientHeight;
            const scrollTop = this.container.scrollTop;
            const itemHeight = this.config.itemHeight;
            const buffer = this.config.bufferSize;

            // Calculate visible range
            const firstVisible = Math.floor(scrollTop / itemHeight);
            const visibleCount = Math.ceil(containerHeight / itemHeight);

            const startIndex = Math.max(0, firstVisible - buffer);
            const endIndex = Math.min(this.items.length, firstVisible + visibleCount + buffer);

            // Only re-render if range changed
            if (startIndex !== this.viewport.startIndex || endIndex !== this.viewport.endIndex) {
                this.viewport.startIndex = startIndex;
                this.viewport.endIndex = endIndex;
                this.render();
            }
        }

        render() {
            const { startIndex, endIndex } = this.viewport;
            const itemHeight = this.config.itemHeight;

            // Recycle elements outside viewport
            for (const [index, element] of this.renderedItems) {
                if (index < startIndex || index >= endIndex) {
                    this.recycleElement(element);
                    this.renderedItems.delete(index);
                }
            }

            // Render items in viewport
            for (let i = startIndex; i < endIndex; i++) {
                if (!this.renderedItems.has(i)) {
                    const element = this.getElement();
                    this.updateElement(element, i);
                    element.style.transform = `translateY(${i * itemHeight}px)`;
                    this.content.appendChild(element);
                    this.renderedItems.set(i, element);
                }
            }

            this.stats.renderedItems = this.renderedItems.size;

            // Dispatch render event
            this.container.dispatchEvent(new CustomEvent('virtualrender', {
                detail: {
                    startIndex,
                    endIndex,
                    renderedCount: this.renderedItems.size,
                    totalItems: this.items.length
                }
            }));
        }

        getElement() {
            if (this.pool.length > 0) {
                this.stats.recycledCount++;
                const element = this.pool.pop();
                element.classList.remove('pooled');
                return element;
            }

            this.stats.createdCount++;
            const element = document.createElement('div');
            element.className = 'virtual-item';
            return element;
        }

        recycleElement(element) {
            if (this.pool.length < this.config.maxDomNodes) {
                element.classList.add('pooled');
                element.innerHTML = '';
                this.pool.push(element);
            } else {
                element.remove();
            }
        }

        updateElement(element, index) {
            const item = this.items[index];
            if (item) {
                if (typeof item === 'string') {
                    element.innerHTML = item;
                } else if (item.html) {
                    element.innerHTML = item.html;
                } else if (item.render) {
                    item.render(element, index);
                }
                element.dataset.index = index;
            }
        }

        setItems(items) {
            this.items = items;
            this.stats.totalItems = items.length;

            // Update spacer height
            const totalHeight = items.length * this.config.itemHeight;
            this.spacer.style.height = `${totalHeight}px`;

            // Force re-render
            this.viewport.startIndex = -1;
            this.updateViewport();
        }

        appendItem(item) {
            this.items.push(item);
            this.stats.totalItems = this.items.length;

            // Update spacer height
            const totalHeight = this.items.length * this.config.itemHeight;
            this.spacer.style.height = `${totalHeight}px`;

            // Check if should auto-scroll
            const wasAtBottom = this.isAtBottom();

            this.updateViewport();

            if (wasAtBottom) {
                this.scrollToBottom();
            }
        }

        isAtBottom(threshold = 50) {
            const { scrollTop, scrollHeight, clientHeight } = this.container;
            return scrollHeight - scrollTop - clientHeight <= threshold;
        }

        scrollToBottom() {
            this.container.scrollTop = this.container.scrollHeight;
        }

        scrollToTop() {
            this.container.scrollTop = 0;
        }

        scrollToIndex(index) {
            const top = index * this.config.itemHeight;
            this.container.scrollTop = top;
        }

        getStats() {
            return {
                ...this.stats,
                poolSize: this.pool.length,
                memorySavings: this.stats.totalItems > 0
                    ? ((this.stats.totalItems - this.stats.renderedItems) / this.stats.totalItems * 100).toFixed(1)
                    : 0
            };
        }

        destroy() {
            this.container.removeEventListener('scroll', this.handleScroll);
            if (this.resizeObserver) {
                this.resizeObserver.disconnect();
            }
            this.renderedItems.clear();
            this.pool = [];
            this.content.remove();
            this.spacer.remove();
        }
    }

    // Expose globally
    window.VirtualScrollManager = VirtualScrollManager;

    // Auto-initialize for marked containers
    document.addEventListener('DOMContentLoaded', () => {
        document.querySelectorAll('[data-virtual-scroll]').forEach(container => {
            const config = JSON.parse(container.dataset.virtualScroll || '{}');
            container._virtualScroll = new VirtualScrollManager(container, config);
        });
    });
})();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = VirtualScrollConfig::default();
        assert_eq!(config.item_height, 24);
        assert_eq!(config.buffer_size, 10);
        assert_eq!(config.max_dom_nodes, 200);
        assert!(config.smooth_scroll);
    }

    #[test]
    fn test_viewport_state_new() {
        let state = ViewportState::new(100, 24, 480.0);
        assert_eq!(state.start_index, 0);
        assert_eq!(state.total_items, 100);
        assert_eq!(state.total_height, 2400.0);
    }

    #[test]
    fn test_viewport_update_scroll() {
        let mut state = ViewportState::new(100, 24, 480.0);
        state.update_scroll(240.0, 24, 5);

        // After scrolling 240px (10 items), first visible is index 10
        // With buffer 5, start should be 5
        assert_eq!(state.start_index, 5);
        assert!(state.end_index > state.start_index);
    }

    #[test]
    fn test_viewport_render_range() {
        let state = ViewportState::new(100, 24, 480.0);
        let range = state.render_range();
        assert_eq!(range.start, 0);
        assert!(range.end > 0);
    }

    #[test]
    fn test_viewport_offset_top() {
        let mut state = ViewportState::new(100, 24, 480.0);
        state.start_index = 10;
        let offset = state.offset_top(24);
        assert_eq!(offset, 240.0);
    }

    #[test]
    fn test_viewport_is_at_bottom() {
        let mut state = ViewportState::new(100, 24, 480.0);
        state.scroll_top = 0.0;
        assert!(!state.is_at_bottom(50.0));

        state.scroll_top = state.total_height - state.viewport_height;
        assert!(state.is_at_bottom(50.0));
    }

    #[test]
    fn test_viewport_is_at_top() {
        let mut state = ViewportState::new(100, 24, 480.0);
        assert!(state.is_at_top(10.0));

        state.scroll_top = 100.0;
        assert!(!state.is_at_top(10.0));
    }

    #[test]
    fn test_dom_pool_config() {
        let config = DomPoolConfig::default();
        assert_eq!(config.name, "output-lines");
        assert_eq!(config.max_size, 100);
        assert_eq!(config.tag_name, "div");
    }

    #[test]
    fn test_stats_default() {
        let stats = VirtualScrollStats::default();
        assert_eq!(stats.total_items, 0);
        assert_eq!(stats.rendered_items, 0);
        assert_eq!(stats.recycled_count, 0);
    }

    #[test]
    fn test_stats_memory_savings() {
        let mut stats = VirtualScrollStats::default();
        stats.total_items = 1000;
        stats.rendered_items = 50;

        let savings = stats.memory_savings();
        assert!((savings - 95.0).abs() < 0.1);
    }

    #[test]
    fn test_stats_memory_savings_zero_items() {
        let stats = VirtualScrollStats::default();
        assert_eq!(stats.memory_savings(), 0.0);
    }

    #[test]
    fn test_css_not_empty() {
        assert!(!VIRTUAL_SCROLL_CSS.is_empty());
        assert!(VIRTUAL_SCROLL_CSS.contains("virtual-scroll-container"));
    }

    #[test]
    fn test_css_has_spacer() {
        assert!(VIRTUAL_SCROLL_CSS.contains("virtual-scroll-spacer"));
    }

    #[test]
    fn test_css_has_content() {
        assert!(VIRTUAL_SCROLL_CSS.contains("virtual-scroll-content"));
    }

    #[test]
    fn test_js_not_empty() {
        assert!(!VIRTUAL_SCROLL_JS.is_empty());
        assert!(VIRTUAL_SCROLL_JS.contains("VirtualScrollManager"));
    }

    #[test]
    fn test_js_has_scroll_handler() {
        assert!(VIRTUAL_SCROLL_JS.contains("handleScroll"));
    }

    #[test]
    fn test_js_has_recycling() {
        assert!(VIRTUAL_SCROLL_JS.contains("recycleElement"));
        assert!(VIRTUAL_SCROLL_JS.contains("getElement"));
    }

    #[test]
    fn test_js_has_viewport_update() {
        assert!(VIRTUAL_SCROLL_JS.contains("updateViewport"));
    }

    #[test]
    fn test_js_exposes_global() {
        assert!(VIRTUAL_SCROLL_JS.contains("window.VirtualScrollManager"));
    }
}
