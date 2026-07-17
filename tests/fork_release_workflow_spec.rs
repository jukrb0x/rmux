use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(root.join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn indented_block(contents: &str, header: &str, indent: usize) -> String {
    let marker = format!("{}{}", " ".repeat(indent), header);
    let lines = contents.lines().collect::<Vec<_>>();
    let start = lines
        .iter()
        .position(|line| *line == marker)
        .unwrap_or_else(|| panic!("missing workflow section: {marker}"));

    lines[start + 1..]
        .iter()
        .take_while(|line| {
            line.trim().is_empty() || line.len().saturating_sub(line.trim_start().len()) > indent
        })
        .copied()
        .collect::<Vec<_>>()
        .join("\n")
}

fn indented_list(contents: &str, header: &str, indent: usize) -> Vec<String> {
    indented_block(contents, header, indent)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_owned())
        .collect()
}

fn push_trigger(contents: &str) -> String {
    let on = indented_block(contents, "on:", 0);
    indented_block(&on, "push:", 2)
}

fn is_reserved_fork_tag(candidate: &str) -> bool {
    let tag = candidate.strip_prefix("refs/tags/").unwrap_or(candidate);
    tag.starts_with('v') && tag.contains("-jukrb0x.")
}

#[test]
fn fork_release_is_windows_only_immutable_and_tmux_bound() {
    let workflow = repo_file(".github/workflows/windows-fork-release.yml");

    for required in [
        "v*-jukrb0x.*",
        "workflow_dispatch:",
        "contents: write",
        "runs-on: windows-latest",
        "toolchain: \"1.96.1\"",
        "refs/remotes/origin/tmux",
        "git merge-base --is-ancestor",
        "^v[0-9]+\\.[0-9]+\\.[0-9]+-jukrb0x\\.[1-9][0-9]*$",
        "assert-cargo-filter-nonempty.ps1",
        "foreground_command_",
        "process_handle_liveness_rejects_an_exited_process_with_an_open_handle",
        "job_object_lists_active_root_and_descendant_processes",
        "status_left_click_uses_explicit_status_format_window_ranges",
        "windows_cmd_ping_updates_current_command_and_automatic_name_until_exit",
        "scripts/package-windows.ps1",
        "scripts/verify-package-windows.ps1",
        "SHA256SUMS",
        "gh release create",
        "if: github.event_name == 'push'",
    ] {
        assert!(
            workflow.contains(required),
            "fork release workflow lost required contract: {required}"
        );
    }

    for forbidden in [
        "gh release upload",
        "--clobber",
        "chocolatey",
        "snapcraft",
        "winget-pkgs",
        "self-hosted",
    ] {
        assert!(
            !workflow.to_ascii_lowercase().contains(forbidden),
            "fork release workflow contains forbidden publisher behavior: {forbidden}"
        );
    }
}

#[test]
fn inherited_release_push_trigger_excludes_personal_tags_after_the_public_pattern() {
    let release = repo_file(".github/workflows/release.yml");
    let push = push_trigger(&release);
    let patterns = indented_list(&push, "tags:", 4);

    assert_eq!(
        patterns,
        ["- \"v*\"", "- \"!v*-jukrb0x.*\""],
        "upstream release push tags must include public tags first and then exclude personal tags"
    );
}

#[test]
fn inherited_release_rejects_personal_tags_before_source_work() {
    let release = repo_file(".github/workflows/release.yml");
    let jobs = indented_block(&release, "jobs:", 0);
    let source_gates = indented_block(&jobs, "source-gates:", 2);
    let source_gate_env = indented_block(&source_gates, "env:", 4);
    let steps = indented_block(&source_gates, "steps:", 4);
    let first_step = steps
        .lines()
        .find(|line| !line.trim().is_empty())
        .expect("source-gates must contain a step")
        .trim();

    assert_eq!(
        first_step, "- name: Reject personal fork release tags",
        "personal-tag rejection must be the first source gate"
    );
    assert!(
        source_gate_env.contains(
            "RELEASE_REF: ${{ github.event_name == 'workflow_dispatch' && inputs.ref || github.ref_name }}"
        ),
        "source-gates must expose the manual-dispatch input to the rejection gate"
    );

    let rejection_gate = indented_block(&steps, "- name: Reject personal fork release tags", 6);
    for required in [
        "shell: pwsh",
        "foreach ($candidate in @($env:GITHUB_REF, $env:RELEASE_REF))",
        "$tag = $candidate -replace '^refs/tags/', ''",
        "if ($tag -like 'v*-jukrb0x.*')",
        "throw \"Personal fork tag $tag must use the fork-only release workflow.\"",
    ] {
        assert!(
            rejection_gate.contains(required),
            "early personal-tag rejection gate lost required behavior: {required}"
        );
    }
    assert!(
        !rejection_gate.contains("^v[0-9]+\\.[0-9]+\\.[0-9]+-jukrb0x\\.[1-9][0-9]*$"),
        "strict valid-release grammar belongs only in the fork workflow"
    );

    for reserved in [
        "v0.9.0-jukrb0x.0",
        "refs/tags/v0.9.0-jukrb0x.preview",
        "vnext-jukrb0x.1",
    ] {
        assert!(
            is_reserved_fork_tag(reserved),
            "test fixture must exercise the reserved fork tag namespace: {reserved}"
        );
    }
    assert!(
        !is_reserved_fork_tag("v0.9.0"),
        "official release tags must remain outside the reserved fork namespace"
    );

    let validate_position = steps
        .find("- name: Validate release event ref")
        .expect("release event validation step");
    let checkout_position = steps
        .find("- uses: actions/checkout@")
        .expect("release source checkout step");
    let rejection_position = steps
        .find("- name: Reject personal fork release tags")
        .expect("personal tag rejection step");
    assert!(
        rejection_position < validate_position && rejection_position < checkout_position,
        "personal tags must be rejected before release validation or source checkout"
    );

    for job in ["build:", "publish:"] {
        let job = indented_block(&jobs, job, 2);
        let needs = indented_block(&job, "needs:", 4);
        assert!(
            needs.lines().any(|line| line.trim() == "- source-gates"),
            "release build and publish jobs must depend on source-gates"
        );
    }
}

#[test]
fn inherited_ci_and_scorecard_target_tmux_without_personal_tag_ci() {
    let ci = repo_file(".github/workflows/ci.yml");
    let scorecard = repo_file(".github/workflows/scorecard.yml");
    let ci_on = indented_block(&ci, "on:", 0);
    let ci_push = indented_block(&ci_on, "push:", 2);
    let ci_pull_request = indented_block(&ci_on, "pull_request:", 2);
    let scorecard_push = push_trigger(&scorecard);

    assert_eq!(
        indented_list(&ci_push, "tags:", 4),
        ["- \"v*\"", "- \"!v*-jukrb0x.*\""],
        "upstream CI push tags must include public tags first and then exclude personal tags"
    );
    assert_eq!(
        indented_list(&ci_push, "branches:", 4),
        ["- tmux"],
        "CI push must target the default branch"
    );
    assert_eq!(
        indented_list(&ci_pull_request, "branches:", 4),
        ["- tmux"],
        "CI pull requests must target the default branch"
    );
    assert_eq!(
        indented_list(&scorecard_push, "branches:", 4),
        ["- tmux"],
        "Scorecard push must target the default branch"
    );
}
