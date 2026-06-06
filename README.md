# ternary-complexity

Kolmogorov complexity proxies for ternary sequences. Compress, measure, compare.

How complex is a ternary sequence? A string of all zeros is trivial—it compresses to nothing. A truly random sequence is incompressible—every symbol carries information. This crate estimates where a sequence falls on that spectrum using three complementary approaches: LZ77 compression ratio, Lempel-Ziv substring counting, and conditional entropy rate. Plus a unique `forgiveness_compression` function that measures how much complexity drops when you replace defection patterns (-1) with neutral (0).

## Why this exists

In multi-agent systems, behavioral sequences encode strategy. A simple strategy (always cooperate) produces a low-complexity sequence. A sophisticated strategy (tit-for-tat with forgiveness) produces a medium-complexity sequence. A random strategy produces high complexity. By measuring the Kolmogorov complexity of an agent's action history, you can classify its sophistication without understanding its internals.

The forgiveness compression is the crate's signature feature. It measures how much a sequence's complexity decreases when you replace defection (-1) with neutrality (0) at a given rate. This quantifies the *cost of conflict*: high forgiveness compression means the sequence contains a lot of adversarial structure that disappears when you stop fighting.

## The key insight

Kolmogorov complexity is uncomputable, but you can approximate it from above with compression algorithms. If a compressor can shrink a sequence significantly, the sequence has low Kolmogorov complexity—the compressor found and exploited structure.

For ternary sequences, LZ77 compression is surprisingly effective because:
1. The alphabet is tiny (3 symbols), so matching is common
2. Ternary patterns tend to be repetitive (strategies repeat)
3. The search window (256 positions, max match 16) is tuned for typical strategy sequences

The three measures give different perspectives:

| Measure | What it captures | Best for |
|---------|-----------------|----------|
| `k_proxy` (LZ77 ratio) | Overall compressibility | Long sequences (>50 elements) |
| `lz_complexity` (distinct substrings) | Vocabulary richness | Short-to-medium sequences |
| `entropy_rate` (conditional entropy) | Predictability given context | Detecting periodic patterns |

## Quick start

```rust
use ternary_complexity::*;

// A repetitive sequence (low complexity)
let repetitive = vec![1, 0, -1, 1, 0, -1, 1, 0, -1, 1, 0, -1];
let k = k_proxy(&repetitive);
assert!(k < 0.5, "Repetitive sequence should compress well: ratio={}", k);

// A complex sequence (high complexity)
let complex: Vec<i8> = (0..100)
    .map(|i| ((i * 31 + 17) % 3) as i8 - 1)
    .collect();
let k = k_proxy(&complex);
// Won't compress as well as the repetitive one

// Lempel-Ziv complexity (count distinct substrings)
let c = lz_complexity(&vec![1, 1, 1, 1]);
assert!(c <= 5, "Constant has low LZ complexity");

// Entropy rate with order-2 context
let alternating = vec![1, -1, 1, -1, 1, -1, 1, -1];
let er = entropy_rate(&alternating, 2);
assert!(er < 0.5, "Alternating pattern is predictable");
```

## API reference

### `lz77_compress(seq) → (usize, usize, f64)`

LZ77 compression for ternary sequences. Returns `(compressed_length, original_length, compression_ratio)`.

- Search window: 256 positions back
- Maximum match length: 16
- Minimum match length: 3 (shorter matches emitted as literals)

```rust
let (comp_len, orig_len, ratio) = lz77_compress(&seq);
// ratio < 1.0 means the sequence compressed (has structure)
// ratio ≈ 1.0 means incompressible (random or complex)
```

### `k_proxy(seq) → f64`

Kolmogorov complexity proxy: the LZ77 compression ratio. Lower = more compressible = lower complexity.

### `entropy_rate(seq, order) → f64`

Conditional entropy estimator. Builds a frequency table of contexts of length `order`, then computes the average conditional entropy of the next symbol given the context.

- `order=0`: marginal entropy (symbol frequency)
- `order=1`: entropy given previous symbol
- `order=2`: entropy given previous two symbols

Higher-order models capture more structure but need longer sequences to be reliable.

### `lz_complexity(seq) → usize`

Lempel-Ziv complexity: the number of distinct substrings encountered when parsing left-to-right, always extending by the shortest substring not seen before.

### `forgiveness_compression(genome, forgiveness_rate) → (f64, f64)`

Measures how much complexity changes when you replace defection (-1) with neutral (0) at the given rate. Returns `(original_complexity, forgiven_complexity)`.

```rust
let genome = vec![1, -1, 1, -1, 1, -1];  // alternating cooperate/defect
let (original, forgiven) = forgiveness_compression(&genome, 1.0);
// At rate 1.0, all -1s become 0: [1, 0, 1, 0, 1, 0]
// This has lower complexity than the original
assert!(forgiven <= original);
```

## The forgiveness insight

Consider two agent genomes:
- Agent A: `[1, -1, 1, -1, 1, -1]` — pure tit-for-tat
- Agent B: `[1, 0, 1, 0, 1, 0]` — tit-for-tat with forgiveness

Agent B's genome has lower complexity because the -1→0 substitution removes the adversarial structure. The `forgiveness_compression` function quantifies this difference. A high forgiveness advantage means the agent's strategy is built around conflict—removing conflict simplifies it significantly.

This has implications for evolutionary game theory: strategies with high forgiveness compression are *brittle*—they depend on conflict for their identity. Strategies with low forgiveness compression are *robust*—they maintain complexity even when you remove the adversarial dynamics.

## Real-world example: Agent strategy analysis

```rust
use ternary_complexity::*;

fn analyze_strategy(name: &str, history: &[i8]) {
    let k = k_proxy(history);
    let lz = lz_complexity(history);
    let er = entropy_rate(history, 2);
    let (orig_k, forg_k) = forgiveness_compression(history, 0.5);
    
    println!("Strategy: {}", name);
    println!("  K-proxy:          {:.3} (lower = simpler)", k);
    println!("  LZ complexity:    {} distinct substrings", lz);
    println!("  Entropy rate:     {:.3} bits", er);
    println!("  Forgiveness Δ:    {:.3} → {:.3} (drop = {:.3})",
        orig_k, forg_k, orig_k - forg_k);
    println!();
}

// Always cooperate
analyze_strategy("Saint", &vec![1; 50]);

// Pure tit-for-tat
let tft: Vec<i8> = (0..50).map(|i| if i % 2 == 0 { 1 } else { -1 }).collect();
analyze_strategy("Tit-for-tat", &tft);

// Random
let random: Vec<i8> = (0..50).map(|i| ((i * 6364136223846793005 + 1442695040888963407) as i64 % 3) as i8 - 1).collect();
analyze_strategy("Random", &random);
```

## Architecture

All functions are pure: `&[i8]` in, scalar or tuple out. No state, no mutability, no side effects. The LZ77 compressor builds a temporary `Vec` of (offset, length) pairs internally. The entropy rate estimator builds a `HashMap` of contexts.

The crate uses `#![forbid(unsafe_code)]` and has zero dependencies beyond `std`.

## Ecosystem connections

- **ternary-classifier** — complexity measures can be features for species classification (high-complexity agents are Explorers, low-complexity are Specialists)
- **ternary-gauge** — track complexity over time to detect strategy shifts (an agent switching from low to high complexity is adapting)
- **ternary-bite** — crushing and downsampling are forms of lossy compression; compare their effect on complexity

## Stats

| Metric | Value |
|--------|-------|
| Tests | 18 |
| Public functions | 5 |
| Lines of code | ~114 |
| License | MIT |
| Unsafe | 0 |

## Installation

```toml
[dependencies]
ternary-complexity = "0.1.0"
```

## License

MIT
