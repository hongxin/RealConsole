//! 协作功能基础架构
//!
//! v1.103.0 新增：多用户协作基础设施
//!
//! # 功能特性
//! - 会话共享：生成共享链接
//! - 权限控制：只读/编辑/管理员
//! - 实时同步：基础设施准备
//! - 参与者管理：加入/离开/踢出
//!
//! # 使用示例
//! ```ignore
//! use crate::web::collaboration::{CollaborationSession, Permission};
//!
//! // 创建协作会话
//! let session = CollaborationSession::new("session-123", "owner-id");
//!
//! // 生成共享链接
//! let link = session.generate_share_link(Permission::ReadOnly, None);
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 参与者 ID
pub type ParticipantId = String;

/// 共享令牌
pub type ShareToken = String;

/// 权限级别（一分为三）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    /// 只读：只能查看会话内容
    #[default]
    ReadOnly,
    /// 编辑：可以执行命令
    Edit,
    /// 管理员：完全控制（可以踢出其他参与者）
    Admin,
}

impl Permission {
    /// 是否可以执行命令
    pub fn can_execute(&self) -> bool {
        matches!(self, Permission::Edit | Permission::Admin)
    }

    /// 是否可以管理参与者
    pub fn can_manage(&self) -> bool {
        matches!(self, Permission::Admin)
    }

    /// 是否可以查看
    pub fn can_view(&self) -> bool {
        true // 所有权限都可以查看
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::ReadOnly => "readonly",
            Permission::Edit => "edit",
            Permission::Admin => "admin",
        }
    }
}

/// 参与者状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParticipantStatus {
    /// 在线
    #[default]
    Online,
    /// 离开
    Away,
    /// 离线
    Offline,
}

/// 参与者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    /// 参与者 ID
    pub id: ParticipantId,
    /// 显示名称
    pub display_name: String,
    /// 权限级别
    pub permission: Permission,
    /// 状态
    pub status: ParticipantStatus,
    /// 加入时间
    pub joined_at: DateTime<Utc>,
    /// 最后活跃时间
    pub last_active_at: DateTime<Utc>,
    /// 是否是会话所有者
    pub is_owner: bool,
    /// 头像 URL（可选）
    pub avatar_url: Option<String>,
    /// 光标位置（用于实时协作）
    pub cursor_position: Option<CursorPosition>,
}

impl Participant {
    /// 创建新参与者
    pub fn new(id: ParticipantId, display_name: String, permission: Permission) -> Self {
        let now = Utc::now();
        Self {
            id,
            display_name,
            permission,
            status: ParticipantStatus::Online,
            joined_at: now,
            last_active_at: now,
            is_owner: false,
            avatar_url: None,
            cursor_position: None,
        }
    }

    /// 创建所有者
    pub fn owner(id: ParticipantId, display_name: String) -> Self {
        let mut participant = Self::new(id, display_name, Permission::Admin);
        participant.is_owner = true;
        participant
    }

    /// 更新活跃时间
    pub fn touch(&mut self) {
        self.last_active_at = Utc::now();
    }

    /// 设置状态
    pub fn set_status(&mut self, status: ParticipantStatus) {
        self.status = status;
        self.touch();
    }

    /// 更新光标位置
    pub fn update_cursor(&mut self, position: CursorPosition) {
        self.cursor_position = Some(position);
        self.touch();
    }
}

/// 光标位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    /// 行号
    pub line: usize,
    /// 列号
    pub column: usize,
    /// 选择区域（可选）
    pub selection: Option<SelectionRange>,
}

/// 选择区域
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionRange {
    /// 起始行
    pub start_line: usize,
    /// 起始列
    pub start_column: usize,
    /// 结束行
    pub end_line: usize,
    /// 结束列
    pub end_column: usize,
}

/// 共享链接
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareLink {
    /// 令牌
    pub token: ShareToken,
    /// 授予的权限
    pub permission: Permission,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 过期时间（可选）
    pub expires_at: Option<DateTime<Utc>>,
    /// 最大使用次数（可选）
    pub max_uses: Option<u32>,
    /// 已使用次数
    pub use_count: u32,
    /// 是否激活
    pub is_active: bool,
}

impl ShareLink {
    /// 创建新的共享链接
    pub fn new(permission: Permission, expires_in_hours: Option<u64>, max_uses: Option<u32>) -> Self {
        let now = Utc::now();
        let expires_at = expires_in_hours.map(|h| now + chrono::Duration::hours(h as i64));

        Self {
            token: format!("share-{}", Uuid::new_v4()),
            permission,
            created_at: now,
            expires_at,
            max_uses,
            use_count: 0,
            is_active: true,
        }
    }

    /// 是否有效
    pub fn is_valid(&self) -> bool {
        if !self.is_active {
            return false;
        }

        // 检查过期
        if let Some(expires) = self.expires_at {
            if Utc::now() > expires {
                return false;
            }
        }

        // 检查使用次数
        if let Some(max) = self.max_uses {
            if self.use_count >= max {
                return false;
            }
        }

        true
    }

    /// 使用一次
    pub fn use_once(&mut self) -> bool {
        if self.is_valid() {
            self.use_count += 1;
            true
        } else {
            false
        }
    }

    /// 撤销链接
    pub fn revoke(&mut self) {
        self.is_active = false;
    }
}

/// 协作配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationConfig {
    /// 最大参与者数
    pub max_participants: usize,
    /// 最大共享链接数
    pub max_share_links: usize,
    /// 默认共享链接有效期（小时）
    pub default_link_expiry_hours: u64,
    /// 离线超时时间（秒）
    pub offline_timeout_secs: u64,
    /// 允许匿名参与
    pub allow_anonymous: bool,
    /// 广播输入内容
    pub broadcast_input: bool,
    /// 广播光标位置
    pub broadcast_cursors: bool,
}

impl Default for CollaborationConfig {
    fn default() -> Self {
        Self {
            max_participants: 10,
            max_share_links: 5,
            default_link_expiry_hours: 24,
            offline_timeout_secs: 300, // 5 minutes
            allow_anonymous: false,
            broadcast_input: true,
            broadcast_cursors: true,
        }
    }
}

/// 协作会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSession {
    /// 会话 ID
    pub session_id: String,
    /// 所有者 ID
    pub owner_id: ParticipantId,
    /// 参与者列表
    pub participants: HashMap<ParticipantId, Participant>,
    /// 共享链接
    pub share_links: HashMap<ShareToken, ShareLink>,
    /// 配置
    pub config: CollaborationConfig,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 是否启用协作
    pub is_enabled: bool,
}

impl CollaborationSession {
    /// 创建新的协作会话
    pub fn new(session_id: impl Into<String>, owner_id: impl Into<String>) -> Self {
        let owner_id = owner_id.into();
        let owner = Participant::owner(owner_id.clone(), "Owner".to_string());

        let mut participants = HashMap::new();
        participants.insert(owner_id.clone(), owner);

        Self {
            session_id: session_id.into(),
            owner_id,
            participants,
            share_links: HashMap::new(),
            config: CollaborationConfig::default(),
            created_at: Utc::now(),
            is_enabled: true,
        }
    }

    /// 带配置创建
    pub fn with_config(mut self, config: CollaborationConfig) -> Self {
        self.config = config;
        self
    }

    /// 生成共享链接
    pub fn generate_share_link(&mut self, permission: Permission, expires_in_hours: Option<u64>) -> Result<ShareLink, String> {
        if self.share_links.len() >= self.config.max_share_links {
            return Err("Maximum share links reached".to_string());
        }

        let expiry = expires_in_hours.unwrap_or(self.config.default_link_expiry_hours);
        let link = ShareLink::new(permission, Some(expiry), None);

        self.share_links.insert(link.token.clone(), link.clone());
        Ok(link)
    }

    /// 通过共享链接加入
    pub fn join_with_link(&mut self, token: &ShareToken, participant_id: ParticipantId, display_name: String) -> Result<Participant, String> {
        let link = self.share_links.get_mut(token).ok_or("Invalid share link")?;

        if !link.is_valid() {
            return Err("Share link expired or invalid".to_string());
        }

        if self.participants.len() >= self.config.max_participants {
            return Err("Maximum participants reached".to_string());
        }

        link.use_once();

        let participant = Participant::new(participant_id.clone(), display_name, link.permission);
        self.participants.insert(participant_id, participant.clone());

        Ok(participant)
    }

    /// 直接加入（需要所有者邀请）
    pub fn add_participant(&mut self, participant_id: ParticipantId, display_name: String, permission: Permission) -> Result<Participant, String> {
        if self.participants.len() >= self.config.max_participants {
            return Err("Maximum participants reached".to_string());
        }

        if self.participants.contains_key(&participant_id) {
            return Err("Participant already exists".to_string());
        }

        let participant = Participant::new(participant_id.clone(), display_name, permission);
        self.participants.insert(participant_id, participant.clone());

        Ok(participant)
    }

    /// 移除参与者
    pub fn remove_participant(&mut self, remover_id: &ParticipantId, target_id: &ParticipantId) -> Result<(), String> {
        // 检查权限
        let remover = self.participants.get(remover_id).ok_or("Remover not found")?;
        if !remover.permission.can_manage() && remover_id != target_id {
            return Err("No permission to remove participants".to_string());
        }

        // 不能移除所有者
        if target_id == &self.owner_id {
            return Err("Cannot remove session owner".to_string());
        }

        self.participants.remove(target_id).ok_or("Participant not found")?;
        Ok(())
    }

    /// 更新参与者权限
    pub fn update_permission(&mut self, admin_id: &ParticipantId, target_id: &ParticipantId, permission: Permission) -> Result<(), String> {
        // 检查管理员权限
        let admin = self.participants.get(admin_id).ok_or("Admin not found")?;
        if !admin.permission.can_manage() {
            return Err("No permission to update permissions".to_string());
        }

        // 不能更改所有者权限
        if target_id == &self.owner_id {
            return Err("Cannot change owner permission".to_string());
        }

        let target = self.participants.get_mut(target_id).ok_or("Target not found")?;
        target.permission = permission;
        target.touch();

        Ok(())
    }

    /// 获取参与者
    pub fn get_participant(&self, participant_id: &ParticipantId) -> Option<&Participant> {
        self.participants.get(participant_id)
    }

    /// 获取参与者（可变）
    pub fn get_participant_mut(&mut self, participant_id: &ParticipantId) -> Option<&mut Participant> {
        self.participants.get_mut(participant_id)
    }

    /// 获取在线参与者列表
    pub fn online_participants(&self) -> Vec<&Participant> {
        self.participants.values()
            .filter(|p| p.status == ParticipantStatus::Online)
            .collect()
    }

    /// 获取所有参与者列表
    pub fn list_participants(&self) -> Vec<ParticipantInfo> {
        self.participants.values().map(ParticipantInfo::from).collect()
    }

    /// 撤销共享链接
    pub fn revoke_share_link(&mut self, admin_id: &ParticipantId, token: &ShareToken) -> Result<(), String> {
        let admin = self.participants.get(admin_id).ok_or("Admin not found")?;
        if !admin.permission.can_manage() {
            return Err("No permission to revoke share links".to_string());
        }

        let link = self.share_links.get_mut(token).ok_or("Link not found")?;
        link.revoke();

        Ok(())
    }

    /// 清理过期的共享链接
    pub fn cleanup_expired_links(&mut self) {
        self.share_links.retain(|_, link| link.is_valid());
    }

    /// 更新参与者状态（基于最后活跃时间）
    pub fn update_participant_status(&mut self) {
        let now = Utc::now();
        let timeout = chrono::Duration::seconds(self.config.offline_timeout_secs as i64);

        for participant in self.participants.values_mut() {
            if participant.status == ParticipantStatus::Online
                && now.signed_duration_since(participant.last_active_at) > timeout
            {
                participant.status = ParticipantStatus::Away;
            }
        }
    }

    /// 检查参与者是否有执行权限
    pub fn can_execute(&self, participant_id: &ParticipantId) -> bool {
        self.participants.get(participant_id)
            .map(|p| p.permission.can_execute())
            .unwrap_or(false)
    }
}

/// 参与者信息（轻量级）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantInfo {
    /// 参与者 ID
    pub id: ParticipantId,
    /// 显示名称
    pub display_name: String,
    /// 权限级别
    pub permission: Permission,
    /// 状态
    pub status: ParticipantStatus,
    /// 是否是所有者
    pub is_owner: bool,
    /// 头像 URL
    pub avatar_url: Option<String>,
}

impl From<&Participant> for ParticipantInfo {
    fn from(p: &Participant) -> Self {
        Self {
            id: p.id.clone(),
            display_name: p.display_name.clone(),
            permission: p.permission,
            status: p.status,
            is_owner: p.is_owner,
            avatar_url: p.avatar_url.clone(),
        }
    }
}

/// 协作事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CollaborationEvent {
    /// 参与者加入
    ParticipantJoined { participant: ParticipantInfo },
    /// 参与者离开
    ParticipantLeft { participant_id: ParticipantId },
    /// 参与者状态变化
    ParticipantStatusChanged { participant_id: ParticipantId, status: ParticipantStatus },
    /// 权限变更
    PermissionChanged { participant_id: ParticipantId, permission: Permission },
    /// 光标移动
    CursorMoved { participant_id: ParticipantId, position: CursorPosition },
    /// 输入广播
    InputBroadcast { participant_id: ParticipantId, input: String },
    /// 共享链接创建
    ShareLinkCreated { token: ShareToken, permission: Permission },
    /// 共享链接撤销
    ShareLinkRevoked { token: ShareToken },
}

// ============ 前端 JavaScript 生成 ============

/// 生成协作功能 JavaScript 代码
pub fn generate_collaboration_js() -> &'static str {
    r#"
// ============ Collaboration Manager UI (v1.103.0) ============

class CollaborationUI {
    constructor() {
        this.participants = new Map();
        this.isOwner = false;
        this.currentUserId = null;
        this.onCollabEvent = null;
    }

    // 初始化协作 UI
    init(containerId, onCollabEvent) {
        this.container = document.getElementById(containerId);
        this.onCollabEvent = onCollabEvent;
        this.injectStyles();
    }

    // 更新参与者列表
    updateParticipants(participants, currentUserId, isOwner) {
        this.participants.clear();
        participants.forEach(p => this.participants.set(p.id, p));
        this.currentUserId = currentUserId;
        this.isOwner = isOwner;
        this.render();
    }

    // 渲染协作面板
    render() {
        if (!this.container) return;

        const participantsArray = Array.from(this.participants.values());

        this.container.innerHTML = `
            <div class="collab-panel">
                <div class="collab-header">
                    <span class="collab-title">Collaboration</span>
                    <span class="collab-count">${participantsArray.length} participant(s)</span>
                </div>
                <div class="collab-participants">
                    ${participantsArray.map(p => this.renderParticipant(p)).join('')}
                </div>
                ${this.isOwner ? `
                    <div class="collab-actions">
                        <button class="collab-share-btn" onclick="collab.shareSession()">Share Session</button>
                    </div>
                ` : ''}
            </div>
        `;
    }

    // 渲染参与者
    renderParticipant(participant) {
        const statusClass = `status-${participant.status}`;
        const permissionIcon = this.getPermissionIcon(participant.permission);
        const isCurrentUser = participant.id === this.currentUserId;

        return `
            <div class="collab-participant ${statusClass} ${isCurrentUser ? 'current-user' : ''}"
                 data-participant-id="${participant.id}">
                <div class="participant-avatar">
                    ${participant.avatar_url
                        ? `<img src="${participant.avatar_url}" alt="${participant.display_name}">`
                        : `<span class="avatar-placeholder">${participant.display_name.charAt(0).toUpperCase()}</span>`
                    }
                    <span class="status-indicator"></span>
                </div>
                <div class="participant-info">
                    <span class="participant-name">
                        ${this.escapeHtml(participant.display_name)}
                        ${participant.is_owner ? '<span class="owner-badge">Owner</span>' : ''}
                    </span>
                    <span class="participant-permission">${permissionIcon} ${participant.permission}</span>
                </div>
                ${this.isOwner && !participant.is_owner ? `
                    <div class="participant-actions">
                        <button onclick="collab.showPermissionMenu('${participant.id}')" title="Change permission">...</button>
                    </div>
                ` : ''}
            </div>
        `;
    }

    // 获取权限图标
    getPermissionIcon(permission) {
        switch (permission) {
            case 'admin': return '👑';
            case 'edit': return '✏️';
            case 'readonly': return '👁';
            default: return '👤';
        }
    }

    // 分享会话
    shareSession() {
        this.showShareDialog();
    }

    // 显示分享对话框
    showShareDialog() {
        const dialog = document.createElement('div');
        dialog.className = 'collab-dialog-overlay';
        dialog.innerHTML = `
            <div class="collab-dialog">
                <div class="dialog-header">
                    <h3>Share Session</h3>
                    <button class="dialog-close" onclick="this.closest('.collab-dialog-overlay').remove()">×</button>
                </div>
                <div class="dialog-body">
                    <div class="form-group">
                        <label>Permission Level</label>
                        <select id="share-permission">
                            <option value="readonly">Read Only - Can view only</option>
                            <option value="edit">Edit - Can execute commands</option>
                            <option value="admin">Admin - Full control</option>
                        </select>
                    </div>
                    <div class="form-group">
                        <label>Expires In</label>
                        <select id="share-expiry">
                            <option value="1">1 hour</option>
                            <option value="24" selected>24 hours</option>
                            <option value="168">7 days</option>
                            <option value="0">Never</option>
                        </select>
                    </div>
                </div>
                <div class="dialog-footer">
                    <button class="btn-secondary" onclick="this.closest('.collab-dialog-overlay').remove()">Cancel</button>
                    <button class="btn-primary" onclick="collab.createShareLink()">Generate Link</button>
                </div>
            </div>
        `;
        document.body.appendChild(dialog);
    }

    // 创建共享链接
    createShareLink() {
        const permission = document.getElementById('share-permission').value;
        const expiry = parseInt(document.getElementById('share-expiry').value);

        this.sendMessage({
            type: 'create_share_link',
            permission: permission,
            expires_in_hours: expiry > 0 ? expiry : null
        });

        document.querySelector('.collab-dialog-overlay')?.remove();
    }

    // 显示共享链接
    showShareLinkResult(token, baseUrl) {
        const fullUrl = `${baseUrl}?share=${token}`;

        const dialog = document.createElement('div');
        dialog.className = 'collab-dialog-overlay';
        dialog.innerHTML = `
            <div class="collab-dialog">
                <div class="dialog-header">
                    <h3>Share Link Created</h3>
                    <button class="dialog-close" onclick="this.closest('.collab-dialog-overlay').remove()">×</button>
                </div>
                <div class="dialog-body">
                    <p>Share this link with others to collaborate:</p>
                    <div class="share-link-box">
                        <input type="text" id="share-link-input" value="${fullUrl}" readonly>
                        <button onclick="collab.copyShareLink()" title="Copy">📋</button>
                    </div>
                </div>
                <div class="dialog-footer">
                    <button class="btn-primary" onclick="this.closest('.collab-dialog-overlay').remove()">Done</button>
                </div>
            </div>
        `;
        document.body.appendChild(dialog);
    }

    // 复制共享链接
    copyShareLink() {
        const input = document.getElementById('share-link-input');
        input.select();
        document.execCommand('copy');

        // 显示复制成功提示
        const btn = input.nextElementSibling;
        btn.textContent = '✓';
        setTimeout(() => { btn.textContent = '📋'; }, 2000);
    }

    // 显示权限菜单
    showPermissionMenu(participantId) {
        // 简单实现：弹出对话框
        const newPermission = prompt('Enter new permission (readonly/edit/admin):');
        if (newPermission && ['readonly', 'edit', 'admin'].includes(newPermission)) {
            this.sendMessage({
                type: 'update_permission',
                participant_id: participantId,
                permission: newPermission
            });
        }
    }

    // 发送消息
    sendMessage(msg) {
        if (this.onCollabEvent) {
            this.onCollabEvent(msg);
        }
    }

    // 处理协作事件
    handleEvent(event) {
        switch (event.type) {
            case 'participant_joined':
                this.participants.set(event.participant.id, event.participant);
                this.render();
                this.showNotification(`${event.participant.display_name} joined the session`);
                break;

            case 'participant_left':
                const left = this.participants.get(event.participant_id);
                this.participants.delete(event.participant_id);
                this.render();
                if (left) {
                    this.showNotification(`${left.display_name} left the session`);
                }
                break;

            case 'participant_status_changed':
                const p = this.participants.get(event.participant_id);
                if (p) {
                    p.status = event.status;
                    this.render();
                }
                break;

            case 'permission_changed':
                const target = this.participants.get(event.participant_id);
                if (target) {
                    target.permission = event.permission;
                    this.render();
                }
                break;

            case 'share_link_created':
                this.showShareLinkResult(event.token, window.location.href.split('?')[0]);
                break;
        }
    }

    // 显示通知
    showNotification(message) {
        const notification = document.createElement('div');
        notification.className = 'collab-notification';
        notification.textContent = message;
        document.body.appendChild(notification);

        setTimeout(() => {
            notification.classList.add('fade-out');
            setTimeout(() => notification.remove(), 300);
        }, 3000);
    }

    // HTML 转义
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    // 注入样式
    injectStyles() {
        if (document.getElementById('collaboration-styles')) return;

        const style = document.createElement('style');
        style.id = 'collaboration-styles';
        style.textContent = `
            .collab-panel {
                background: #161B22;
                border: 1px solid #30363D;
                border-radius: 6px;
                overflow: hidden;
            }

            .collab-header {
                display: flex;
                justify-content: space-between;
                align-items: center;
                padding: 12px 16px;
                border-bottom: 1px solid #30363D;
            }

            .collab-title {
                font-weight: 600;
                color: #E6EDF3;
            }

            .collab-count {
                font-size: 12px;
                color: #8B949E;
            }

            .collab-participants {
                max-height: 200px;
                overflow-y: auto;
            }

            .collab-participant {
                display: flex;
                align-items: center;
                padding: 8px 16px;
                border-bottom: 1px solid #21262D;
            }

            .collab-participant:last-child {
                border-bottom: none;
            }

            .collab-participant.current-user {
                background: rgba(88, 166, 255, 0.1);
            }

            .participant-avatar {
                position: relative;
                width: 32px;
                height: 32px;
                margin-right: 12px;
            }

            .participant-avatar img,
            .avatar-placeholder {
                width: 32px;
                height: 32px;
                border-radius: 50%;
                object-fit: cover;
            }

            .avatar-placeholder {
                display: flex;
                align-items: center;
                justify-content: center;
                background: #30363D;
                color: #E6EDF3;
                font-weight: 600;
            }

            .status-indicator {
                position: absolute;
                bottom: 0;
                right: 0;
                width: 10px;
                height: 10px;
                border-radius: 50%;
                border: 2px solid #161B22;
            }

            .status-online .status-indicator {
                background: #3FB950;
            }

            .status-away .status-indicator {
                background: #F0B90B;
            }

            .status-offline .status-indicator {
                background: #8B949E;
            }

            .participant-info {
                flex: 1;
            }

            .participant-name {
                display: flex;
                align-items: center;
                gap: 8px;
                font-size: 13px;
                color: #E6EDF3;
            }

            .owner-badge {
                font-size: 10px;
                padding: 2px 6px;
                background: #F0B90B;
                color: #000;
                border-radius: 4px;
            }

            .participant-permission {
                font-size: 11px;
                color: #8B949E;
            }

            .participant-actions button {
                padding: 4px 8px;
                background: transparent;
                border: none;
                color: #8B949E;
                cursor: pointer;
            }

            .participant-actions button:hover {
                color: #E6EDF3;
            }

            .collab-actions {
                padding: 12px 16px;
                border-top: 1px solid #30363D;
            }

            .collab-share-btn {
                width: 100%;
                padding: 8px 16px;
                background: #238636;
                border: none;
                border-radius: 6px;
                color: white;
                cursor: pointer;
                font-weight: 500;
            }

            .collab-share-btn:hover {
                background: #2EA043;
            }

            .collab-dialog-overlay {
                position: fixed;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                background: rgba(0, 0, 0, 0.5);
                display: flex;
                align-items: center;
                justify-content: center;
                z-index: 1000;
            }

            .collab-dialog {
                background: #161B22;
                border: 1px solid #30363D;
                border-radius: 8px;
                width: 400px;
                max-width: 90%;
            }

            .dialog-header {
                display: flex;
                justify-content: space-between;
                align-items: center;
                padding: 16px;
                border-bottom: 1px solid #30363D;
            }

            .dialog-header h3 {
                margin: 0;
                color: #E6EDF3;
            }

            .dialog-close {
                background: none;
                border: none;
                color: #8B949E;
                font-size: 20px;
                cursor: pointer;
            }

            .dialog-body {
                padding: 16px;
            }

            .form-group {
                margin-bottom: 16px;
            }

            .form-group label {
                display: block;
                margin-bottom: 8px;
                color: #E6EDF3;
                font-size: 13px;
            }

            .form-group select,
            .form-group input {
                width: 100%;
                padding: 8px 12px;
                background: #0D1117;
                border: 1px solid #30363D;
                border-radius: 6px;
                color: #E6EDF3;
            }

            .dialog-footer {
                display: flex;
                justify-content: flex-end;
                gap: 8px;
                padding: 16px;
                border-top: 1px solid #30363D;
            }

            .btn-primary {
                padding: 8px 16px;
                background: #238636;
                border: none;
                border-radius: 6px;
                color: white;
                cursor: pointer;
            }

            .btn-secondary {
                padding: 8px 16px;
                background: #21262D;
                border: 1px solid #30363D;
                border-radius: 6px;
                color: #E6EDF3;
                cursor: pointer;
            }

            .share-link-box {
                display: flex;
                gap: 8px;
            }

            .share-link-box input {
                flex: 1;
            }

            .share-link-box button {
                padding: 8px 12px;
                background: #21262D;
                border: 1px solid #30363D;
                border-radius: 6px;
                color: #E6EDF3;
                cursor: pointer;
            }

            .collab-notification {
                position: fixed;
                bottom: 20px;
                right: 20px;
                padding: 12px 20px;
                background: #161B22;
                border: 1px solid #30363D;
                border-radius: 6px;
                color: #E6EDF3;
                z-index: 1001;
                animation: slideIn 0.3s ease;
            }

            .collab-notification.fade-out {
                animation: fadeOut 0.3s ease;
            }

            @keyframes slideIn {
                from { transform: translateX(100%); opacity: 0; }
                to { transform: translateX(0); opacity: 1; }
            }

            @keyframes fadeOut {
                from { opacity: 1; }
                to { opacity: 0; }
            }
        `;
        document.head.appendChild(style);
    }
}

// 全局实例
const collab = new CollaborationUI();
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_methods() {
        assert!(!Permission::ReadOnly.can_execute());
        assert!(Permission::Edit.can_execute());
        assert!(Permission::Admin.can_execute());

        assert!(!Permission::ReadOnly.can_manage());
        assert!(!Permission::Edit.can_manage());
        assert!(Permission::Admin.can_manage());

        assert!(Permission::ReadOnly.can_view());
    }

    #[test]
    fn test_participant_new() {
        let p = Participant::new("user-1".to_string(), "Test User".to_string(), Permission::Edit);

        assert_eq!(p.id, "user-1");
        assert_eq!(p.display_name, "Test User");
        assert_eq!(p.permission, Permission::Edit);
        assert_eq!(p.status, ParticipantStatus::Online);
        assert!(!p.is_owner);
    }

    #[test]
    fn test_participant_owner() {
        let p = Participant::owner("owner-1".to_string(), "Owner".to_string());

        assert!(p.is_owner);
        assert_eq!(p.permission, Permission::Admin);
    }

    #[test]
    fn test_share_link_new() {
        let link = ShareLink::new(Permission::ReadOnly, Some(24), Some(10));

        assert!(link.token.starts_with("share-"));
        assert_eq!(link.permission, Permission::ReadOnly);
        assert!(link.is_active);
        assert_eq!(link.use_count, 0);
        assert!(link.expires_at.is_some());
        assert_eq!(link.max_uses, Some(10));
    }

    #[test]
    fn test_share_link_validity() {
        let mut link = ShareLink::new(Permission::Edit, Some(24), Some(2));

        assert!(link.is_valid());

        // Use twice
        assert!(link.use_once());
        assert!(link.use_once());

        // Third use should fail
        assert!(!link.is_valid());
        assert!(!link.use_once());
    }

    #[test]
    fn test_share_link_revoke() {
        let mut link = ShareLink::new(Permission::Edit, None, None);

        assert!(link.is_valid());
        link.revoke();
        assert!(!link.is_valid());
    }

    #[test]
    fn test_collaboration_session_new() {
        let session = CollaborationSession::new("session-123", "owner-1");

        assert_eq!(session.session_id, "session-123");
        assert_eq!(session.owner_id, "owner-1");
        assert_eq!(session.participants.len(), 1);
        assert!(session.participants.contains_key("owner-1"));
    }

    #[test]
    fn test_collaboration_generate_share_link() {
        let mut session = CollaborationSession::new("session-123", "owner-1");

        let link = session.generate_share_link(Permission::ReadOnly, None).unwrap();

        assert!(link.token.starts_with("share-"));
        assert_eq!(link.permission, Permission::ReadOnly);
        assert_eq!(session.share_links.len(), 1);
    }

    #[test]
    fn test_collaboration_join_with_link() {
        let mut session = CollaborationSession::new("session-123", "owner-1");
        let link = session.generate_share_link(Permission::Edit, None).unwrap();

        let participant = session.join_with_link(
            &link.token,
            "user-2".to_string(),
            "New User".to_string(),
        ).unwrap();

        assert_eq!(participant.id, "user-2");
        assert_eq!(participant.permission, Permission::Edit);
        assert_eq!(session.participants.len(), 2);
    }

    #[test]
    fn test_collaboration_add_participant() {
        let mut session = CollaborationSession::new("session-123", "owner-1");

        let participant = session.add_participant(
            "user-2".to_string(),
            "User 2".to_string(),
            Permission::ReadOnly,
        ).unwrap();

        assert_eq!(participant.id, "user-2");
        assert_eq!(session.participants.len(), 2);
    }

    #[test]
    fn test_collaboration_remove_participant() {
        let mut session = CollaborationSession::new("session-123", "owner-1");
        session.add_participant("user-2".to_string(), "User 2".to_string(), Permission::Edit).unwrap();

        // Owner can remove
        assert!(session.remove_participant(&"owner-1".to_string(), &"user-2".to_string()).is_ok());
        assert_eq!(session.participants.len(), 1);
    }

    #[test]
    fn test_collaboration_cannot_remove_owner() {
        let mut session = CollaborationSession::new("session-123", "owner-1");
        session.add_participant("admin-2".to_string(), "Admin 2".to_string(), Permission::Admin).unwrap();

        // Cannot remove owner
        let result = session.remove_participant(&"admin-2".to_string(), &"owner-1".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("owner"));
    }

    #[test]
    fn test_collaboration_update_permission() {
        let mut session = CollaborationSession::new("session-123", "owner-1");
        session.add_participant("user-2".to_string(), "User 2".to_string(), Permission::ReadOnly).unwrap();

        // Owner updates permission
        session.update_permission(&"owner-1".to_string(), &"user-2".to_string(), Permission::Edit).unwrap();

        let user = session.get_participant(&"user-2".to_string()).unwrap();
        assert_eq!(user.permission, Permission::Edit);
    }

    #[test]
    fn test_collaboration_can_execute() {
        let mut session = CollaborationSession::new("session-123", "owner-1");
        session.add_participant("reader".to_string(), "Reader".to_string(), Permission::ReadOnly).unwrap();
        session.add_participant("editor".to_string(), "Editor".to_string(), Permission::Edit).unwrap();

        assert!(session.can_execute(&"owner-1".to_string()));
        assert!(!session.can_execute(&"reader".to_string()));
        assert!(session.can_execute(&"editor".to_string()));
        assert!(!session.can_execute(&"unknown".to_string()));
    }

    #[test]
    fn test_participant_info_from_participant() {
        let p = Participant::owner("owner-1".to_string(), "Owner".to_string());
        let info = ParticipantInfo::from(&p);

        assert_eq!(info.id, "owner-1");
        assert_eq!(info.display_name, "Owner");
        assert!(info.is_owner);
    }

    #[test]
    fn test_generate_collaboration_js() {
        let js = generate_collaboration_js();

        assert!(js.contains("class CollaborationUI"));
        assert!(js.contains("shareSession"));
        assert!(js.contains("createShareLink"));
        assert!(js.contains("handleEvent"));
        assert!(js.contains(".collab-panel"));
    }
}
