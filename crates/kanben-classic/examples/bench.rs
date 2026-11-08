//! 粗測各步耗時：cargo run --release -p kanben-classic --example bench
use std::time::Instant;

use kanben_classic::{marks, text};

fn main() {
    let src = include_str!("../tests/corpus/00-sample.txt").repeat(4);
    let env = text::Env::default();
    let t = |name: &str, f: &dyn Fn() -> String| {
        let t0 = Instant::now();
        let mut n = 0;
        for _ in 0..20 {
            n += f().len();
        }
        println!(
            "{name:8} {:7.2} ms  ({n})",
            t0.elapsed().as_secs_f64() * 1000.0 / 20.0
        );
    };
    let html = text::to_html(&src, &env);
    let (body, _) = text::punctuate_vh_1("grebdriuo", &html);
    t("toHTML", &|| text::to_html(&src, &env));
    t("點行間", &|| marks::點行間(&body));
    t("圈點抹", &|| marks::圈點抹(&body));
    t("陰文墨圍", &|| marks::陰文和墨圍(&body));
}
