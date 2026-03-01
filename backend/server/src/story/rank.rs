/// Fractional indexing for story rank ordering.
///
/// Ranks are lexicographically-ordered strings using lowercase letters 'a'–'z'.
/// A rank string sorts correctly with standard string comparison operators.
///
/// Initial rank: "n" (near the middle of the alphabet, leaving space on both sides).

use thiserror::Error;

const MIN_CHAR: u8 = b'a';
const MAX_CHAR: u8 = b'z';
const BASE: i64 = 26;

#[derive(Debug, Error, PartialEq)]
pub enum RankError {
    #[error("Cannot insert a rank below the minimum ('a')")]
    BelowMinimum,
    #[error("lo must be lexicographically less than hi")]
    InvalidOrder,
    #[error("Rank computation exceeded maximum recursion depth")]
    Overflow,
}

/// Generate a key that sorts between `lo` and `hi`.
///
/// - `(None, None)` → initial rank "n"
/// - `(Some(l), None)` → key that sorts after `l`
/// - `(None, Some(h))` → key that sorts before `h`
/// - `(Some(l), Some(h))` → key strictly between `l` and `h`
pub fn generate_key_between(lo: Option<&str>, hi: Option<&str>) -> Result<String, RankError> {
    match (lo, hi) {
        (None, None) => Ok("n".to_string()),
        (Some(lo), None) => key_after(lo),
        (None, Some(hi)) => key_before(hi),
        (Some(lo), Some(hi)) => {
            if lo >= hi {
                return Err(RankError::InvalidOrder);
            }
            key_between(lo, hi, 0)
        }
    }
}

fn key_after(lo: &str) -> Result<String, RankError> {
    let bytes: Vec<u8> = lo.bytes().collect();
    // Increment the rightmost character that is not MAX_CHAR ('z')
    if let Some(pos) = bytes.iter().rposition(|&b| b < MAX_CHAR) {
        let mut result = bytes[..pos].to_vec();
        result.push(bytes[pos] + 1);
        Ok(String::from_utf8(result).unwrap())
    } else {
        // All characters are 'z' — append 'n' to produce a longer, larger key
        Ok(format!("{}n", lo))
    }
}

fn key_before(hi: &str) -> Result<String, RankError> {
    if hi == "a" {
        return Err(RankError::BelowMinimum);
    }
    // Generate a key strictly between the implicit floor "a" and hi
    key_between("a", hi, 0)
}

/// Compute a key strictly between `lo` and `hi` (both must be in the valid alphabet).
/// `depth` limits recursion when strings are lexicographically adjacent.
fn key_between(lo: &str, hi: &str, depth: u32) -> Result<String, RankError> {
    if depth > 64 {
        return Err(RankError::Overflow);
    }

    let n = lo.len().max(hi.len());

    // Pad both strings to length `n` with MIN_CHAR ('a' = digit 0)
    let lo_d: Vec<i64> = lo
        .bytes()
        .chain(std::iter::repeat(MIN_CHAR))
        .take(n)
        .map(|b| (b - MIN_CHAR) as i64)
        .collect();
    let hi_d: Vec<i64> = hi
        .bytes()
        .chain(std::iter::repeat(MIN_CHAR))
        .take(n)
        .map(|b| (b - MIN_CHAR) as i64)
        .collect();

    // Compute sum = lo + hi in base-26, big-endian
    let mut sum = vec![0i64; n + 1];
    for i in (0..n).rev() {
        sum[i + 1] += lo_d[i] + hi_d[i];
        if sum[i + 1] >= BASE {
            sum[i] += sum[i + 1] / BASE;
            sum[i + 1] %= BASE;
        }
    }

    // Divide sum by 2 (floor division), propagating remainders
    let mut mid = vec![0i64; n + 1];
    let mut rem: i64 = 0;
    for i in 0..=n {
        let val = sum[i] + rem * BASE;
        mid[i] = val / 2;
        rem = val % 2;
    }

    // Drop the carry slot (index 0) and convert digits back to characters
    let mid_str: String = mid[1..]
        .iter()
        .map(|&d| (MIN_CHAR + d as u8) as char)
        .collect::<String>()
        .trim_end_matches(MIN_CHAR as char)
        .to_string();

    let mid_str = if mid_str.is_empty() {
        (MIN_CHAR as char).to_string()
    } else {
        mid_str
    };

    // Verify mid_str is strictly between lo and hi
    if mid_str.as_str() > lo && mid_str.as_str() < hi {
        Ok(mid_str)
    } else {
        // The two strings are adjacent at this length — extend lo by one digit
        // and retry, which gives more room for a midpoint.
        let lo_ext = format!("{}{}", lo, MIN_CHAR as char);
        key_between(&lo_ext, hi, depth + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_rank_is_n() {
        assert_eq!(generate_key_between(None, None).unwrap(), "n");
    }

    #[test]
    fn append_to_end_increments() {
        let a = "n";
        let b = generate_key_between(Some(a), None).unwrap();
        assert!(b.as_str() > a, "{b:?} must be > {a:?}");
    }

    #[test]
    fn insert_before_first() {
        let hi = "n";
        let before = generate_key_between(None, Some(hi)).unwrap();
        assert!(before.as_str() < hi, "{before:?} must be < {hi:?}");
    }

    #[test]
    fn midpoint_basic() {
        let lo = "a";
        let hi = "z";
        let mid = generate_key_between(Some(lo), Some(hi)).unwrap();
        assert!(mid.as_str() > lo && mid.as_str() < hi);
    }

    #[test]
    fn midpoint_adjacent_single_chars() {
        let lo = "m";
        let hi = "n";
        let mid = generate_key_between(Some(lo), Some(hi)).unwrap();
        assert!(mid.as_str() > lo && mid.as_str() < hi, "mid={mid:?}, lo={lo:?}, hi={hi:?}");
    }

    #[test]
    fn midpoint_adjacent_strings() {
        let lo = "az";
        let hi = "b";
        let mid = generate_key_between(Some(lo), Some(hi)).unwrap();
        assert!(mid.as_str() > lo && mid.as_str() < hi, "mid={mid:?}");
    }

    #[test]
    fn key_after_all_z() {
        let lo = "zzz";
        let result = generate_key_between(Some(lo), None).unwrap();
        assert!(result.as_str() > lo, "{result:?} must be > {lo:?}");
    }

    #[test]
    fn ordering_preserved_across_appends() {
        let mut ranks = vec![generate_key_between(None, None).unwrap()];
        for _ in 0..20 {
            let last = ranks.last().unwrap().clone();
            ranks.push(generate_key_between(Some(&last), None).unwrap());
        }
        let mut sorted = ranks.clone();
        sorted.sort();
        assert_eq!(ranks, sorted, "appended ranks must already be in order");
    }

    #[test]
    fn ordering_preserved_across_prepends() {
        let first = "n".to_string();
        let mut ranks = vec![first.clone()];
        let mut hi = first;
        for _ in 0..20 {
            let new_rank = generate_key_between(None, Some(&hi)).unwrap();
            ranks.insert(0, new_rank.clone());
            hi = new_rank;
        }
        let mut sorted = ranks.clone();
        sorted.sort();
        assert_eq!(ranks, sorted, "prepended ranks must already be in order");
    }

    #[test]
    fn reorder_between_two_stories() {
        let lo = "m";
        let hi = "z";
        let a = generate_key_between(Some(lo), Some(hi)).unwrap();
        let b = generate_key_between(Some(&a), Some(hi)).unwrap();
        let c = generate_key_between(Some(lo), Some(&a)).unwrap();
        assert!(c.as_str() > lo && c < a);
        assert!(a < b && b.as_str() < hi);
    }

    #[test]
    fn invalid_order_errors() {
        let result = generate_key_between(Some("z"), Some("a"));
        assert_eq!(result, Err(RankError::InvalidOrder));
    }

    #[test]
    fn below_minimum_errors() {
        let result = generate_key_between(None, Some("a"));
        assert_eq!(result, Err(RankError::BelowMinimum));
    }
}
