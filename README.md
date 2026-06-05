# ternary-complexity

**Kolmogorov complexity and compressibility for ternary sequences.**

How complex is a signal, really? Kolmogorov complexity says: the shortest program that produces it. Since we can't compute that directly, we approximate with compression. A sequence that compresses well is *simple* — it has regularity, structure, pattern. A sequence that doesn't compress is *complex* — it's close to random.

This crate provides three complementary measures of complexity for ternary sequences (`{-1, 0, +1}`):

1. **LZ77 compression** — classic sliding-window compression ratio
2. **Entropy rate** — conditional entropy from n-gram statistics
3. **LZ complexity** — count of distinct substrings (Lempel-Ziv complexity)

Together, they give a fingerprint of how much *information* a ternary signal actually carries.

## What's Inside

- **`lz77_compress(seq)`** — sliding-window compression. Returns `(compressed_len, original_len, ratio)`
- **`k_proxy(seq)`** — Kolmogorov complexity proxy: the compression ratio (lower = simpler)
- **`entropy_rate(seq, order)`** — n-gram conditional entropy. Higher = more random per symbol
- **`lz_complexity(seq)`** — Lempel-Ziv complexity: count of distinct blocks. More blocks = more complex

## Quick Example

```rust
use ternary_complexity::*;

// A repeating pattern — very simple
let simple = vec![1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1, 0];
let (len, orig, ratio) = lz77_compress(&simple);
assert!(ratio < 1.0); // compresses well
assert!(k_proxy(&simple) < 0.5); // low Kolmogorov proxy

// A random-ish sequence — much more complex
let complex = vec![1, -1, 1, 0, -1, 1, 0, 0, -1, 1, -1, 0];
let k_complex = k_proxy(&complex);
assert!(k_complex > k_proxy(&simple)); // harder to compress

// Entropy rate: how predictable is the next symbol?
let h = entropy_rate(&complex, 2); // order-2 conditional entropy
// h ∈ [0, log₂(3)] ≈ [0, 1.585] bits per symbol

// LZ complexity: distinct blocks
let blocks = lz_complexity(&simple);
let blocks_random = lz_complexity(&complex);
assert!(blocks < blocks_random); // simpler = fewer distinct blocks
```

## Why Measure Ternary Complexity?

**Structure vs. randomness is the fundamental question.** In agent systems, cellular automata, genetic algorithms, and any generative process, you need to know: is the output doing something *interesting*, or just making noise? Complexity measures answer that quantitatively.

Ternary sequences are the simplest non-binary case — rich enough to show structure, simple enough to compute fast. This makes them ideal testbeds for complexity research.

**Use cases:**
- **Cellular automata** — classify rule complexity (Wolfram Class 1-4)
- **Genetic algorithms** — measure diversity of populations
- **Anomaly detection** — sequences with unusual complexity are suspicious
- **Music/information dynamics** — track information content over time in a composition
- **Agent behavior analysis** — is an agent's action sequence structured or random?

## See Also
- **ternary-entropy** — related
- **ternary-mutual-info** — related
- **ternary-chaos** — related
- **ternary-fib** — related
- **ternary-collatz** — related

## Install

```bash
cargo add ternary-complexity
```

## License

MIT
