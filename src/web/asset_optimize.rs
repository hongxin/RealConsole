//! Asset Optimization - v1.100.0
//!
//! Provides asset optimization for web performance:
//! - Caching strategies with ETags and Cache-Control
//! - Asset compression (gzip, brotli)
//! - Lazy loading for deferred resources
//! - Code splitting support

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Asset type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetType {
    /// JavaScript files
    JavaScript,
    /// CSS stylesheets
    Css,
    /// Image files
    Image,
    /// Font files
    Font,
    /// JSON data
    Json,
    /// HTML documents
    Html,
    /// Other assets
    Other,
}

impl AssetType {
    /// Get MIME type for asset
    pub fn mime_type(&self) -> &'static str {
        match self {
            AssetType::JavaScript => "application/javascript",
            AssetType::Css => "text/css",
            AssetType::Image => "image/png",
            AssetType::Font => "font/woff2",
            AssetType::Json => "application/json",
            AssetType::Html => "text/html",
            AssetType::Other => "application/octet-stream",
        }
    }

    /// Check if asset is compressible
    pub fn is_compressible(&self) -> bool {
        matches!(
            self,
            AssetType::JavaScript | AssetType::Css | AssetType::Json | AssetType::Html
        )
    }

    /// Detect asset type from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "js" | "mjs" => AssetType::JavaScript,
            "css" => AssetType::Css,
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "ico" => AssetType::Image,
            "woff" | "woff2" | "ttf" | "otf" | "eot" => AssetType::Font,
            "json" => AssetType::Json,
            "html" | "htm" => AssetType::Html,
            _ => AssetType::Other,
        }
    }
}

/// Caching strategy for assets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CacheStrategy {
    /// No caching
    NoCache,
    /// Short-term cache (5 minutes)
    ShortTerm,
    /// Medium-term cache (1 hour)
    MediumTerm,
    /// Long-term cache (1 day)
    LongTerm,
    /// Immutable cache (1 year, for versioned assets)
    Immutable,
}

impl CacheStrategy {
    /// Get max-age in seconds
    pub fn max_age_secs(&self) -> u64 {
        match self {
            CacheStrategy::NoCache => 0,
            CacheStrategy::ShortTerm => 300,        // 5 minutes
            CacheStrategy::MediumTerm => 3600,      // 1 hour
            CacheStrategy::LongTerm => 86400,       // 1 day
            CacheStrategy::Immutable => 31536000,   // 1 year
        }
    }

    /// Get Cache-Control header value
    pub fn cache_control(&self) -> String {
        match self {
            CacheStrategy::NoCache => "no-cache, no-store, must-revalidate".to_string(),
            CacheStrategy::ShortTerm => format!("public, max-age={}", self.max_age_secs()),
            CacheStrategy::MediumTerm => format!("public, max-age={}", self.max_age_secs()),
            CacheStrategy::LongTerm => format!("public, max-age={}", self.max_age_secs()),
            CacheStrategy::Immutable => {
                format!("public, max-age={}, immutable", self.max_age_secs())
            }
        }
    }

    /// Get recommended strategy for asset type
    pub fn for_asset_type(asset_type: AssetType) -> Self {
        match asset_type {
            AssetType::Html => CacheStrategy::NoCache,
            AssetType::Json => CacheStrategy::ShortTerm,
            AssetType::JavaScript | AssetType::Css => CacheStrategy::LongTerm,
            AssetType::Image | AssetType::Font => CacheStrategy::Immutable,
            AssetType::Other => CacheStrategy::MediumTerm,
        }
    }
}

/// Asset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    /// Asset path/URL
    pub path: String,
    /// Asset type
    pub asset_type: AssetType,
    /// Original size in bytes
    pub original_size: usize,
    /// Compressed size (if applicable)
    pub compressed_size: Option<usize>,
    /// ETag for caching
    pub etag: Option<String>,
    /// Cache strategy
    pub cache_strategy: CacheStrategy,
    /// Whether asset supports lazy loading
    pub lazy_loadable: bool,
    /// Dependencies (for code splitting)
    pub dependencies: Vec<String>,
}

impl AssetMetadata {
    /// Create new asset metadata
    pub fn new(path: &str, asset_type: AssetType, size: usize) -> Self {
        Self {
            path: path.to_string(),
            asset_type,
            original_size: size,
            compressed_size: None,
            etag: None,
            cache_strategy: CacheStrategy::for_asset_type(asset_type),
            lazy_loadable: matches!(asset_type, AssetType::JavaScript | AssetType::Css | AssetType::Image),
            dependencies: Vec::new(),
        }
    }

    /// Set ETag from content hash
    pub fn with_etag(mut self, hash: &str) -> Self {
        self.etag = Some(format!("\"{}\"", hash));
        self
    }

    /// Set compressed size
    pub fn with_compressed_size(mut self, size: usize) -> Self {
        self.compressed_size = Some(size);
        self
    }

    /// Add dependency
    pub fn with_dependency(mut self, dep: &str) -> Self {
        self.dependencies.push(dep.to_string());
        self
    }

    /// Calculate compression ratio
    pub fn compression_ratio(&self) -> Option<f64> {
        self.compressed_size.map(|compressed| {
            if self.original_size > 0 {
                compressed as f64 / self.original_size as f64
            } else {
                1.0
            }
        })
    }

    /// Get bytes saved by compression
    pub fn bytes_saved(&self) -> usize {
        self.compressed_size
            .map(|c| self.original_size.saturating_sub(c))
            .unwrap_or(0)
    }
}

/// Configuration for asset optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetOptimizeConfig {
    /// Enable compression
    pub compression_enabled: bool,
    /// Minimum size for compression (bytes)
    pub compression_threshold: usize,
    /// Preferred compression algorithm
    pub compression_algorithm: CompressionAlgorithm,
    /// Enable lazy loading
    pub lazy_loading_enabled: bool,
    /// Enable code splitting
    pub code_splitting_enabled: bool,
    /// Enable ETags
    pub etag_enabled: bool,
    /// Default cache strategy
    pub default_cache_strategy: CacheStrategy,
}

impl Default for AssetOptimizeConfig {
    fn default() -> Self {
        Self {
            compression_enabled: true,
            compression_threshold: 1024, // 1KB
            compression_algorithm: CompressionAlgorithm::Gzip,
            lazy_loading_enabled: true,
            code_splitting_enabled: true,
            etag_enabled: true,
            default_cache_strategy: CacheStrategy::MediumTerm,
        }
    }
}

/// Compression algorithm options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompressionAlgorithm {
    /// No compression
    None,
    /// Gzip compression
    Gzip,
    /// Brotli compression
    Brotli,
    /// Deflate compression
    Deflate,
}

impl CompressionAlgorithm {
    /// Get Accept-Encoding value
    pub fn encoding(&self) -> &'static str {
        match self {
            CompressionAlgorithm::None => "identity",
            CompressionAlgorithm::Gzip => "gzip",
            CompressionAlgorithm::Brotli => "br",
            CompressionAlgorithm::Deflate => "deflate",
        }
    }

    /// Parse from Accept-Encoding header
    pub fn from_accept_encoding(header: &str) -> Self {
        let header_lower = header.to_lowercase();
        if header_lower.contains("br") {
            CompressionAlgorithm::Brotli
        } else if header_lower.contains("gzip") {
            CompressionAlgorithm::Gzip
        } else if header_lower.contains("deflate") {
            CompressionAlgorithm::Deflate
        } else {
            CompressionAlgorithm::None
        }
    }
}

/// Asset registry for managing assets
#[derive(Debug, Clone, Default)]
pub struct AssetRegistry {
    assets: HashMap<String, AssetMetadata>,
    config: AssetOptimizeConfig,
}

impl AssetRegistry {
    /// Create new registry with config
    pub fn new(config: AssetOptimizeConfig) -> Self {
        Self {
            assets: HashMap::new(),
            config,
        }
    }

    /// Register an asset
    pub fn register(&mut self, metadata: AssetMetadata) {
        self.assets.insert(metadata.path.clone(), metadata);
    }

    /// Get asset by path
    pub fn get(&self, path: &str) -> Option<&AssetMetadata> {
        self.assets.get(path)
    }

    /// List all assets
    pub fn list(&self) -> Vec<&AssetMetadata> {
        self.assets.values().collect()
    }

    /// List assets by type
    pub fn list_by_type(&self, asset_type: AssetType) -> Vec<&AssetMetadata> {
        self.assets
            .values()
            .filter(|a| a.asset_type == asset_type)
            .collect()
    }

    /// Get total original size
    pub fn total_original_size(&self) -> usize {
        self.assets.values().map(|a| a.original_size).sum()
    }

    /// Get total compressed size
    pub fn total_compressed_size(&self) -> usize {
        self.assets
            .values()
            .map(|a| a.compressed_size.unwrap_or(a.original_size))
            .sum()
    }

    /// Get total bytes saved
    pub fn total_bytes_saved(&self) -> usize {
        self.assets.values().map(|a| a.bytes_saved()).sum()
    }

    /// Generate HTTP headers for asset
    pub fn http_headers(&self, path: &str) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        if let Some(asset) = self.get(path) {
            headers.insert("Content-Type".to_string(), asset.asset_type.mime_type().to_string());
            headers.insert("Cache-Control".to_string(), asset.cache_strategy.cache_control());

            if let Some(etag) = &asset.etag {
                headers.insert("ETag".to_string(), etag.clone());
            }

            if asset.compressed_size.is_some() && self.config.compression_enabled {
                headers.insert(
                    "Content-Encoding".to_string(),
                    self.config.compression_algorithm.encoding().to_string(),
                );
            }
        }

        headers
    }
}

/// Statistics for asset optimization
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssetOptimizeStats {
    /// Total assets
    pub total_assets: usize,
    /// Total original size
    pub total_original_bytes: usize,
    /// Total compressed size
    pub total_compressed_bytes: usize,
    /// Bytes saved
    pub bytes_saved: usize,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Lazy loaded assets
    pub lazy_loaded: u64,
}

impl AssetOptimizeStats {
    /// Calculate overall compression ratio
    pub fn compression_ratio(&self) -> f64 {
        if self.total_original_bytes > 0 {
            self.total_compressed_bytes as f64 / self.total_original_bytes as f64
        } else {
            1.0
        }
    }

    /// Calculate cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            self.cache_hits as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Record cache hit
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    /// Record cache miss
    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }
}

/// Lazy loading CSS styles
pub const ASSET_OPTIMIZE_CSS: &str = r#"
/* ============================================
   Asset Optimization v1.100.0
   Lazy loading and performance UI
   ============================================ */

/* Lazy load container */
.lazy-load {
    opacity: 0;
    transition: opacity 0.3s ease;
}

.lazy-load.loaded {
    opacity: 1;
}

/* Lazy load placeholder */
.lazy-placeholder {
    background: var(--bg-tertiary, #1a0b2e);
    animation: lazyPulse 1.5s ease-in-out infinite;
    min-height: 100px;
    border-radius: 8px;
}

@keyframes lazyPulse {
    0%, 100% { opacity: 0.4; }
    50% { opacity: 0.7; }
}

/* Asset loading indicator */
.asset-loading {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 16px;
    color: var(--text-secondary, #8b949e);
    font-size: 14px;
}

.asset-loading-spinner {
    width: 20px;
    height: 20px;
    border: 2px solid var(--border, #30363d);
    border-top-color: var(--accent, #a371f7);
    border-radius: 50%;
    animation: assetSpin 0.8s linear infinite;
}

@keyframes assetSpin {
    to { transform: rotate(360deg); }
}

/* Performance metrics display */
.asset-metrics {
    display: flex;
    gap: 16px;
    padding: 12px 16px;
    background: var(--bg-secondary, #0d1117);
    border: 1px solid var(--border, #30363d);
    border-radius: 8px;
    font-size: 12px;
}

.asset-metric {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.asset-metric-label {
    color: var(--text-secondary, #8b949e);
}

.asset-metric-value {
    color: var(--text-primary, #e6edf3);
    font-weight: 500;
}

.asset-metric-value.good {
    color: var(--success, #0ecb81);
}

.asset-metric-value.warning {
    color: var(--warning, #f0b90b);
}

/* Code splitting chunk indicator */
.chunk-badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    background: var(--bg-tertiary, #1a0b2e);
    border-radius: 4px;
    font-size: 11px;
    color: var(--text-secondary, #8b949e);
}

.chunk-badge.loaded {
    background: rgba(14, 203, 129, 0.1);
    color: var(--success, #0ecb81);
}

/* Light theme */
[data-theme="light"] .lazy-placeholder {
    background: #f0f0f0;
}

[data-theme="light"] .asset-metrics {
    background: #ffffff;
    border-color: #d0d7de;
}

[data-theme="light"] .chunk-badge {
    background: #f6f8fa;
}
"#;

/// Asset optimization JavaScript
pub const ASSET_OPTIMIZE_JS: &str = r#"
// ============================================
// Asset Optimization v1.100.0
// Lazy loading and code splitting
// ============================================

(function() {
    'use strict';

    // Configuration
    const DEFAULT_CONFIG = {
        lazyLoadThreshold: 100,
        lazyLoadRootMargin: '100px',
        preloadEnabled: true,
        chunkTimeout: 10000
    };

    // Loaded chunks tracking
    const loadedChunks = new Set();
    const pendingChunks = new Map();

    /**
     * Lazy load manager using Intersection Observer
     */
    class LazyLoadManager {
        constructor(config = {}) {
            this.config = { ...DEFAULT_CONFIG, ...config };
            this.observer = null;
            this.stats = {
                observed: 0,
                loaded: 0,
                failed: 0
            };

            this.init();
        }

        init() {
            if ('IntersectionObserver' in window) {
                this.observer = new IntersectionObserver(
                    this.handleIntersection.bind(this),
                    {
                        rootMargin: this.config.lazyLoadRootMargin,
                        threshold: 0.01
                    }
                );

                // Observe existing lazy elements
                this.observeAll();

                // Watch for new elements
                this.watchDOM();
            } else {
                // Fallback: load all immediately
                this.loadAllImmediate();
            }
        }

        handleIntersection(entries) {
            entries.forEach(entry => {
                if (entry.isIntersecting) {
                    this.loadElement(entry.target);
                    this.observer.unobserve(entry.target);
                }
            });
        }

        loadElement(element) {
            const src = element.dataset.src;
            const type = element.dataset.type || 'image';

            if (!src) return;

            switch (type) {
                case 'image':
                    this.loadImage(element, src);
                    break;
                case 'script':
                    this.loadScript(src);
                    break;
                case 'style':
                    this.loadStyle(src);
                    break;
                default:
                    this.loadImage(element, src);
            }
        }

        loadImage(element, src) {
            const img = new Image();
            img.onload = () => {
                if (element.tagName === 'IMG') {
                    element.src = src;
                } else {
                    element.style.backgroundImage = `url(${src})`;
                }
                element.classList.remove('lazy-load');
                element.classList.add('loaded');
                this.stats.loaded++;
            };
            img.onerror = () => {
                element.classList.add('error');
                this.stats.failed++;
            };
            img.src = src;
        }

        loadScript(src) {
            return new Promise((resolve, reject) => {
                if (loadedChunks.has(src)) {
                    resolve();
                    return;
                }

                const script = document.createElement('script');
                script.src = src;
                script.async = true;
                script.onload = () => {
                    loadedChunks.add(src);
                    this.stats.loaded++;
                    resolve();
                };
                script.onerror = () => {
                    this.stats.failed++;
                    reject(new Error(`Failed to load: ${src}`));
                };
                document.head.appendChild(script);
            });
        }

        loadStyle(src) {
            return new Promise((resolve, reject) => {
                if (loadedChunks.has(src)) {
                    resolve();
                    return;
                }

                const link = document.createElement('link');
                link.rel = 'stylesheet';
                link.href = src;
                link.onload = () => {
                    loadedChunks.add(src);
                    this.stats.loaded++;
                    resolve();
                };
                link.onerror = () => {
                    this.stats.failed++;
                    reject(new Error(`Failed to load: ${src}`));
                };
                document.head.appendChild(link);
            });
        }

        observeAll() {
            document.querySelectorAll('[data-src], .lazy-load').forEach(el => {
                this.observe(el);
            });
        }

        observe(element) {
            if (this.observer) {
                this.observer.observe(element);
                this.stats.observed++;
            }
        }

        watchDOM() {
            const mutationObserver = new MutationObserver(mutations => {
                mutations.forEach(mutation => {
                    mutation.addedNodes.forEach(node => {
                        if (node.nodeType === 1) {
                            if (node.dataset && node.dataset.src) {
                                this.observe(node);
                            }
                            node.querySelectorAll && node.querySelectorAll('[data-src]').forEach(el => {
                                this.observe(el);
                            });
                        }
                    });
                });
            });

            mutationObserver.observe(document.body, {
                childList: true,
                subtree: true
            });
        }

        loadAllImmediate() {
            document.querySelectorAll('[data-src]').forEach(el => {
                this.loadElement(el);
            });
        }

        getStats() {
            return { ...this.stats };
        }
    }

    /**
     * Code splitting / chunk loader
     */
    class ChunkLoader {
        constructor() {
            this.loaded = loadedChunks;
            this.pending = pendingChunks;
        }

        async load(chunkName, url) {
            // Already loaded
            if (this.loaded.has(chunkName)) {
                return Promise.resolve();
            }

            // Already loading
            if (this.pending.has(chunkName)) {
                return this.pending.get(chunkName);
            }

            // Start loading
            const promise = this.fetchChunk(url)
                .then(() => {
                    this.loaded.add(chunkName);
                    this.pending.delete(chunkName);
                })
                .catch(err => {
                    this.pending.delete(chunkName);
                    throw err;
                });

            this.pending.set(chunkName, promise);
            return promise;
        }

        async fetchChunk(url) {
            return new Promise((resolve, reject) => {
                const script = document.createElement('script');
                script.src = url;
                script.async = true;

                const timeout = setTimeout(() => {
                    reject(new Error(`Chunk load timeout: ${url}`));
                }, DEFAULT_CONFIG.chunkTimeout);

                script.onload = () => {
                    clearTimeout(timeout);
                    resolve();
                };

                script.onerror = () => {
                    clearTimeout(timeout);
                    reject(new Error(`Failed to load chunk: ${url}`));
                };

                document.head.appendChild(script);
            });
        }

        isLoaded(chunkName) {
            return this.loaded.has(chunkName);
        }

        getLoadedChunks() {
            return Array.from(this.loaded);
        }
    }

    /**
     * Preload manager for critical resources
     */
    class PreloadManager {
        preload(url, as = 'script') {
            const link = document.createElement('link');
            link.rel = 'preload';
            link.href = url;
            link.as = as;
            document.head.appendChild(link);
        }

        prefetch(url) {
            const link = document.createElement('link');
            link.rel = 'prefetch';
            link.href = url;
            document.head.appendChild(link);
        }

        preconnect(origin) {
            const link = document.createElement('link');
            link.rel = 'preconnect';
            link.href = origin;
            document.head.appendChild(link);
        }
    }

    // Create instances
    const lazyLoader = new LazyLoadManager();
    const chunkLoader = new ChunkLoader();
    const preloader = new PreloadManager();

    // Expose globally
    window.AssetOptimize = {
        lazyLoader,
        chunkLoader,
        preloader,
        loadChunk: (name, url) => chunkLoader.load(name, url),
        preload: (url, as) => preloader.preload(url, as),
        prefetch: (url) => preloader.prefetch(url),
        getStats: () => lazyLoader.getStats()
    };
})();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_type_mime() {
        assert_eq!(AssetType::JavaScript.mime_type(), "application/javascript");
        assert_eq!(AssetType::Css.mime_type(), "text/css");
        assert_eq!(AssetType::Image.mime_type(), "image/png");
    }

    #[test]
    fn test_asset_type_compressible() {
        assert!(AssetType::JavaScript.is_compressible());
        assert!(AssetType::Css.is_compressible());
        assert!(!AssetType::Image.is_compressible());
        assert!(!AssetType::Font.is_compressible());
    }

    #[test]
    fn test_asset_type_from_extension() {
        assert_eq!(AssetType::from_extension("js"), AssetType::JavaScript);
        assert_eq!(AssetType::from_extension("CSS"), AssetType::Css);
        assert_eq!(AssetType::from_extension("png"), AssetType::Image);
        assert_eq!(AssetType::from_extension("woff2"), AssetType::Font);
        assert_eq!(AssetType::from_extension("xyz"), AssetType::Other);
    }

    #[test]
    fn test_cache_strategy_max_age() {
        assert_eq!(CacheStrategy::NoCache.max_age_secs(), 0);
        assert_eq!(CacheStrategy::ShortTerm.max_age_secs(), 300);
        assert_eq!(CacheStrategy::Immutable.max_age_secs(), 31536000);
    }

    #[test]
    fn test_cache_strategy_header() {
        let header = CacheStrategy::NoCache.cache_control();
        assert!(header.contains("no-cache"));

        let header = CacheStrategy::Immutable.cache_control();
        assert!(header.contains("immutable"));
    }

    #[test]
    fn test_cache_strategy_for_type() {
        assert_eq!(CacheStrategy::for_asset_type(AssetType::Html), CacheStrategy::NoCache);
        assert_eq!(CacheStrategy::for_asset_type(AssetType::Image), CacheStrategy::Immutable);
    }

    #[test]
    fn test_asset_metadata_new() {
        let meta = AssetMetadata::new("/app.js", AssetType::JavaScript, 5000);
        assert_eq!(meta.path, "/app.js");
        assert_eq!(meta.original_size, 5000);
        assert!(meta.lazy_loadable);
    }

    #[test]
    fn test_asset_metadata_with_etag() {
        let meta = AssetMetadata::new("/app.js", AssetType::JavaScript, 5000)
            .with_etag("abc123");
        assert_eq!(meta.etag, Some("\"abc123\"".to_string()));
    }

    #[test]
    fn test_asset_metadata_compression() {
        let meta = AssetMetadata::new("/app.js", AssetType::JavaScript, 1000)
            .with_compressed_size(400);

        assert_eq!(meta.bytes_saved(), 600);
        assert!((meta.compression_ratio().unwrap() - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_config_default() {
        let config = AssetOptimizeConfig::default();
        assert!(config.compression_enabled);
        assert!(config.lazy_loading_enabled);
        assert_eq!(config.compression_threshold, 1024);
    }

    #[test]
    fn test_compression_algorithm() {
        assert_eq!(CompressionAlgorithm::Gzip.encoding(), "gzip");
        assert_eq!(CompressionAlgorithm::Brotli.encoding(), "br");
    }

    #[test]
    fn test_compression_from_header() {
        assert_eq!(
            CompressionAlgorithm::from_accept_encoding("gzip, deflate, br"),
            CompressionAlgorithm::Brotli
        );
        assert_eq!(
            CompressionAlgorithm::from_accept_encoding("gzip, deflate"),
            CompressionAlgorithm::Gzip
        );
    }

    #[test]
    fn test_asset_registry_new() {
        let registry = AssetRegistry::new(AssetOptimizeConfig::default());
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_asset_registry_register() {
        let mut registry = AssetRegistry::new(AssetOptimizeConfig::default());
        let meta = AssetMetadata::new("/app.js", AssetType::JavaScript, 5000);
        registry.register(meta);

        assert!(registry.get("/app.js").is_some());
        assert_eq!(registry.list().len(), 1);
    }

    #[test]
    fn test_registry_list_by_type() {
        let mut registry = AssetRegistry::new(AssetOptimizeConfig::default());
        registry.register(AssetMetadata::new("/app.js", AssetType::JavaScript, 5000));
        registry.register(AssetMetadata::new("/style.css", AssetType::Css, 2000));
        registry.register(AssetMetadata::new("/vendor.js", AssetType::JavaScript, 3000));

        let js_assets = registry.list_by_type(AssetType::JavaScript);
        assert_eq!(js_assets.len(), 2);
    }

    #[test]
    fn test_registry_totals() {
        let mut registry = AssetRegistry::new(AssetOptimizeConfig::default());
        registry.register(
            AssetMetadata::new("/app.js", AssetType::JavaScript, 5000)
                .with_compressed_size(2000)
        );
        registry.register(
            AssetMetadata::new("/style.css", AssetType::Css, 3000)
                .with_compressed_size(1000)
        );

        assert_eq!(registry.total_original_size(), 8000);
        assert_eq!(registry.total_compressed_size(), 3000);
        assert_eq!(registry.total_bytes_saved(), 5000);
    }

    #[test]
    fn test_registry_http_headers() {
        let mut registry = AssetRegistry::new(AssetOptimizeConfig::default());
        registry.register(
            AssetMetadata::new("/app.js", AssetType::JavaScript, 5000)
                .with_etag("abc123")
                .with_compressed_size(2000)
        );

        let headers = registry.http_headers("/app.js");
        assert!(headers.contains_key("Content-Type"));
        assert!(headers.contains_key("Cache-Control"));
        assert!(headers.contains_key("ETag"));
    }

    #[test]
    fn test_stats_default() {
        let stats = AssetOptimizeStats::default();
        assert_eq!(stats.total_assets, 0);
        assert_eq!(stats.cache_hits, 0);
    }

    #[test]
    fn test_stats_compression_ratio() {
        let mut stats = AssetOptimizeStats::default();
        stats.total_original_bytes = 10000;
        stats.total_compressed_bytes = 4000;

        assert!((stats.compression_ratio() - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_stats_cache_hit_rate() {
        let mut stats = AssetOptimizeStats::default();
        stats.cache_hits = 80;
        stats.cache_misses = 20;

        assert!((stats.cache_hit_rate() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_css_not_empty() {
        assert!(!ASSET_OPTIMIZE_CSS.is_empty());
        assert!(ASSET_OPTIMIZE_CSS.contains("lazy-load"));
    }

    #[test]
    fn test_js_not_empty() {
        assert!(!ASSET_OPTIMIZE_JS.is_empty());
        assert!(ASSET_OPTIMIZE_JS.contains("LazyLoadManager"));
    }

    #[test]
    fn test_js_has_chunk_loader() {
        assert!(ASSET_OPTIMIZE_JS.contains("ChunkLoader"));
        assert!(ASSET_OPTIMIZE_JS.contains("loadChunk"));
    }

    #[test]
    fn test_js_has_preload() {
        assert!(ASSET_OPTIMIZE_JS.contains("PreloadManager"));
        assert!(ASSET_OPTIMIZE_JS.contains("preload"));
        assert!(ASSET_OPTIMIZE_JS.contains("prefetch"));
    }
}
