/// Mix two PCM i16 sample buffers together with clamping.
/// If buffers are different lengths, the shorter one is padded with silence (0).
pub fn mix_samples(a: &[i16], b: &[i16]) -> Vec<i16> {
    let len = a.len().max(b.len());
    (0..len)
        .map(|i| {
            let sa = *a.get(i).unwrap_or(&0) as i32;
            let sb = *b.get(i).unwrap_or(&0) as i32;
            (sa + sb).clamp(i16::MIN as i32, i16::MAX as i32) as i16
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mix_two_streams() {
        let a: Vec<i16> = vec![1000, 2000, 3000];
        let b: Vec<i16> = vec![500, 1000, 1500];
        let mixed = mix_samples(&a, &b);
        assert_eq!(mixed, vec![1500, 3000, 4500]);
    }

    #[test]
    fn test_mix_clamps_to_i16_max() {
        let a: Vec<i16> = vec![i16::MAX];
        let b: Vec<i16> = vec![1000];
        let mixed = mix_samples(&a, &b);
        assert_eq!(mixed, vec![i16::MAX]);
    }

    #[test]
    fn test_mix_clamps_to_i16_min() {
        let a: Vec<i16> = vec![i16::MIN];
        let b: Vec<i16> = vec![-1000];
        let mixed = mix_samples(&a, &b);
        assert_eq!(mixed, vec![i16::MIN]);
    }

    #[test]
    fn test_mix_different_lengths() {
        let a: Vec<i16> = vec![1000, 2000, 3000];
        let b: Vec<i16> = vec![500];
        let mixed = mix_samples(&a, &b);
        assert_eq!(mixed, vec![1500, 2000, 3000]);
    }

    #[test]
    fn test_mix_empty() {
        let a: Vec<i16> = vec![];
        let b: Vec<i16> = vec![];
        let mixed = mix_samples(&a, &b);
        let expected: Vec<i16> = vec![];
        assert_eq!(mixed, expected);
    }
}
