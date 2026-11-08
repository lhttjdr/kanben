//! 原版給已成 HTML 的正文、夾注加標點樣式、圈點、陰文墨圍等（點行間、點字下 …）。

use crate::re;
use crate::text::{Env, num};

/// `陰文和墨圍`
pub fn 陰文和墨圍(x: &str) -> String {
    let mut x = re!(r"\[【(.+?)】\]", "g").replace(
        x,
        r#"<i class="hideCopy">[</i><cite class="imVang">$1</cite><i class="hideCopy">]</i>"#,
    );
    x = re!(r"\(【(.+?)】\)", "g").replace(
        &x,
        r#"<i class="hideCopy">(</i><cite class="imHiuan">$1</cite><i class="hideCopy">)</i>"#,
    );
    x = re!(r"〔【(.+?)】〕", "g").replace(
        &x,
        r#"<i class="hideCopy">〔</i><cite class="im6grug">$1</cite><i class="hideCopy">〕</i>"#,
    );
    x = re!(r"\!\[⬣(.+?)⬣\]¡", "g").replace(&x, r#"<cite class="imLantn">$1</cite>"#);
    x = re!(r"\!\[●(.+?)●\]¡", "g").replace(&x, r#"<cite class="imCirc">$1</cite>"#);
    x = re!(r"\[〖(.+?)〗\]", "g").replace(
        &x,
        r#"<i class="hideCopy">[</i><cite class="mhVang">$1</cite><i class="hideCopy">]</i>"#,
    );
    x = re!(r"\(〖(.+?)〗\)", "g").replace(
        &x,
        r#"<i class="hideCopy">(</i><cite class="mhHiuan">$1</cite><i class="hideCopy">)</i>"#,
    );
    x = re!(r"〔〖(.+?)〗〕", "g").replace(
        &x,
        r#"<i class="hideCopy">〔</i><cite class="mh6grug">$1</cite><i class="hideCopy">〕</i>"#,
    );
    x = re!(r"\!\[⎔(.+?)⎔\]¡", "g").replace(&x, r#"<cite class="mhLantn">$1</cite>"#);
    x = re!(r"\!\[○(.+?)○\]¡", "g").replace(&x, r#"<cite class="mhCirc">$1</cite>"#);
    x = re!(r"﹙〖(.+?)〗﹚", "g").replace(
        &x,
        r#"<i class="hideCopy">﹙</i><cite class="mhHiuanNR">$1</cite><i class="hideCopy">﹚</i>"#,
    );
    re!(r"﹝〖(.+?)〗﹞", "g").replace(
        &x,
        r#"<i class="hideCopy">﹝</i><cite class="mh8grugNR"><span>$1</span></cite><i class="hideCopy">﹞</i>"#,
    )
}

/// `圈點抹`
pub fn 圈點抹(x: &str) -> String {
    let mut x = re!(r"\?\?•(.+?)•¿¿", "g").replace(x, r#"<mark class="colateOF">$1</mark>"#);
    x = re!(r"\?\?◦(.+?)◦¿¿", "g").replace(&x, r#"<mark class="colateOO">$1</mark>"#);
    x = re!(r"\?\?\-(.+?)\-¿¿", "g").replace(&x, r#"<mark class="colateMS">$1</mark>"#);
    x = re!(r"\?\?(.+?)¿¿", "g").replace(&x, r#"<mark class="colateTC">$1</mark>"#);
    x = re!(r"\!\!¯•(.+?)•¯¡¡", "g").replace(&x, r#"<strong class="strROF">$1</strong>"#);
    x = re!(r"\!\!¯◦(.+?)◦¯¡¡", "g").replace(&x, r#"<strong class="strROO">$1</strong>"#);
    旁點(&x)
}

/// `圈點抹` 裏的 `旁點`：逐字後綴 `¯¯﹆`，標籤裏的不動。
fn 旁點(txt: &str) -> String {
    let sentens = re!(r"\!\!¯([﹆﹅]).+?\1¯¡¡", "g").captures_all(txt);
    if sentens.is_empty() {
        return txt.to_string();
    }
    let mut txt = re!(r"\!\!¯([﹆﹅]).+?\1¯¡¡", "g").replace(txt, "｛$1｝");
    for s in sentens {
        let mark = s[1].clone().unwrap_or_default();
        let mut s0 = s[0].clone().unwrap_or_default();
        s0 = re!(r"([〇㐀-鿿])", "g").replace(&s0, &format!("$1¯¯{mark}"));
        s0 = re!(r"([\u{20000}-\u{2A6DF}\u{2A700}-\u{2CEAF}])", "gu")
            .replace(&s0, &format!("$1¯¯{mark}"));
        if let Some(tgs) = re!(r"<[^\/].+?>", "g").match_all(&s0) {
            s0 = re!(r"<[^\/].+?>", "g").replace(&s0, "｛〈〉｝");
            for t in tgs {
                let t = re!(r"([〇㐀-鿿])¯¯[﹆﹅]", "g").replace(&t, "$1");
                s0 = re!("｛〈〉｝").replace(&s0, &t);
            }
        }
        // 原版 slice(4,-4)：去掉頭尾各 4 個 UTF-16 單元（`!!¯﹆` 與 `﹆¯¡¡`，都在 BMP）。
        let chars: Vec<char> = s0.chars().collect();
        let inner: String = chars[4..chars.len().saturating_sub(4).max(4)]
            .iter()
            .collect();
        txt = re!(r"｛[﹆﹅]｝").replace(&txt, &inner);
    }
    txt
}

/// `點行間`：正文 HTML 加標點樣式等。`clr` 原版不用（只在註釋掉的代碼裏）。
pub fn 點行間(x: &str) -> String {
    let mut x = re!(r"([^「『])([「『])(?![「『』」])", "g")
        .replace(x, r#"$1<span class="hgbdSysQt1">$2</span>"#);
    x = re!(r"([^「『』」])([』」])(?![』」])", "g")
        .replace(&x, r#"$1<span class="hgbdSysQt2">$2</span>"#);
    x = re!(r"([，、。．])", "g").replace(
        &x,
        r#"<span class="jmhgbd hgbdSysSm" lang="zh-Hans-CN">$1</span>"#,
    );
    x = re!(r"([：；])", "g").replace(
        &x,
        r#"<span class="jmhgbd hgbdSysMd" lang="zh-Hans-CN">$1</span>"#,
    );
    x = re!(r"([？！])", "g").replace(
        &x,
        r#"<span class="jmhgbd hgbdSysTl" lang="zh-Hans-CN">$1</span>"#,
    );
    x = re!("』」", "g").replace(
        &x,
        r#"<span class="hgbd2Qt2"><span>』</span><span>」</span></span>"#,
    );
    x = re!("「『", "g").replace(&x, r#"<span class="hgbd2Qt1"><span>「</span>『</span>"#);
    x = re!("）（", "g").replace(&x, r#"<span style="letter-spacing: -1em">）</span>（"#);
    x = 圈點抹(&x);
    x = re!(r"¯¯([﹅﹆ロ])", "g").replace(
        &x,
        r#"<span class="jmhgbd hgbdRightEm" lang="zh-Hans-CN">$1</span>"#,
    );
    x = re!("(〪)", "g").replace(&x, r#"<span class="sigaugxieng fkLevel lrtb">$1</span>"#);
    x = re!("(〫)", "g").replace(&x, r#"<span class="sigaugxieng fkRise lrtb">$1</span>"#);
    x = re!("(〬)", "g").replace(&x, r#"<span class="sigaugxieng fkDept">$1</span>"#);
    x = re!("(〭)", "g").replace(&x, r#"<span class="sigaugxieng fkEnter lrtb">$1</span>"#);
    x = 陰文和墨圍(&x);
    x = re!("〖(.+?)〗", "g").replace(
        &x,
        r#"<span style="padding:0 4px; margin:-2px -2px; border-radius:5px; border:1px solid gray">$1</span>"#,
    );
    x = re!(r"\{（(.+?)）\}", "g").replace(&x, "<del>$1</del>");
    x = re!(r"\{［(.+?)］\}", "g").replace(&x, "<ins>$1</ins>");
    x = re!("《(.+?)》", "g").replace(
        &x,
        r#"<i class="hideCopy">《</i><u class="xiomeng">$1</u><i class="hideCopy">》</i>"#,
    );
    x = re!("〈(.+?)〉", "g").replace(
        &x,
        r#"<i class="hideCopy">〈</i><u class="xiomengRight">$1</u><i class="hideCopy">〉</i>"#,
    );
    x = re!("(·)", "g").replace(
        &x,
        r#"<span class="midDotPrd" style="bottom: 0.5em">$1</span>"#,
    );
    x = re!("‹(.+?)›", "g").replace(
        &x,
        r#"<i class="hideCopy">‹</i><u class="juenmeng"><span>$1</span></u><i class="hideCopy">›</i>"#,
    );
    x = re!(r"\{(.+?)\|(.+?)\}", "g")
        .replace(&x, r#"<span class="rightRuby" data-rrt="$2">$1</span>"#);
    re!(r"([…〻])(?!$)", "g").replace(&x, r#"<span style="display:inline-block">$1</span>"#)
}

/// `點字下`：夾注（已換直排標點）裏的標點、陰文、書名等。
pub fn 點字下(y: &str, env: &Env) -> String {
    let mut y = re!(r"(^|[^﹁﹃])([﹁﹃])(?![﹁﹃﹄﹂])", "g")
        .replace(y, r#"$1<span class="zzbdQt1">$2</span>"#);
    y = re!(r"([^﹁﹃﹄﹂])([﹄﹂])(?=$|[^﹄﹂])", "g")
        .replace(&y, r#"$1<span class="zzbdQt2">$2</span>"#);
    if env.系統 == "Android" || env.系統 == "iPhone" {
        y = re!("([︐︑︒・])", "g").replace(&y, r#"<span class="gdzhbd zzbdMSm">$1</span>"#);
        y = re!("([︓︔])", "g").replace(&y, r#"<span class="gdzhbd zzbdMMd">$1</span>"#);
    } else {
        y = re!("([︐︑︒・])", "g").replace(&y, r#"<span class="gdzhbd zzbdSm">$1</span>"#);
        y = re!("([︓︔])", "g").replace(&y, r#"<span class="gdzhbd zzbdMd">$1</span>"#);
    }
    y = re!("([︖︕])", "g").replace(&y, r#"<span class="gdzhbd zzbdTl">$1</span>"#);
    y = re!("﹄﹂", "g").replace(&y, r#"<span class="zzbd2Qt2"><span>﹄</span>﹂</span>"#);
    y = re!("﹁﹃", "g").replace(&y, r#"<span class="zzbd2Qt1"><span>﹁</span>﹃</span>"#);
    y = re!("︶︵", "g").replace(&y, r#"<span style="letter-spacing: -1em">︶</span>︵"#);
    y = re!("([¯ˇˋˆ])", "g").replace(&y, r#"<span class="sigaugxieng">$1</span>"#);
    let cite = |cls: &str, open: &str, close: &str, inner: &str| {
        format!(
            r#"<i class="hideCopy">{open}</i><cite class="{cls}">{inner}</cite><i class="hideCopy">{close}</i>"#
        )
    };
    y = re!(r"\[︻(.+?)︼\]", "g").replace(&y, &cite("imVang", "[", "]", "<span>$1</span>"));
    y = re!(r"\(︻(.+?)︼\)", "g").replace(&y, &cite("imHiuan", "(", ")", "<span>$1</span>"));
    y = re!(r"︹︻(.+?)︼︺", "g").replace(&y, &cite("im6grug", "〔", "〕", "<span>$1</span>"));
    y = re!(r"\[\u{E794}(.+?)\u{E795}\]", "g")
        .replace(&y, &cite("mhVang", "[", "]", "<span>$1</span>"));
    y = re!(r"\(\u{E794}(.+?)\u{E795}\)", "g")
        .replace(&y, &cite("mhHiuan", "(", ")", "<span>$1</span>"));
    y = re!(r"︹\u{E794}(.+?)\u{E795}︺", "g").replace(
        &y,
        &cite("mh6grug", "〔", "〕", "<span><span>$1</span></span>"),
    );
    y = re!(r"\[︻(.+?)$", "g").replace(&y, r#"<cite class="imVangSt">$1</cite>"#);
    y = re!(r"\(︻(.+?)$", "g").replace(&y, r#"<cite class="imHiuanSt">$1</cite>"#);
    y = re!(r"︹︻(.+?)$", "g").replace(&y, r#"<cite class="im6grugSt">$1</cite>"#);
    y = re!(r"\[\u{E794}(.+?)$", "g")
        .replace(&y, r#"<cite class="mhVangSt"><span>$1</span></cite>"#);
    y = re!(r"\(\u{E794}(.+?)$", "g").replace(&y, r#"<cite class="mhHiuanSt">$1</cite>"#);
    y = re!(r"︹\u{E794}(.+?)$", "g").replace(&y, r#"<cite class="mh6grugSt">$1</cite>"#);
    y = re!(r"^(.+?)︼\]", "g").replace(&y, r#"<cite class="imVangEd"><span>$1</span></cite>"#);
    y = re!(r"^(.+?)︼\)", "g").replace(&y, r#"<cite class="imHiuanEd"><span>$1</span></cite>"#);
    y = re!(r"^(.+?)︼︺", "g").replace(&y, r#"<cite class="im6grugEd"><span>$1</span></cite>"#);
    y = re!(r"^(.+?)\u{E795}\]", "g")
        .replace(&y, r#"<cite class="mhVangEd"><span>$1</span></cite>"#);
    y = re!(r"^(.+?)\u{E795}\)", "g")
        .replace(&y, r#"<cite class="mhHiuanEd"><span>$1</span></cite>"#);
    y = re!(r"^(.+?)\u{E795}︺", "g")
        .replace(&y, r#"<cite class="mh6grugEd"><span>$1</span></cite>"#);
    y = re!(r"︻(.+?)︼", "g").replace(&y, r#"<span class="imHiuan"><span>$1</span></span>"#);
    y = re!(r"\u{E794}(.+?)\u{E795}", "g").replace(
        &y,
        r#"<span style="padding:0 4px; margin:-2px -2px; border-radius:5px; border:1px solid gray">$1</span>"#,
    );
    y = re!(r"\{︵(.+?)︶\}", "g").replace(&y, "<del>$1</del>");
    y = re!(r"\{﹇(.+?)﹈\}", "g").replace(&y, "<ins>$1</ins>");
    y = re!(r"︽(.+?)︾", "g").replace(
        &y,
        r#"<i class="hideCopy">《</i><u class="xiomeng"><span>$1</span></u><i class="hideCopy">》</i>"#,
    );
    y = re!(r"︿(.+?)﹀", "g").replace(&y, r#"<span style="color:purple">$1</span>"#);
    y = re!(r"︽([^︾]+)$", "g").replace(&y, r#"<u class="xiomengSt"><span>$1</span></u>"#);
    y = re!(r"^([^︾]+)︾", "g").replace(&y, r#"<u class="xiomengEd"><span>$1</span></u>"#);
    y = re!("(･)", "g").replace(
        &y,
        r#"<span class="midDotPrd" style="bottom: 0.625em">$1</span>"#,
    );
    y = re!(r"﹤(.+?)﹥", "g").replace(
        &y,
        r#"<i class="hideCopy">‹</i><u class="juenmeng"><span>$1</span></u><i class="hideCopy">›</i>"#,
    );
    y = re!(r"﹤([^﹥]+)$", "g").replace(&y, r#"<u class="juenmengSt"><span>$1</span></u>"#);
    y = re!(r"^([^﹥]+)﹥", "g").replace(&y, r#"<u class="juenmengEd"><span>$1</span></u>"#);
    if env.注中小字 == -1.0 {
        y = re!(r"﹇﹇(.+?)﹈﹈", "g").replace(&y, r#"<small class="DRinDRhf">$1</small>"#);
        y = re!(r"﹇﹇([^﹈]+)$", "g").replace(&y, r#"<small class="DRinDRhf">$1</small>"#);
        y = re!(r"^(.+?)﹈﹈", "g").replace(&y, r#"<small class="DRinDRhf">$1</small>"#);
    } else {
        let ls = num(env.注中小字 + 2.0);
        let t = format!(r#"<small class="DRinDRhf" style="letter-spacing:{ls}em">$1</small>"#);
        y = re!(r"﹇﹇(.+?)﹈﹈", "g").replace(&y, &t);
        y = re!(r"﹇﹇([^﹈]+)$", "g").replace(&y, &t);
        y = re!(r"^(.+?)﹈﹈", "g").replace(&y, &t);
    }
    if env.注中注 == 0.0 {
        y = re!(r"﹇(.+?)﹈", "g").replace(&y, r#"<span class="DRinDR">$1</span>"#);
        y = re!(r"﹇([^﹈]+)$", "g").replace(&y, r#"<span class="DRinDR">$1</span>"#);
        y = re!(r"^(.+?)﹈", "g").replace(&y, r#"<span class="DRinDR">$1</span>"#);
    } else {
        let ls = num(env.注中注 + 0.25);
        let t = format!(r#"<span class="DRinDR" style="letter-spacing:{ls}em">$1</span>"#);
        y = re!(r"﹇(.+?)﹈", "g").replace(&y, &t);
        y = re!(r"﹇([^﹈]+)$", "g").replace(&y, &t);
        y = re!(r"^(.+?)﹈", "g").replace(&y, &t);
    }
    re!(r"([　︙〻])(?!$)", "g").replace(&y, r#"<span style="display:inline-block">$1</span>"#)
}

/// `行間標點`（天頭、目錄等用；`clr` 為 None 時不著色）。
pub fn 行間標點(x: &str, clr: Option<&str>) -> String {
    let mut x = re!("？+！+", "g").replace(x, "⁈");
    x = re!("！+？+", "g").replace(&x, "⁉");
    x = re!("？{2,}", "g").replace(&x, "⁇");
    x = re!("！{2,}", "g").replace(&x, "‼");
    let set = r"([，、､。｡．：；？！⁇⁈⁉‼「」『』（）…〻々])";
    x = match clr {
        None => re!(r"([，、､。｡．：；？！⁇⁈⁉‼「」『』（）…〻々])", "g")
            .replace(&x, r#"<span class="jmhgbd0">$1</span>"#),
        Some(c) if re!(r"^#[A-Fa-f0-9]|^rgb\(").test(c) => crate::jsre::Re::new(set, "g").replace(
            &x,
            &format!(r#"<span class="jmhgbd0" style="color:{c}">$1</span>"#),
        ),
        Some(c) => crate::jsre::Re::new(set, "g").replace(
            &x,
            &format!(r#"<span class="jmhgbd0" style="color:var(--{c})">$1</span>"#),
        ),
    };
    x = 圈點抹(&x);
    x = re!("([﹅﹆])", "g").replace(&x, r#"<span class="jmhgbd0">$1</span>"#);
    x = re!("([〪〭〫〬])", "g").replace(&x, r#"<span class="sigrugxieng">$1</span>"#);
    x = 陰文和墨圍(&x);
    x = re!("〖(.+?)〗", "g").replace(
        &x,
        r#"<span style="padding:0 4px; margin:-2px -2px; border-radius:5px; border:1px solid gray">$1</span>"#,
    );
    x = re!("《(.+?)》", "g").replace(&x, r#"<u class="xiomeng">$1</u>"#);
    x = re!("〈(.+?)〉", "g").replace(&x, r#"<u class="xiomengRight">$1</u>"#);
    x = re!("(·)", "g").replace(
        &x,
        r#"<span class="midDotPrd" style="bottom: 0.5em; color:var(--標點色)">$1</span>"#,
    );
    x = re!("‹(.+?)›", "g").replace(&x, r#"<u class="juenmeng"><span>$1</span></u>"#);
    re!(r"\{(.+?)\|(.+?)\}", "g").replace(&x, r#"<span class="rightRuby" data-rrt="$2">$1</span>"#)
}
