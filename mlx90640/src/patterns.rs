const fn gen_patterns() -> ([i8; 768], [i8; 768], [i8; 768]) {
    let mut il = [0i8; 768];
    let mut chess = [0i8; 768];
    let mut conv = [0i8; 768];
    let mut i = 0;
    while i < 768 {
        let p = i as i32;
        let il_pat = p / 32 - (p / 64) * 2;
        let chess_pat = il_pat ^ (p - (p / 2) * 2);
        let conv_pat = ((p + 2) / 4 - (p + 3) / 4 + (p + 1) / 4 - p / 4) * (1 - 2 * il_pat);
        il[i] = il_pat as i8;
        chess[i] = chess_pat as i8;
        conv[i] = conv_pat as i8;
        i += 1;
    }
    (il, chess, conv)
}

const PATTERNS: ([i8; 768], [i8; 768], [i8; 768]) = gen_patterns();

pub(crate) const IL_PATTERN: [i8; 768] = PATTERNS.0;
pub(crate) const CHESS_PATTERN: [i8; 768] = PATTERNS.1;
pub(crate) const CONVERSION_PATTERN: [i8; 768] = PATTERNS.2;
