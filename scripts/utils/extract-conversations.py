#!/usr/bin/env python3
"""
Claude Code 对话历史全量抽取工具

将 Claude Code 的对话历史从 ~/.claude/projects/ 目录抽取并备份为 JSON 文件。
保留完整内容包括 thinking 字段，便于长期观察和分析。

用法:
    python scripts/utils/extract-conversations.py

输出:
    docs/04-reports/features/session-conversations/backups/YYYY-MM-DD-full-backup.json
"""

from __future__ import annotations

import json
import os
from pathlib import Path
from datetime import datetime
from typing import Any, Dict, List, Optional

# 配置
CLAUDE_PROJECT_DIR = Path.home() / ".claude/projects/-Users-hongxin-Workspace-claude-ai-playground-RealConsole"
OUTPUT_DIR = Path("docs/04-reports/features/session-conversations/backups")


def parse_jsonl(filepath: Path) -> List[Dict[str, Any]]:
    """解析 JSONL 文件，保留完整内容包括 thinking"""
    messages = []
    with open(filepath, 'r', encoding='utf-8') as f:
        for line_num, line in enumerate(f, 1):
            if line.strip():
                try:
                    messages.append(json.loads(line))
                except json.JSONDecodeError as e:
                    print(f"警告: {filepath.name} 第 {line_num} 行解析失败: {e}")
    return messages


def get_date_range(sessions: List[Dict], agents: List[Dict]) -> Dict[str, Optional[str]]:
    """从所有消息中提取时间范围"""
    all_timestamps = []

    for session in sessions:
        for msg in session.get("messages", []):
            if "timestamp" in msg:
                all_timestamps.append(msg["timestamp"])

    for agent in agents:
        for msg in agent.get("messages", []):
            if "timestamp" in msg:
                all_timestamps.append(msg["timestamp"])

    if not all_timestamps:
        return {"start": None, "end": None}

    all_timestamps.sort()
    return {
        "start": all_timestamps[0],
        "end": all_timestamps[-1]
    }


def count_total_messages(sessions: List[Dict], agents: List[Dict]) -> int:
    """统计总消息数"""
    total = 0
    for session in sessions:
        total += len(session.get("messages", []))
    for agent in agents:
        total += len(agent.get("messages", []))
    return total


def extract_all() -> None:
    """执行全量抽取"""
    print(f"开始抽取对话历史...")
    print(f"数据源: {CLAUDE_PROJECT_DIR}")

    # 检查数据源目录
    if not CLAUDE_PROJECT_DIR.exists():
        print(f"错误: 数据源目录不存在: {CLAUDE_PROJECT_DIR}")
        return

    # 1. 读取会话索引
    sessions_index = {}
    sessions_index_file = CLAUDE_PROJECT_DIR / "sessions-index.json"
    if sessions_index_file.exists():
        with open(sessions_index_file, 'r', encoding='utf-8') as f:
            sessions_index = json.load(f)
        print(f"读取会话索引: {len(sessions_index.get('sessions', []))} 个会话")
    else:
        print("警告: 未找到 sessions-index.json")

    # 2. 提取所有会话
    sessions = []
    agents = []
    jsonl_files = list(CLAUDE_PROJECT_DIR.glob("*.jsonl"))
    print(f"发现 {len(jsonl_files)} 个 JSONL 文件")

    for file in sorted(jsonl_files):
        messages = parse_jsonl(file)
        file_size = file.stat().st_size / 1024 / 1024  # MB

        if file.name.startswith("agent-"):
            agent_id = file.stem.replace("agent-", "")
            agents.append({
                "agent_id": agent_id,
                "filename": file.name,
                "file_size_mb": round(file_size, 2),
                "message_count": len(messages),
                "messages": messages
            })
            print(f"  代理会话: {file.name} ({len(messages)} 条消息, {file_size:.2f}MB)")
        else:
            session_id = file.stem
            # 从索引中查找会话元数据
            session_meta = {}
            for s in sessions_index.get("sessions", []):
                if s.get("id") == session_id:
                    session_meta = s
                    break

            sessions.append({
                "session_id": session_id,
                "filename": file.name,
                "file_size_mb": round(file_size, 2),
                "summary": session_meta.get("summary", ""),
                "created": session_meta.get("created", ""),
                "modified": session_meta.get("modified", ""),
                "message_count": len(messages),
                "messages": messages
            })
            print(f"  主会话: {file.name} ({len(messages)} 条消息, {file_size:.2f}MB)")

    # 3. 统计信息
    total_messages = count_total_messages(sessions, agents)
    date_range = get_date_range(sessions, agents)
    total_size_mb = sum(s["file_size_mb"] for s in sessions) + sum(a["file_size_mb"] for a in agents)

    # 4. 构建输出
    output = {
        "metadata": {
            "project": "RealConsole",
            "extraction_time": datetime.now().isoformat(),
            "source_path": str(CLAUDE_PROJECT_DIR),
            "total_sessions": len(sessions),
            "total_agents": len(agents),
            "total_files": len(sessions) + len(agents),
            "total_messages": total_messages,
            "total_size_mb": round(total_size_mb, 2),
            "date_range": date_range
        },
        "sessions_index": sessions_index,
        "sessions": sessions,
        "agents": agents
    }

    # 5. 写入备份文件
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    output_file = OUTPUT_DIR / f"{datetime.now().strftime('%Y-%m-%d')}-full-backup.json"

    print(f"\n写入备份文件: {output_file}")
    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(output, f, ensure_ascii=False, indent=2)

    output_size_mb = output_file.stat().st_size / 1024 / 1024

    # 6. 输出统计摘要
    print(f"\n{'='*50}")
    print(f"备份完成!")
    print(f"{'='*50}")
    print(f"  主会话数: {len(sessions)}")
    print(f"  代理会话数: {len(agents)}")
    print(f"  总消息数: {total_messages}")
    print(f"  源文件总大小: {total_size_mb:.2f} MB")
    print(f"  备份文件大小: {output_size_mb:.2f} MB")
    print(f"  时间范围: {date_range['start']} ~ {date_range['end']}")
    print(f"  输出文件: {output_file}")


if __name__ == "__main__":
    extract_all()
