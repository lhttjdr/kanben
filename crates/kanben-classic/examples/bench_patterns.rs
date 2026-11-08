//! 逐條測 點行間 裏各正則的耗時。
use std::time::Instant;

use kanben_classic::jsre::Re;
use kanben_classic::text;

fn main() {
    let src = include_str!("../tests/corpus/00-sample.txt").repeat(4);
    let html = text::to_html(&src, &text::Env::default());
    let (body, _) = text::punctuate_vh_1("grebdriuo", &html);
    let pats = [
        r"([^「『])([「『])(?![「『』」])",
        r"([^「『』」])([』」])(?![』」])",
        r"([，、。．])",
        r"([：；])",
        r"([？！])",
        "』」",
        r"\?\?•(.+?)•¿¿",
        r"\?\?(.+?)¿¿",
        r"\!\!¯([﹆﹅]).+?\1¯¡¡",
        r"¯¯([﹅﹆ロ])",
        "(〪)",
        r"\[【(.+?)】\]",
        "〖(.+?)〗",
        r"\{（(.+?)）\}",
        "《(.+?)》",
        "‹(.+?)›",
        r"\{(.+?)\|(.+?)\}",
        r"([…〻])(?!$)",
    ];
    for p in pats {
        let re = Re::new(p, "g");
        let t0 = Instant::now();
        let mut n = 0;
        for _ in 0..20 {
            n += re.replace(&body, "<x>$1</x>").len();
        }
        println!(
            "{:8.3} ms  {p}  ({n})",
            t0.elapsed().as_secs_f64() * 1000.0 / 20.0
        );
    }
}
