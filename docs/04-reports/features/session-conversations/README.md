# Claude Code 对话历史备份

本目录存储 RealConsole 项目与 Claude Code 的对话历史备份，用于长期观察、分析和知识积累。

## 目录结构

```
session-conversations/
├── README.md              # 本说明文档
└── backups/
    └── YYYY-MM-DD-full-backup.json  # 全量备份文件
```

## 备份目的

1. **知识积累**: 保留开发过程中的决策讨论、问题解决思路
2. **长期观察**: 分析 AI 辅助开发的模式和演进
3. **项目历史**: 记录项目从创立到成长的完整对话轨迹
4. **数据安全**: 防止 Claude Code 本地数据意外丢失

## JSON 格式说明

```json
{
  "metadata": {
    "project": "RealConsole",
    "extraction_time": "2026-01-30T...",
    "source_path": "~/.claude/projects/...",
    "total_sessions": 2,
    "total_agents": 26,
    "total_messages": 8800,
    "total_size_mb": 73.5,
    "date_range": {
      "start": "2026-01-08T...",
      "end": "2026-01-30T..."
    }
  },
  "sessions_index": { ... },  // Claude Code 原始会话索引
  "sessions": [
    {
      "session_id": "uuid",
      "filename": "uuid.jsonl",
      "summary": "会话摘要",
      "created": "创建时间",
      "modified": "修改时间",
      "message_count": 100,
      "messages": [ ... ]  // 完整消息数组，包含 thinking
    }
  ],
  "agents": [
    {
      "agent_id": "short-id",
      "filename": "agent-xxx.jsonl",
      "message_count": 50,
      "messages": [ ... ]
    }
  ]
}
```

## 使用方法

### 执行备份

```bash
# 在项目根目录执行
python scripts/utils/extract-conversations.py
```

### 验证备份

```bash
# 查看元数据
jq '.metadata' backups/*.json

# 统计会话数
jq '.sessions | length' backups/*.json

# 统计代理数
jq '.agents | length' backups/*.json

# 查看时间范围
jq '.metadata.date_range' backups/*.json
```

### 数据分析示例

```bash
# 提取所有用户消息
jq '[.sessions[].messages[] | select(.type == "user")] | length' backups/*.json

# 查看特定会话的摘要
jq '.sessions[] | {session_id, summary, message_count}' backups/*.json

# 搜索特定关键词（需要 grep）
jq -r '.sessions[].messages[].content' backups/*.json | grep -i "关键词"
```

## 数据来源

备份数据来自 Claude Code 的本地存储目录：

```
~/.claude/projects/-Users-hongxin-Workspace-claude-ai-playground-RealConsole/
├── sessions-index.json    # 会话索引元数据
├── {session-uuid}.jsonl   # 主会话文件
└── agent-{id}.jsonl       # 代理子会话文件
```

## 维护建议

- **定期备份**: 建议在重要里程碑（版本发布、重大功能完成）后执行备份
- **命名规范**: 备份文件以日期命名 `YYYY-MM-DD-full-backup.json`
- **存储管理**: 备份文件较大（~100MB），注意磁盘空间
- **版本控制**: 备份文件已加入 `.gitignore`，不提交到 Git

## 注意事项

1. **隐私保护**: 备份文件可能包含敏感信息，请妥善保管
2. **文件大小**: 包含完整 thinking 内容，单次备份可达 100MB+
3. **编码格式**: 使用 UTF-8 编码，确保中文正确显示

---

*最后更新: 2026-01-30*
