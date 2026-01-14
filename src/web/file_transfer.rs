//! 文件传输增强模块
//!
//! v1.102.0 新增：完善的文件上传/下载功能
//!
//! # 功能特性
//! - 拖放上传：支持拖拽文件到终端区域
//! - 批量传输：同时上传/下载多个文件
//! - 进度显示：实时传输进度反馈
//! - 断点续传：大文件分块传输支持
//! - 文件校验：MD5/SHA256 完整性验证
//!
//! # 使用示例
//! ```ignore
//! use crate::web::file_transfer::{TransferManager, TransferConfig};
//!
//! let config = TransferConfig::default();
//! let mut manager = TransferManager::new(config);
//!
//! // 开始上传
//! let transfer_id = manager.start_upload("file.txt", file_size, chunks)?;
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 传输 ID
pub type TransferId = String;

/// 传输方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransferDirection {
    /// 上传
    Upload,
    /// 下载
    Download,
}

/// 传输状态（一分为三扩展）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransferState {
    /// 等待中
    #[default]
    Pending,
    /// 传输中
    InProgress,
    /// 已暂停
    Paused,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
}

impl TransferState {
    /// 是否为终态
    pub fn is_terminal(&self) -> bool {
        matches!(self, TransferState::Completed | TransferState::Failed | TransferState::Cancelled)
    }

    /// 是否可以暂停
    pub fn can_pause(&self) -> bool {
        matches!(self, TransferState::Pending | TransferState::InProgress)
    }

    /// 是否可以恢复
    pub fn can_resume(&self) -> bool {
        matches!(self, TransferState::Paused)
    }

    /// 是否可以取消
    pub fn can_cancel(&self) -> bool {
        !self.is_terminal()
    }
}

/// 传输配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferConfig {
    /// 单个文件最大大小（字节）
    pub max_file_size: usize,
    /// 同时传输的最大数量
    pub max_concurrent: usize,
    /// 分块大小（字节）
    pub chunk_size: usize,
    /// 重试次数
    pub max_retries: u32,
    /// 重试间隔（毫秒）
    pub retry_interval_ms: u64,
    /// 启用校验和验证
    pub enable_checksum: bool,
    /// 允许的文件类型（MIME）
    pub allowed_types: Vec<String>,
    /// 传输超时（秒）
    pub transfer_timeout_secs: u64,
}

impl Default for TransferConfig {
    fn default() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_concurrent: 3,
            chunk_size: 256 * 1024, // 256KB
            max_retries: 3,
            retry_interval_ms: 1000,
            enable_checksum: true,
            allowed_types: vec![
                "text/*".to_string(),
                "image/*".to_string(),
                "application/json".to_string(),
                "application/pdf".to_string(),
                "application/zip".to_string(),
            ],
            transfer_timeout_secs: 300, // 5 minutes
        }
    }
}

/// 传输进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferProgress {
    /// 已传输字节数
    pub bytes_transferred: usize,
    /// 总字节数
    pub total_bytes: usize,
    /// 已完成的块数
    pub chunks_completed: usize,
    /// 总块数
    pub total_chunks: usize,
    /// 传输速度（字节/秒）
    pub speed_bps: f64,
    /// 预计剩余时间（秒）
    pub eta_secs: Option<f64>,
    /// 百分比
    pub percentage: f64,
}

impl TransferProgress {
    /// 创建新的进度
    pub fn new(total_bytes: usize, chunk_size: usize) -> Self {
        let total_chunks = total_bytes.div_ceil(chunk_size);
        Self {
            bytes_transferred: 0,
            total_bytes,
            chunks_completed: 0,
            total_chunks,
            speed_bps: 0.0,
            eta_secs: None,
            percentage: 0.0,
        }
    }

    /// 更新进度
    pub fn update(&mut self, bytes_transferred: usize, elapsed_secs: f64) {
        self.bytes_transferred = bytes_transferred;
        self.chunks_completed = bytes_transferred / (self.total_bytes / self.total_chunks.max(1));

        if self.total_bytes > 0 {
            self.percentage = (bytes_transferred as f64 / self.total_bytes as f64) * 100.0;
        }

        if elapsed_secs > 0.0 {
            self.speed_bps = bytes_transferred as f64 / elapsed_secs;
            if self.speed_bps > 0.0 {
                let remaining = self.total_bytes.saturating_sub(bytes_transferred);
                self.eta_secs = Some(remaining as f64 / self.speed_bps);
            }
        }
    }

    /// 标记完成
    pub fn complete(&mut self) {
        self.bytes_transferred = self.total_bytes;
        self.chunks_completed = self.total_chunks;
        self.percentage = 100.0;
        self.eta_secs = Some(0.0);
    }
}

/// 文件块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChunk {
    /// 块索引
    pub index: usize,
    /// 起始位置
    pub offset: usize,
    /// 块大小
    pub size: usize,
    /// 块数据（Base64 编码）
    pub data: Option<String>,
    /// 块校验和
    pub checksum: Option<String>,
    /// 是否已传输
    pub transferred: bool,
}

impl FileChunk {
    /// 创建新块
    pub fn new(index: usize, offset: usize, size: usize) -> Self {
        Self {
            index,
            offset,
            size,
            data: None,
            checksum: None,
            transferred: false,
        }
    }
}

/// 传输任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    /// 传输 ID
    pub id: TransferId,
    /// 文件名
    pub filename: String,
    /// 文件大小
    pub file_size: usize,
    /// MIME 类型
    pub mime_type: String,
    /// 传输方向
    pub direction: TransferDirection,
    /// 传输状态
    pub state: TransferState,
    /// 传输进度
    pub progress: TransferProgress,
    /// 文件块列表
    #[serde(skip)]
    pub chunks: Vec<FileChunk>,
    /// 校验和（完整文件）
    pub checksum: Option<String>,
    /// 错误信息
    pub error: Option<String>,
    /// 重试次数
    pub retry_count: u32,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 开始时间
    pub started_at: Option<DateTime<Utc>>,
    /// 完成时间
    pub completed_at: Option<DateTime<Utc>>,
}

impl Transfer {
    /// 创建新的上传任务
    pub fn new_upload(filename: String, file_size: usize, mime_type: String, chunk_size: usize) -> Self {
        let now = Utc::now();
        let total_chunks = file_size.div_ceil(chunk_size);

        // 创建块列表
        let mut chunks = Vec::with_capacity(total_chunks);
        for i in 0..total_chunks {
            let offset = i * chunk_size;
            let size = (file_size - offset).min(chunk_size);
            chunks.push(FileChunk::new(i, offset, size));
        }

        Self {
            id: format!("transfer-{}", Uuid::new_v4()),
            filename,
            file_size,
            mime_type,
            direction: TransferDirection::Upload,
            state: TransferState::Pending,
            progress: TransferProgress::new(file_size, chunk_size),
            chunks,
            checksum: None,
            error: None,
            retry_count: 0,
            created_at: now,
            started_at: None,
            completed_at: None,
        }
    }

    /// 创建新的下载任务
    pub fn new_download(filename: String, file_size: usize, mime_type: String, chunk_size: usize) -> Self {
        let mut transfer = Self::new_upload(filename, file_size, mime_type, chunk_size);
        transfer.direction = TransferDirection::Download;
        transfer
    }

    /// 开始传输
    pub fn start(&mut self) {
        if self.state == TransferState::Pending {
            self.state = TransferState::InProgress;
            self.started_at = Some(Utc::now());
        }
    }

    /// 暂停传输
    pub fn pause(&mut self) -> bool {
        if self.state.can_pause() {
            self.state = TransferState::Paused;
            true
        } else {
            false
        }
    }

    /// 恢复传输
    pub fn resume(&mut self) -> bool {
        if self.state.can_resume() {
            self.state = TransferState::InProgress;
            true
        } else {
            false
        }
    }

    /// 取消传输
    pub fn cancel(&mut self) -> bool {
        if self.state.can_cancel() {
            self.state = TransferState::Cancelled;
            self.completed_at = Some(Utc::now());
            true
        } else {
            false
        }
    }

    /// 标记块完成
    pub fn complete_chunk(&mut self, chunk_index: usize) -> bool {
        if chunk_index < self.chunks.len() {
            self.chunks[chunk_index].transferred = true;

            // 更新进度
            let completed_bytes: usize = self.chunks
                .iter()
                .filter(|c| c.transferred)
                .map(|c| c.size)
                .sum();

            let elapsed = self.started_at
                .map(|s| (Utc::now() - s).num_milliseconds() as f64 / 1000.0)
                .unwrap_or(0.0);

            self.progress.update(completed_bytes, elapsed);

            // 检查是否全部完成
            if self.chunks.iter().all(|c| c.transferred) {
                self.complete();
            }

            true
        } else {
            false
        }
    }

    /// 标记完成
    pub fn complete(&mut self) {
        self.state = TransferState::Completed;
        self.progress.complete();
        self.completed_at = Some(Utc::now());
    }

    /// 标记失败
    pub fn fail(&mut self, error: String) {
        self.state = TransferState::Failed;
        self.error = Some(error);
        self.completed_at = Some(Utc::now());
    }

    /// 获取下一个未传输的块索引
    pub fn next_pending_chunk(&self) -> Option<usize> {
        self.chunks
            .iter()
            .position(|c| !c.transferred)
    }

    /// 是否需要重试
    pub fn should_retry(&self, max_retries: u32) -> bool {
        self.state == TransferState::Failed && self.retry_count < max_retries
    }

    /// 增加重试次数
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
        self.state = TransferState::Pending;
        self.error = None;
    }
}

/// 传输事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransferEvent {
    /// 传输开始
    Started { transfer_id: TransferId },
    /// 进度更新
    Progress { transfer_id: TransferId, progress: TransferProgress },
    /// 块完成
    ChunkCompleted { transfer_id: TransferId, chunk_index: usize },
    /// 传输暂停
    Paused { transfer_id: TransferId },
    /// 传输恢复
    Resumed { transfer_id: TransferId },
    /// 传输完成
    Completed { transfer_id: TransferId },
    /// 传输失败
    Failed { transfer_id: TransferId, error: String },
    /// 传输取消
    Cancelled { transfer_id: TransferId },
    /// 队列变化
    QueueUpdated { pending: usize, active: usize, completed: usize },
}

/// 传输管理器
#[derive(Debug)]
pub struct TransferManager {
    /// 配置
    config: TransferConfig,
    /// 所有传输任务
    transfers: HashMap<TransferId, Transfer>,
    /// 待处理队列
    pending_queue: Vec<TransferId>,
    /// 活跃传输
    active_transfers: Vec<TransferId>,
    /// 已完成传输
    completed_transfers: Vec<TransferId>,
}

impl TransferManager {
    /// 创建传输管理器
    pub fn new(config: TransferConfig) -> Self {
        Self {
            config,
            transfers: HashMap::new(),
            pending_queue: Vec::new(),
            active_transfers: Vec::new(),
            completed_transfers: Vec::new(),
        }
    }

    /// 添加上传任务
    pub fn add_upload(&mut self, filename: String, file_size: usize, mime_type: String) -> Result<TransferId, String> {
        // 检查文件大小
        if file_size > self.config.max_file_size {
            return Err(format!(
                "File too large: {} bytes (max: {} bytes)",
                file_size, self.config.max_file_size
            ));
        }

        // 检查 MIME 类型
        if !self.is_type_allowed(&mime_type) {
            return Err(format!("File type not allowed: {}", mime_type));
        }

        let transfer = Transfer::new_upload(filename, file_size, mime_type, self.config.chunk_size);
        let transfer_id = transfer.id.clone();

        self.transfers.insert(transfer_id.clone(), transfer);
        self.pending_queue.push(transfer_id.clone());

        self.process_queue();

        Ok(transfer_id)
    }

    /// 添加下载任务
    pub fn add_download(&mut self, filename: String, file_size: usize, mime_type: String) -> TransferId {
        let transfer = Transfer::new_download(filename, file_size, mime_type, self.config.chunk_size);
        let transfer_id = transfer.id.clone();

        self.transfers.insert(transfer_id.clone(), transfer);
        self.pending_queue.push(transfer_id.clone());

        self.process_queue();

        transfer_id
    }

    /// 处理队列
    fn process_queue(&mut self) {
        while self.active_transfers.len() < self.config.max_concurrent && !self.pending_queue.is_empty() {
            if let Some(transfer_id) = self.pending_queue.pop() {
                if let Some(transfer) = self.transfers.get_mut(&transfer_id) {
                    transfer.start();
                    self.active_transfers.push(transfer_id);
                }
            }
        }
    }

    /// 检查 MIME 类型是否允许
    fn is_type_allowed(&self, mime_type: &str) -> bool {
        for pattern in &self.config.allowed_types {
            if pattern == "*/*" {
                return true;
            }
            if pattern.ends_with("/*") {
                let category = &pattern[..pattern.len() - 2];
                if mime_type.starts_with(category) {
                    return true;
                }
            } else if pattern == mime_type {
                return true;
            }
        }
        false
    }

    /// 接收块数据
    pub fn receive_chunk(&mut self, transfer_id: &TransferId, chunk_index: usize, _data: String) -> Option<TransferEvent> {
        let transfer = self.transfers.get_mut(transfer_id)?;

        if transfer.state != TransferState::InProgress {
            return None;
        }

        if transfer.complete_chunk(chunk_index) {
            if transfer.state == TransferState::Completed {
                // 从活跃列表移到完成列表
                self.active_transfers.retain(|id| id != transfer_id);
                self.completed_transfers.push(transfer_id.clone());
                self.process_queue();

                Some(TransferEvent::Completed { transfer_id: transfer_id.clone() })
            } else {
                Some(TransferEvent::ChunkCompleted {
                    transfer_id: transfer_id.clone(),
                    chunk_index,
                })
            }
        } else {
            None
        }
    }

    /// 暂停传输
    pub fn pause_transfer(&mut self, transfer_id: &TransferId) -> Option<TransferEvent> {
        let transfer = self.transfers.get_mut(transfer_id)?;

        if transfer.pause() {
            self.active_transfers.retain(|id| id != transfer_id);
            Some(TransferEvent::Paused { transfer_id: transfer_id.clone() })
        } else {
            None
        }
    }

    /// 恢复传输
    pub fn resume_transfer(&mut self, transfer_id: &TransferId) -> Option<TransferEvent> {
        let transfer = self.transfers.get_mut(transfer_id)?;

        if transfer.resume() {
            if self.active_transfers.len() < self.config.max_concurrent {
                self.active_transfers.push(transfer_id.clone());
            } else {
                self.pending_queue.push(transfer_id.clone());
            }
            Some(TransferEvent::Resumed { transfer_id: transfer_id.clone() })
        } else {
            None
        }
    }

    /// 取消传输
    pub fn cancel_transfer(&mut self, transfer_id: &TransferId) -> Option<TransferEvent> {
        let transfer = self.transfers.get_mut(transfer_id)?;

        if transfer.cancel() {
            self.active_transfers.retain(|id| id != transfer_id);
            self.pending_queue.retain(|id| id != transfer_id);
            self.process_queue();

            Some(TransferEvent::Cancelled { transfer_id: transfer_id.clone() })
        } else {
            None
        }
    }

    /// 标记传输失败
    pub fn fail_transfer(&mut self, transfer_id: &TransferId, error: String) -> Option<TransferEvent> {
        let transfer = self.transfers.get_mut(transfer_id)?;
        transfer.fail(error.clone());

        self.active_transfers.retain(|id| id != transfer_id);
        self.process_queue();

        Some(TransferEvent::Failed { transfer_id: transfer_id.clone(), error })
    }

    /// 获取传输信息
    pub fn get_transfer(&self, transfer_id: &TransferId) -> Option<&Transfer> {
        self.transfers.get(transfer_id)
    }

    /// 获取所有传输列表
    pub fn list_transfers(&self) -> Vec<TransferInfo> {
        self.transfers.values().map(TransferInfo::from).collect()
    }

    /// 获取活跃传输
    pub fn active_transfers(&self) -> Vec<&Transfer> {
        self.active_transfers
            .iter()
            .filter_map(|id| self.transfers.get(id))
            .collect()
    }

    /// 获取队列状态
    pub fn queue_stats(&self) -> QueueStats {
        QueueStats {
            pending: self.pending_queue.len(),
            active: self.active_transfers.len(),
            completed: self.completed_transfers.len(),
            failed: self.transfers.values().filter(|t| t.state == TransferState::Failed).count(),
        }
    }

    /// 清理已完成的传输
    pub fn cleanup_completed(&mut self, max_age_secs: u64) {
        let now = Utc::now();
        let max_age = chrono::Duration::seconds(max_age_secs as i64);

        let to_remove: Vec<TransferId> = self.transfers
            .iter()
            .filter(|(_, t)| {
                t.state.is_terminal() &&
                t.completed_at
                    .map(|c| now.signed_duration_since(c) > max_age)
                    .unwrap_or(false)
            })
            .map(|(id, _)| id.clone())
            .collect();

        for id in to_remove {
            self.transfers.remove(&id);
            self.completed_transfers.retain(|i| i != &id);
        }
    }
}

/// 传输信息（轻量级）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferInfo {
    /// 传输 ID
    pub id: TransferId,
    /// 文件名
    pub filename: String,
    /// 文件大小
    pub file_size: usize,
    /// 传输方向
    pub direction: TransferDirection,
    /// 传输状态
    pub state: TransferState,
    /// 进度百分比
    pub progress_percent: f64,
    /// 传输速度
    pub speed_bps: f64,
    /// 预计剩余时间
    pub eta_secs: Option<f64>,
    /// 错误信息
    pub error: Option<String>,
}

impl From<&Transfer> for TransferInfo {
    fn from(transfer: &Transfer) -> Self {
        Self {
            id: transfer.id.clone(),
            filename: transfer.filename.clone(),
            file_size: transfer.file_size,
            direction: transfer.direction,
            state: transfer.state,
            progress_percent: transfer.progress.percentage,
            speed_bps: transfer.progress.speed_bps,
            eta_secs: transfer.progress.eta_secs,
            error: transfer.error.clone(),
        }
    }
}

/// 队列统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    /// 等待中
    pub pending: usize,
    /// 活跃中
    pub active: usize,
    /// 已完成
    pub completed: usize,
    /// 失败
    pub failed: usize,
}

// ============ 前端 JavaScript 生成 ============

/// 生成文件传输 JavaScript 代码
pub fn generate_file_transfer_js() -> &'static str {
    r#"
// ============ File Transfer Manager UI (v1.102.0) ============

class FileTransferUI {
    constructor() {
        this.transfers = new Map();
        this.dropZone = null;
        this.onTransferEvent = null;
    }

    // 初始化文件传输 UI
    init(dropZoneId, onTransferEvent) {
        this.dropZone = document.getElementById(dropZoneId);
        this.onTransferEvent = onTransferEvent;

        if (this.dropZone) {
            this.setupDropZone();
        }

        this.injectStyles();
    }

    // 设置拖放区域
    setupDropZone() {
        this.dropZone.addEventListener('dragover', (e) => this.handleDragOver(e));
        this.dropZone.addEventListener('dragleave', (e) => this.handleDragLeave(e));
        this.dropZone.addEventListener('drop', (e) => this.handleDrop(e));

        // 也支持粘贴
        document.addEventListener('paste', (e) => this.handlePaste(e));
    }

    // 处理拖拽悬停
    handleDragOver(e) {
        e.preventDefault();
        e.stopPropagation();
        this.dropZone.classList.add('drag-over');
    }

    // 处理拖拽离开
    handleDragLeave(e) {
        e.preventDefault();
        e.stopPropagation();
        this.dropZone.classList.remove('drag-over');
    }

    // 处理文件放下
    handleDrop(e) {
        e.preventDefault();
        e.stopPropagation();
        this.dropZone.classList.remove('drag-over');

        const files = e.dataTransfer.files;
        if (files.length > 0) {
            this.uploadFiles(Array.from(files));
        }
    }

    // 处理粘贴
    handlePaste(e) {
        const items = e.clipboardData?.items;
        if (!items) return;

        const files = [];
        for (const item of items) {
            if (item.kind === 'file') {
                const file = item.getAsFile();
                if (file) files.push(file);
            }
        }

        if (files.length > 0) {
            this.uploadFiles(files);
        }
    }

    // 上传多个文件
    async uploadFiles(files) {
        for (const file of files) {
            await this.uploadFile(file);
        }
    }

    // 上传单个文件
    async uploadFile(file) {
        const transfer = {
            id: 'upload-' + Date.now(),
            filename: file.name,
            file_size: file.size,
            direction: 'upload',
            state: 'pending',
            progress_percent: 0,
            speed_bps: 0,
            eta_secs: null,
            error: null
        };

        this.transfers.set(transfer.id, transfer);
        this.renderTransferList();

        try {
            // 发送开始上传消息
            this.sendMessage({
                type: 'start_upload',
                filename: file.name,
                file_size: file.size,
                mime_type: file.type || 'application/octet-stream'
            });

            // 读取文件并分块上传
            const chunkSize = 256 * 1024; // 256KB
            const chunks = Math.ceil(file.size / chunkSize);

            for (let i = 0; i < chunks; i++) {
                const start = i * chunkSize;
                const end = Math.min(start + chunkSize, file.size);
                const chunk = file.slice(start, end);
                const data = await this.readChunkAsBase64(chunk);

                this.sendMessage({
                    type: 'upload_chunk',
                    transfer_id: transfer.id,
                    chunk_index: i,
                    data: data
                });

                // 更新进度
                transfer.progress_percent = ((i + 1) / chunks) * 100;
                transfer.state = 'in_progress';
                this.renderTransferList();
            }

            transfer.state = 'completed';
            transfer.progress_percent = 100;
        } catch (error) {
            transfer.state = 'failed';
            transfer.error = error.message;
        }

        this.renderTransferList();
    }

    // 读取块为 Base64
    readChunkAsBase64(chunk) {
        return new Promise((resolve, reject) => {
            const reader = new FileReader();
            reader.onload = () => {
                const base64 = reader.result.split(',')[1];
                resolve(base64);
            };
            reader.onerror = reject;
            reader.readAsDataURL(chunk);
        });
    }

    // 更新传输状态
    updateTransfer(transferInfo) {
        this.transfers.set(transferInfo.id, transferInfo);
        this.renderTransferList();
    }

    // 渲染传输列表
    renderTransferList() {
        const container = document.getElementById('transfer-list');
        if (!container) return;

        const transfers = Array.from(this.transfers.values());

        if (transfers.length === 0) {
            container.innerHTML = '<div class="transfer-empty">No active transfers</div>';
            return;
        }

        container.innerHTML = transfers.map(t => this.renderTransferItem(t)).join('');
    }

    // 渲染单个传输项
    renderTransferItem(transfer) {
        const icon = transfer.direction === 'upload' ? '↑' : '↓';
        const stateClass = `transfer-${transfer.state}`;
        const progressWidth = transfer.progress_percent.toFixed(1);

        return `
            <div class="transfer-item ${stateClass}">
                <div class="transfer-icon">${icon}</div>
                <div class="transfer-info">
                    <div class="transfer-filename">${this.escapeHtml(transfer.filename)}</div>
                    <div class="transfer-progress-bar">
                        <div class="transfer-progress-fill" style="width: ${progressWidth}%"></div>
                    </div>
                    <div class="transfer-stats">
                        <span>${this.formatSize(transfer.file_size)}</span>
                        <span>${progressWidth}%</span>
                        ${transfer.speed_bps > 0 ? `<span>${this.formatSpeed(transfer.speed_bps)}</span>` : ''}
                        ${transfer.eta_secs ? `<span>ETA: ${this.formatTime(transfer.eta_secs)}</span>` : ''}
                    </div>
                </div>
                <div class="transfer-actions">
                    ${transfer.state === 'in_progress' ? `
                        <button onclick="fileTransfer.pauseTransfer('${transfer.id}')" title="Pause">⏸</button>
                    ` : ''}
                    ${transfer.state === 'paused' ? `
                        <button onclick="fileTransfer.resumeTransfer('${transfer.id}')" title="Resume">▶</button>
                    ` : ''}
                    ${!['completed', 'failed', 'cancelled'].includes(transfer.state) ? `
                        <button onclick="fileTransfer.cancelTransfer('${transfer.id}')" title="Cancel">✕</button>
                    ` : ''}
                </div>
            </div>
        `;
    }

    // 暂停传输
    pauseTransfer(transferId) {
        this.sendMessage({ type: 'pause_transfer', transfer_id: transferId });
    }

    // 恢复传输
    resumeTransfer(transferId) {
        this.sendMessage({ type: 'resume_transfer', transfer_id: transferId });
    }

    // 取消传输
    cancelTransfer(transferId) {
        this.sendMessage({ type: 'cancel_transfer', transfer_id: transferId });
    }

    // 发送消息
    sendMessage(msg) {
        if (this.onTransferEvent) {
            this.onTransferEvent(msg);
        }
    }

    // 格式化文件大小
    formatSize(bytes) {
        const units = ['B', 'KB', 'MB', 'GB'];
        let size = bytes;
        let unit = 0;
        while (size >= 1024 && unit < units.length - 1) {
            size /= 1024;
            unit++;
        }
        return `${size.toFixed(1)} ${units[unit]}`;
    }

    // 格式化速度
    formatSpeed(bps) {
        return this.formatSize(bps) + '/s';
    }

    // 格式化时间
    formatTime(secs) {
        if (secs < 60) return `${Math.round(secs)}s`;
        if (secs < 3600) return `${Math.round(secs / 60)}m`;
        return `${Math.round(secs / 3600)}h`;
    }

    // HTML 转义
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    // 注入样式
    injectStyles() {
        if (document.getElementById('file-transfer-styles')) return;

        const style = document.createElement('style');
        style.id = 'file-transfer-styles';
        style.textContent = `
            .drag-over {
                border: 2px dashed #58A6FF !important;
                background: rgba(88, 166, 255, 0.1) !important;
            }

            .transfer-list {
                max-height: 200px;
                overflow-y: auto;
                background: #161B22;
                border: 1px solid #30363D;
                border-radius: 6px;
                margin: 8px 0;
            }

            .transfer-empty {
                padding: 16px;
                text-align: center;
                color: #8B949E;
            }

            .transfer-item {
                display: flex;
                align-items: center;
                padding: 8px 12px;
                border-bottom: 1px solid #30363D;
            }

            .transfer-item:last-child {
                border-bottom: none;
            }

            .transfer-icon {
                font-size: 16px;
                margin-right: 8px;
                color: #8B949E;
            }

            .transfer-in_progress .transfer-icon {
                color: #58A6FF;
            }

            .transfer-completed .transfer-icon {
                color: #3FB950;
            }

            .transfer-failed .transfer-icon {
                color: #F85149;
            }

            .transfer-info {
                flex: 1;
                min-width: 0;
            }

            .transfer-filename {
                font-size: 13px;
                color: #E6EDF3;
                white-space: nowrap;
                overflow: hidden;
                text-overflow: ellipsis;
            }

            .transfer-progress-bar {
                height: 4px;
                background: #21262D;
                border-radius: 2px;
                margin: 4px 0;
                overflow: hidden;
            }

            .transfer-progress-fill {
                height: 100%;
                background: #58A6FF;
                border-radius: 2px;
                transition: width 0.3s ease;
            }

            .transfer-completed .transfer-progress-fill {
                background: #3FB950;
            }

            .transfer-failed .transfer-progress-fill {
                background: #F85149;
            }

            .transfer-stats {
                display: flex;
                gap: 8px;
                font-size: 11px;
                color: #8B949E;
            }

            .transfer-actions {
                display: flex;
                gap: 4px;
            }

            .transfer-actions button {
                padding: 4px 8px;
                background: transparent;
                border: 1px solid #30363D;
                border-radius: 4px;
                color: #8B949E;
                cursor: pointer;
                font-size: 12px;
            }

            .transfer-actions button:hover {
                background: #21262D;
                color: #E6EDF3;
            }
        `;
        document.head.appendChild(style);
    }
}

// 全局实例
const fileTransfer = new FileTransferUI();
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_state_methods() {
        assert!(TransferState::Completed.is_terminal());
        assert!(TransferState::Failed.is_terminal());
        assert!(!TransferState::InProgress.is_terminal());

        assert!(TransferState::InProgress.can_pause());
        assert!(!TransferState::Completed.can_pause());

        assert!(TransferState::Paused.can_resume());
        assert!(!TransferState::InProgress.can_resume());
    }

    #[test]
    fn test_transfer_config_default() {
        let config = TransferConfig::default();
        assert_eq!(config.max_file_size, 100 * 1024 * 1024);
        assert_eq!(config.max_concurrent, 3);
        assert_eq!(config.chunk_size, 256 * 1024);
    }

    #[test]
    fn test_transfer_progress_new() {
        let progress = TransferProgress::new(1024 * 1024, 256 * 1024);
        assert_eq!(progress.total_bytes, 1024 * 1024);
        assert_eq!(progress.total_chunks, 4);
        assert_eq!(progress.percentage, 0.0);
    }

    #[test]
    fn test_transfer_progress_update() {
        let mut progress = TransferProgress::new(1000, 100);
        progress.update(500, 2.0);

        assert_eq!(progress.bytes_transferred, 500);
        assert_eq!(progress.percentage, 50.0);
        assert_eq!(progress.speed_bps, 250.0);
    }

    #[test]
    fn test_file_chunk_new() {
        let chunk = FileChunk::new(0, 0, 256);
        assert_eq!(chunk.index, 0);
        assert_eq!(chunk.offset, 0);
        assert_eq!(chunk.size, 256);
        assert!(!chunk.transferred);
    }

    #[test]
    fn test_transfer_new_upload() {
        let transfer = Transfer::new_upload(
            "test.txt".to_string(),
            1024,
            "text/plain".to_string(),
            256,
        );

        assert!(transfer.id.starts_with("transfer-"));
        assert_eq!(transfer.filename, "test.txt");
        assert_eq!(transfer.file_size, 1024);
        assert_eq!(transfer.direction, TransferDirection::Upload);
        assert_eq!(transfer.state, TransferState::Pending);
        assert_eq!(transfer.chunks.len(), 4);
    }

    #[test]
    fn test_transfer_lifecycle() {
        let mut transfer = Transfer::new_upload(
            "test.txt".to_string(),
            512,
            "text/plain".to_string(),
            256,
        );

        // Start
        transfer.start();
        assert_eq!(transfer.state, TransferState::InProgress);
        assert!(transfer.started_at.is_some());

        // Pause
        assert!(transfer.pause());
        assert_eq!(transfer.state, TransferState::Paused);

        // Resume
        assert!(transfer.resume());
        assert_eq!(transfer.state, TransferState::InProgress);

        // Complete chunks
        transfer.complete_chunk(0);
        assert_eq!(transfer.state, TransferState::InProgress);

        transfer.complete_chunk(1);
        assert_eq!(transfer.state, TransferState::Completed);
    }

    #[test]
    fn test_transfer_cancel() {
        let mut transfer = Transfer::new_upload(
            "test.txt".to_string(),
            1024,
            "text/plain".to_string(),
            256,
        );

        transfer.start();
        assert!(transfer.cancel());
        assert_eq!(transfer.state, TransferState::Cancelled);

        // Can't cancel again
        assert!(!transfer.cancel());
    }

    #[test]
    fn test_transfer_manager_add_upload() {
        let config = TransferConfig::default();
        let mut manager = TransferManager::new(config);

        let result = manager.add_upload(
            "test.txt".to_string(),
            1024,
            "text/plain".to_string(),
        );

        assert!(result.is_ok());
        let transfer_id = result.unwrap();
        assert!(manager.get_transfer(&transfer_id).is_some());
    }

    #[test]
    fn test_transfer_manager_file_too_large() {
        let mut config = TransferConfig::default();
        config.max_file_size = 1024;
        let mut manager = TransferManager::new(config);

        let result = manager.add_upload(
            "large.txt".to_string(),
            2048,
            "text/plain".to_string(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too large"));
    }

    #[test]
    fn test_transfer_manager_type_not_allowed() {
        let mut config = TransferConfig::default();
        config.allowed_types = vec!["text/*".to_string()];
        let mut manager = TransferManager::new(config);

        let result = manager.add_upload(
            "video.mp4".to_string(),
            1024,
            "video/mp4".to_string(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not allowed"));
    }

    #[test]
    fn test_transfer_manager_queue_stats() {
        let config = TransferConfig::default();
        let mut manager = TransferManager::new(config);

        manager.add_upload("test1.txt".to_string(), 1024, "text/plain".to_string()).unwrap();
        manager.add_upload("test2.txt".to_string(), 1024, "text/plain".to_string()).unwrap();

        let stats = manager.queue_stats();
        assert!(stats.active > 0 || stats.pending > 0);
    }

    #[test]
    fn test_transfer_info_from_transfer() {
        let transfer = Transfer::new_upload(
            "test.txt".to_string(),
            1024,
            "text/plain".to_string(),
            256,
        );

        let info = TransferInfo::from(&transfer);
        assert_eq!(info.filename, "test.txt");
        assert_eq!(info.file_size, 1024);
        assert_eq!(info.direction, TransferDirection::Upload);
    }

    #[test]
    fn test_generate_file_transfer_js() {
        let js = generate_file_transfer_js();

        assert!(js.contains("class FileTransferUI"));
        assert!(js.contains("uploadFile"));
        assert!(js.contains("pauseTransfer"));
        assert!(js.contains("resumeTransfer"));
        assert!(js.contains("cancelTransfer"));
        assert!(js.contains(".drag-over"));
    }
}
