//! Ratio-based space distribution — equivalent to Rich's `_ratio.py`.
//!
//! Used by `Table` and `Layout` to distribute available width among flexible
//! columns using proportional ratios and minimum-size constraints.

/// Distribute `total` space among `n` items using `ratios` and `minimums`.
/// Returns a `Vec<usize>` where each element respects its minimum.
///
/// Equivalent to Python Rich's `ratio_distribute()`.
pub fn ratio_distribute(total: usize, ratios: &[usize], minimums: &[usize]) -> Vec<usize> {
    let n = ratios.len();
    if n == 0 {
        return Vec::new();
    }

    let ratio_sum: usize = ratios.iter().sum();
    if ratio_sum == 0 {
        // Equal distribution
        let each = total / n;
        return (0..n).map(|_| each.max(1)).collect();
    }

    let mut sizes: Vec<usize> = ratios
        .iter()
        .enumerate()
        .map(|(i, &r)| {
            let min = minimums.get(i).copied().unwrap_or(1);
            let size = (total * r) / ratio_sum;
            size.max(min)
        })
        .collect();

    // Adjust for rounding error
    let current_sum: usize = sizes.iter().sum();
    if current_sum < total && n > 0 {
        sizes[n - 1] += total - current_sum;
    }

    sizes
}

/// Resolve flexible layout sizes with optional fixed `sizes`, `ratios`, and
/// `minimums`. Items with a fixed `size` get that value; remaining space is
/// distributed to items with `ratio` values.
///
/// Equivalent to Python Rich's `ratio_resolve()`.
pub fn ratio_resolve(
    total: usize,
    fixed_sizes: &[Option<usize>],
    ratios: &[usize],
    minimums: &[usize],
) -> Vec<usize> {
    let n = fixed_sizes.len();
    if n == 0 {
        return Vec::new();
    }

    let mut result = vec![0usize; n];
    let mut used = 0usize;
    let mut flex_indices: Vec<usize> = Vec::new();

    for (i, item) in result.iter_mut().enumerate() {
        if let Some(fixed) = fixed_sizes.get(i).copied().flatten() {
            *item = fixed.max(minimums.get(i).copied().unwrap_or(1));
            used += *item;
        } else {
            flex_indices.push(i);
        }
    }

    if flex_indices.is_empty() {
        return result;
    }

    let remaining = total.saturating_sub(used);
    let flex_ratios: Vec<usize> = flex_indices
        .iter()
        .map(|&i| ratios.get(i).copied().unwrap_or(1))
        .collect();
    let flex_minimums: Vec<usize> = flex_indices
        .iter()
        .map(|&i| minimums.get(i).copied().unwrap_or(1))
        .collect();

    let distributed = ratio_distribute(remaining, &flex_ratios, &flex_minimums);

    for (idx, &flex_idx) in flex_indices.iter().enumerate() {
        result[flex_idx] = distributed[idx];
    }

    result
}

/// Reduce a set of values proportionally so their sum fits within `total`,
/// respecting `maximums`.
///
/// Equivalent to Python Rich's `ratio_reduce()`.
pub fn ratio_reduce(
    total: usize,
    ratios: &[usize],
    maximums: &[usize],
    values: &[usize],
) -> Vec<usize> {
    let n = values.len();
    if n == 0 {
        return Vec::new();
    }

    let current_sum: usize = values.iter().sum();
    if current_sum <= total {
        return values.to_vec();
    }

    let excess = current_sum - total;
    let ratio_sum: usize = ratios.iter().sum();

    if ratio_sum == 0 {
        return values.to_vec();
    }

    let mut result = values.to_vec();
    for i in 0..n {
        let reduction = (excess * ratios[i]) / ratio_sum;
        let max = maximums.get(i).copied().unwrap_or(usize::MAX);
        result[i] = values[i].saturating_sub(reduction).max(1).min(max);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ratio_distribute_equal() {
        let result = ratio_distribute(100, &[1, 1], &[1, 1]);
        assert_eq!(result, vec![50, 50]);
    }

    #[test]
    fn test_ratio_distribute_weighted() {
        let result = ratio_distribute(100, &[3, 1], &[1, 1]);
        assert_eq!(result, vec![75, 25]);
    }

    #[test]
    fn test_ratio_distribute_minimum() {
        let result = ratio_distribute(10, &[1, 1], &[8, 1]);
        assert_eq!(result[0], 8); // first gets at least 8
        assert!(result[1] >= 1);
    }

    #[test]
    fn test_ratio_resolve_fixed() {
        let result = ratio_resolve(100, &[Some(30), None, None], &[1, 2, 1], &[1, 1, 1]);
        assert_eq!(result[0], 30);
        assert!(result[1] > result[2]); // ratio 2:1
    }

    #[test]
    fn test_ratio_reduce() {
        let result = ratio_reduce(50, &[1, 1], &[100, 100], &[60, 40]);
        let sum: usize = result.iter().sum();
        assert!(sum <= 50);
    }
}
