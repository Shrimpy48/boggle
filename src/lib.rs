pub mod io;
#[cfg(feature = "serde")]
mod serde;
#[cfg(test)]
mod tests;

use enum_map::{Enum, EnumMap};
use rand::prelude::*;
use std::error;
use std::fmt;
use std::iter::FromIterator;
use std::mem;
use std::hash;
use std::ops::Deref;
use std::borrow::Borrow;
use std::ops::{Index, IndexMut};
use std::str::FromStr;
#[cfg(feature = "serde")]
use ::serde::{Serialize, Deserialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Enum, Hash)]
pub enum BChar {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Qu,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
}
use BChar::*;

impl Default for BChar {
    fn default() -> Self {
        A
    }
}

impl fmt::Display for BChar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            A => "A",
            B => "B",
            C => "C",
            D => "D",
            E => "E",
            F => "F",
            G => "G",
            H => "H",
            I => "I",
            J => "J",
            K => "K",
            L => "L",
            M => "M",
            N => "N",
            O => "O",
            P => "P",
            Qu => "Qu",
            R => "R",
            S => "S",
            T => "T",
            U => "U",
            V => "V",
            W => "W",
            X => "X",
            Y => "Y",
            Z => "Z",
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct BString(Vec<BChar>);

impl BString {
    pub fn to_vec(self) -> Vec<BChar> {
        self.0
    }

    pub fn push(&mut self, ch: BChar) {
        self.0.push(ch)
    }

    pub fn pop(&mut self) -> Option<BChar> {
        self.0.pop()
    }
}

impl FromStr for BString {
    type Err = ParseBoggleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut out = Vec::new();
        let mut chars = s.chars().map(|c| c.to_ascii_uppercase());
        while let Some(c) = chars.next() {
            match c {
                'A' => out.push(A),
                'B' => out.push(B),
                'C' => out.push(C),
                'D' => out.push(D),
                'E' => out.push(E),
                'F' => out.push(F),
                'G' => out.push(G),
                'H' => out.push(H),
                'I' => out.push(I),
                'J' => out.push(J),
                'K' => out.push(K),
                'L' => out.push(L),
                'M' => out.push(M),
                'N' => out.push(N),
                'O' => out.push(O),
                'P' => out.push(P),
                'Q' => match chars.next() {
                    Some('U') => out.push(Qu),
                    _ => return Err(ParseBoggleError),
                },
                'R' => out.push(R),
                'S' => out.push(S),
                'T' => out.push(T),
                'U' => out.push(U),
                'V' => out.push(V),
                'W' => out.push(W),
                'X' => out.push(X),
                'Y' => out.push(Y),
                'Z' => out.push(Z),
                _ => return Err(ParseBoggleError),
            }
        }
        Ok(BString(out))
    }
}

impl hash::Hash for BString {
    #[inline]
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        (**self).hash(hasher)
    }
}

#[derive(Debug)]
pub struct ParseBoggleError;

impl fmt::Display for ParseBoggleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid character found in string")
    }
}

impl error::Error for ParseBoggleError {}

#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct BStr([BChar]);

impl BStr {
    pub fn split_first(&self) -> Option<(&BChar, &Self)> {
        self.0
            .split_first()
            .map(|(c, rem)| (c, BStr::from_slice(rem)))
    }

    pub fn from_slice(v: &[BChar]) -> &BStr {
        // SAFETY: BStr is a "newtype" of [BChar] with repr(transparent)
        unsafe { mem::transmute(v) }
    }
}

impl fmt::Display for BStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(
            &self
                .0
                .iter()
                .map(|bc| match bc {
                    A => "a",
                    B => "b",
                    C => "c",
                    D => "d",
                    E => "e",
                    F => "f",
                    G => "g",
                    H => "h",
                    I => "i",
                    J => "j",
                    K => "k",
                    L => "l",
                    M => "m",
                    N => "n",
                    O => "o",
                    P => "p",
                    Qu => "qu",
                    R => "r",
                    S => "s",
                    T => "t",
                    U => "u",
                    V => "v",
                    W => "w",
                    X => "x",
                    Y => "y",
                    Z => "z",
                })
                .collect::<Vec<&str>>()
                .concat(),
        )
    }
}

impl fmt::Display for BString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl Deref for BString {
    type Target = BStr;

    fn deref(&self) -> &BStr {
        BStr::from_slice(&self.0)
    }
}

impl Borrow<BStr> for BString {
    fn borrow(&self) -> &BStr {
        BStr::from_slice(&self.0)
    }
}

impl ToOwned for BStr {
    type Owned = BString;

    fn to_owned(&self) -> Self::Owned {
        BString(self.0.to_owned())
    }
}

#[derive(Default, PartialEq, Eq, Clone)]
#[repr(transparent)]
struct DictChildren(EnumMap<BChar, Option<Box<Dict>>>);

impl DictChildren {
    fn values(&self) -> enum_map::Values<Option<Box<Dict>>> {
        self.0.values()
    }

    fn values_mut(&mut self) -> enum_map::ValuesMut<Option<Box<Dict>>> {
        self.0.values_mut()
    }

    fn iter(&self) -> enum_map::Iter<BChar, Option<Box<Dict>>> {
        self.0.iter()
    }
}

impl Index<BChar> for DictChildren {
    type Output = <EnumMap<BChar, Option<Box<Dict>>> as Index<BChar>>::Output;

    fn index(&self, index: BChar) -> &Self::Output {
        self.0.index(index)
    }
}

impl IndexMut<BChar> for DictChildren {
    fn index_mut(&mut self, index: BChar) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl fmt::Debug for DictChildren {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map()
            .entries(
                self.iter()
                    .filter_map(|(k, ov)| ov.as_ref().map(|v| (k, v))),
            )
            .finish()
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Dict {
    val: bool,
    children: DictChildren,
}

impl Dict {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert(&mut self, word: &BStr) {
        match word.split_first() {
            None => {
                self.val = true;
            }
            Some((&c, rem)) => {
                self.children[c]
                    .get_or_insert_with(Default::default)
                    .insert(rem);
            }
        }
    }

    pub fn remove(&mut self, word: &BStr) {
        match word.split_first() {
            None => {
                self.val = false;
            }
            Some((&c, rem)) => match &mut self.children[c] {
                None => {}
                Some(dict) => {
                    dict.remove(rem);
                    if dict.is_empty() {
                        self.children[c] = None;
                    }
                }
            },
        }
    }

    pub fn is_empty(&self) -> bool {
        !self.val && self.children.values().all(Option::is_none)
    }

    pub fn contains(&self, word: &BStr) -> bool {
        match word.split_first() {
            None => self.val,
            Some((&c, rem)) => match &self.children[c] {
                None => false,
                Some(dict) => dict.contains(rem),
            },
        }
    }

    pub fn words(&self) -> Vec<BString> {
        let mut out = Vec::new();
        self.traverse(|w| out.push(w.to_owned()));
        out
    }

    pub fn traverse<F>(&self, mut f: F)
    where
        F: FnMut(&BStr),
    {
        let mut current_str = BString::default();
        self.traverse_impl(&mut current_str, &mut f);
    }

    fn traverse_impl<F>(&self, current_str: &mut BString, f: &mut F)
    where
        F: FnMut(&BStr),
    {
        if self.val {
            f(&current_str);
        }
        for (ch, v) in self.children.iter() {
            if let Some(d) = v {
                current_str.push(ch);
                d.traverse_impl(current_str, f);
                current_str.pop();
            }
        }
    }

    pub fn try_traverse<F, E>(&self, mut f: F) -> Result<(), E>
    where
        F: FnMut(&BStr) -> Result<(), E>,
    {
        let mut current_str = BString::default();
        self.try_traverse_impl(&mut current_str, &mut f)
    }

    fn try_traverse_impl<F, E>(&self, current_str: &mut BString, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&BStr) -> Result<(), E>,
    {
        if self.val {
            f(&current_str)?;
        }
        for (ch, v) in self.children.iter() {
            if let Some(d) = v {
                current_str.push(ch);
                d.try_traverse_impl(current_str, f)?;
                current_str.pop();
            }
        }
        Ok(())
    }

    fn prune(&mut self) {
        for v in self.children.values_mut() {
            if let Some(d) = v {
                d.prune();
                if d.is_empty() {
                    *v = None;
                }
            }
        }
    }

    // pub fn iter(&self) -> impl Iterator<Item = BString> + '_ {
    //     self.iter_impl().map(|w| BString(Vec::from(w)))
    // }

    // fn iter_impl(&self) -> Box<dyn Iterator<Item = VecDeque<BChar>> + '_> {
    //     Box::new(
    //         self.val.then(Default::default).into_iter().chain(
    //             self.children
    //                 .iter()
    //                 .filter_map(|(c, v)| Some(c).zip(v.as_ref()))
    //                 .flat_map(|(c, v)| {
    //                     v.iter_impl().map(move |mut w| {
    //                         w.push_front(c);
    //                         w
    //                     })
    //                 }),
    //         ),
    //     )
    // }

    // fn into_iter_impl(self) -> Box<dyn Iterator<Item = VecDeque<BChar>>> {
    //     Box::new(
    //         self.val.then(Default::default).into_iter().chain(
    //             self.children
    //                 .into_iter()
    //                 .filter_map(|(c, v)| Some(c).zip(v))
    //                 .flat_map(|(c, v)| {
    //                     v.into_iter_impl().map(move |mut w| {
    //                         w.push_front(c);
    //                         w
    //                     })
    //                 }),
    //         ),
    //     )
    // }
}

// impl IntoIterator for Dict {
//     type Item = BString;
//     type IntoIter = Box<dyn Iterator<Item = BString>>;
//     fn into_iter(self) -> Self::IntoIter {
//         Box::new(self.into_iter_impl().map(|w| BString(Vec::from(w))))
//     }
// }
//
impl<'a> FromIterator<&'a BStr> for Dict {
    fn from_iter<I: IntoIterator<Item = &'a BStr>>(iter: I) -> Self {
        let mut dict = Dict::default();
        for word in iter {
            dict.insert(word);
        }
        dict
    }
}

impl<'a> Extend<&'a BStr> for Dict {
    fn extend<T: IntoIterator<Item = &'a BStr>>(&mut self, iter: T) {
        for word in iter {
            self.insert(word);
        }
    }
}

impl FromIterator<BString> for Dict {
    fn from_iter<I: IntoIterator<Item = BString>>(iter: I) -> Self {
        let mut dict = Dict::default();
        for word in iter {
            dict.insert(&word);
        }
        dict
    }
}

impl Extend<BString> for Dict {
    fn extend<T: IntoIterator<Item = BString>>(&mut self, iter: T) {
        for word in iter {
            self.insert(&word);
        }
    }
}

pub type Dice = [[BChar; 6]; 16];

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Default)]
#[repr(transparent)]
pub struct Board([[BChar; 4]; 4]);

impl Board {
    // pub fn words_vec(&self, dict: &Dict) -> Vec<BString> {
    //     let mut visited = [[false; 4]; 4];
    //     let mut out = Vec::new();
    //     let mut current_str = BString::default();
    //     if dict.val {
    //         out.push(current_str.clone());
    //     }
    //     for r in 0..4 {
    //         for c in 0..4 {
    //             self.visit_pos_vec(r, c, &mut current_str, &mut visited, dict, &mut out);
    //         }
    //     }
    //     out
    // }

    // fn visit_pos_vec(
    //     &self,
    //     r: usize,
    //     c: usize,
    //     current_str: &mut BString,
    //     visited: &mut [[bool; 4]; 4],
    //     dict: &Dict,
    //     out: &mut Vec<BString>,
    // ) {
    //     let char_at_pos = self.0[r][c];
    //     if let Some(d) = &dict.children[char_at_pos] {
    //         visited[r][c] = true;
    //         current_str.push(char_at_pos);
    //         if d.val {
    //             out.push(current_str.clone());
    //         }
    //         for (row, col) in neighbours((r, c)) {
    //             if visited[row][col] {
    //                 continue;
    //             }
    //             self.visit_pos_vec(row, col, current_str, visited, &d, out);
    //         }
    //         visited[r][c] = false;
    //         current_str.pop();
    //     }
    // }

    pub fn words_trie(&self, dict: &Dict) -> Dict {
        let mut visited = [[false; 4]; 4];
        // let mut out = Dict::default();
        // out.val = dict.val;
        let mut out = Dict {
            val: dict.val,
            ..Default::default()
        };
        for r in 0..4 {
            for c in 0..4 {
                let char_at_pos = self.0[r][c];
                if let Some(d) = &dict.children[char_at_pos] {
                    self.visit_pos_trie(
                        r,
                        c,
                        &mut visited,
                        d,
                        out.children[char_at_pos].get_or_insert_with(Default::default),
                    );
                }
            }
        }
        out.prune();
        out
    }

    fn visit_pos_trie(
        &self,
        row: usize,
        col: usize,
        visited: &mut [[bool; 4]; 4],
        dict: &Dict,
        out: &mut Dict,
    ) {
        visited[row][col] = true;
        if dict.val {
            out.val = true;
        }
        for (r, c) in neighbours((row, col)) {
            if visited[r][c] {
                continue;
            }
            let char_at_pos = self.0[r][c];
            if let Some(d) = &dict.children[char_at_pos] {
                self.visit_pos_trie(
                    r,
                    c,
                    visited,
                    d,
                    out.children[char_at_pos].get_or_insert_with(Default::default),
                );
            }
        }
        visited[row][col] = false;
    }

    pub fn contains(&self, word: &BStr) -> bool {
        let mut dict = Dict::new();
        dict.insert(word);
        return !self.words_trie(&dict).is_empty()
    }
}

fn neighbours(p: (usize, usize)) -> impl Iterator<Item = (usize, usize)> {
    let (r, c) = (p.0 as i8, p.1 as i8);
    IntoIterator::into_iter([
        (r - 1, c - 1),
        (r - 1, c),
        (r - 1, c + 1),
        (r, c - 1),
        (r, c + 1),
        (r + 1, c - 1),
        (r + 1, c),
        (r + 1, c + 1),
    ])
    .filter_map(|(row, col)| {
        if row < 0 || col < 0 || row >= 4 || col >= 4 {
            None
        } else {
            Some((row as usize, col as usize))
        }
    })
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in self.0 {
            for c in line {
                write!(f, "{:3}", c)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Index<usize> for Board {
    type Output = [BChar; 4];
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

pub fn roll<R: Rng + ?Sized>(dice: &Dice, rng: &mut R) -> Board {
    let mut board = <[[BChar; 4]; 4]>::default();
    for (i, die) in dice.choose_multiple(rng, dice.len()).enumerate() {
        board[i / 4][i % 4] = *die.choose(rng).unwrap();
    }
    Board(board)
}

pub fn score(word: &str) -> u8 {
    match word.len() {
        0 | 1 | 2 => 0,
        3 | 4 => 1,
        5 => 2,
        6 => 3,
        7 => 5,
        _ => 11,
    }
}
