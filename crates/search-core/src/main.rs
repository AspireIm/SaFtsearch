//! 命令行工具，用于扫描目录和搜索文件内容。
//!
//! 支持子命令：
//! - `config`   : 打印默认配置（可扩展为读取配置文件）
//! - `scan`     : 扫描目录并输出文件特征（JSON）
//! - `search`   : 执行搜索，输出匹配结果（JSON）
//!
//! 所有子命令均可通过参数调整排除模式、符号链接行为等。

use anyhow::{bail, Result};
use saftsearch_core::{search_with_config, IndexConfig};
use std::{env, path::PathBuf};

// ========== 命令行参数解析（手动实现，但支持丰富的选项） ==========

/// 解析命令行参数，返回子命令和对应的参数结构
fn parse_args() -> Result<Command> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_help();
        bail!("Missing subcommand");
    }

    let subcommand = args[1].as_str();
    match subcommand {
        "config" => Ok(Command::Config),
        "scan" => {
            // scan [ROOT] [--exclude PATTERN]... [--follow-symlinks]
            let (root, excludes, follow_symlinks) = parse_common_options(&args, 2)?;
            Ok(Command::Scan {
                root,
                excludes,
                follow_symlinks,
            })
        }
        "search" => {
            // search <QUERY> [ROOT] [--exclude PATTERN]... [--follow-symlinks] [--limit N]
            let query = args.get(2).map(String::as_str).unwrap_or("");
            if query.is_empty() {
                bail!("Search query cannot be empty");
            }
            let (root, excludes, follow_symlinks) = parse_common_options(&args, 3)?;
            let limit = parse_limit(&args)?.unwrap_or(50);
            Ok(Command::Search {
                query: query.to_string(),
                root,
                excludes,
                follow_symlinks,
                limit,
            })
        }
        other => bail!("Unknown command: {}", other),
    }
}

/// 解析公共选项：根目录、排除模式、是否跟随符号链接
fn parse_common_options(args: &[String], root_pos: usize) -> Result<(PathBuf, Vec<String>, bool)> {
    let mut root = None;
    let mut excludes = Vec::new();
    let mut follow_symlinks = false;

    let mut i = root_pos;
    while i < args.len() {
        match args[i].as_str() {
            "--exclude" => {
                if i + 1 >= args.len() {
                    bail!("Missing pattern for --exclude");
                }
                excludes.push(args[i + 1].clone());
                i += 2;
            }
            "--follow-symlink" | "--follow-symlinks" => {
                follow_symlinks = true;
                i += 1;
            }
            "--limit" => {
                if i + 1 >= args.len() {
                    bail!("Missing value for --limit");
                }
                i += 2;
            }
            option if option.starts_with("--") => {
                bail!("Unknown option: {}", option);
            }
            value => {
                if root.is_some() {
                    bail!("Unexpected positional argument: {}", value);
                }
                root = Some(PathBuf::from(value));
                i += 1;
            }
        }
    }

    Ok((root.unwrap_or_else(|| PathBuf::from(".")), excludes, follow_symlinks))
}

/// 解析 --limit 参数
fn parse_limit(args: &[String]) -> Result<Option<usize>> {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--limit" {
            let value = args
                .get(i + 1)
                .ok_or_else(|| anyhow::anyhow!("Missing value for --limit"))?;
            let limit = value
                .parse::<usize>()
                .map_err(|_| anyhow::anyhow!("Invalid --limit value: {}", value))?;
            return Ok(Some(limit));
        }
    }
    Ok(None)
}

/// 打印简易帮助信息
fn print_help() {
    eprintln!(
        r#"Usage: saftsearch <COMMAND> [OPTIONS]

Commands:
  config                                 Print default configuration
  scan [ROOT] [--exclude PATTERN]... [--follow-symlinks]
                                         Scan directory and output features
  search <QUERY> [ROOT] [--exclude PATTERN]... [--follow-symlinks] [--limit N]
                                         Search for query in directory

Examples:
  saftsearch scan ./src --exclude tests
  saftsearch search "fn main" . --limit 10 --follow-symlinks
"#
    );
}

// ========== 命令枚举 ==========

#[derive(Debug)]
enum Command {
    Config,
    Scan {
        root: PathBuf,
        excludes: Vec<String>,
        follow_symlinks: bool,
    },
    Search {
        query: String,
        root: PathBuf,
        excludes: Vec<String>,
        follow_symlinks: bool,
        limit: usize,
    },
}

// ========== 主函数 ==========

fn main() -> Result<()> {
    let command = parse_args()?;

    match command {
        Command::Config => {
            let config = saftsearch_core::default_config(".");
            println!("{}", serde_json::to_string_pretty(&config)?);
        }
        Command::Scan {
            root,
            excludes,
            follow_symlinks,
        } => {
            let config = IndexConfig {
                roots: vec![root],
                exclude_patterns: excludes,
                follow_symlinks,
            };
            let root_path = &config.roots[0];
            let features = saftsearch_core::scanner::scan_root(
                root_path,
                &config.exclude_patterns,
                config.follow_symlinks,
            )?;
            println!("{}", serde_json::to_string_pretty(&features)?);
        }
        Command::Search {
            query,
            root,
            excludes,
            follow_symlinks,
            limit,
        } => {
            let config = IndexConfig {
                roots: vec![root],
                exclude_patterns: excludes,
                follow_symlinks,
            };
            let hits = search_with_config(&config, &query, limit)?;
            println!("{}", serde_json::to_string_pretty(&hits)?);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn common_options_allow_options_without_root() {
        let parsed = parse_common_options(&args(&["saftsearch", "scan", "--exclude", "target"]), 2)
            .expect("options without root should parse");

        assert_eq!(parsed.0, PathBuf::from("."));
        assert_eq!(parsed.1, vec!["target".to_string()]);
        assert!(!parsed.2);
    }

    #[test]
    fn common_options_parse_root_and_follow_symlinks() {
        let parsed = parse_common_options(
            &args(&["saftsearch", "scan", "src", "--follow-symlinks"]),
            2,
        )
        .expect("root and follow flag should parse");

        assert_eq!(parsed.0, PathBuf::from("src"));
        assert!(parsed.2);
    }

    #[test]
    fn limit_requires_numeric_value() {
        let result = parse_limit(&args(&["saftsearch", "search", "toml", ".", "--limit", "x"]));
        assert!(result.is_err());
    }
}
