use boggle::file::stdio::*;
use boggle::*;
use rand::thread_rng;
use std::io;
use std::io::prelude::*;

fn main() -> Result<(), Error> {
    let dict = read_dict("dictionaries/custom.txt")?;
    let dice = read_dice("dice.txt")?;
    // let dice = [
    //     [A, B, B, O, O, J],
    //     [D, E, Y, L, R, V],
    //     [D, E, X, L, I, R],
    //     [M, U, Qu, H, I, N],
    //     [T, E, R, W, H, V],
    //     [S, S, O, I, E, T],
    //     [F, F, K, S, A, P],
    //     [T, T, R, E, L, Y],
    //     [M, U, O, C, T, I],
    //     [Z, N, R, N, H, L],
    //     [O, O, W, T, A, T],
    //     [P, S, H, A, O, C],
    //     [E, E, G, N, A, A],
    //     [T, I, T, S, D, Y],
    //     [E, E, U, S, N, I],
    //     [E, E, N, H, W, G],
    // ];

    cli(&dict, &dice)?;

    write_dict("dictionaries/custom.txt", &dict)
}

fn cli(dict: &Dict, dice: &Dice) -> io::Result<()> {
    let mut rng = thread_rng();
    loop {
        print!("Play? (Y/n) ");
        io::stdout().flush()?;
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        match buf.trim() {
            "" | "y" | "Y" => {}
            _ => break,
        }
        let board = roll(&dice, &mut rng);
        print!("{}", board);
        let mut input_lines = Vec::new();
        let mut correct = Vec::new();
        let mut not_present = Vec::new();
        let mut too_short = Vec::new();
        let mut not_word = Vec::new();
        let mut not_bword = Vec::new();
        loop {
            buf.clear();
            io::stdin().read_line(&mut buf)?;
            let trimmed = buf.trim();
            if trimmed.is_empty() {
                break;
            }
            input_lines.push(String::from(trimmed));
        }
        let mut present = board.words_trie(&dict);
        for (sword, word_res) in input_lines.iter().map(|w| (w, w.parse::<BString>())) {
            match word_res {
                Ok(bword) => {
                    if present.contains(&bword) {
                        correct.push(sword);
                        present.remove(&bword);
                    } else if dict.contains(&bword) {
                        not_present.push(sword);
                    } else if sword.len() < 3 {
                        too_short.push(sword);
                    } else {
                        not_word.push(sword);
                    }
                }
                Err(_) => {
                    not_bword.push(sword);
                }
            }
        }
        let mut sum = 0;
        if !correct.is_empty() {
            println!("Correct:");
            for word in correct.iter() {
                let score = score(word);
                println!("{}: {}", word, score);
                sum += score;
            }
        }
        if !not_present.is_empty() {
            println!("Repeated or not on the board:");
            for word in not_present.iter() {
                println!("{}", word);
            }
        }
        if !too_short.is_empty() {
            println!("Too short:");
            for word in too_short.iter() {
                println!("{}", word);
            }
        }
        if !not_word.is_empty() {
            println!("Not in the dictionary:");
            for word in not_word.iter() {
                println!("{}", word);
            }
        }
        if !not_bword.is_empty() {
            println!("Not possible in Boggle:");
            for word in not_bword.iter() {
                println!("{}", word);
            }
        }
        println!("{}", "-".repeat(80));
        println!("Score: {}", sum);
        println!("{}", "-".repeat(80));
        let mut other_words: Vec<String> =
            present.words().into_iter().map(|w| w.to_string()).collect();
        other_words.sort_unstable_by_key(|w| -(w.len() as i8));
        if !other_words.is_empty() {
            println!("Some other words on the board:");
            for word in other_words.into_iter().take(16) {
                println!("{}: {}", word, score(&word));
            }
        }
    }
    Ok(())
}
