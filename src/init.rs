use std::fs;
use std::path::{Path, PathBuf};

const STARTER_HELL_CODE_JSON: &str = concat!(
    "{\n",
    "  \"permissions\": {\n",
    "    \"defaultMode\": \"dontAsk\"\n",
    "  }\n",
    "}\n",
);
const GITIGNORE_COMMENT: &str = "# HELL-CODE local artifacts";
const GITIGNORE_ENTRIES: [&str; 3] = [".hell-code/settings.local.json", ".hell-code/sessions/", ".hell-code/logs/"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InitStatus {
    Created,
    Updated,
    Skipped,
}

impl InitStatus {
    #[must_use]
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Updated => "updated",
            Self::Skipped => "skipped (already exists)",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InitArtifact {
    pub(crate) name: &'static str,
    pub(crate) status: InitStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InitReport {
    pub(crate) project_root: PathBuf,
    pub(crate) artifacts: Vec<InitArtifact>,
}

impl InitReport {
    #[must_use]
    pub(crate) fn render(&self) -> String {
        let mut lines = vec![
            "Initialize HELL-CODE Workspace".to_string(),
            format!("  Project          {}", self.project_root.display()),
        ];
        for artifact in &self.artifacts {
            lines.push(format!(
                "  {:<20} {}",
                artifact.name,
                artifact.status.label()
            ));
        }
        lines.push("  Next step        Review HELL-CODE.md and customize your verified commands".to_string());
        lines.join("\n")
    }
}

pub(crate) fn initialize_repo(cwd: &Path) -> Result<InitReport, Box<dyn std::error::Error>> {
    let mut artifacts = Vec::new();

    let hc_dir = cwd.join(".hell-code");
    artifacts.push(InitArtifact {
        name: ".hell-code/",
        status: ensure_dir(&hc_dir)?,
    });

    let hc_json = cwd.join(".hell-code.json");
    artifacts.push(InitArtifact {
        name: ".hell-code.json",
        status: write_file_if_missing(&hc_json, STARTER_HELL_CODE_JSON)?,
    });

    let gitignore = cwd.join(".gitignore");
    artifacts.push(InitArtifact {
        name: ".gitignore",
        status: ensure_gitignore_entries(&gitignore)?,
    });

    let guidance_md = cwd.join("HELL-CODE.md");
    let content = render_init_hell_code_md(cwd);
    artifacts.push(InitArtifact {
        name: "HELL-CODE.md",
        status: write_file_if_missing(&guidance_md, &content)?,
    });

    Ok(InitReport {
        project_root: cwd.to_path_buf(),
        artifacts,
    })
}

fn ensure_dir(path: &Path) -> Result<InitStatus, std::io::Error> {
    if path.is_dir() {
        return Ok(InitStatus::Skipped);
    }
    fs::create_dir_all(path)?;
    Ok(InitStatus::Created)
}

fn write_file_if_missing(path: &Path, content: &str) -> Result<InitStatus, std::io::Error> {
    if path.exists() {
        return Ok(InitStatus::Skipped);
    }
    fs::write(path, content)?;
    Ok(InitStatus::Created)
}

fn ensure_gitignore_entries(path: &Path) -> Result<InitStatus, std::io::Error> {
    if !path.exists() {
        let mut lines = vec![GITIGNORE_COMMENT.to_string()];
        lines.extend(GITIGNORE_ENTRIES.iter().map(|entry| (*entry).to_string()));
        fs::write(path, format!("{}\n", lines.join("\n")))?;
        return Ok(InitStatus::Created);
    }

    let existing = fs::read_to_string(path)?;
    let mut lines = existing.lines().map(ToOwned::to_owned).collect::<Vec<_>>();
    let mut changed = false;

    if !lines.iter().any(|line| line == GITIGNORE_COMMENT) {
        lines.push(GITIGNORE_COMMENT.to_string());
        changed = true;
    }

    for entry in GITIGNORE_ENTRIES {
        if !lines.iter().any(|line| line == entry) {
            lines.push(entry.to_string());
            changed = true;
        }
    }

    if !changed {
        return Ok(InitStatus::Skipped);
    }

    fs::write(path, format!("{}\n", lines.join("\n")))?;
    Ok(InitStatus::Updated)
}

pub(crate) fn render_init_hell_code_md(cwd: &Path) -> String {
    let detection = detect_repo(cwd);
    let mut lines = vec![
        "# HELL-CODE Guidance".to_string(),
        String::new(),
        "This file provides context and operational rules for HELL-CODE when working in this repository.".to_string(),
        String::new(),
    ];

    let detected_languages = detected_languages(&detection);
    let detected_frameworks = detected_frameworks(&detection);
    lines.push("## Detected Stack".to_string());
    if detected_languages.is_empty() {
        lines.push("- No specific language markers were detected yet.".to_string());
    } else {
        lines.push(format!("- **Languages**: {}.", detected_languages.join(", ")));
    }
    if detected_frameworks.is_empty() {
        lines.push("- **Frameworks**: none detected.".to_string());
    } else {
        lines.push(format!(
            "- **Frameworks/Tooling**: {}.",
            detected_frameworks.join(", ")
        ));
    }
    lines.push(String::new());

    let verification_lines = verification_lines(cwd, &detection);
    if !verification_lines.is_empty() {
        lines.push("## Verification Commands".to_string());
        lines.extend(verification_lines);
        lines.push(String::new());
    }

    let structure_lines = repository_shape_lines(&detection);
    if !structure_lines.is_empty() {
        lines.push("## Repository Structure".to_string());
        lines.extend(structure_lines);
        lines.push(String::new());
    }

    let framework_lines = framework_notes(&detection);
    if !framework_lines.is_empty() {
        lines.push("## Framework Guidance".to_string());
        lines.extend(framework_lines);
        lines.push(String::new());
    }

    lines.push("## Working Agreement".to_string());
    lines.push("- Prefer small, reviewable changes.".to_string());
    lines.push("- Use `.hell-code.json` for project-wide rules.".to_string());
    lines.push("- Update this file (`HELL-CODE.md`) if repository workflows evolve.".to_string());
    lines.push(String::new());

    lines.join("\n")
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct RepoDetection {
    rust_workspace: bool,
    rust_root: bool,
    python: bool,
    package_json: bool,
    typescript: bool,
    nextjs: bool,
    react: bool,
    vite: bool,
    nest: bool,
    src_dir: bool,
    tests_dir: bool,
    rust_dir: bool,
}

fn detect_repo(cwd: &Path) -> RepoDetection {
    let package_json_contents = fs::read_to_string(cwd.join("package.json"))
        .unwrap_or_default()
        .to_ascii_lowercase();
    RepoDetection {
        rust_workspace: cwd.join("rust").join("Cargo.toml").is_file(),
        rust_root: cwd.join("Cargo.toml").is_file(),
        python: cwd.join("pyproject.toml").is_file()
            || cwd.join("requirements.txt").is_file()
            || cwd.join("setup.py").is_file(),
        package_json: cwd.join("package.json").is_file(),
        typescript: cwd.join("tsconfig.json").is_file()
            || package_json_contents.contains("typescript"),
        nextjs: package_json_contents.contains("\"next\""),
        react: package_json_contents.contains("\"react\""),
        vite: package_json_contents.contains("\"vite\""),
        nest: package_json_contents.contains("@nestjs"),
        src_dir: cwd.join("src").is_dir(),
        tests_dir: cwd.join("tests").is_dir(),
        rust_dir: cwd.join("rust").is_dir(),
    }
}

fn detected_languages(detection: &RepoDetection) -> Vec<&'static str> {
    let mut languages = Vec::new();
    if detection.rust_workspace || detection.rust_root {
        languages.push("Rust");
    }
    if detection.python {
        languages.push("Python");
    }
    if detection.typescript {
        languages.push("TypeScript");
    } else if detection.package_json {
        languages.push("JavaScript/Node.js");
    }
    languages
}

fn detected_frameworks(detection: &RepoDetection) -> Vec<&'static str> {
    let mut frameworks = Vec::new();
    if detection.nextjs {
        frameworks.push("Next.js");
    }
    if detection.react {
        frameworks.push("React");
    }
    if detection.vite {
        frameworks.push("Vite");
    }
    if detection.nest {
        frameworks.push("NestJS");
    }
    frameworks
}

fn verification_lines(cwd: &Path, detection: &RepoDetection) -> Vec<String> {
    let mut lines = Vec::new();
    if detection.rust_workspace {
        lines.push("- Run Rust verification from `rust/`: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`".to_string());
    } else if detection.rust_root {
        lines.push("- Run Rust verification from the repo root: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`".to_string());
    }
    if detection.python {
        if cwd.join("pyproject.toml").is_file() {
            lines.push("- Run Python checks: `pytest`, `ruff check`, and `mypy`.".to_string());
        } else {
            lines.push("- Run Python test/lint commands before shipping changes.".to_string());
        }
    }
    if detection.package_json {
        lines.push("- Run JS/TS checks from `package.json`: `npm test`, `npm run lint`, etc.".to_string());
    }
    if detection.tests_dir && detection.src_dir {
        lines.push("- Both `src/` and `tests/` are present; keep them in sync.".to_string());
    }
    lines
}

fn repository_shape_lines(detection: &RepoDetection) -> Vec<String> {
    let mut lines = Vec::new();
    if detection.rust_dir {
        lines.push("- `rust/` contains the Rust workspace implementation.".to_string());
    }
    if detection.src_dir {
        lines.push("- `src/` contains source files.".to_string());
    }
    if detection.tests_dir {
        lines.push("- `tests/` contains validation tests.".to_string());
    }
    lines
}

fn framework_notes(detection: &RepoDetection) -> Vec<String> {
    let mut lines = Vec::new();
    if detection.nextjs {
        lines.push("- Next.js: preserve routing conventions and verify builds structured.".to_string());
    }
    if detection.react && !detection.nextjs {
        lines.push("- React: keep components tested and avoid unnecessary prop churn.".to_string());
    }
    if detection.vite {
        lines.push("- Vite: validate production bundle after config changes.".to_string());
    }
    if detection.nest {
        lines.push("- NestJS: preserve module boundaries and verify wiring.".to_string());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::{initialize_repo};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("hell-code-init-{nanos}"))
    }

    #[test]
    fn initialize_repo_creates_expected_files() {
        let root = temp_dir();
        fs::create_dir_all(root.join("rust")).expect("create rust dir");
        fs::write(root.join("rust").join("Cargo.toml"), "[workspace]\n").expect("write cargo");

        let report = initialize_repo(&root).expect("init should succeed");
        let rendered = report.render();
        assert!(rendered.contains(".hell-code/"));
        assert!(rendered.contains(".hell-code.json"));
        assert!(rendered.contains("created"));
        assert!(rendered.contains(".gitignore"));
        assert!(rendered.contains("HELL-CODE.md"));
        assert!(root.join(".hell-code").is_dir());
        assert!(root.join(".hell-code.json").is_file());
        assert!(root.join("HELL-CODE.md").is_file());
        
        fs::remove_dir_all(root).expect("cleanup temp dir");
    }
}
