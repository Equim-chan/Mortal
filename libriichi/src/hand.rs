//! Hand format conversions, usually only useful for testing and debugging.
//!
//! Note that all functions in this mod that take or produce strings are dealing
//! with tenhou.net/2 format tile description (like 0m 123z) instead of mjai (like
//! 5mr ESW).

use crate::tile::Tile;
use crate::vec_ops::vec_add_assign;
use crate::{must_tile, tuz};

use anyhow::{Result, bail, ensure};

/// Spaces are allowed.
pub fn hand_with_aka(s: &str) -> Result<[u8; 37]> {
    // We will be using bytes instead of chars afterwards.
    ensure!(s.is_ascii(), "hand {s} contains non-ascii content");

    let mut ret = [0; 37];
    let mut stack = vec![];

    for b in s.as_bytes() {
        match b {
            b'0'..=b'9' => stack.push((b - b'0') as usize),
            b'm' | b'p' | b's' | b'z' => {
                for t in stack.drain(..) {
                    let idx = if t == 0 {
                        match b {
                            b'm' => tuz!(5mr),
                            b'p' => tuz!(5pr),
                            b's' => tuz!(5sr),
                            _ => bail!("unexpected byte {b}"),
                        }
                    } else {
                        let kind = match b {
                            b'm' => 0,
                            b'p' => 1,
                            b's' => 2,
                            b'z' => 3,
                            _ => unreachable!(),
                        };
                        kind * 9 + t - 1
                    };
                    ret[idx] += 1;
                }
            }
            b' ' | b'\t' | b'\n' => (),
            _ => bail!("unexpected byte {b}"),
        };
    }

    Ok(ret)
}

/// Spaces are allowed.
pub fn hand(s: &str) -> Result<[u8; 34]> {
    let mut ret = [0; 34];
    let hand = hand_with_aka(s)?;
    vec_add_assign(&mut ret, &hand);
    ret[tuz!(5m)] += hand[tuz!(5mr)];
    ret[tuz!(5p)] += hand[tuz!(5pr)];
    ret[tuz!(5s)] += hand[tuz!(5sr)];

    Ok(ret)
}

#[must_use]
pub fn tile37_to_vec(tiles: &[u8; 37]) -> Vec<Tile> {
    let mut ret = vec![];
    tiles
        .iter()
        .enumerate()
        .filter(|&(_, &count)| count > 0)
        .for_each(|(tid, &count)| {
            if tid < 34 {
                ret.resize(ret.len() + count as usize, must_tile!(tid));
            } else {
                ret.push(must_tile!(tid));
            }
        });
    ret
}

#[must_use]
pub fn tile34_to_vec(tiles: &[u8; 34]) -> Vec<Tile> {
    let mut ret = vec![];
    tiles
        .iter()
        .enumerate()
        .filter(|&(_, &count)| count > 0)
        .for_each(|(tid, &count)| {
            ret.resize(ret.len() + count as usize, must_tile!(tid));
        });
    ret
}

#[must_use]
pub fn tiles_to_string(tiles: &[u8; 34], aka: [bool; 3]) -> String {
    let suhai = tiles[..3 * 9]
        .chunks_exact(9)
        .enumerate()
        .map(|(kind, chunk)| {
            let mut partial = String::new();
            let mut not_empty = false;
            chunk
                .iter()
                .enumerate()
                .filter(|&(_, &count)| count > 0)
                .for_each(|(num, &count)| {
                    let literal_num = num + 1;
                    if literal_num == 5 && aka[kind] {
                        partial.push('0');
                        partial += &literal_num.to_string().repeat(count as usize - 1);
                    } else {
                        partial += &literal_num.to_string().repeat(count as usize);
                    }
                    not_empty = true;
                });

            if not_empty {
                let c = match kind {
                    0 => 'm',
                    1 => 'p',
                    2 => 's',
                    _ => unreachable!(),
                };
                partial.push(c);
            }
            partial
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    let jihai: String = tiles[3 * 9..]
        .iter()
        .enumerate()
        .filter(|&(_, &count)| count > 0)
        .map(|(num, &count)| (num + 1).to_string().repeat(count as usize))
        .collect();

    if jihai.is_empty() {
        suhai
    } else {
        format!("{suhai} {jihai}z")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        assert_eq!(
            hand("1111m 333p 222s 444z").unwrap(),
            [
                4, 0, 0, 0, 0, 0, 0, 0, 0, // m
                0, 0, 3, 0, 0, 0, 0, 0, 0, // p
                0, 3, 0, 0, 0, 0, 0, 0, 0, // s
                0, 0, 0, 3, 0, 0, 0, // z
            ]
        );

        assert_eq!(
            hand_with_aka("22334450m234p2s3s4s").unwrap(),
            [
                0, 2, 2, 2, 1, 0, 0, 0, 0, // m
                0, 1, 1, 1, 0, 0, 0, 0, 0, // p
                0, 1, 1, 1, 0, 0, 0, 0, 0, // s
                0, 0, 0, 0, 0, 0, 0, // z
                1, 0, 0, // a
            ]
        );

        assert_eq!(
            hand("456m 6p 7899p 77z 987s 9p").unwrap(),
            [
                0, 0, 0, 1, 1, 1, 0, 0, 0, // m
                0, 0, 0, 0, 0, 1, 1, 1, 3, // p
                0, 0, 0, 0, 0, 0, 1, 1, 1, // s
                0, 0, 0, 0, 0, 0, 2, // z
            ]
        );
    }

    #[test]
    fn string() {
        assert_eq!(
            tiles_to_string(
                &[
                    0, 0, 2, 0, 1, 1, 1, 0, 0, // m
                    0, 0, 1, 1, 1, 1, 1, 1, 0, // p
                    0, 0, 0, 0, 0, 1, 1, 1, 0, // s
                    0, 0, 0, 0, 0, 0, 0, // z
                ],
                [true, false, false]
            ),
            "33067m 345678p 678s"
        );
    }
}
