/// djb hash (times33) algorithm implementation
/// This is the hash algorithm proposed by Daniel J. Bernstein
pub fn djb_hash(input: &str) -> u64 {
    let mut hash: u64 = 5381;
    
    for byte in input.bytes() {
        // Use wrapping arithmetic to avoid overflow panic
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_djb_hash() {
        let hash1 = djb_hash("hello");
        let hash2 = djb_hash("world");
        
        assert_ne!(hash1, hash2);
        assert!(hash1 > 0);
        assert!(hash2 > 0);
    }

    #[test]
    fn test_djb_hash_consistency() {
        let hash1 = djb_hash("test");
        let hash2 = djb_hash("test");
        
        assert_eq!(hash1, hash2);
    }
}

