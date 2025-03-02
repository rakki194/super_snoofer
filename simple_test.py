#!/usr/bin/env python3
"""
Simple test script for Super Snoofer basic functionality
"""

import os
import subprocess
import sys

def main():
    # Determine the path to super_snoofer
    super_snoofer_path = None
    if os.path.exists("./target/debug/super_snoofer"):
        super_snoofer_path = "./target/debug/super_snoofer"
    elif os.path.exists("./target/release/super_snoofer"):
        super_snoofer_path = "./target/release/super_snoofer"
    else:
        super_snoofer_path = "super_snoofer"
    
    # Test commands
    test_commands = [
        ["--suggest-full-completion", "git s"],
        ["--suggest-completion", "gi"],
        ["--suggest-frequent-command", ""],
        ["--help"]
    ]
    
    # Run each test command and print results
    for cmd in test_commands:
        full_cmd = [super_snoofer_path] + cmd
        print(f"\n\n=== Testing: {' '.join(full_cmd)} ===")
        
        result = subprocess.run(
            full_cmd,
            capture_output=True,
            text=True,
            check=False
        )
        
        print(f"Return code: {result.returncode}")
        print(f"Stdout ({len(result.stdout)} bytes): {repr(result.stdout)}")
        print(f"Stderr ({len(result.stderr)} bytes): {repr(result.stderr)}")
        
        # Print stdout lines for easier reading if there's output
        if result.stdout:
            print("\nStdout lines:")
            for i, line in enumerate(result.stdout.splitlines()):
                print(f"  {i+1}: {line}")

if __name__ == "__main__":
    main() 