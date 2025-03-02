use super_snoofer::cache::CommandCache;

fn main() {
    // Test completion for "cat " command
    let cmd = "cat ";
    println!("Testing completions for: '{}'", cmd);
    
    // Call the generate_full_completion function directly
    let completions = super_snoofer::cache::generate_full_completion(cmd);
    
    // Print the completions
    if completions.is_empty() {
        println!("No completions found.");
    } else {
        println!("Found {} completions:", completions.len());
        for (i, completion) in completions.iter().enumerate() {
            println!("  {}. '{}'", i + 1, completion);
        }
    }
} 