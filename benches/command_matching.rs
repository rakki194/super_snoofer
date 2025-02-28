#![warn(clippy::all, clippy::pedantic)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use super_snoofer::CommandCache;

fn setup_test_cache() -> CommandCache {
    let mut cache = CommandCache::default();
    
    // Add a realistic set of common commands
    let common_commands = [
        "git", "cargo", "python", "python3", "npm", "node", "rustc", "gcc",
        "clang", "make", "cmake", "docker", "kubectl", "vim", "nvim", "code",
        "ls", "cd", "cp", "mv", "rm", "cat", "grep", "find", "sed", "awk",
        "curl", "wget", "tar", "zip", "unzip", "ssh", "scp", "rsync",
        "systemctl", "journalctl", "top", "htop", "ps", "kill", "pkill",
    ];
    
    for cmd in &common_commands {
        cache.insert(cmd);
    }
    
    cache
}

fn bench_command_matching(c: &mut Criterion) {
    let cache = setup_test_cache();
    
    let mut group = c.benchmark_group("command_matching");
    
    // Benchmark exact matches
    group.bench_function("exact_match", |b| {
        b.iter(|| cache.find_similar(black_box("git")));
    });
    
    // Benchmark common typos
    group.bench_function("common_typo", |b| {
        b.iter(|| cache.find_similar(black_box("gti")));
    });
    
    // Benchmark learned corrections
    group.bench_function("learned_correction", |b| {
        let mut cache = setup_test_cache();
        cache.learn_correction("gti", "git").unwrap();
        b.iter(|| cache.find_similar(black_box("gti")));
    });
    
    // Benchmark no match
    group.bench_function("no_match", |b| {
        b.iter(|| cache.find_similar(black_box("xyzabc")));
    });
    
    // Benchmark with longer command names
    group.bench_function("long_command", |b| {
        b.iter(|| cache.find_similar(black_box("systemctll")));
    });
    
    group.finish();
}

criterion_group!(benches, bench_command_matching);
criterion_main!(benches); 