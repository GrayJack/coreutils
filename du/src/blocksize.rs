use self::BlocksizeSuffix::*;
use std::{fmt, slice::Iter};

// Reference:
// https://www.gnu.org/software/coreutils/manual/html_node/Block-size.html#Block-size

#[derive(Debug)]
pub enum BlocksizeError {
    InvalidBlocksize,
    InvalidSuffixError(String),
}

#[derive(Debug, Clone, Copy)]
enum BlocksizeSuffix {
    KB,
    K,
    KiB,
    MB,
    M,
    MiB,
    GB,
    G,
    GiB,
    TB,
    T,
    TiB,
    PB,
    P,
    PiB,
    EB,
    E,
    EiB,
}

impl BlocksizeSuffix {
    fn iter() -> Iter<'static, BlocksizeSuffix> {
        static SUFFIXES: [BlocksizeSuffix; 18] =
            [KB, K, KiB, MB, M, MiB, GB, G, GiB, TB, T, TiB, PB, P, PiB, EB, E, EiB];
        SUFFIXES.iter()
    }

    fn value(self) -> u64 {
        match self {
            KB => 10u64.pow(3),
            K | KiB => 2u64.pow(10),
            MB => 10u64.pow(6),
            M | MiB => 2u64.pow(20),
            GB => 10u64.pow(9),
            G | GiB => 2u64.pow(30),
            TB => 10u64.pow(12),
            T | TiB => 2u64.pow(40),
            PB => 10u64.pow(15),
            P | PiB => 2u64.pow(50),
            EB => 10u64.pow(18),
            E | EiB => 2u64.pow(60),
        }
    }

    fn to_str(self) -> &'static str {
        match self {
            KB => "kB",
            K => "K",
            KiB => "KiB",
            MB => "MB",
            M => "M",
            MiB => "MiB",
            GB => "GB",
            G => "G",
            GiB => "GiB",
            TB => "TB",
            T => "T",
            TiB => "TiB",
            PB => "PB",
            P => "P",
            PiB => "PiB",
            EB => "EB",
            E => "E",
            EiB => "EiB",
        }
    }

    fn largest_from_value(value: u64, use_si: bool) -> Option<BlocksizeSuffix> {
        let mut largest_suffix: Option<BlocksizeSuffix> = None;

        for (count, suffix) in BlocksizeSuffix::iter().enumerate() {
            // take base 10 suffixes
            if use_si && count % 3 == 0 && value / suffix.value() > 1 {
                largest_suffix = Some(*suffix);
            }
            // take single letter base 2 suffixes
            if !use_si && count % 3 == 1 && value / suffix.value() > 1 {
                largest_suffix = Some(*suffix);
            }
        }

        largest_suffix
    }
}

#[derive(Debug)]
pub struct Blocksize {
    value:  u64,
    suffix: Option<BlocksizeSuffix>,
    use_si: bool,
}

impl Blocksize {
    pub fn new() -> Self { Blocksize { value: init_blocksize(), suffix: None, use_si: false } }

    pub fn from_str(size: &str) -> Result<Self, BlocksizeError> {
        let init = Blocksize::new();

        let value = size
            .chars()
            .take_while(|c| c.is_digit(10))
            .map(|c| c.to_digit(10).map(u64::from).unwrap())
            .fold(0u64, |acc, d| acc * 10 + d);

        let suffix = &size.chars().skip_while(|c| c.is_digit(10)).collect::<String>();

        match init.with_suffix(suffix) {
            Ok(blocksize) => {
                if value == 0 {
                    if blocksize.suffix.is_some() {
                        Ok(blocksize.with_value(1))
                    } else {
                        Err(BlocksizeError::InvalidBlocksize)
                    }
                } else {
                    Ok(blocksize.with_value(value))
                }
            },
            Err(err) => {
                if value == 0 {
                    Err(BlocksizeError::InvalidBlocksize)
                } else {
                    Err(err)
                }
            },
        }
    }

    pub fn with_value(mut self, size: u64) -> Self {
        self.value = size;
        self
    }

    pub fn with_suffix(mut self, suffix: &str) -> Result<Self, BlocksizeError> {
        match suffix {
            "KB" | "kB" => self.suffix = Some(KB),
            "K" | "k" => self.suffix = Some(K),
            "KiB" => self.suffix = Some(KiB),
            "MB" => self.suffix = Some(MB),
            "M" => self.suffix = Some(M),
            "MiB" => self.suffix = Some(MiB),
            "GB" => self.suffix = Some(GB),
            "G" => self.suffix = Some(G),
            "GiB" => self.suffix = Some(GiB),
            "TB" => self.suffix = Some(TB),
            "T" => self.suffix = Some(T),
            "TiB" => self.suffix = Some(TiB),
            "PB" => self.suffix = Some(PB),
            "P" => self.suffix = Some(P),
            "PiB" => self.suffix = Some(PiB),
            "EB" => self.suffix = Some(EB),
            "E" => self.suffix = Some(E),
            "EiB" => self.suffix = Some(EiB),
            "" => self.suffix = None,
            _ => return Err(BlocksizeError::InvalidSuffixError(suffix.to_owned())),
        }
        Ok(self)
    }

    pub fn use_si(&mut self) { self.use_si = true; }

    pub fn use_largest_suffix(self) -> Self {
        let total = self.value();
        let suffix = BlocksizeSuffix::largest_from_value(total, self.use_si);

        Blocksize { value: total / suffix.map_or(1, |s| s.value()), suffix, use_si: self.use_si }
    }

    pub fn value(&self) -> u64 { self.value * self.suffix.map_or(1, |s| s.value()) }

    pub fn human_readable(&self) -> String { format!("{}", self) }

    pub fn suffix_str(&self) -> &'static str {
        match self.suffix {
            Some(s) => s.to_str(),
            None => "",
        }
    }
}

impl fmt::Display for Blocksize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.value, self.suffix.map_or("", |s| s.to_str()))
    }
}

fn init_blocksize() -> u64 { 1024 }
