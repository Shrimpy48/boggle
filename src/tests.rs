use crate::*;
use rand::SeedableRng;
use rand_pcg::Pcg32;

fn board_from_u64(seed: u64) -> Board {
    let mut rng = Pcg32::seed_from_u64(seed);
    let dice = [
        [A, B, B, O, O, J],
        [D, E, Y, L, R, V],
        [D, E, X, L, I, R],
        [M, U, Qu, H, I, N],
        [T, E, R, W, H, V],
        [S, S, O, I, E, T],
        [F, F, K, S, A, P],
        [T, T, R, E, L, Y],
        [M, U, O, C, T, I],
        [Z, N, R, N, H, L],
        [O, O, W, T, A, T],
        [P, S, H, A, O, C],
        [E, E, G, N, A, A],
        [T, I, T, S, D, Y],
        [E, E, U, S, N, I],
        [E, E, N, H, W, G],
    ];
    roll(&dice, &mut rng)
}

#[test]
fn test_roll() {
    let board = board_from_u64(7);
    let exp = Board([
        [R, L, T, T],
        [E, F, O, E],
        [M, P, I, T],
        [E, H, V, L]
    ]);
    assert_eq!(board.0, exp.0);
}

#[test]
fn find_words() {
    let board = board_from_u64(7);
    let dict = ["ref", "remep", "world", "pit", "pity", "toe", "vile", "ferler"].into_iter().flat_map(|s| s.parse::<BString>()).collect();
    let words = board.words_trie(&dict);
    assert!(words.contains(&"ref".parse::<BString>().unwrap()));
    assert!(words.contains(&"remep".parse::<BString>().unwrap()));
    assert!(words.contains(&"pit".parse::<BString>().unwrap()));
    assert!(words.contains(&"toe".parse::<BString>().unwrap()));
    assert!(!words.contains(&"pity".parse::<BString>().unwrap()));
    assert!(!words.contains(&"world".parse::<BString>().unwrap()));
    assert!(!words.contains(&"vile".parse::<BString>().unwrap()));
    assert!(!words.contains(&"ferler".parse::<BString>().unwrap()));
    assert!(!words.contains(&"hello".parse::<BString>().unwrap()));
}
