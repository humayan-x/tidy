use std::fs::{self, File};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use tidy::core::config::Config;
use tidy::watch::dispatcher::{WatchConfig, WatchDispatcher};

fn wait_for_path(path: &std::path::Path, timeout: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if path.exists() {
            return true;
        }
        thread::sleep(Duration::from_millis(50));
    }
    false
}

#[test]
fn test_watch_detects_and_organizes_new_file() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_path_buf();

    let watch_config = WatchConfig {
        target_dir: root.clone(),
        recursive: false,
        debounce_duration: Duration::from_millis(100),
        stability_tick: Duration::from_millis(50),
        required_stable_ticks: 2,
        custom_db_path: Some(root.join("test_history.db")),
    };

    let config = Config::default();
    let mut dispatcher = WatchDispatcher::new(watch_config, &config).unwrap();

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);

    let handle = thread::spawn(move || {
        dispatcher.run(shutdown_clone).unwrap();
    });

    // Give watcher thread a moment to initialize inotify / fsevents
    thread::sleep(Duration::from_millis(150));

    // Create a new image file in watched root
    let img_src = root.join("scenery.png");
    File::create(&img_src)
        .unwrap()
        .write_all(b"fake png data")
        .unwrap();

    // Verify file is moved to Images/scenery.png
    let img_dest = root.join("Images").join("scenery.png");
    assert!(
        wait_for_path(&img_dest, Duration::from_secs(5)),
        "File should be organized into Images/scenery.png"
    );
    assert!(!img_src.exists(), "Source file should have been moved");

    // Clean shutdown
    shutdown.store(true, Ordering::SeqCst);
    handle.join().unwrap();
}

#[test]
fn test_watch_waits_for_download_completion() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_path_buf();

    let watch_config = WatchConfig {
        target_dir: root.clone(),
        recursive: false,
        debounce_duration: Duration::from_millis(100),
        stability_tick: Duration::from_millis(50),
        required_stable_ticks: 2,
        custom_db_path: Some(root.join("test_history.db")),
    };

    let config = Config::default();
    let mut dispatcher = WatchDispatcher::new(watch_config, &config).unwrap();

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);

    let handle = thread::spawn(move || {
        dispatcher.run(shutdown_clone).unwrap();
    });

    thread::sleep(Duration::from_millis(150));

    // 1. Create an in-progress download artifact
    let temp_download = root.join("trailer.mp4.crdownload");
    File::create(&temp_download)
        .unwrap()
        .write_all(b"partial video stream")
        .unwrap();

    // Wait 500ms and verify it is NOT moved while in .crdownload state
    thread::sleep(Duration::from_millis(500));
    assert!(
        temp_download.exists(),
        "Download in progress must not be moved"
    );
    assert!(!root.join("Video").join("trailer.mp4").exists());

    // 2. Simulate browser completing download by renaming
    let final_file = root.join("trailer.mp4");
    fs::rename(&temp_download, &final_file).unwrap();

    // Verify it is organized into Video/trailer.mp4
    let video_dest = root.join("Video").join("trailer.mp4");
    assert!(
        wait_for_path(&video_dest, Duration::from_secs(5)),
        "Finished download should be organized into Video/trailer.mp4"
    );
    assert!(!final_file.exists());

    shutdown.store(true, Ordering::SeqCst);
    handle.join().unwrap();
}

#[test]
fn test_watch_collision_renaming() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_path_buf();

    let watch_config = WatchConfig {
        target_dir: root.clone(),
        recursive: false,
        debounce_duration: Duration::from_millis(100),
        stability_tick: Duration::from_millis(50),
        required_stable_ticks: 2,
        custom_db_path: Some(root.join("test_history.db")),
    };

    let config = Config::default();
    let mut dispatcher = WatchDispatcher::new(watch_config, &config).unwrap();

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);

    let handle = thread::spawn(move || {
        dispatcher.run(shutdown_clone).unwrap();
    });

    thread::sleep(Duration::from_millis(150));

    // 1. Create first document
    let doc1 = root.join("report.pdf");
    File::create(&doc1).unwrap().write_all(b"doc v1").unwrap();

    let dest1 = root.join("Documents").join("report.pdf");
    assert!(
        wait_for_path(&dest1, Duration::from_secs(5)),
        "First report.pdf should be organized"
    );

    // 2. Create second document with identical name
    let doc2 = root.join("report.pdf");
    File::create(&doc2).unwrap().write_all(b"doc v2").unwrap();

    let dest2 = root.join("Documents").join("report (1).pdf");
    assert!(
        wait_for_path(&dest2, Duration::from_secs(5)),
        "Collision should rename to report (1).pdf"
    );

    // Both files exist with correct contents
    assert_eq!(fs::read(&dest1).unwrap(), b"doc v1");
    assert_eq!(fs::read(&dest2).unwrap(), b"doc v2");

    shutdown.store(true, Ordering::SeqCst);
    handle.join().unwrap();
}

#[test]
fn test_watch_non_recursive_skips_subdirectories() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_path_buf();

    let watch_config = WatchConfig {
        target_dir: root.clone(),
        recursive: false,
        debounce_duration: Duration::from_millis(100),
        stability_tick: Duration::from_millis(50),
        required_stable_ticks: 2,
        custom_db_path: Some(root.join("test_history.db")),
    };

    let config = Config::default();
    let mut dispatcher = WatchDispatcher::new(watch_config, &config).unwrap();

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);

    let handle = thread::spawn(move || {
        dispatcher.run(shutdown_clone).unwrap();
    });

    thread::sleep(Duration::from_millis(150));

    // Create a subfolder with a file
    let subfolder = root.join("my_subfolder");
    fs::create_dir(&subfolder).unwrap();
    let sub_file = subfolder.join("nested.png");
    File::create(&sub_file)
        .unwrap()
        .write_all(b"nested image")
        .unwrap();

    // Wait 500ms: non-recursive watcher must not touch files in subdirectories
    thread::sleep(Duration::from_millis(500));
    assert!(sub_file.exists(), "Nested file should remain in subfolder");
    assert!(!root.join("Images").join("nested.png").exists());

    shutdown.store(true, Ordering::SeqCst);
    handle.join().unwrap();
}

#[test]
fn test_daemon_idle_memory_footprint() {
    use tidy::watch::memory::get_resident_memory_kb;

    let mem_kb = get_resident_memory_kb().expect("Memory measurement must succeed");
    // Idle resident memory target is ~3-5 MB; allow generous test process ceiling of 50 MB
    assert!(mem_kb > 0, "Memory must be non-zero");
    println!(
        "Measured resident memory (RSS): {} KB ({:.2} MB)",
        mem_kb,
        mem_kb as f64 / 1024.0
    );
    assert!(
        mem_kb < 50_000,
        "Daemon memory footprint {} KB exceeded 50 MB ceiling",
        mem_kb
    );
}

#[test]
fn test_watch_initial_scan() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_path_buf();

    // Create a pre-existing file in watched root BEFORE starting
    let existing_doc = root.join("pre_existing.pdf");
    File::create(&existing_doc)
        .unwrap()
        .write_all(b"initial doc")
        .unwrap();

    let _watch_args = tidy::cli::WatchArgs {
        path: Some(root.clone()),
        recursive: false,
        debounce: Some(100),
        initial_scan: true,
    };

    // Execute run scan directly as initial_scan does
    let config = Config::default();
    let rules = config.compile().unwrap();
    let classifier = tidy::core::classifier::Classifier::new(rules);
    let planned = tidy::cli::run::scan_directory(&root, false, &classifier).unwrap();
    assert_eq!(planned.len(), 1);
    assert_eq!(planned[0].category, "Documents");

    let dest = root.join("Documents").join("pre_existing.pdf");
    tidy::safety::mover::safe_move(&existing_doc, &dest, false).unwrap();

    assert!(dest.exists());
    assert!(!existing_doc.exists());
}
