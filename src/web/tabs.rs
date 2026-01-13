//! Web 终端多标签支持
//!
//! v1.101.0 新增：支持多个会话标签页同时运行
//!
//! # 功能特性
//! - 标签管理器：创建、切换、关闭标签
//! - 标签状态：活跃/后台/关闭三态
//! - 跨标签会话隔离
//! - 标签持久化支持
//!
//! # 使用示例
//! ```ignore
//! use crate::web::tabs::{TabManager, TabConfig};
//!
//! let config = TabConfig::default();
//! let mut manager = TabManager::new(config);
//!
//! // 创建新标签
//! let tab_id = manager.create_tab(None);
//!
//! // 切换标签
//! manager.switch_to_tab(&tab_id);
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 标签 ID
pub type TabId = String;

/// 标签状态（一分为三：活跃、后台、关闭）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TabState {
    /// 活跃状态：当前正在显示和交互的标签
    #[default]
    Active,
    /// 后台状态：保持会话但不显示
    Background,
    /// 关闭状态：标签已关闭但可恢复
    Closed,
}

impl TabState {
    /// 是否可以执行命令
    pub fn can_execute(&self) -> bool {
        matches!(self, TabState::Active | TabState::Background)
    }

    /// 是否可以接收通知
    pub fn can_notify(&self) -> bool {
        matches!(self, TabState::Active | TabState::Background)
    }

    /// 状态转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            TabState::Active => "active",
            TabState::Background => "background",
            TabState::Closed => "closed",
        }
    }
}

/// 标签配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabConfig {
    /// 最大标签数
    pub max_tabs: usize,
    /// 最大后台标签数
    pub max_background_tabs: usize,
    /// 后台标签超时时间（秒）
    pub background_timeout_secs: u64,
    /// 自动保存后台标签
    pub auto_save_background: bool,
    /// 关闭标签时提示确认
    pub confirm_close: bool,
    /// 允许恢复已关闭的标签
    pub allow_restore: bool,
    /// 保留已关闭标签的数量
    pub closed_tabs_limit: usize,
}

impl Default for TabConfig {
    fn default() -> Self {
        Self {
            max_tabs: 10,
            max_background_tabs: 5,
            background_timeout_secs: 3600, // 1 hour
            auto_save_background: true,
            confirm_close: true,
            allow_restore: true,
            closed_tabs_limit: 5,
        }
    }
}

/// 标签元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabMetadata {
    /// 标签标题
    pub title: String,
    /// 标签图标（可选）
    pub icon: Option<String>,
    /// 工作目录
    pub working_dir: Option<String>,
    /// 是否有未读消息
    pub has_unread: bool,
    /// 是否正在执行命令
    pub is_executing: bool,
    /// 最后输入的命令
    pub last_command: Option<String>,
}

impl Default for TabMetadata {
    fn default() -> Self {
        Self {
            title: "New Tab".to_string(),
            icon: None,
            working_dir: None,
            has_unread: false,
            is_executing: false,
            last_command: None,
        }
    }
}

/// 单个标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    /// 标签 ID
    pub id: TabId,
    /// 关联的会话 ID
    pub session_id: String,
    /// 标签状态
    pub state: TabState,
    /// 标签元数据
    pub metadata: TabMetadata,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后活跃时间
    pub last_active_at: DateTime<Utc>,
    /// 标签索引（用于排序）
    pub index: usize,
}

impl Tab {
    /// 创建新标签
    pub fn new(session_id: String, index: usize) -> Self {
        let now = Utc::now();
        Self {
            id: format!("tab-{}", Uuid::new_v4()),
            session_id,
            state: TabState::Active,
            metadata: TabMetadata::default(),
            created_at: now,
            last_active_at: now,
            index,
        }
    }

    /// 带标题创建标签
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.metadata.title = title.into();
        self
    }

    /// 设置工作目录
    pub fn with_working_dir(mut self, dir: impl Into<String>) -> Self {
        self.metadata.working_dir = Some(dir.into());
        self
    }

    /// 激活标签
    pub fn activate(&mut self) {
        self.state = TabState::Active;
        self.last_active_at = Utc::now();
        self.metadata.has_unread = false;
    }

    /// 移到后台
    pub fn move_to_background(&mut self) {
        self.state = TabState::Background;
    }

    /// 关闭标签
    pub fn close(&mut self) {
        self.state = TabState::Closed;
    }

    /// 标记为有未读消息
    pub fn mark_unread(&mut self) {
        if self.state == TabState::Background {
            self.metadata.has_unread = true;
        }
    }

    /// 设置执行状态
    pub fn set_executing(&mut self, executing: bool) {
        self.metadata.is_executing = executing;
    }

    /// 更新最后命令
    pub fn update_last_command(&mut self, command: impl Into<String>) {
        self.metadata.last_command = Some(command.into());
        self.last_active_at = Utc::now();
    }
}

/// 标签事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TabEvent {
    /// 标签创建
    Created { tab_id: TabId },
    /// 标签激活
    Activated { tab_id: TabId },
    /// 标签移到后台
    Backgrounded { tab_id: TabId },
    /// 标签关闭
    Closed { tab_id: TabId },
    /// 标签恢复
    Restored { tab_id: TabId },
    /// 标签更新
    Updated { tab_id: TabId },
    /// 达到最大标签数
    MaxTabsReached { max: usize },
}

/// 标签管理器
#[derive(Debug)]
pub struct TabManager {
    /// 配置
    config: TabConfig,
    /// 所有标签
    tabs: HashMap<TabId, Tab>,
    /// 当前活跃标签 ID
    active_tab_id: Option<TabId>,
    /// 标签顺序（用于 UI 显示）
    tab_order: Vec<TabId>,
    /// 已关闭的标签（用于恢复）
    closed_tabs: Vec<Tab>,
    /// 下一个标签索引
    next_index: usize,
}

impl TabManager {
    /// 创建标签管理器
    pub fn new(config: TabConfig) -> Self {
        Self {
            config,
            tabs: HashMap::new(),
            active_tab_id: None,
            tab_order: Vec::new(),
            closed_tabs: Vec::new(),
            next_index: 0,
        }
    }

    /// 创建新标签
    pub fn create_tab(&mut self, session_id: Option<String>) -> Result<TabId, TabEvent> {
        // 检查是否达到最大标签数
        let active_count = self.tabs.values()
            .filter(|t| t.state != TabState::Closed)
            .count();

        if active_count >= self.config.max_tabs {
            return Err(TabEvent::MaxTabsReached { max: self.config.max_tabs });
        }

        // 将当前活跃标签移到后台
        if let Some(current_id) = &self.active_tab_id {
            if let Some(tab) = self.tabs.get_mut(current_id) {
                tab.move_to_background();
            }
        }

        // 创建新标签
        let session_id = session_id.unwrap_or_else(|| format!("session-{}", Uuid::new_v4()));
        let tab = Tab::new(session_id, self.next_index);
        let tab_id = tab.id.clone();

        self.next_index += 1;
        self.tab_order.push(tab_id.clone());
        self.tabs.insert(tab_id.clone(), tab);
        self.active_tab_id = Some(tab_id.clone());

        Ok(tab_id)
    }

    /// 切换到指定标签
    pub fn switch_to_tab(&mut self, tab_id: &TabId) -> Option<TabEvent> {
        // 检查标签是否存在
        if !self.tabs.contains_key(tab_id) {
            return None;
        }

        // 检查是否已关闭
        if let Some(tab) = self.tabs.get(tab_id) {
            if tab.state == TabState::Closed {
                return None;
            }
        }

        // 将当前活跃标签移到后台
        if let Some(current_id) = &self.active_tab_id {
            if current_id != tab_id {
                if let Some(tab) = self.tabs.get_mut(current_id) {
                    tab.move_to_background();
                }
            }
        }

        // 激活目标标签
        if let Some(tab) = self.tabs.get_mut(tab_id) {
            tab.activate();
        }

        self.active_tab_id = Some(tab_id.clone());
        Some(TabEvent::Activated { tab_id: tab_id.clone() })
    }

    /// 关闭标签
    pub fn close_tab(&mut self, tab_id: &TabId) -> Option<TabEvent> {
        let tab = self.tabs.get_mut(tab_id)?;

        // 标记为关闭
        tab.close();

        // 保存到已关闭列表
        if self.config.allow_restore {
            let closed_tab = tab.clone();
            self.closed_tabs.push(closed_tab);

            // 限制已关闭标签数量
            while self.closed_tabs.len() > self.config.closed_tabs_limit {
                self.closed_tabs.remove(0);
            }
        }

        // 从显示顺序中移除
        self.tab_order.retain(|id| id != tab_id);

        // 如果关闭的是活跃标签，切换到最后一个
        if self.active_tab_id.as_ref() == Some(tab_id) {
            self.active_tab_id = self.tab_order.last().cloned();
            if let Some(new_active_id) = &self.active_tab_id {
                if let Some(tab) = self.tabs.get_mut(new_active_id) {
                    tab.activate();
                }
            }
        }

        Some(TabEvent::Closed { tab_id: tab_id.clone() })
    }

    /// 恢复最近关闭的标签
    pub fn restore_last_closed(&mut self) -> Option<TabEvent> {
        if !self.config.allow_restore || self.closed_tabs.is_empty() {
            return None;
        }

        let mut tab = self.closed_tabs.pop()?;

        // 检查是否超过最大标签数
        let active_count = self.tabs.values()
            .filter(|t| t.state != TabState::Closed)
            .count();

        if active_count >= self.config.max_tabs {
            self.closed_tabs.push(tab);
            return None;
        }

        // 激活标签
        tab.activate();
        let tab_id = tab.id.clone();

        // 将当前活跃标签移到后台
        if let Some(current_id) = &self.active_tab_id {
            if let Some(current_tab) = self.tabs.get_mut(current_id) {
                current_tab.move_to_background();
            }
        }

        self.tab_order.push(tab_id.clone());
        self.tabs.insert(tab_id.clone(), tab);
        self.active_tab_id = Some(tab_id.clone());

        Some(TabEvent::Restored { tab_id })
    }

    /// 获取当前活跃标签
    pub fn active_tab(&self) -> Option<&Tab> {
        self.active_tab_id.as_ref().and_then(|id| self.tabs.get(id))
    }

    /// 获取当前活跃标签（可变）
    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        let id = self.active_tab_id.clone()?;
        self.tabs.get_mut(&id)
    }

    /// 获取所有活跃标签（按顺序）
    pub fn list_tabs(&self) -> Vec<&Tab> {
        self.tab_order
            .iter()
            .filter_map(|id| self.tabs.get(id))
            .filter(|tab| tab.state != TabState::Closed)
            .collect()
    }

    /// 获取标签数量
    pub fn tab_count(&self) -> usize {
        self.tabs.values()
            .filter(|t| t.state != TabState::Closed)
            .count()
    }

    /// 获取后台标签数量
    pub fn background_count(&self) -> usize {
        self.tabs.values()
            .filter(|t| t.state == TabState::Background)
            .count()
    }

    /// 获取可恢复的已关闭标签
    pub fn closed_tabs(&self) -> &[Tab] {
        &self.closed_tabs
    }

    /// 获取指定标签
    pub fn get_tab(&self, tab_id: &TabId) -> Option<&Tab> {
        self.tabs.get(tab_id)
    }

    /// 获取指定标签（可变）
    pub fn get_tab_mut(&mut self, tab_id: &TabId) -> Option<&mut Tab> {
        self.tabs.get_mut(tab_id)
    }

    /// 更新标签标题
    pub fn update_title(&mut self, tab_id: &TabId, title: impl Into<String>) -> Option<TabEvent> {
        let tab = self.tabs.get_mut(tab_id)?;
        tab.metadata.title = title.into();
        Some(TabEvent::Updated { tab_id: tab_id.clone() })
    }

    /// 标记标签有未读消息
    pub fn mark_unread(&mut self, tab_id: &TabId) {
        if let Some(tab) = self.tabs.get_mut(tab_id) {
            tab.mark_unread();
        }
    }

    /// 移动标签位置
    pub fn move_tab(&mut self, tab_id: &TabId, new_index: usize) {
        if let Some(pos) = self.tab_order.iter().position(|id| id == tab_id) {
            let id = self.tab_order.remove(pos);
            let new_pos = new_index.min(self.tab_order.len());
            self.tab_order.insert(new_pos, id);
        }
    }

    /// 下一个标签
    pub fn next_tab(&mut self) -> Option<TabEvent> {
        if self.tab_order.len() <= 1 {
            return None;
        }

        let current_pos = self.active_tab_id.as_ref()
            .and_then(|id| self.tab_order.iter().position(|t| t == id))
            .unwrap_or(0);

        let next_pos = (current_pos + 1) % self.tab_order.len();
        let next_id = self.tab_order[next_pos].clone();

        self.switch_to_tab(&next_id)
    }

    /// 上一个标签
    pub fn prev_tab(&mut self) -> Option<TabEvent> {
        if self.tab_order.len() <= 1 {
            return None;
        }

        let current_pos = self.active_tab_id.as_ref()
            .and_then(|id| self.tab_order.iter().position(|t| t == id))
            .unwrap_or(0);

        let prev_pos = if current_pos == 0 {
            self.tab_order.len() - 1
        } else {
            current_pos - 1
        };
        let prev_id = self.tab_order[prev_pos].clone();

        self.switch_to_tab(&prev_id)
    }

    /// 清理超时的后台标签
    pub fn cleanup_background_tabs(&mut self) -> Vec<TabEvent> {
        let timeout = chrono::Duration::seconds(self.config.background_timeout_secs as i64);
        let now = Utc::now();
        let mut events = Vec::new();

        let tabs_to_close: Vec<TabId> = self.tabs
            .iter()
            .filter(|(_, tab)| {
                tab.state == TabState::Background &&
                now.signed_duration_since(tab.last_active_at) > timeout
            })
            .map(|(id, _)| id.clone())
            .collect();

        for tab_id in tabs_to_close {
            if let Some(event) = self.close_tab(&tab_id) {
                events.push(event);
            }
        }

        events
    }
}

/// 标签列表信息（用于前端显示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabListInfo {
    /// 标签列表
    pub tabs: Vec<TabInfo>,
    /// 当前活跃标签 ID
    pub active_tab_id: Option<TabId>,
    /// 可恢复的已关闭标签数
    pub closed_count: usize,
}

/// 单个标签信息（轻量级）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    /// 标签 ID
    pub id: TabId,
    /// 标签标题
    pub title: String,
    /// 标签状态
    pub state: TabState,
    /// 是否有未读消息
    pub has_unread: bool,
    /// 是否正在执行
    pub is_executing: bool,
    /// 标签索引
    pub index: usize,
}

impl From<&Tab> for TabInfo {
    fn from(tab: &Tab) -> Self {
        Self {
            id: tab.id.clone(),
            title: tab.metadata.title.clone(),
            state: tab.state,
            has_unread: tab.metadata.has_unread,
            is_executing: tab.metadata.is_executing,
            index: tab.index,
        }
    }
}

impl TabManager {
    /// 获取标签列表信息（用于前端）
    pub fn get_list_info(&self) -> TabListInfo {
        TabListInfo {
            tabs: self.list_tabs().iter().map(|t| TabInfo::from(*t)).collect(),
            active_tab_id: self.active_tab_id.clone(),
            closed_count: self.closed_tabs.len(),
        }
    }
}

// ============ 前端 JavaScript 生成 ============

/// 生成 Tab UI JavaScript 代码
pub fn generate_tab_ui_js() -> &'static str {
    r#"
// ============ Tab Manager UI (v1.101.0) ============

class TabManagerUI {
    constructor() {
        this.tabs = new Map();
        this.activeTabId = null;
        this.closedCount = 0;
        this.onTabChange = null;
        this.container = null;
    }

    // 初始化 Tab UI
    init(containerId, onTabChange) {
        this.container = document.getElementById(containerId);
        this.onTabChange = onTabChange;
        if (this.container) {
            this.render();
        }
    }

    // 更新标签列表
    updateTabs(tabListInfo) {
        this.tabs.clear();
        tabListInfo.tabs.forEach(tab => {
            this.tabs.set(tab.id, tab);
        });
        this.activeTabId = tabListInfo.active_tab_id;
        this.closedCount = tabListInfo.closed_count;
        this.render();
    }

    // 渲染标签栏
    render() {
        if (!this.container) return;

        const tabsArray = Array.from(this.tabs.values())
            .sort((a, b) => a.index - b.index);

        this.container.innerHTML = `
            <div class="tab-bar">
                <div class="tab-list">
                    ${tabsArray.map(tab => this.renderTab(tab)).join('')}
                    <button class="tab-new" onclick="tabManager.createTab()" title="New Tab (Ctrl+T)">+</button>
                </div>
                <div class="tab-actions">
                    ${this.closedCount > 0 ? `
                        <button class="tab-restore" onclick="tabManager.restoreTab()" title="Restore Closed Tab (Ctrl+Shift+T)">
                            <span class="badge">${this.closedCount}</span>
                        </button>
                    ` : ''}
                </div>
            </div>
        `;

        // 绑定事件
        this.bindEvents();
    }

    // 渲染单个标签
    renderTab(tab) {
        const isActive = tab.id === this.activeTabId;
        const stateClass = `tab-${tab.state}`;
        const unreadClass = tab.has_unread ? 'tab-unread' : '';
        const executingClass = tab.is_executing ? 'tab-executing' : '';

        return `
            <div class="tab ${stateClass} ${unreadClass} ${executingClass} ${isActive ? 'tab-active' : ''}"
                 data-tab-id="${tab.id}"
                 onclick="tabManager.switchTab('${tab.id}')"
                 draggable="true">
                <span class="tab-title">${this.escapeHtml(tab.title)}</span>
                ${tab.is_executing ? '<span class="tab-spinner"></span>' : ''}
                ${tab.has_unread ? '<span class="tab-dot"></span>' : ''}
                <button class="tab-close" onclick="event.stopPropagation(); tabManager.closeTab('${tab.id}')" title="Close Tab">×</button>
            </div>
        `;
    }

    // 绑定拖拽事件
    bindEvents() {
        const tabElements = this.container.querySelectorAll('.tab');
        tabElements.forEach(tab => {
            tab.addEventListener('dragstart', (e) => this.handleDragStart(e));
            tab.addEventListener('dragover', (e) => this.handleDragOver(e));
            tab.addEventListener('drop', (e) => this.handleDrop(e));
            tab.addEventListener('dragend', (e) => this.handleDragEnd(e));
        });
    }

    // 拖拽处理
    handleDragStart(e) {
        e.dataTransfer.setData('text/plain', e.target.dataset.tabId);
        e.target.classList.add('dragging');
    }

    handleDragOver(e) {
        e.preventDefault();
        e.target.closest('.tab')?.classList.add('drag-over');
    }

    handleDrop(e) {
        e.preventDefault();
        const sourceId = e.dataTransfer.getData('text/plain');
        const targetTab = e.target.closest('.tab');
        if (targetTab) {
            const targetId = targetTab.dataset.tabId;
            this.sendMessage({ type: 'move_tab', source_id: sourceId, target_id: targetId });
        }
    }

    handleDragEnd(e) {
        document.querySelectorAll('.tab').forEach(tab => {
            tab.classList.remove('dragging', 'drag-over');
        });
    }

    // 发送消息到 WebSocket
    sendMessage(msg) {
        if (this.onTabChange) {
            this.onTabChange(msg);
        }
    }

    // 创建新标签
    createTab() {
        this.sendMessage({ type: 'create_tab' });
    }

    // 切换标签
    switchTab(tabId) {
        if (tabId !== this.activeTabId) {
            this.sendMessage({ type: 'switch_tab', tab_id: tabId });
        }
    }

    // 关闭标签
    closeTab(tabId) {
        this.sendMessage({ type: 'close_tab', tab_id: tabId });
    }

    // 恢复标签
    restoreTab() {
        this.sendMessage({ type: 'restore_tab' });
    }

    // HTML 转义
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
}

// Tab 样式
const tabStyles = `
    .tab-bar {
        display: flex;
        justify-content: space-between;
        align-items: center;
        background: #161B22;
        border-bottom: 1px solid #30363D;
        padding: 0 8px;
        height: 36px;
    }

    .tab-list {
        display: flex;
        align-items: center;
        gap: 2px;
        overflow-x: auto;
        flex: 1;
    }

    .tab {
        display: flex;
        align-items: center;
        padding: 6px 12px;
        background: #21262D;
        border: 1px solid #30363D;
        border-bottom: none;
        border-radius: 6px 6px 0 0;
        cursor: pointer;
        min-width: 100px;
        max-width: 200px;
        transition: all 0.2s ease;
    }

    .tab:hover {
        background: #30363D;
    }

    .tab-active {
        background: #0D1117;
        border-color: #58A6FF;
        border-bottom: 1px solid #0D1117;
    }

    .tab-background {
        opacity: 0.7;
    }

    .tab-title {
        flex: 1;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        font-size: 13px;
        color: #E6EDF3;
    }

    .tab-close {
        margin-left: 8px;
        padding: 2px 6px;
        background: transparent;
        border: none;
        color: #8B949E;
        cursor: pointer;
        border-radius: 4px;
        font-size: 14px;
    }

    .tab-close:hover {
        background: #F85149;
        color: white;
    }

    .tab-new {
        padding: 4px 12px;
        background: transparent;
        border: 1px dashed #30363D;
        border-radius: 6px;
        color: #8B949E;
        cursor: pointer;
        font-size: 18px;
    }

    .tab-new:hover {
        background: #21262D;
        border-color: #58A6FF;
        color: #58A6FF;
    }

    .tab-unread .tab-dot {
        width: 6px;
        height: 6px;
        background: #F85149;
        border-radius: 50%;
        margin-left: 4px;
    }

    .tab-executing .tab-spinner {
        width: 12px;
        height: 12px;
        border: 2px solid #58A6FF;
        border-top-color: transparent;
        border-radius: 50%;
        animation: spin 1s linear infinite;
        margin-left: 4px;
    }

    @keyframes spin {
        to { transform: rotate(360deg); }
    }

    .tab-actions {
        display: flex;
        gap: 4px;
    }

    .tab-restore {
        position: relative;
        padding: 4px 8px;
        background: #21262D;
        border: 1px solid #30363D;
        border-radius: 4px;
        color: #8B949E;
        cursor: pointer;
    }

    .tab-restore:hover {
        background: #30363D;
        color: #E6EDF3;
    }

    .tab-restore .badge {
        position: absolute;
        top: -4px;
        right: -4px;
        min-width: 14px;
        height: 14px;
        background: #F85149;
        border-radius: 7px;
        font-size: 10px;
        color: white;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .tab.dragging {
        opacity: 0.5;
    }

    .tab.drag-over {
        border-left: 2px solid #58A6FF;
    }
`;

// 注入样式
function injectTabStyles() {
    if (!document.getElementById('tab-manager-styles')) {
        const style = document.createElement('style');
        style.id = 'tab-manager-styles';
        style.textContent = tabStyles;
        document.head.appendChild(style);
    }
}

// 全局实例
const tabManager = new TabManagerUI();

// 初始化
document.addEventListener('DOMContentLoaded', () => {
    injectTabStyles();
});
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_state_methods() {
        assert!(TabState::Active.can_execute());
        assert!(TabState::Background.can_execute());
        assert!(!TabState::Closed.can_execute());

        assert!(TabState::Active.can_notify());
        assert!(TabState::Background.can_notify());
        assert!(!TabState::Closed.can_notify());
    }

    #[test]
    fn test_tab_config_default() {
        let config = TabConfig::default();
        assert_eq!(config.max_tabs, 10);
        assert_eq!(config.max_background_tabs, 5);
        assert!(config.confirm_close);
    }

    #[test]
    fn test_tab_creation() {
        let tab = Tab::new("session-123".to_string(), 0);

        assert!(tab.id.starts_with("tab-"));
        assert_eq!(tab.session_id, "session-123");
        assert_eq!(tab.state, TabState::Active);
        assert_eq!(tab.index, 0);
    }

    #[test]
    fn test_tab_with_title() {
        let tab = Tab::new("session-123".to_string(), 0)
            .with_title("My Tab");

        assert_eq!(tab.metadata.title, "My Tab");
    }

    #[test]
    fn test_tab_state_transitions() {
        let mut tab = Tab::new("session-123".to_string(), 0);

        assert_eq!(tab.state, TabState::Active);

        tab.move_to_background();
        assert_eq!(tab.state, TabState::Background);

        tab.activate();
        assert_eq!(tab.state, TabState::Active);

        tab.close();
        assert_eq!(tab.state, TabState::Closed);
    }

    #[test]
    fn test_tab_unread_marking() {
        let mut tab = Tab::new("session-123".to_string(), 0);

        // Active 标签不会标记未读
        tab.mark_unread();
        assert!(!tab.metadata.has_unread);

        // Background 标签会标记未读
        tab.move_to_background();
        tab.mark_unread();
        assert!(tab.metadata.has_unread);

        // 激活后清除未读
        tab.activate();
        assert!(!tab.metadata.has_unread);
    }

    #[test]
    fn test_tab_manager_creation() {
        let config = TabConfig::default();
        let manager = TabManager::new(config);

        assert_eq!(manager.tab_count(), 0);
        assert!(manager.active_tab().is_none());
    }

    #[test]
    fn test_tab_manager_create_tab() {
        let config = TabConfig::default();
        let mut manager = TabManager::new(config);

        let tab_id = manager.create_tab(None).unwrap();

        assert_eq!(manager.tab_count(), 1);
        assert!(manager.active_tab().is_some());
        assert_eq!(manager.active_tab().unwrap().id, tab_id);
    }

    #[test]
    fn test_tab_manager_max_tabs() {
        let mut config = TabConfig::default();
        config.max_tabs = 2;
        let mut manager = TabManager::new(config);

        // 创建两个标签成功
        assert!(manager.create_tab(None).is_ok());
        assert!(manager.create_tab(None).is_ok());

        // 第三个失败
        let result = manager.create_tab(None);
        assert!(matches!(result, Err(TabEvent::MaxTabsReached { max: 2 })));
    }

    #[test]
    fn test_tab_manager_switch_tab() {
        let config = TabConfig::default();
        let mut manager = TabManager::new(config);

        let tab1_id = manager.create_tab(None).unwrap();
        let tab2_id = manager.create_tab(None).unwrap();

        // tab2 现在是活跃的
        assert_eq!(manager.active_tab().unwrap().id, tab2_id);

        // tab1 在后台
        assert_eq!(manager.get_tab(&tab1_id).unwrap().state, TabState::Background);

        // 切换到 tab1
        manager.switch_to_tab(&tab1_id);
        assert_eq!(manager.active_tab().unwrap().id, tab1_id);
        assert_eq!(manager.get_tab(&tab2_id).unwrap().state, TabState::Background);
    }

    #[test]
    fn test_tab_manager_close_tab() {
        let config = TabConfig::default();
        let mut manager = TabManager::new(config);

        let tab1_id = manager.create_tab(None).unwrap();
        let tab2_id = manager.create_tab(None).unwrap();

        // 关闭 tab2
        manager.close_tab(&tab2_id);

        assert_eq!(manager.tab_count(), 1);
        // tab1 应该成为活跃标签
        assert_eq!(manager.active_tab().unwrap().id, tab1_id);
        // 已关闭标签可恢复
        assert_eq!(manager.closed_tabs().len(), 1);
    }

    #[test]
    fn test_tab_manager_restore_tab() {
        let config = TabConfig::default();
        let mut manager = TabManager::new(config);

        let tab1_id = manager.create_tab(None).unwrap();
        manager.close_tab(&tab1_id);

        assert_eq!(manager.tab_count(), 0);
        assert_eq!(manager.closed_tabs().len(), 1);

        // 恢复标签
        let event = manager.restore_last_closed();
        assert!(event.is_some());
        assert_eq!(manager.tab_count(), 1);
        assert_eq!(manager.closed_tabs().len(), 0);
    }

    #[test]
    fn test_tab_manager_next_prev() {
        let config = TabConfig::default();
        let mut manager = TabManager::new(config);

        let tab1_id = manager.create_tab(None).unwrap();
        let _tab2_id = manager.create_tab(None).unwrap();
        let tab3_id = manager.create_tab(None).unwrap();

        // tab3 是活跃的
        assert_eq!(manager.active_tab().unwrap().id, tab3_id);

        // 下一个 -> 回到 tab1（循环）
        manager.next_tab();
        assert_eq!(manager.active_tab().unwrap().id, tab1_id);

        // 上一个 -> 回到 tab3
        manager.prev_tab();
        assert_eq!(manager.active_tab().unwrap().id, tab3_id);
    }

    #[test]
    fn test_tab_list_info() {
        let config = TabConfig::default();
        let mut manager = TabManager::new(config);

        manager.create_tab(None).unwrap();
        manager.create_tab(None).unwrap();

        let info = manager.get_list_info();

        assert_eq!(info.tabs.len(), 2);
        assert!(info.active_tab_id.is_some());
        assert_eq!(info.closed_count, 0);
    }

    #[test]
    fn test_tab_info_from_tab() {
        let tab = Tab::new("session-123".to_string(), 0)
            .with_title("Test Tab");

        let info = TabInfo::from(&tab);

        assert_eq!(info.id, tab.id);
        assert_eq!(info.title, "Test Tab");
        assert_eq!(info.state, TabState::Active);
        assert!(!info.has_unread);
        assert!(!info.is_executing);
    }

    #[test]
    fn test_generate_tab_ui_js() {
        let js = generate_tab_ui_js();

        assert!(js.contains("class TabManagerUI"));
        assert!(js.contains("createTab"));
        assert!(js.contains("switchTab"));
        assert!(js.contains("closeTab"));
        assert!(js.contains("restoreTab"));
        assert!(js.contains(".tab-bar"));
    }
}
