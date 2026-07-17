// Conformance probes: each tests/conformance/probes/<name>.p8 is a minimal
// cart whose printh() output was captured from official PICO-8 into
// tests/conformance/golden/<name>.txt via tools/oracle.sh. Running it through
// run-cart must reproduce that output byte-for-byte.
//
// Probes for known, not-yet-fixed gaps are listed in
// tests/conformance/expected_failures.txt (one stem per line) so the suite
// stays green while the gap is still open.

use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

fn expected_failures() -> HashSet<String> {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/conformance/expected_failures.txt");
    match std::fs::read_to_string(path) {
        Ok(s) => s
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(String::from)
            .collect(),
        Err(_) => HashSet::new(),
    }
}

#[test]
fn probes_match_official_pico8_golden() {
    let probes_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/conformance/probes");
    let golden_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/conformance/golden");
    let xfail = expected_failures();

    let Ok(entries) = std::fs::read_dir(&probes_dir) else {
        return; // no probes yet
    };

    let mut failures = Vec::new();
    let mut ran = 0;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("p8") {
            continue;
        }
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let golden_path = golden_dir.join(format!("{stem}.txt"));
        let Ok(golden) = std::fs::read_to_string(&golden_path) else {
            panic!("probe {stem}.p8 has no golden at {golden_path:?}; run tools/oracle.sh");
        };

        let output = Command::new(env!("CARGO_BIN_EXE_run-cart"))
            .arg(&path)
            .arg("1")
            .output()
            .expect("run-cart executes");

        let actual: String = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|l| l.starts_with("RESULT") || l.starts_with("DONE"))
            .map(|l| format!("{l}\n"))
            .collect();

        ran += 1;
        if actual != golden {
            if xfail.contains(&stem) {
                continue; // known gap, documented
            }
            failures.push(format!(
                "probe {stem}:\n--- expected (official PICO-8) ---\n{golden}--- actual (pico-r) ---\n{actual}"
            ));
        } else if xfail.contains(&stem) {
            panic!(
                "probe {stem} is listed in expected_failures.txt but now matches the golden — remove it from the xfail list"
            );
        }
    }

    assert!(
        failures.is_empty(),
        "{} conformance probe(s) diverge from official PICO-8:\n\n{}",
        failures.len(),
        failures.join("\n")
    );
    if ran == 0 {
        eprintln!("note: no conformance probes found");
    }
}
