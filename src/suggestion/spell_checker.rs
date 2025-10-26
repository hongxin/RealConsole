//! 命令拼写纠错模块
//!
//! 基于 Levenshtein 距离算法检测命令拼写错误并提供修正建议
//!
//! ## 设计理念
//!
//! 遵循"一分为三"哲学：
//! - **精确匹配** (距离=0)：完全正确，不需要建议
//! - **模糊匹配** (距离=1-2)：轻微拼写错误，高置信度建议
//! - **远距离匹配** (距离>2)：可能的误输入，低置信度建议
//!
//! ## 示例
//!
//! ```rust
//! use realconsole::suggestion::SpellChecker;
//!
//! let checker = SpellChecker::new();
//! let suggestions = checker.check_and_suggest("cago", None);
//!
//! // 预期建议 "cargo" with high score
//! assert!(!suggestions.is_empty());
//! assert_eq!(suggestions[0].command, "cargo");
//! ```

use super::types::{Suggestion, SuggestionCategory, SuggestionSource};
use std::collections::HashSet;

/// 拼写检查器
///
/// 使用 Levenshtein 距离算法检测命令拼写错误
pub struct SpellChecker {
    /// 常用命令词典（用于匹配）
    common_commands: HashSet<String>,
}

impl SpellChecker {
    /// 创建新的拼写检查器
    pub fn new() -> Self {
        Self {
            common_commands: Self::build_common_commands(),
        }
    }

    /// 添加自定义命令到词典
    pub fn add_command(&mut self, command: String) {
        self.common_commands.insert(command);
    }

    /// 批量添加命令
    pub fn add_commands(&mut self, commands: Vec<String>) {
        for cmd in commands {
            self.common_commands.insert(cmd);
        }
    }

    /// 检查命令拼写并生成建议
    ///
    /// # 参数
    /// - `input`: 用户输入的命令（可能有拼写错误）
    /// - `last_output`: 上一次命令的输出（用于检测"command not found"）
    ///
    /// # 返回
    /// 建议列表，按相似度排序
    pub fn check_and_suggest(&self, input: &str, last_output: Option<&str>) -> Vec<Suggestion> {
        // 提取输入的第一个词（命令部分）
        let command = input.split_whitespace().next().unwrap_or(input);

        // 如果命令在词典中，不需要建议
        if self.common_commands.contains(command) {
            return Vec::new();
        }

        // 如果有输出，检查是否包含 "command not found"
        if let Some(output) = last_output {
            if !Self::is_command_not_found_error(output) {
                // 不是拼写错误，而是其他错误，不提供拼写建议
                return Vec::new();
            }
        }

        // 计算与所有常用命令的距离，找到最相似的
        let mut candidates: Vec<(String, usize)> = self
            .common_commands
            .iter()
            .map(|cmd| {
                let distance = Self::levenshtein_distance(command, cmd);
                (cmd.clone(), distance)
            })
            .collect();

        // 按距离排序
        candidates.sort_by_key(|(_, dist)| *dist);

        // 生成建议
        let mut suggestions = Vec::new();
        for (cmd, distance) in candidates.into_iter().take(5) {
            // 距离太大，不建议
            if distance > 3 {
                break;
            }

            // 根据距离计算分数
            let score = Self::distance_to_score(distance, command.len());

            // 只保留高质量建议（分数 > 0.5）
            if score < 0.5 {
                break;
            }

            suggestions.push(
                Suggestion::new(
                    cmd.clone(),
                    format!("Did you mean '{}'?", cmd),
                    score,
                    SuggestionSource::Rule,
                )
                .with_category(SuggestionCategory::Diagnostic),
            );
        }

        suggestions
    }

    /// 计算两个字符串的 Levenshtein 距离
    ///
    /// Levenshtein 距离是将一个字符串转换为另一个字符串所需的最少单字符编辑次数
    /// （插入、删除、替换）
    ///
    /// # 算法复杂度
    /// - 时间复杂度: O(m×n)
    /// - 空间复杂度: O(n)
    fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();

        // 边界情况
        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        // 动态规划：使用滚动数组优化空间
        let mut prev_row: Vec<usize> = (0..=len2).collect();
        let mut curr_row: Vec<usize> = vec![0; len2 + 1];

        for (i, c1) in s1.chars().enumerate() {
            curr_row[0] = i + 1;

            for (j, c2) in s2.chars().enumerate() {
                let cost = if c1 == c2 { 0 } else { 1 };

                curr_row[j + 1] = *[
                    prev_row[j + 1] + 1,    // 删除
                    curr_row[j] + 1,        // 插入
                    prev_row[j] + cost,     // 替换
                ]
                .iter()
                .min()
                .unwrap();
            }

            std::mem::swap(&mut prev_row, &mut curr_row);
        }

        prev_row[len2]
    }

    /// 将编辑距离转换为建议分数
    ///
    /// 分数计算考虑：
    /// - 编辑距离
    /// - 原始字符串长度（短字符串的误差影响更大）
    ///
    /// 一分为三的评分策略：
    /// - 距离 1: 高置信度 (0.9-0.95)
    /// - 距离 2: 中等置信度 (0.7-0.8)
    /// - 距离 3: 低置信度 (0.5-0.6)
    fn distance_to_score(distance: usize, input_len: usize) -> f64 {
        match distance {
            0 => 1.0,  // 完全匹配（不应出现在建议中）
            1 => {
                // 距离1：高置信度，长度越短分数略低
                if input_len <= 3 {
                    0.85  // 短命令，单字符错误影响较大
                } else {
                    0.93  // 长命令，单字符错误影响小
                }
            }
            2 => {
                // 距离2：中等置信度
                if input_len <= 4 {
                    0.65
                } else {
                    0.78
                }
            }
            3 => {
                // 距离3：低置信度
                if input_len <= 5 {
                    0.45  // 太短，可能不是拼写错误
                } else {
                    0.58
                }
            }
            _ => 0.3,  // 距离太大，不太可能是拼写错误
        }
    }

    /// 检测错误输出是否为 "command not found"
    fn is_command_not_found_error(output: &str) -> bool {
        let lower = output.to_lowercase();
        lower.contains("command not found")
            || lower.contains("not found")
            || lower.contains("no such file or directory")
    }

    /// 构建常用命令词典
    ///
    /// 包含：
    /// - 系统命令 (ls, cd, cat, etc.)
    /// - 开发工具 (git, cargo, npm, etc.)
    /// - 常用工具 (grep, find, curl, etc.)
    fn build_common_commands() -> HashSet<String> {
        let commands = vec![
            // 系统基础命令
            "ls", "cd", "pwd", "cat", "echo", "mkdir", "rm", "cp", "mv", "touch", "chmod",
            "chown", "ln", "find", "grep", "sed", "awk", "cut", "sort", "uniq", "head", "tail",
            "less", "more", "diff", "which", "whereis", "man", "history", "clear", "exit",
            // 文件操作
            "tar", "zip", "unzip", "gzip", "gunzip", "bzip2", "xz",
            // 网络工具
            "curl", "wget", "ssh", "scp", "rsync", "ping", "nc", "netstat", "ifconfig", "ip",
            // 系统监控
            "top", "htop", "ps", "kill", "killall", "df", "du", "free", "uptime", "w", "who",
            // Git 命令
            "git", "gitk", "tig",
            // 开发工具 - Rust
            "cargo", "rustc", "rustup", "rustfmt", "clippy",
            // 开发工具 - Node.js
            "npm", "npx", "yarn", "pnpm", "node",
            // 开发工具 - Python
            "python", "python3", "pip", "pip3", "poetry", "pipenv", "pytest",
            // 开发工具 - Go
            "go", "gofmt",
            // 开发工具 - Java
            "java", "javac", "mvn", "gradle",
            // 容器工具
            "docker", "docker-compose", "podman", "kubectl", "helm",
            // 编辑器
            "vim", "nvim", "nano", "emacs", "code", "subl",
            // 构建工具
            "make", "cmake", "ninja",
            // 包管理器
            "brew", "apt", "yum", "dnf", "pacman",
            // 其他常用工具
            "tmux", "screen", "tree", "jq", "yq", "fd", "rg", "bat", "exa", "zoxide",
        ];

        commands.into_iter().map(String::from).collect()
    }
}

impl Default for SpellChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(SpellChecker::levenshtein_distance("", ""), 0);
        assert_eq!(SpellChecker::levenshtein_distance("a", ""), 1);
        assert_eq!(SpellChecker::levenshtein_distance("", "a"), 1);
        assert_eq!(SpellChecker::levenshtein_distance("abc", "abc"), 0);
        assert_eq!(SpellChecker::levenshtein_distance("abc", "abd"), 1);
        assert_eq!(SpellChecker::levenshtein_distance("abc", "ac"), 1);
        assert_eq!(SpellChecker::levenshtein_distance("abc", "abcd"), 1);
        assert_eq!(SpellChecker::levenshtein_distance("kitten", "sitting"), 3);
    }

    #[test]
    fn test_distance_to_score() {
        // 距离 1
        assert!(SpellChecker::distance_to_score(1, 5) > 0.9);
        assert!(SpellChecker::distance_to_score(1, 3) > 0.8);

        // 距离 2
        assert!(SpellChecker::distance_to_score(2, 5) > 0.7);
        assert!(SpellChecker::distance_to_score(2, 5) < 0.8);

        // 距离 3
        assert!(SpellChecker::distance_to_score(3, 6) > 0.5);
        assert!(SpellChecker::distance_to_score(3, 6) < 0.6);

        // 距离太大
        assert!(SpellChecker::distance_to_score(5, 10) < 0.5);
    }

    #[test]
    fn test_spell_checker_cargo_typo() {
        let checker = SpellChecker::new();

        // "cago" -> "cargo"
        let suggestions = checker.check_and_suggest(
            "cago",
            Some("zsh: command not found: cago"),
        );

        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].command, "cargo");
        assert!(suggestions[0].score > 0.9);
        assert_eq!(suggestions[0].source, SuggestionSource::Rule);
    }

    #[test]
    fn test_spell_checker_git_typo() {
        let checker = SpellChecker::new();

        // "gi" -> "git" (distance = 1, 缺少最后一个字符)
        let suggestions = checker.check_and_suggest(
            "gi",
            Some("bash: gi: command not found"),
        );

        assert!(!suggestions.is_empty());
        // git 应该在建议列表中
        assert!(suggestions.iter().any(|s| s.command == "git"));

        // 找到 git 建议并验证分数
        let git_suggestion = suggestions.iter().find(|s| s.command == "git").unwrap();
        assert!(git_suggestion.score >= 0.85, "git suggestion score should be >= 0.85, got {}", git_suggestion.score);
    }

    #[test]
    fn test_spell_checker_npm_typo() {
        let checker = SpellChecker::new();

        // "npn" -> "npm" (distance = 1, 替换最后一个字符)
        // 使用更好的例子：npn 而不是 nmp
        let suggestions = checker.check_and_suggest(
            "npn",
            Some("command not found: npn"),
        );

        assert!(!suggestions.is_empty());
        // npm 应该在建议中（可能不是第一个，但应该在前几个）
        assert!(suggestions.iter().any(|s| s.command == "npm"));

        // 第一个建议应该是距离最小的
        let first_distance = SpellChecker::levenshtein_distance("npn", &suggestions[0].command);
        assert_eq!(first_distance, 1);
    }

    #[test]
    fn test_spell_checker_no_suggestion_for_correct_command() {
        let checker = SpellChecker::new();

        // "ls" 是正确的命令，不应该有建议
        let suggestions = checker.check_and_suggest("ls", None);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_spell_checker_no_suggestion_for_other_errors() {
        let checker = SpellChecker::new();

        // 不是 "command not found" 错误，不应该有拼写建议
        let suggestions = checker.check_and_suggest(
            "cargo",
            Some("error: could not compile"),
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_spell_checker_multiple_candidates() {
        let checker = SpellChecker::new();

        // "car" 可能匹配多个命令（cargo, cat, tar, ...）
        let suggestions = checker.check_and_suggest(
            "car",
            Some("command not found: car"),
        );

        // 应该有多个建议
        assert!(!suggestions.is_empty());
        // 第一个建议应该是距离最小的
        assert!(suggestions[0].score >= suggestions.last().unwrap().score);
    }

    #[test]
    fn test_spell_checker_add_custom_command() {
        let mut checker = SpellChecker::new();

        // 添加自定义命令
        checker.add_command("realconsole".to_string());

        // "realconsol" -> "realconsole"
        let suggestions = checker.check_and_suggest(
            "realconsol",
            Some("command not found: realconsol"),
        );

        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].command, "realconsole");
        assert!(suggestions[0].score > 0.9);
    }

    #[test]
    fn test_spell_checker_distance_too_large() {
        let checker = SpellChecker::new();

        // "abcdefgh" 与任何常用命令的距离都很大
        let suggestions = checker.check_and_suggest(
            "abcdefgh",
            Some("command not found: abcdefgh"),
        );

        // 应该没有高质量建议，或者建议分数很低
        if !suggestions.is_empty() {
            assert!(suggestions[0].score < 0.7);
        }
    }

    #[test]
    fn test_is_command_not_found_error() {
        assert!(SpellChecker::is_command_not_found_error(
            "zsh: command not found: cago"
        ));
        assert!(SpellChecker::is_command_not_found_error(
            "bash: gti: command not found"
        ));
        assert!(SpellChecker::is_command_not_found_error(
            "Command not found"
        ));
        assert!(SpellChecker::is_command_not_found_error(
            "no such file or directory"
        ));

        assert!(!SpellChecker::is_command_not_found_error(
            "error: could not compile"
        ));
        assert!(!SpellChecker::is_command_not_found_error(
            "permission denied"
        ));
    }

    #[test]
    fn test_common_commands_contains_expected() {
        let checker = SpellChecker::new();
        assert!(checker.common_commands.contains("git"));
        assert!(checker.common_commands.contains("cargo"));
        assert!(checker.common_commands.contains("npm"));
        assert!(checker.common_commands.contains("docker"));
        assert!(checker.common_commands.contains("ls"));
    }
}
