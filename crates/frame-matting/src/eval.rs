//! Matte quality metrics for regression testing across model versions.

use crate::Matte;

/// Mean absolute error between two mattes (0.0 = identical).
pub fn mae(a: &Matte, b: &Matte) -> f32 {
    assert_eq!(a.alpha.len(), b.alpha.len(), "matte size mismatch");
    let sum: f32 = a
        .alpha
        .iter()
        .zip(&b.alpha)
        .map(|(x, y)| (x - y).abs())
        .sum();
    sum / a.alpha.len() as f32
}

/// IoU of the thresholded (>0.5) foreground masks (1.0 = identical support).
pub fn iou(a: &Matte, b: &Matte, threshold: f32) -> f32 {
    assert_eq!(a.alpha.len(), b.alpha.len(), "matte size mismatch");
    let (mut inter, mut union) = (0u32, 0u32);
    for (x, y) in a.alpha.iter().zip(&b.alpha) {
        let ax = *x > threshold;
        let bx = *y > threshold;
        if ax && bx {
            inter += 1;
        }
        if ax || bx {
            union += 1;
        }
    }
    if union == 0 {
        1.0
    } else {
        inter as f32 / union as f32
    }
}

/// Foreground coverage fraction (>threshold), for sanity-checking a matte
/// isn't degenerate (all-background or all-foreground).
pub fn coverage(m: &Matte, threshold: f32) -> f32 {
    m.alpha.iter().filter(|a| **a > threshold).count() as f32 / m.alpha.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matte(alpha: Vec<f32>) -> Matte {
        Matte {
            width: alpha.len() as u32,
            height: 1,
            alpha,
        }
    }

    #[test]
    fn identical_mattes_score_perfectly() {
        let a = matte(vec![0.0, 1.0, 1.0, 0.0]);
        let b = matte(vec![0.0, 1.0, 1.0, 0.0]);
        assert_eq!(mae(&a, &b), 0.0);
        assert_eq!(iou(&a, &b, 0.5), 1.0);
    }

    #[test]
    fn disjoint_mattes_score_zero_iou() {
        let a = matte(vec![1.0, 1.0, 0.0, 0.0]);
        let b = matte(vec![0.0, 0.0, 1.0, 1.0]);
        assert_eq!(iou(&a, &b, 0.5), 0.0);
        assert!((mae(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn partial_overlap_iou() {
        // fg at indices {0,1,2} vs {1,2,3}: intersection 2, union 4.
        let a = matte(vec![1.0, 1.0, 1.0, 0.0]);
        let b = matte(vec![0.0, 1.0, 1.0, 1.0]);
        assert!((iou(&a, &b, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn coverage_counts_foreground_fraction() {
        let m = matte(vec![1.0, 1.0, 0.0, 0.0]);
        assert!((coverage(&m, 0.5) - 0.5).abs() < 1e-6);
    }
}
