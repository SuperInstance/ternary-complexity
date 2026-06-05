#![forbid(unsafe_code)]
//! Kolmogorov complexity proxy via LZ77 compression for ternary genomes.

/// Simple LZ77 compression for ternary sequences.
/// Returns (compressed_length, original_length, compression_ratio).
pub fn lz77_compress(seq: &[i8]) -> (usize, usize, f64) {
    if seq.is_empty() { return (0, 0, 0.0); }
    let mut compressed = Vec::new();
    let mut pos = 0;
    while pos < seq.len() {
        let mut best_len = 0;
        let mut best_offset = 0;
        let search_start = if pos > 256 { pos - 256 } else { 0 };
        for offset in search_start..pos {
            let mut len = 0;
            while pos + len < seq.len() && len < 16 {
                if seq[offset + (len % (pos - offset))] == seq[pos + len] { len += 1; } else { break; }
            }
            if len > best_len { best_len = len; best_offset = pos - offset; }
        }
        if best_len >= 3 {
            compressed.push((-(best_offset as i8), best_len as u8));
            pos += best_len;
        } else {
            compressed.push((seq[pos], 1));
            pos += 1;
        }
    }
    let ratio = compressed.len() as f64 / seq.len() as f64;
    (compressed.len(), seq.len(), ratio)
}

/// Kolmogorov complexity proxy: compression ratio (lower = more compressible = lower K).
pub fn k_proxy(seq: &[i8]) -> f64 {
    let (_, _, ratio) = lz77_compress(seq);
    ratio
}

/// Entropy rate estimator: conditional entropy from frequency table.
pub fn entropy_rate(seq: &[i8], order: usize) -> f64 {
    if seq.len() <= order { return 0.0; }
    let mut contexts: std::collections::HashMap<Vec<i8>, Vec<i8>> = std::collections::HashMap::new();
    for i in 0..seq.len() - order {
        let ctx = seq[i..i+order].to_vec();
        contexts.entry(ctx).or_default().push(seq[i + order]);
    }
    let mut total_h = 0.0; let mut count = 0;
    for (_ctx, nexts) in &contexts {
        let mut freq = [0usize; 3];
        for &v in nexts { let idx = (v + 1) as usize; if idx < 3 { freq[idx] += 1; } }
        let n = nexts.len() as f64;
        let h: f64 = freq.iter().filter(|&&f| f > 0).map(|&f| { let p = f as f64 / n; -p * p.log2() }).sum();
        total_h += h * nexts.len() as f64;
        count += nexts.len();
    }
    if count == 0 { 0.0 } else { total_h / count as f64 }
}

/// Lempel-Ziv complexity: count distinct substrings.
pub fn lz_complexity(seq: &[i8]) -> usize {
    if seq.is_empty() { return 0; }
    let mut complexity = 1;
    let mut i = 0;
    while i < seq.len() {
        let mut max_len = 1;
        for len in 1..=seq.len() - i {
            let substring = &seq[i..i+len];
            let mut found = false;
            for start in 0..i {
                if start + len <= i && &seq[start..start+len] == substring { found = true; break; }
            }
            if !found { max_len = len; break; }
            max_len = len;
        }
        i += max_len.max(1);
        complexity += 1;
    }
    complexity
}

/// Forgiveness compression advantage: measure how forgiveness reduces genome complexity.
pub fn forgiveness_compression(genome: &[i8], forgiveness_rate: f64) -> (f64, f64) {
    let original_k = k_proxy(genome);
    // Apply forgiveness: replace defection patterns with zeros
    let mut forgiven = genome.to_vec();
    let mut rng_s: u64 = 42;
    let mut rng = || -> f64 { rng_s = rng_s.wrapping_mul(6364136223846793005).wrapping_add(1); (rng_s >> 33) as f64 / (1u64 << 31) as f64 };
    for v in &mut forgiven { if *v == -1 && rng() < forgiveness_rate { *v = 0; } }
    let forgiven_k = k_proxy(&forgiven);
    (original_k, forgiven_k)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_compress_constant() { let seq = vec![1,1,1,1,1,1,1,1]; let (_, _, r) = lz77_compress(&seq); assert!(r < 1.0, "Constant should compress well"); }
    #[test] fn test_compress_random() { let seq: Vec<i8> = (0..100).map(|i| (((i as u64).wrapping_mul(1103515245).wrapping_add(12345) >> 16) % 3) as i8 - 1).collect(); let (_, _, r) = lz77_compress(&seq); assert!(r > 0.05, "random compression ratio was {:.3}", r); }
    #[test] fn test_k_proxy_low_for_constant() { assert!(k_proxy(&vec![1,1,1,1,1,1]) < 0.8); }
    #[test] fn test_k_proxy_high_for_complex() { let seq: Vec<i8> = (0..50).map(|i| ((i * 13 + 7) % 3) as i8 - 1).collect(); assert!(k_proxy(&seq) > 0.1); }
    #[test] fn test_entropy_rate_constant() { assert!(entropy_rate(&vec![1,1,1,1,1,1], 2) < 0.1); }
    #[test] fn test_entropy_rate_alternating() { let seq = vec![1,-1,1,-1,1,-1,1,-1]; let er = entropy_rate(&seq, 2); assert!(er < 0.5, "Alternating should be predictable"); }
    #[test] fn test_lz_complexity_constant() { let c = lz_complexity(&vec![1,1,1,1]); assert!(c <= 5, "constant seq complexity was {}", c); }
    #[test] fn test_lz_complexity_random() { let seq: Vec<i8> = (0..30).map(|i| ((i * 31 + 17) % 3) as i8 - 1).collect(); assert!(lz_complexity(&seq) > 5); }
    #[test] fn test_forgiveness_reduces_complexity() { let genome = vec![1,-1,1,-1,1,-1]; let (orig, forg) = forgiveness_compression(&genome, 1.0); assert!(forg <= orig); }
    #[test] fn test_compress_empty() { assert_eq!(lz77_compress(&[]), (0, 0, 0.0)); }
    #[test] fn test_compress_single() { let (_, n, r) = lz77_compress(&[1]); assert_eq!(n, 1); assert!((r - 1.0).abs() < 0.01); }
    #[test] fn test_k_proxy_empty() { assert_eq!(k_proxy(&[]), 0.0); }
    #[test] fn test_entropy_rate_empty() { assert_eq!(entropy_rate(&[], 2), 0.0); }
    #[test] fn test_lz_complexity_empty() { assert_eq!(lz_complexity(&[]), 0); }
    #[test] fn test_lz_complexity_single() { assert_eq!(lz_complexity(&[1]), 2); }
    #[test] fn test_compress_repetitive() { let seq = vec![1,0,-1,1,0,-1,1,0,-1]; let (_, _, r) = lz77_compress(&seq); assert!(r < 0.9, "Repetitive should compress: ratio={}", r); }
    #[test] fn test_entropy_rate_high_for_random() { let seq: Vec<i8> = (0..200).map(|i| (((i as u64).wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407) >> 33) % 3) as i8 - 1).collect(); let er = entropy_rate(&seq, 2); assert!(er > 0.2, "entropy rate was {:.3}", er); }
    #[test] fn test_forgiveness_zero_rate() { let genome = vec![1,-1,1,-1]; let (o, f) = forgiveness_compression(&genome, 0.0); assert!((o - f).abs() < 0.01); }
}
