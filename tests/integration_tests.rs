//! Comprehensive end-to-end integration tests for `tidy`.
//!
//! Covers:
//! - Single-pass file organization (`tidy run`)
//! - Dry-run guarantee (zero modifications on disk)
//! - Atomic undo and directory rollback (`tidy undo`)
//! - Partial failure recording and rollback resilience
//! - Deep directory pruning in reverse depth order
//! - Configuration initialization (`tidy init`)
//! - Native shell completions (`tidy completions`)

use std::fs::{self, File};
use std::io::Write;

use tidy::cli::completions::{generate_completions, SupportedShell};
use tidy::cli::config_cmd::execute_init;
use tidy::cli::run::execute_run;
use tidy::cli::RunArgs;
use tidy::core::config::Config;
use tidy::safety::ledger::Ledger;
use tidy::safety::mover::safe_move;

#[test]
fn test_single_run_organization() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();

    // Direct ledger to temp dir for isolated test environment
    std::env::set_var("TIDY_STATE_DIR", root);

    // Create candidate files in root
    let f_img = root.join("photo.jpg");
    let f_doc = root.join("contract.pdf");
    let f_aud = root.join("song.mp3");
    let f_arc = root.join("archive.tar.gz");
    let f_code = root.join("script.py");

    File::create(&f_img).unwrap().write_all(b"image").unwrap();
    File::create(&f_doc).unwrap().write_all(b"doc").unwrap();
    File::create(&f_aud).unwrap().write_all(b"audio").unwrap();
    File::create(&f_arc).unwrap().write_all(b"archive").unwrap();
    File::create(&f_code).unwrap().write_all(b"code").unwrap();

    let args = RunArgs {
        path: Some(root.to_path_buf()),
        dry_run: false,
        recursive: false,
    };

    execute_run(&args, None).unwrap();

    // Verify original files are gone
    assert!(!f_img.exists());
    assert!(!f_doc.exists());
    assert!(!f_aud.exists());
    assert!(!f_arc.exists());
    assert!(!f_code.exists());

    // Verify files are sorted into appropriate category and extension subfolders
    assert!(root.join("Images").join("JPG").join("photo.jpg").exists());
    assert!(root.join("Documents").join("PDF").join("contract.pdf").exists());
    assert!(root.join("Audio").join("MP3").join("song.mp3").exists());
    assert!(root.join("Archives").join("TAR.GZ").join("archive.tar.gz").exists());
    assert!(root.join("Code").join("PY").join("script.py").exists());
}

#[test]
fn test_dry_run_no_side_effects() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();

    let f_img = root.join("photo.png");
    let f_doc = root.join("notes.txt");
    File::create(&f_img).unwrap().write_all(b"png").unwrap();
    File::create(&f_doc).unwrap().write_all(b"txt").unwrap();

    let args = RunArgs {
        path: Some(root.to_path_buf()),
        dry_run: true,
        recursive: false,
    };

    execute_run(&args, None).unwrap();

    // Verify files remain in root untouched
    assert!(f_img.exists());
    assert!(f_doc.exists());

    // Verify category folders were NOT created
    assert!(!root.join("Images").exists());
    assert!(!root.join("Documents").exists());
}

#[test]
fn test_undo_restores_exact_state() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let db_path = root.join("history.db");

    let f1 = root.join("vacation.jpg");
    let f2 = root.join("report.docx");
    File::create(&f1).unwrap().write_all(b"beach").unwrap();
    File::create(&f2).unwrap().write_all(b"work").unwrap();

    let mut ledger = Ledger::open_at(&db_path).unwrap();

    let d1 = root.join("Images").join("vacation.jpg");
    let d2 = root.join("Documents").join("report.docx");
    let m1 = safe_move(&f1, &d1, false).unwrap();
    let m2 = safe_move(&f2, &d2, false).unwrap();

    let run = ledger.record_run("tidy run", root, &[m1, m2]).unwrap();

    assert!(!f1.exists());
    assert!(!f2.exists());
    assert!(d1.exists());
    assert!(d2.exists());

    // Execute undo
    let report = tidy::safety::undo::execute_undo(&mut ledger, Some(&run.uuid)).unwrap();
    assert_eq!(report.restored_count, 2);

    // Verify restored files
    assert!(f1.exists());
    assert!(f2.exists());
    assert_eq!(fs::read(&f1).unwrap(), b"beach");
    assert_eq!(fs::read(&f2).unwrap(), b"work");

    // Verify category folders were pruned
    assert!(!root.join("Images").exists());
    assert!(!root.join("Documents").exists());
}

#[test]
fn test_partial_failure_ledger_and_undo() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let db_path = root.join("history.db");
    let mut ledger = Ledger::open_at(&db_path).unwrap();

    let f1 = root.join("file1.jpg");
    File::create(&f1).unwrap().write_all(b"data1").unwrap();
    let d1 = root.join("Images").join("file1.jpg");
    let m1 = safe_move(&f1, &d1, false).unwrap();

    // Record with PARTIAL_FAILURE
    let run = ledger
        .record_run_with_status("tidy run", root, &[m1], "PARTIAL_FAILURE")
        .unwrap();
    assert_eq!(run.status, "PARTIAL_FAILURE");

    // get_latest_completed_run must find PARTIAL_FAILURE runs
    let latest = ledger.get_latest_completed_run().unwrap().unwrap();
    assert_eq!(latest.uuid, run.uuid);

    // Undo should be able to roll back the partial run
    let report = tidy::safety::undo::execute_undo(&mut ledger, None).unwrap();
    assert_eq!(report.restored_count, 1);
    assert!(f1.exists());
    assert!(!d1.exists());
}

#[test]
fn test_deep_nested_undo_pruning() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let db_path = root.join("history.db");
    let mut ledger = Ledger::open_at(&db_path).unwrap();

    let src = root.join("nested_photo.jpg");
    File::create(&src).unwrap().write_all(b"nested").unwrap();

    // Move to a multi-level nested destination: Images/2026/Summer/nested_photo.jpg
    let dest = root
        .join("Images")
        .join("2026")
        .join("Summer")
        .join("nested_photo.jpg");
    let outcome = safe_move(&src, &dest, false).unwrap();
    ledger.record_run("tidy run", root, &[outcome]).unwrap();

    assert!(!src.exists());
    assert!(dest.exists());

    // Undo must prune Summer/, 2026/, and Images/ up to root
    let report = tidy::safety::undo::execute_undo(&mut ledger, None).unwrap();
    assert_eq!(report.restored_count, 1);
    assert!(src.exists());

    // Entire hierarchy must be pruned
    assert!(!root.join("Images").join("2026").join("Summer").exists());
    assert!(!root.join("Images").join("2026").exists());
    assert!(!root.join("Images").exists());
    // Root directory itself remains
    assert!(root.exists());
}

#[test]
fn test_config_init_and_parse() {
    let temp = tempfile::tempdir().unwrap();
    let config_file = temp.path().join("config.toml");

    // Initialize config
    execute_init(Some(&config_file), false).unwrap();
    assert!(config_file.exists());

    // Verify it parses cleanly with Config loader
    let loaded = Config::load_or_default(Some(&config_file)).unwrap();
    let rules = loaded.compile().unwrap();
    assert!(rules.is_ignored(".DS_Store"));
    assert!(rules.is_ignored("test.crdownload"));
    assert_eq!(
        rules.extension_to_category.get("png").map(String::as_str),
        Some("Images")
    );
}

#[test]
fn test_completions_generation() {
    // Generate bash, zsh, and fish completions without panic
    assert!(generate_completions(SupportedShell::Bash).is_ok());
    assert!(generate_completions(SupportedShell::Zsh).is_ok());
    assert!(generate_completions(SupportedShell::Fish).is_ok());
}

#[test]
fn test_recursive_scan_preserves_nested_category_folders() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();

    // Create nested directory tree: root/nested_proj/sub/Images/screenshot.png
    let nested_img_dir = root.join("nested_proj").join("sub").join("Images");
    fs::create_dir_all(&nested_img_dir).unwrap();
    let nested_file = nested_img_dir.join("screenshot.png");
    File::create(&nested_file)
        .unwrap()
        .write_all(b"png data")
        .unwrap();

    let config = Config::default();
    let rules = config.compile().unwrap();
    let classifier = tidy::core::classifier::Classifier::new(rules);

    // In recursive mode, scan_directory must NOT skip nested_proj/sub/Images
    let planned = tidy::cli::run::scan_directory(root, true, &classifier).unwrap();
    assert_eq!(planned.len(), 1);
    assert_eq!(planned[0].source, nested_file);
    assert_eq!(planned[0].category, "Images");
}

#[test]
fn test_undo_dry_run_and_all() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let db_path = root.join("history.db");
    let mut ledger = Ledger::open_at(&db_path).unwrap();

    let src1 = root.join("doc1.pdf");
    File::create(&src1).unwrap().write_all(b"doc1").unwrap();
    let dest1 = root.join("Documents").join("doc1.pdf");
    let m1 = safe_move(&src1, &dest1, false).unwrap();
    let run1 = ledger.record_run("tidy run", root, &[m1]).unwrap();

    let src2 = root.join("doc2.pdf");
    File::create(&src2).unwrap().write_all(b"doc2").unwrap();
    let dest2 = root.join("Documents").join("doc2.pdf");
    let m2 = safe_move(&src2, &dest2, false).unwrap();
    let run2 = ledger.record_run("tidy run", root, &[m2]).unwrap();

    // 1. Test execute_undo_opt with dry_run = true
    let dry_report = tidy::safety::undo::execute_undo_opt(&mut ledger, None, true).unwrap();
    assert!(dry_report.is_dry_run);
    assert_eq!(dry_report.restored_count, 1);
    assert_eq!(dry_report.run_uuid, run2.uuid);

    // Verify files on disk were NOT modified
    assert!(dest2.exists());
    assert!(!src2.exists());

    // Verify ledger status was NOT changed
    let latest = ledger.get_latest_completed_run().unwrap().unwrap();
    assert_eq!(latest.uuid, run2.uuid);

    // 2. Test undoing all runs
    let report2 = tidy::safety::undo::execute_undo(&mut ledger, None).unwrap();
    assert_eq!(report2.run_uuid, run2.uuid);
    assert!(src2.exists());
    assert!(!dest2.exists());

    let report1 = tidy::safety::undo::execute_undo(&mut ledger, None).unwrap();
    assert_eq!(report1.run_uuid, run1.uuid);
    assert!(src1.exists());
    assert!(!dest1.exists());

    // No runs left to undo
    let none = ledger.get_latest_completed_run().unwrap();
    assert!(none.is_none());
}
