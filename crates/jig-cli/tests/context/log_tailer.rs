//! Following a log file as it grows — what `ps --watch` and
//! `jig daemon logs -f` do.

use std::io::Write;

use jig_cli::context::log::LogFile;

#[test]
fn tailer_returns_only_appended_complete_lines() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("d.log");
    std::fs::write(&path, "old\n").unwrap();

    let mut tailer = LogFile::at(&path).follow();
    assert!(tailer.poll(10).is_empty());

    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    write!(f, "one\ntw").unwrap();
    assert_eq!(tailer.poll(10), vec!["one"]);

    writeln!(f, "o").unwrap();
    assert_eq!(tailer.poll(10), vec!["two"]);
}

#[test]
fn tailer_restarts_after_truncation() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("d.log");
    std::fs::write(&path, "a long first line\n").unwrap();
    let mut tailer = LogFile::at(&path).follow();

    std::fs::write(&path, "new\n").unwrap();
    assert_eq!(tailer.poll(10), vec!["new"]);
}

#[test]
fn tailer_waits_for_missing_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("later.log");
    let mut tailer = LogFile::at(&path).follow();
    assert!(tailer.poll(10).is_empty());

    std::fs::write(&path, "hi\n").unwrap();
    assert_eq!(tailer.poll(10), vec!["hi"]);
}

#[test]
fn tail_keeps_last_n() {
    let tmp = tempfile::tempdir().unwrap();
    let log = LogFile::at(tmp.path().join("d.log"));
    std::fs::write(log.path(), "1\n2\n3\n4\n").unwrap();
    assert_eq!(log.tail(2).unwrap(), vec!["3", "4"]);
    assert_eq!(log.tail(10).unwrap().len(), 4);
    assert!(log.tail(0).unwrap().is_empty());
}

#[test]
fn tail_of_a_missing_log_is_an_error() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(LogFile::at(tmp.path().join("nope.log")).tail(5).is_err());
}

/// Bigger than the chunk `tail` reads back in, so the lines it keeps span
/// a chunk boundary and the cut first line has to be dropped.
#[test]
fn tail_of_a_large_log_reads_back_across_chunks() {
    let tmp = tempfile::tempdir().unwrap();
    let log = LogFile::at(tmp.path().join("big.log"));
    let body: String = (0..20_000)
        .map(|i| format!("line {i:05} {}\n", "x".repeat(40)))
        .collect();
    std::fs::write(log.path(), &body).unwrap();

    let tail = log.tail(3000).unwrap();
    assert_eq!(tail.len(), 3000);
    assert!(tail[0].starts_with("line 17000 "), "{}", tail[0]);
    assert!(tail[2999].starts_with("line 19999 "), "{}", tail[2999]);
    assert!(tail.iter().all(|l| l.len() == "line 00000 ".len() + 40));
}

#[test]
fn create_makes_the_directory_and_truncates() {
    let tmp = tempfile::tempdir().unwrap();
    let log = LogFile::at(tmp.path().join("a/b/setup.log"));
    std::fs::create_dir_all(log.path().parent().unwrap()).unwrap();
    std::fs::write(log.path(), "stale\n").unwrap();

    let mut file = log.create().unwrap();
    writeln!(file, "fresh").unwrap();
    assert_eq!(log.tail(10).unwrap(), vec!["fresh"]);

    let nested = LogFile::at(tmp.path().join("x/y/z.log"));
    nested.create().unwrap();
    assert!(nested.exists());
}

#[test]
fn removing_a_missing_log_is_fine() {
    let tmp = tempfile::tempdir().unwrap();
    let log = LogFile::at(tmp.path().join("gone.log"));
    log.remove().unwrap();
    log.create().unwrap();
    log.remove().unwrap();
    assert!(!log.exists());
}

#[test]
fn an_excerpt_quotes_the_end_and_says_where_the_rest_is() {
    let tmp = tempfile::tempdir().unwrap();
    let log = LogFile::at(tmp.path().join("setup.log"));
    std::fs::write(log.path(), "resolving\n\nERR_PNPM_OUTDATED_LOCKFILE\n").unwrap();

    assert_eq!(
        log.excerpt(5),
        format!(
            "  resolving\n  ERR_PNPM_OUTDATED_LOCKFILE\n  (full log: {})",
            log.path().display()
        )
    );
}

#[test]
fn an_excerpt_is_bounded_however_much_was_written() {
    let tmp = tempfile::tempdir().unwrap();
    let log = LogFile::at(tmp.path().join("setup.log"));
    let body: String = (0..5000)
        .map(|i| format!("line {i} {}\n", "x".repeat(1000)))
        .collect();
    std::fs::write(log.path(), body).unwrap();

    let excerpt = log.excerpt(20);
    let lines: Vec<&str> = excerpt.lines().collect();
    assert_eq!(lines.len(), 21, "20 quoted lines and the path");
    assert!(lines[19].starts_with("  line 4999 "), "{}", lines[19]);
    assert!(lines[0].ends_with('…'));
    assert!(excerpt.len() < 8 * 1024, "{}", excerpt.len());
}

#[test]
fn an_excerpt_keeps_only_the_last_redraw_of_a_line() {
    let tmp = tempfile::tempdir().unwrap();
    let log = LogFile::at(tmp.path().join("setup.log"));
    std::fs::write(log.path(), "10%\r50%\r100%\n   \ndone\n").unwrap();
    assert!(log.excerpt(5).starts_with("  100%\n  done\n"));
}

#[test]
fn an_excerpt_of_nothing_says_so() {
    let tmp = tempfile::tempdir().unwrap();
    let log = LogFile::at(tmp.path().join("setup.log"));
    log.create().unwrap();
    assert!(log.excerpt(5).starts_with("  (no output)\n"));
}
