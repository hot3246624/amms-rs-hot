use std::cmp;

// Copy of the logic from mod.rs
fn tick_to_word(tick: i32, tick_spacing: i32) -> i32 {
    let mut compressed = tick / tick_spacing;
    if tick < 0 && tick % tick_spacing != 0 {
        compressed -= 1;
    }

    compressed >> 8
}

fn simulate_bug() {
    println!("Simulating bug with tick_spacing = 1");
    let tick_spacing = 1;
    let min_tick = -887272;
    let max_tick = 887272;
    
    let mut min_word = tick_to_word(min_tick, tick_spacing);
    let max_word = tick_to_word(max_tick, tick_spacing);
    let mut word_range = max_word - min_word + 1;
    
    println!("Initial State:");
    println!("Min Word: {}", min_word);
    println!("Max Word: {}", max_word);
    println!("Word Range: {}", word_range);
    
    let max_range = 6900;
    let mut group_range = 0;
    
    let mut covered_words = std::collections::HashSet::new();
    
    let mut iteration = 0;
    while word_range > 0 {
        iteration += 1;
        let remaining_range = max_range - group_range;
        let range = cmp::min(word_range, remaining_range);
        
        println!("\nIteration {}:", iteration);
        println!("  Current Min Word: {}", min_word);
        println!("  Range to fetch: {}", range);
        println!("  Requesting words [{}, {})", min_word, min_word + range);
        
        // Simulate fetching
        for i in 0..range {
            covered_words.insert(min_word + i);
        }
        
        word_range -= range;
        min_word += range; // FIXED LOGIC
        group_range += range;
        
        if group_range >= max_range {
             group_range = 0;
        }
    }
    
    println!("\nSimulation Result:");
    let expected_total = tick_to_word(max_tick, tick_spacing) - tick_to_word(min_tick, tick_spacing) + 1;
    println!("Expected words (approx): {}", expected_total);
    println!("Actually covered unique words: {}", covered_words.len());
    
    let target_max_word = 3465;
    if !covered_words.contains(&target_max_word) {
        println!("FAILURE: Did not cover the max word {}", target_max_word);
    } else {
        println!("SUCCESS: Covered max word");
    }
    
    if !covered_words.contains(&(target_max_word - 1)) {
        println!("Checking word {}: MISSING (Expected as incorrect overlap might miss end)", target_max_word - 1);
    }
}

fn main() {
    simulate_bug();
}
