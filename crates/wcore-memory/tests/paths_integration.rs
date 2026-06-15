// Integration tests for the memory path system.
//
// These tests target the functional requirements from test-plan.md TC-2,
// treating the public API as a black box.

use std::fs;
use std::path::{Path, PathBuf};

use serial_test::serial;
use wcore_memory::paths;

// -- TC-2.1: Default memory base directory ------------------------------------

#[test]
#[serial(env)]
fn tc_2_1_default_base_dir_uses_platform_config() {
    // Ensure env override is NOT set
    let saved = std::env::var(env_key()).ok();
    // SAFETY: #[serial(env)] ensures no concurrent env mutation.
    unsafe { std::env::remove_var(env_key()) };

    let base = paths::memory_base_dir();
    // Should return Some (platform provides a config dir in CI/test envs)
    assert!(
        base.is_some(),
        "memory_base_dir should return Some on this platform"
    );
    let base = base.unwrap();
    // Should end with "apexrouter-cli" (the brand, not "claude")
    assert!(
        base.to_string_lossy().contains("apexrouter-cli"),
        "base dir should use apexrouter-cli brand: {base:?}"
    );

    restore_env(saved);
}

// -- TC-2.2: Environment variable overrides base directory --------------------

#[cfg(unix)]
#[test]
#[serial(env)]
fn tc_2_2_env_var_overrides_base_dir() {
    let saved = std::env::var(env_key()).ok();
    // SAFETY: #[serial(env)] ensures no concurrent env mutation.
    unsafe { std::env::set_var(env_key(), "/custom/memory/path") };

    let base = paths::memory_base_dir();
    assert_eq!(base, Some(PathBuf::from("/custom/memory/path")));

    restore_env(saved);
}

#[cfg(windows)]
#[test]
#[serial(env)]
fn tc_2_2_env_var_overrides_base_dir() {
    let saved = std::env::var(env_key()).ok();
    // SAFETY: #[serial(env)] ensures no concurrent env mutation.
    unsafe { std::env::set_var(env_key(), "C:\\custom\\memory\\path") };

    let base = paths::memory_base_dir();
    assert_eq!(base, Some(PathBuf::from("C:\\custom\\memory\\path")));

    restore_env(saved);
}

// -- TC-2.3: Project memory directory path ------------------------------------

#[cfg(unix)]
#[test]
#[serial(env)]
fn tc_2_3_auto_memory_dir_structure() {
    let saved = std::env::var(env_key()).ok();
    // SAFETY: #[serial(env)] ensures no concurrent env mutation.
    unsafe { std::env::set_var(env_key(), "/base") };

    let dir = paths::auto_memory_dir(Path::new("/home/user/my-project"));
    assert!(dir.is_some());
    let dir = dir.unwrap();

    // Should have the structure: <base>/projects/<sanitized>/memory
    let dir_str = dir.to_string_lossy();
    assert!(
        dir_str.starts_with("/base/projects/"),
        "wrong prefix: {dir_str}"
    );
    assert!(
        dir_str.ends_with("/memory"),
        "should end with /memory: {dir_str}"
    );

    // Sanitized name should not contain `/` (the original separator)
    let sanitized = dir.parent().unwrap().file_name().unwrap().to_string_lossy();
    assert!(
        !sanitized.contains('/'),
        "sanitized name should not contain /: {sanitized}"
    );

    restore_env(saved);
}

#[cfg(windows)]
#[test]
#[serial(env)]
fn tc_2_3_auto_memory_dir_structure() {
    let saved = std::env::var(env_key()).ok();
    // SAFETY: #[serial(env)] ensures no concurrent env mutation.
    unsafe { std::env::set_var(env_key(), "C:\\base") };

    let dir = paths::auto_memory_dir(Path::new("C:\\Users\\user\\my-project"));
    assert!(dir.is_some());
    let dir = dir.unwrap();

    let dir_str = dir.to_string_lossy();
    assert!(
        dir_str.starts_with("C:\\base\\projects\\"),
        "wrong prefix: {dir_str}"
    );
    assert!(
        dir_str.ends_with("\\memory"),
        "should end with \\memory: {dir_str}"
    );

    let sanitized = dir.parent().unwrap().file_name().unwrap().to_string_lossy();
    assert!(
        !sanitized.contains('\\'),
        "sanitized name should not contain \\: {sanitized}"
    );

    restore_env(saved);
}

// -- TC-2.4: Reject relative path ---------------------------------------------

#[test]
fn tc_2_4_reject_relative_path() {
    let result = paths::validate_memory_path(Path::new("relative/path"));
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("absolute"),
        "error should mention 'absolute': {err_msg}"
    );
}

// -- TC-2.5: Reject null byte -------------------------------------------------

#[cfg(unix)]
#[test]
fn tc_2_5_reject_null_byte() {
    let bad_path = PathBuf::from("/tmp/test\0evil");
    let result = paths::validate_memory_path(&bad_path);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("null"),
        "error should mention null: {err_msg}"
    );
}

#[cfg(windows)]
#[test]
fn tc_2_5_reject_null_byte() {
    let bad_path = PathBuf::from("C:\\tmp\\test\0evil");
    let result = paths::validate_memory_path(&bad_path);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("null"),
        "error should mention null: {err_msg}"
    );
}

// -- TC-2.6: Reject path traversal --------------------------------------------

#[cfg(unix)]
#[test]
fn tc_2_6_reject_traversal() {
    let result = paths::validate_memory_path(Path::new("/tmp/../../../etc/passwd"));
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("traversal"),
        "error should mention traversal: {err_msg}"
    );
}

#[cfg(windows)]
#[test]
fn tc_2_6_reject_traversal() {
    let result = paths::validate_memory_path(Path::new("C:\\tmp\\..\\..\\..\\etc\\passwd"));
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("traversal"),
        "error should mention traversal: {err_msg}"
    );
}

// -- TC-2.7: Memory entrypoint path -------------------------------------------

#[test]
fn tc_2_7_entrypoint_path() {
    // memory_entrypoint just appends MEMORY.md — no absolute path requirement,
    // so a platform-neutral relative path works fine here.
    let dir = Path::new("path").join("to").join("memory");
    let ep = paths::memory_entrypoint(&dir);
    assert_eq!(ep, dir.join("MEMORY.md"));
}

// -- TC-2.8: Path membership positive -----------------------------------------

#[test]
fn tc_2_8_is_memory_path_inside() {
    let tmp = tempfile::tempdir().unwrap();
    let mem_dir = tmp.path().join("memory");
    fs::create_dir_all(&mem_dir).unwrap();
    let file = mem_dir.join("user_role.md");
    fs::write(&file, "test").unwrap();

    assert!(
        paths::is_memory_path(&file, &mem_dir),
        "file inside memory dir should be recognized"
    );
}

// -- TC-2.9: Path membership negative -----------------------------------------

#[test]
fn tc_2_9_is_memory_path_outside() {
    let tmp = tempfile::tempdir().unwrap();
    let mem_dir = tmp.path().join("memory");
    fs::create_dir_all(&mem_dir).unwrap();
    let outside = tmp.path().join("other_file.md");
    fs::write(&outside, "test").unwrap();

    assert!(
        !paths::is_memory_path(&outside, &mem_dir),
        "file outside memory dir should not be recognized"
    );
}

// -- TC-2.10: Ensure directory exists -----------------------------------------

#[test]
fn tc_2_10_ensure_dir_creates_and_is_idempotent() {
    let tmp = tempfile::tempdir().unwrap();
    let deep = tmp.path().join("a").join("b").join("c").join("memory");

    // Does not exist yet
    assert!(!deep.exists());

    // First call creates it
    paths::ensure_memory_dir(&deep).unwrap();
    assert!(deep.is_dir());

    // Second call is idempotent
    paths::ensure_memory_dir(&deep).unwrap();
    assert!(deep.is_dir());
}

// -- Additional edge cases from test-plan TC-2 --------------------------------

#[cfg(unix)]
#[test]
fn validate_accepts_valid_absolute_path() {
    let result = paths::validate_memory_path(Path::new("/tmp/memory/test.md"));
    assert!(result.is_ok());
}

#[cfg(windows)]
#[test]
fn validate_accepts_valid_absolute_path() {
    let result = paths::validate_memory_path(Path::new("C:\\tmp\\memory\\test.md"));
    assert!(result.is_ok());
}

#[cfg(unix)]
#[test]
fn validate_rejects_root_path() {
    let result = paths::validate_memory_path(Path::new("/"));
    assert!(result.is_err());
}

#[cfg(windows)]
#[test]
fn validate_rejects_root_path() {
    let result = paths::validate_memory_path(Path::new("C:\\"));
    assert!(result.is_err());
}

#[test]
fn sanitize_produces_deterministic_results() {
    let path = "/home/user/workspace/project";
    assert_eq!(paths::sanitize_path(path), paths::sanitize_path(path));
}

#[test]
fn sanitize_different_paths_produce_different_results() {
    let a = paths::sanitize_path("/home/alice/project");
    let b = paths::sanitize_path("/home/bob/project");
    assert_ne!(a, b);
}

#[test]
fn entrypoint_name_constant_is_memory_md() {
    assert_eq!(paths::ENTRYPOINT_NAME, "MEMORY.md");
}

// -- TC-2.v2: v2 path resolution (W5 Task A.5) --------------------------------

const WCORE_KEY: &str = "WCORE_MEMORY_DIR";

#[cfg(unix)]
#[test]
#[serial(env)]
fn v2_global_session_audit_changelog_paths() {
    let saved_w = std::env::var(WCORE_KEY).ok();
    unsafe {
        std::env::set_var(WCORE_KEY, "/base");
    }

    assert_eq!(
        paths::global_db_path(),
        Some(PathBuf::from("/base/memory/memory.db"))
    );
    assert_eq!(
        paths::session_db_path("s-123"),
        Some(PathBuf::from("/base/memory/sessions/s-123.db"))
    );
    assert_eq!(
        paths::audit_db_path(),
        Some(PathBuf::from("/base/memory/audit.db"))
    );
    assert_eq!(
        paths::changelog_path("project"),
        Some(PathBuf::from(
            "/base/memory/changelog/project.changelog.jsonl"
        ))
    );
    assert_eq!(
        paths::changelog_path("global"),
        Some(PathBuf::from(
            "/base/memory/changelog/global.changelog.jsonl"
        ))
    );
    assert_eq!(
        paths::changelog_path("session"),
        Some(PathBuf::from(
            "/base/memory/changelog/session.changelog.jsonl"
        ))
    );

    restore_single(saved_w);
}

#[test]
fn v2_project_db_path_under_apexrouter_cli() {
    let root = Path::new("/home/user/project");
    let p = paths::project_db_path(root);
    let s = p.to_string_lossy();
    assert!(
        s.ends_with(".apexrouter-cli/memory/memory.db")
            || s.ends_with(".apexrouter-cli\\memory\\memory.db"),
        "{s}"
    );
    assert!(s.starts_with("/home/user/project") || s.starts_with("\\home\\user\\project"));
}

#[cfg(unix)]
#[test]
#[serial(env)]
fn v2_session_path_sanitizes_session_id() {
    let saved_w = std::env::var(WCORE_KEY).ok();
    unsafe {
        std::env::set_var(WCORE_KEY, "/base");
    }

    let p = paths::session_db_path("weird/session id?!").unwrap();
    let leaf = p.file_name().unwrap().to_string_lossy();
    // sanitize_path replaces non-alphanumeric with `-`
    assert!(
        !leaf.contains('/'),
        "session leaf must not contain /: {leaf}"
    );
    assert!(
        !leaf.contains(' '),
        "session leaf must not contain whitespace: {leaf}"
    );
    assert!(leaf.ends_with(".db"));

    restore_single(saved_w);
}

// -- Helpers ------------------------------------------------------------------

// env_key + restore_env are used by both unix and
// Windows path-resolution tests at the top of this file.
fn env_key() -> &'static str {
    WCORE_KEY
}

fn restore_env(saved: Option<String>) {
    // SAFETY: only called from #[serial(env)] tests.
    unsafe {
        match saved {
            Some(v) => std::env::set_var(env_key(), v),
            None => std::env::remove_var(env_key()),
        }
    }
}

#[cfg(unix)]
fn restore_single(saved: Option<String>) {
    // SAFETY: only called from #[serial(env)] tests.
    unsafe {
        match saved {
            Some(v) => std::env::set_var(WCORE_KEY, v),
            None => std::env::remove_var(WCORE_KEY),
        }
    }
}
