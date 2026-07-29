//! # Prime Scheduling Utilities
//!
//! Prime numbers provide non-repeating but deterministic structure for:
//! - **Time signatures**: parameter groups update at different prime intervals (2, 3, 5, 7, 11 frames)
//! - **Channel indexing**: primes select subsets of instances/nodes/layers to modulate
//! - **Sampling windows**: prime-offset windows prevent lock into short loops
//!
//! All logic runs in Rust host — no prime computation in WGSL shaders.
//! Outputs are exposed as uniform blocks, storage buffers, or instance data.

use std::collections::HashMap;

/// A prime-based schedule that determines when parameter groups update.
///
/// Each group is assigned a prime interval (frames between updates).
/// Groups with smaller primes update more frequently (2 = every 2 frames),
/// groups with larger primes update less often (11 = every 11 frames).
#[derive(Debug, Clone)]
pub struct PrimeSchedule {
    /// Prime intervals for each parameter group (group_name -> prime_interval)
    pub groups: HashMap<String, u32>,
    /// Current frame counter
    frame: u64,
    /// Cached which groups should update this frame
    active_groups: Vec<String>,
}

impl PrimeSchedule {
    /// Create a new prime schedule with default groups
    ///
    /// Default groups use the first 5 primes: 2, 3, 5, 7, 11
    /// - Group A (prime 2): fastest modulation (every 2 frames)
    /// - Group B (prime 3): medium-fast (every 3 frames)
    /// - Group C (prime 5): medium (every 5 frames)
    /// - Group D (prime 7): medium-slow (every 7 frames)
    /// - Group E (prime 11): slowest (every 11 frames)
    pub fn new() -> Self {
        let mut groups = HashMap::new();
        groups.insert("group_a".to_string(), 2);
        groups.insert("group_b".to_string(), 3);
        groups.insert("group_c".to_string(), 5);
        groups.insert("group_d".to_string(), 7);
        groups.insert("group_e".to_string(), 11);
        Self {
            groups,
            frame: 0,
            active_groups: Vec::new(),
        }
    }

    /// Create a schedule with custom prime intervals
    pub fn with_groups(groups: HashMap<String, u32>) -> Self {
        Self {
            groups,
            frame: 0,
            active_groups: Vec::new(),
        }
    }

    /// Advance the frame counter and compute which groups are active this frame
    ///
    /// A group is active when `frame % prime_interval == 0`
    pub fn tick(&mut self) {
        self.frame += 1;
        self.active_groups.clear();
        for (name, &prime) in &self.groups {
            if prime > 0 && self.frame % prime as u64 == 0 {
                self.active_groups.push(name.clone());
            }
        }
    }

    /// Get the current frame number
    pub fn frame(&self) -> u64 {
        self.frame
    }

    /// Get which groups are active this frame
    pub fn active_groups(&self) -> &[String] {
        &self.active_groups
    }

    /// Check if a specific group is active this frame
    pub fn is_group_active(&self, group_name: &str) -> bool {
        self.active_groups.contains(&group_name.to_string())
    }

    /// Reset the schedule to frame 0
    pub fn reset(&mut self) {
        self.frame = 0;
        self.active_groups.clear();
    }
}

/// A prime-based bitmask for selecting which instances/nodes receive modulation.
///
/// For N instances, the mask selects instances whose IDs are:
/// - Multiples of a prime (e.g., ID % 2 == 0, ID % 3 == 0, etc.)
/// - Non-multiples (inverse selection)
/// - Composite selections (e.g., ID % 2 == 0 && ID % 3 == 0)
#[derive(Debug, Clone)]
pub struct PrimeMask {
    /// Number of instances to mask
    pub count: usize,
    /// Current prime used for masking
    pub current_prime: u32,
    /// Cached mask bits (true = selected)
    mask: Vec<bool>,
}

impl PrimeMask {
    /// Create a new prime mask for N instances
    pub fn new(count: usize) -> Self {
        // Find first prime suitable for this count
        let p = if count > 1 { 2u32 } else { 2u32 };
        let mut mask = vec![false; count];
        for i in 0..count {
            mask[i] = (i as u32) % p == 0;
        }
        Self {
            count,
            current_prime: p,
            mask,
        }
    }

    /// Create a mask using a specific prime
    pub fn with_prime(count: usize, prime: u32) -> Self {
        let mut mask = vec![false; count];
        for i in 0..count {
            mask[i] = (i as u32) % prime == 0;
        }
        Self {
            count,
            current_prime: prime,
            mask,
        }
    }

    /// Recompute the mask with a different prime
    pub fn set_prime(&mut self, prime: u32) {
        self.current_prime = prime;
        for i in 0..self.count {
            self.mask[i] = (i as u32) % prime == 0;
        }
    }

    /// Get whether instance at index is selected
    pub fn is_selected(&self, index: usize) -> bool {
        index < self.count && self.mask[index]
    }

    /// Get the raw mask vector
    pub fn mask(&self) -> &[bool] {
        &self.mask
    }

    /// Count how many instances are selected
    pub fn selected_count(&self) -> usize {
        self.mask.iter().filter(|&&b| b).count()
    }

    /// Get indices of selected instances
    pub fn selected_indices(&self) -> Vec<usize> {
        self.mask
            .iter()
            .enumerate()
            .filter(|(_, &b)| b)
            .map(|(i, _)| i)
            .collect()
    }
}

/// A rotating set of primes that cycles through a schedule.
///
/// Useful for creating evolving non-repeating patterns:
/// - Cycle through [2, 3, 5, 7, 11] for instance selection
/// - Each tick advances to next prime
/// - Wraps around when reaching the end
#[derive(Debug, Clone)]
pub struct PrimeRotator {
    /// The list of primes to cycle through
    pub primes: Vec<u32>,
    /// Current index in the cycle
    index: usize,
}

impl PrimeRotator {
    /// Create a new rotator with the first N primes
    pub fn new(count: usize) -> Self {
        let primes: Vec<u32> = PRIMES.iter().take(count.max(1)).copied().collect();
        Self { primes, index: 0 }
    }

    /// Create a rotator with custom prime list
    pub fn with_primes(primes: Vec<u32>) -> Self {
        Self { primes, index: 0 }
    }

    /// Advance to the next prime and return it
    pub fn next(&mut self) -> u32 {
        let prime = self.primes[self.index % self.primes.len()];
        self.index = (self.index + 1) % self.primes.len();
        prime
    }

    /// Get the current prime without advancing
    pub fn current(&self) -> u32 {
        self.primes[self.index % self.primes.len()]
    }

    /// Reset the cycle
    pub fn reset(&mut self) {
        self.index = 0;
    }
}

// First 20 primes for convenient use
const PRIMES: [u32; 20] = [
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71,
];

/// Check if a number is prime (simple trial division)
pub fn is_prime(n: u32) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 || n == 3 {
        return true;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return false;
    }
    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 {
            return false;
        }
        i += 6;
    }
    true
}

/// Generate all primes up to a limit using a simple sieve
pub fn prime_sieve(limit: u32) -> Vec<u32> {
    if limit < 2 {
        return Vec::new();
    }
    let mut is_composite = vec![false; limit as usize + 1];
    let mut primes = Vec::new();
    for i in 2..=limit {
        if !is_composite[i as usize] {
            primes.push(i);
            let mut multiple = i * i;
            while multiple <= limit {
                is_composite[multiple as usize] = true;
                multiple += i;
            }
        }
    }
    primes
}

/// Get the prime factors of a number
pub fn prime_factors(n: u32) -> Vec<u32> {
    let mut n = n;
    let mut factors = Vec::new();
    let mut d = 2;
    while d * d <= n {
        while n % d == 0 {
            factors.push(d);
            n /= d;
        }
        d += if d == 2 { 1 } else { 2 };
    }
    if n > 1 {
        factors.push(n);
    }
    factors
}

/// Get the nth prime (1-indexed)
pub fn nth_prime(n: usize) -> u32 {
    PRIMES.get(n.saturating_sub(1)).copied().unwrap_or(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prime_schedule_tick() {
        let mut sched = PrimeSchedule::new();
        assert_eq!(sched.frame(), 0);
        // Group A (prime 2) should be active on even frames
        sched.tick(); // frame 1
        assert!(!sched.is_group_active("group_a"));
        sched.tick(); // frame 2
        assert!(sched.is_group_active("group_a"));
        sched.tick(); // frame 3
        assert!(!sched.is_group_active("group_a"));
        sched.tick(); // frame 4
        assert!(sched.is_group_active("group_a"));
    }

    #[test]
    fn test_prime_schedule_multiple_groups() {
        let mut sched = PrimeSchedule::new();
        for _ in 0..6 {
            sched.tick();
        }
        // Frame 6: divisible by 2 and 3, not by 5, 7, 11
        assert!(sched.is_group_active("group_a")); // prime 2
        assert!(sched.is_group_active("group_b")); // prime 3
        assert!(!sched.is_group_active("group_c")); // prime 5
    }

    #[test]
    fn test_prime_mask() {
        let mask = PrimeMask::new(10);
        assert_eq!(mask.count, 10);
        assert_eq!(mask.current_prime, 2);
        // With prime 2, even indices are selected
        assert!(mask.is_selected(0));
        assert!(!mask.is_selected(1));
        assert!(mask.is_selected(2));
        assert!(!mask.is_selected(3));
    }

    #[test]
    fn test_prime_mask_custom_prime() {
        let mask = PrimeMask::with_prime(10, 3);
        // With prime 3, indices 0, 3, 6, 9 are selected
        assert!(mask.is_selected(0));
        assert!(!mask.is_selected(1));
        assert!(!mask.is_selected(2));
        assert!(mask.is_selected(3));
    }

    #[test]
    fn test_prime_mask_switch_prime() {
        let mut mask = PrimeMask::new(10);
        mask.set_prime(5);
        assert_eq!(mask.current_prime, 5);
        assert!(mask.is_selected(0));
        assert!(mask.is_selected(5));
        assert!(!mask.is_selected(1));
    }

    #[test]
    fn test_prime_rotator() {
        let mut rot = PrimeRotator::new(3);
        assert_eq!(rot.next(), 2);
        assert_eq!(rot.next(), 3);
        assert_eq!(rot.next(), 5);
        assert_eq!(rot.next(), 2); // wraps
    }

    #[test]
    fn test_is_prime() {
        assert!(!is_prime(1));
        assert!(is_prime(2));
        assert!(is_prime(3));
        assert!(!is_prime(4));
        assert!(is_prime(5));
        assert!(!is_prime(6));
        assert!(is_prime(7));
        assert!(is_prime(11));
        assert!(is_prime(13));
        assert!(!is_prime(15));
        assert!(is_prime(97));
    }

    #[test]
    fn test_prime_sieve() {
        let primes = prime_sieve(30);
        assert_eq!(primes, vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
    }

    #[test]
    fn test_prime_factors() {
        assert_eq!(prime_factors(12), vec![2, 2, 3]);
        assert_eq!(prime_factors(17), vec![17]);
        assert_eq!(prime_factors(100), vec![2, 2, 5, 5]);
    }

    #[test]
    fn test_nth_prime() {
        assert_eq!(nth_prime(1), 2);
        assert_eq!(nth_prime(2), 3);
        assert_eq!(nth_prime(5), 11);
    }
}
