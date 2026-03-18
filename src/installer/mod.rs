use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use inquire::MultiSelect;
use inquire::Text;

const SKILL_CONTENT: &str = include_str!("../../skills/SKILL.md");

struct CodingTool {
    name: &'static str,
    binary: &'static str,
    skills_dir: &'static str,
}

const TOOLS: &[CodingTool] = &[
    CodingTool {
        name: "Claude Code",
        binary: "claude",
        skills_dir: ".claude/skills",
    },
    CodingTool {
        name: "OpenCode",
        binary: "opencode",
        skills_dir: ".config/opencode/skills",
    },
    CodingTool {
        name: "Cline",
        binary: "cline",
        skills_dir: ".cline/skills",
    },
    CodingTool {
        name: "Codex",
        binary: "codex",
        skills_dir: ".codex/skills",
    },
    CodingTool {
        name: "Gemini CLI",
        binary: "gemini",
        skills_dir: ".gemini/skills",
    },
    CodingTool {
        name: "GitHub Copilot",
        binary: "copilot",
        skills_dir: ".copilot/skills",
    },
    CodingTool {
        name: "Mistral Vibe",
        binary: "vibe",
        skills_dir: ".vibe/skills",
    },
];

struct InstallOption {
    label: String,
    target_path: PathBuf,
    is_custom: bool,
}

impl fmt::Display for InstallOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}

fn is_tool_installed(tool: &CodingTool, home: &Path) -> bool {
    // Check if binary is on PATH (cross-platform: Command searches PATH natively)
    if Command::new(tool.binary)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
    {
        return true;
    }

    // Fallback: check if config directory exists (e.g. ~/.cline/ for VS Code extensions)
    let config_dir = home.join(
        tool.skills_dir
            .rsplit_once('/')
            .map(|(parent, _)| parent)
            .unwrap_or(tool.skills_dir),
    );
    config_dir.exists()
}

fn target_path(home: &Path, skills_dir: &str) -> PathBuf {
    home.join(skills_dir).join("fretka").join("SKILL.md")
}

fn install_skill(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("failed to create directory: {e}"))?;
    }
    fs::write(path, SKILL_CONTENT).map_err(|e| format!("failed to write file: {e}"))?;
    Ok(())
}

pub fn run() {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => {
            eprintln!("error: could not determine home directory");
            std::process::exit(1);
        }
    };

    let mut options: Vec<InstallOption> = Vec::new();

    for tool in TOOLS {
        if is_tool_installed(tool, &home) {
            let path = target_path(&home, tool.skills_dir);
            let display_path = path.display();
            options.push(InstallOption {
                label: format!("{:<20} ({display_path})", tool.name),
                target_path: path,
                is_custom: false,
            });
        }
    }

    options.push(InstallOption {
        label: format!("{:<20} (custom path, prompted after confirming)", "Custom"),
        target_path: PathBuf::new(),
        is_custom: true,
    });

    let selected = match MultiSelect::new("Install fretka skill to:", options)
        .with_formatter(&|opts| {
            opts.iter()
                .map(|o| o.value.label.split('(').next().unwrap_or("").trim())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .prompt()
    {
        Ok(s) => s,
        Err(_) => {
            eprintln!("cancelled");
            std::process::exit(1);
        }
    };

    if selected.is_empty() {
        eprintln!("no targets selected");
        std::process::exit(1);
    }

    let mut targets: Vec<PathBuf> = Vec::new();

    for option in &selected {
        if option.is_custom {
            let custom = match Text::new("Enter skills directory path:").prompt() {
                Ok(p) => p,
                Err(_) => {
                    eprintln!("cancelled");
                    std::process::exit(1);
                }
            };
            let trimmed = custom.trim();
            let base = if let Some(rest) = trimmed.strip_prefix("~/") {
                home.join(rest)
            } else {
                PathBuf::from(trimmed)
            };
            let path = base.join("fretka").join("SKILL.md");
            targets.push(path);
        } else {
            targets.push(option.target_path.clone());
        }
    }

    let total = targets.len();
    let mut succeeded = 0;
    for path in &targets {
        match install_skill(path) {
            Ok(()) => {
                succeeded += 1;
                println!("  Installed: {}", path.display());
            }
            Err(e) => eprintln!("  Failed {}: {e}", path.display()),
        }
    }
    if succeeded > 0 {
        println!("  Done — {succeeded}/{total} target(s)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn target_path_builds_correct_path() {
        let home = PathBuf::from("/home/user");
        assert_eq!(
            target_path(&home, ".claude/skills"),
            PathBuf::from("/home/user/.claude/skills/fretka/SKILL.md")
        );
    }

    #[test]
    fn target_path_handles_nested_skills_dir() {
        let home = PathBuf::from("/home/user");
        assert_eq!(
            target_path(&home, ".config/opencode/skills"),
            PathBuf::from("/home/user/.config/opencode/skills/fretka/SKILL.md")
        );
    }

    #[test]
    fn install_skill_creates_dirs_and_writes_content() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("fretka").join("SKILL.md");

        install_skill(&path).expect("install should succeed");

        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, SKILL_CONTENT);
    }

    #[test]
    fn install_skill_overwrites_existing_file() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("fretka").join("SKILL.md");

        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "old content").unwrap();

        install_skill(&path).expect("install should succeed");

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, SKILL_CONTENT);
    }

    #[test]
    fn tools_have_valid_fields() {
        for tool in TOOLS {
            assert!(!tool.name.is_empty(), "tool name must not be empty");
            assert!(!tool.binary.is_empty(), "tool binary must not be empty");
            assert!(
                tool.skills_dir.ends_with("skills"),
                "{}: skills_dir '{}' must end with 'skills'",
                tool.name,
                tool.skills_dir
            );
        }
    }

    #[test]
    fn tools_have_unique_names_and_binaries() {
        for (i, a) in TOOLS.iter().enumerate() {
            for b in &TOOLS[i + 1..] {
                assert_ne!(a.name, b.name, "duplicate tool name: {}", a.name);
                assert_ne!(a.binary, b.binary, "duplicate binary: {}", a.binary);
                assert_ne!(
                    a.skills_dir, b.skills_dir,
                    "duplicate skills_dir: {}",
                    a.skills_dir
                );
            }
        }
    }

    #[test]
    fn skill_content_is_not_empty() {
        assert!(!SKILL_CONTENT.is_empty());
        assert!(SKILL_CONTENT.contains("fretka"));
    }
}
