//! Languages. The game is written in English, and the English text is the
//! key for its translations: `tr("You see nothing.")` gives the Polish line
//! from `assets/lang/pl.txt` when Polish is chosen.
//!
//! `t!("You see {}.", noun)` translates a template and fills it in. Templates
//! take `{}` (the next argument) or `{N}` (argument N), and in Polish a noun
//! argument may name its case: `{:acc}`, `{1:gen}` (`{:Nom}` also
//! capitalises it, for the start of a sentence). `{a|b}` picks the first
//! form for a male player and the second for a female one (Polish verbs and
//! adjectives agree with the speaker), and `{N?m|f|n|p}` picks by the
//! grammatical gender of noun argument N.
//!
//! The file holds phrases as pairs of lines (English, then `= Polish`) and
//! nouns as `@english = nom | gen | acc | nom pl | gen pl | gender`, where the
//! gender is `m` (masculine), `ma` (masculine animate), `mp` (masculine
//! personal), `f`, `n`, or `p` (plural only, like "drzwi").

use std::cell::Cell;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, serde::Serialize, serde::Deserialize)]
pub enum Lang {
    #[default]
    En,
    Pl,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::En, Lang::Pl];

    /// The language's name, in that language.
    pub fn name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Pl => "polski",
        }
    }

    pub fn next(self) -> Lang {
        let i = Lang::ALL.iter().position(|l| *l == self).unwrap_or(0);
        Lang::ALL[(i + 1) % Lang::ALL.len()]
    }
}

thread_local! {
    static LANG: Cell<Lang> = const { Cell::new(Lang::En) };
    static FEMALE: Cell<bool> = const { Cell::new(false) };
}

pub fn set(l: Lang) {
    LANG.with(|c| c.set(l));
}

pub fn get() -> Lang {
    LANG.with(|c| c.get())
}

/// Whether the player is a woman, for `{a|b}` in templates.
pub fn set_female(f: bool) {
    FEMALE.with(|c| c.set(f));
}

fn female() -> bool {
    FEMALE.with(|c| c.get())
}

// ------------------------------------------------------------ the tables

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gender {
    /// Masculine inanimate (miecz).
    M,
    /// Masculine animate (goblin): the accusative is the genitive.
    Ma,
    /// Masculine personal (mag).
    Mp,
    F,
    N,
    /// Plural only (drzwi, zwłoki).
    P,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Case {
    Nom,
    Gen,
    Acc,
}

#[derive(Clone, Copy, Debug)]
pub struct PlNoun {
    /// Nominative, genitive and accusative singular.
    pub sg: [&'static str; 3],
    /// Nominative and genitive plural.
    pub pl: [&'static str; 2],
    pub gender: Gender,
}

struct Table {
    /// English to Polish, with `{a|b}` already resolved for a male and a
    /// female player.
    phrases: HashMap<&'static str, (&'static str, &'static str)>,
    nouns: HashMap<&'static str, PlNoun>,
}

const PL_SRC: &str = include_str!("../assets/lang/pl.txt");

fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn unescape(s: &str) -> &'static str {
    if s.contains("\\n") { Box::leak(s.replace("\\n", "\n").into_boxed_str()) } else { Box::leak(s.to_string().into_boxed_str()) }
}

fn gender(s: &str) -> Option<Gender> {
    Some(match s {
        "m" => Gender::M,
        "ma" => Gender::Ma,
        "mp" => Gender::Mp,
        "f" => Gender::F,
        "n" => Gender::N,
        "p" => Gender::P,
        _ => return None,
    })
}

fn parse(src: &'static str) -> Result<Table, String> {
    let mut phrases = HashMap::new();
    let mut nouns = HashMap::new();
    let mut key: Option<(usize, &'static str)> = None;
    for (ln, line) in src.lines().enumerate() {
        let ln = ln + 1;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix('@') {
            let (en, forms) = rest.split_once(" = ").ok_or(format!("line {}: noun without ' = '", ln))?;
            let f: Vec<&'static str> = forms.split('|').map(|s| unescape(s.trim())).collect();
            if f.len() != 6 {
                return Err(format!("line {}: a noun needs six fields", ln));
            }
            let g = gender(f[5]).ok_or(format!("line {}: unknown gender '{}'", ln, f[5]))?;
            if nouns.insert(unescape(en.trim()), PlNoun { sg: [f[0], f[1], f[2]], pl: [f[3], f[4]], gender: g }).is_some() {
                return Err(format!("line {}: noun '{}' is defined twice", ln, en.trim()));
            }
        } else if let Some(pl) = line.strip_prefix("= ") {
            let (kl, en) = key.take().ok_or(format!("line {}: translation without an English line", ln))?;
            let pl = unescape(pl);
            let forms = (leak(by_sex(pl, false)), leak(by_sex(pl, true)));
            if phrases.insert(en, forms).is_some() {
                return Err(format!("line {}: '{}' is translated twice", kl, en));
            }
        } else {
            if let Some((kl, en)) = key {
                return Err(format!("line {}: '{}' has no translation", kl, en));
            }
            key = Some((ln, unescape(line)));
        }
    }
    if let Some((kl, en)) = key {
        return Err(format!("line {}: '{}' has no translation", kl, en));
    }
    Ok(Table { phrases, nouns })
}

fn polish() -> &'static Table {
    static T: OnceLock<Table> = OnceLock::new();
    T.get_or_init(|| parse(PL_SRC).expect("assets/lang/pl.txt is valid"))
}

/// Translate a fixed piece of text. Text with no translation stays English.
pub fn tr(en: &'static str) -> &'static str {
    match get() {
        Lang::En => en,
        Lang::Pl => polish().phrases.get(en).map(|f| if female() { f.1 } else { f.0 }).unwrap_or(en),
    }
}

/// Translate text that is not a literal (level writings, names from saves).
pub fn tr_dyn(en: &str) -> String {
    match get() {
        Lang::En => en.to_string(),
        Lang::Pl => polish().phrases.get(en).map(|f| if female() { f.1 } else { f.0 }).unwrap_or(en).to_string(),
    }
}

fn noun(key: &str) -> Option<&'static PlNoun> {
    polish().nouns.get(key)
}

/// Whether a noun has Polish forms (for tests).
#[cfg(test)]
pub fn has_noun(key: &str) -> bool {
    noun(key).is_some()
}

/// Whether a phrase has a Polish line (for tests).
#[cfg(test)]
pub fn has_phrase(en: &str) -> bool {
    polish().phrases.contains_key(en)
}

// --------------------------------------------------------------- nouns

/// A noun phrase: an item, a creature. In English it is the finished text
/// ("a worn longsword"); in Polish it is built from the noun's forms, the
/// adjectives agree with it, and a template chooses the case.
#[derive(Clone, Debug)]
pub struct Noun {
    pub en: String,
    /// The English base name the Polish forms are kept under.
    key: String,
    /// English adjectives, translated and inflected in Polish.
    adjs: Vec<&'static str>,
    /// Polish text after the noun ("o imieniu Squeak").
    suffix: String,
    qty: u32,
}

impl Noun {
    pub fn new(en: impl Into<String>, key: &str) -> Noun {
        Noun { en: en.into(), key: key.to_string(), adjs: Vec::new(), suffix: String::new(), qty: 1 }
    }

    pub fn adj(mut self, en_adj: &'static str) -> Noun {
        let a = en_adj.trim();
        if !a.is_empty() {
            self.adjs.push(a);
        }
        self
    }

    pub fn suffix(mut self, s: impl Into<String>) -> Noun {
        self.suffix = s.into();
        self
    }

    pub fn qty(mut self, n: u32) -> Noun {
        self.qty = n.max(1);
        self
    }

    fn forms(&self) -> PlNoun {
        noun(&self.key).copied().unwrap_or_else(|| {
            let s: &'static str = Box::leak(self.key.clone().into_boxed_str());
            PlNoun { sg: [s; 3], pl: [s; 2], gender: Gender::M }
        })
    }

    pub fn gender(&self) -> Gender {
        if self.qty > 1 { Gender::P } else { self.forms().gender }
    }

    /// The Polish phrase in a case.
    pub fn pl(&self, case: Case) -> String {
        let f = self.forms();
        let mut words: Vec<String> = Vec::new();
        let noun = if self.qty > 1 {
            words.push(self.qty.to_string());
            let few = matches!(self.qty % 10, 2..=4) && !matches!(self.qty % 100, 12..=14);
            let case = if case != Case::Gen && few { Case::Nom } else { Case::Gen };
            for a in &self.adjs {
                words.push(inflect(tr(a), case, Gender::P));
            }
            f.pl[if case == Case::Nom { 0 } else { 1 }]
        } else {
            for a in &self.adjs {
                words.push(inflect(tr(a), case, f.gender));
            }
            f.sg[case as usize]
        };
        words.push(noun.to_string());
        if !self.suffix.is_empty() {
            words.push(self.suffix.clone());
        }
        words.join(" ")
    }

    /// The text for the current language (nominative).
    pub fn text(&self) -> String {
        match get() {
            Lang::En => self.en.clone(),
            Lang::Pl => self.pl(Case::Nom),
        }
    }
}

/// Inflect the adjectives in a Polish phrase (words ending in -y or -i) to
/// agree with a noun. Other words (adverbs: "mocno", "nieco") stay as they are.
pub fn inflect(phrase: &str, case: Case, g: Gender) -> String {
    phrase.split(' ').map(|w| inflect_word(w, case, g)).collect::<Vec<_>>().join(" ")
}

fn inflect_word(w: &str, case: Case, g: Gender) -> String {
    // A leading _ marks a word that never changes ("bliski _śmierci").
    if let Some(w) = w.strip_prefix('_') {
        return w.to_string();
    }
    let soft = w.ends_with('i');
    if w.chars().count() < 3 || !(soft || w.ends_with('y')) {
        return w.to_string();
    }
    let stem = &w[..w.len() - 1];
    let end = match (g, case) {
        (Gender::P, Case::Gen) => "ych",
        (Gender::P, _) => "e",
        (Gender::F, Case::Nom) => "a",
        (Gender::F, Case::Gen) => "ej",
        (Gender::F, Case::Acc) => "ą",
        (Gender::N, Case::Nom | Case::Acc) => "e",
        (_, Case::Gen) => "ego",
        (Gender::Ma | Gender::Mp, Case::Acc) => "ego",
        _ => "y",
    };
    // Soft stems (wrogi, lekki) keep their i before e: wrogie, wrogiego, wrogich.
    let end = if soft {
        match end {
            "y" => "i",
            "ych" => "ich",
            "e" | "ego" | "ej" => return format!("{}i{}", stem, end),
            e => e,
        }
    } else {
        end
    };
    format!("{}{}", stem, end)
}

/// "a torch, bread and a lantern", in a case for Polish.
pub fn list(nouns: &[Noun], case: Case) -> String {
    let names: Vec<String> = nouns
        .iter()
        .map(|n| match get() {
            Lang::En => n.en.clone(),
            Lang::Pl => n.pl(case),
        })
        .collect();
    match names.len() {
        0 => String::new(),
        1 => names[0].clone(),
        n => format!("{} {} {}", names[..n - 1].join(", "), tr("and"), names[n - 1]),
    }
}

/// An adjective (or participle) that agrees with a noun: "(wounded)" after a
/// creature, "It is now worn".
pub fn agree(en_adj: &'static str, n: &Noun) -> String {
    match get() {
        Lang::En => en_adj.to_string(),
        Lang::Pl => inflect(tr(en_adj), Case::Nom, n.gender()),
    }
}

/// An adjective for a given gender (the player's, say).
pub fn agree_g(en_adj: &'static str, g: Gender) -> String {
    match get() {
        Lang::En => en_adj.to_string(),
        Lang::Pl => inflect(tr(en_adj), Case::Nom, g),
    }
}

/// "1st", "2nd" in English, "1.", "2." in Polish.
pub fn ordinal(n: i32) -> String {
    match get() {
        Lang::Pl => format!("{}.", n),
        Lang::En => {
            let suffix = match (n % 10, n % 100) {
                (1, 11) | (2, 12) | (3, 13) => "th",
                (1, _) => "st",
                (2, _) => "nd",
                (3, _) => "rd",
                _ => "th",
            };
            format!("{}{}", n, suffix)
        }
    }
}

/// An amount of gold: "40 gold" / "40 sztuk złota".
pub fn gold(n: i32) -> String {
    match get() {
        Lang::En => format!("{} gold", n),
        Lang::Pl => {
            let few = matches!(n % 10, 2..=4) && !matches!(n % 100, 12..=14);
            format!("{} {} złota", n, if n == 1 { "sztuka" } else if few { "sztuki" } else { "sztuk" })
        }
    }
}

/// Capitalise the first letter.
pub fn cap(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

// ----------------------------------------------------------- templates

pub enum Arg {
    Text(String),
    Noun(Noun),
}

pub trait IntoArg {
    fn into_arg(self) -> Arg;
}

impl IntoArg for Noun {
    fn into_arg(self) -> Arg {
        Arg::Noun(self)
    }
}

impl IntoArg for &Noun {
    fn into_arg(self) -> Arg {
        Arg::Noun(self.clone())
    }
}

macro_rules! text_args {
    ($($t:ty),*) => { $(impl IntoArg for $t { fn into_arg(self) -> Arg { Arg::Text(self.to_string()) } })* };
}
text_args!(&str, String, &String, i32, i64, u8, u16, u32, usize, char);

/// Fill a template's placeholders (see the module notes).
pub fn fill(template: &str, args: &[Arg]) -> String {
    let mut out = String::new();
    let mut next = 0;
    let mut rest = template;
    while let Some(k) = rest.find('{') {
        out.push_str(&rest[..k]);
        let tail = &rest[k + 1..];
        let Some(end) = tail.find('}') else {
            out.push_str(&rest[k..]);
            return out;
        };
        let spec = &tail[..end];
        rest = &tail[end + 1..];
        // {N?m|f|n|p}: by the gender of a noun.
        if let Some((n, forms)) = spec.split_once('?') {
            let forms: Vec<&str> = forms.split('|').collect();
            let i = match args.get(n.parse::<usize>().unwrap_or(0)) {
                Some(Arg::Noun(n)) => match n.gender() {
                    Gender::M | Gender::Ma | Gender::Mp => 0,
                    Gender::F => 1,
                    Gender::N => 2,
                    Gender::P => 3,
                },
                _ => 0,
            };
            out.push_str(forms.get(i).or(forms.last()).unwrap_or(&""));
            continue;
        }
        // {a|b}: by the player's sex.
        if let Some((m, f)) = spec.split_once('|') {
            out.push_str(if female() { f } else { m });
            continue;
        }
        let (idx, case) = match spec.split_once(':') {
            Some((i, c)) => (i, c),
            None => (spec, ""),
        };
        let i = if idx.is_empty() {
            next += 1;
            next - 1
        } else {
            idx.parse::<usize>().unwrap_or(0)
        };
        // A capital (`{:Nom}`) capitalises the text.
        let upper = case.starts_with(|c: char| c.is_ascii_uppercase());
        let case = match case.to_ascii_lowercase().as_str() {
            "gen" => Case::Gen,
            "acc" => Case::Acc,
            _ => Case::Nom,
        };
        let s = match args.get(i) {
            Some(Arg::Text(s)) => s.clone(),
            Some(Arg::Noun(n)) => match get() {
                Lang::En => n.en.clone(),
                Lang::Pl => n.pl(case),
            },
            None => String::new(),
        };
        out.push_str(&if upper { cap(&s) } else { s });
    }
    out.push_str(rest);
    out
}

/// Resolve `{a|b}` (male and female forms) in text that has other braces
/// too, like dialogue with `{name}` and `{buy N}`.
pub fn by_sex(text: &str, female: bool) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(k) = rest.find('{') {
        out.push_str(&rest[..k]);
        let tail = &rest[k..];
        let Some(end) = tail.find('}') else {
            out.push_str(tail);
            return out;
        };
        let spec = &tail[1..end];
        match spec.split_once('|') {
            Some((m, f)) if !spec.contains('?') => out.push_str(if female { f } else { m }),
            _ => out.push_str(&tail[..=end]),
        }
        rest = &tail[end + 1..];
    }
    out.push_str(rest);
    out
}

/// Translate a template and fill it in: `t!("You see {}.", noun)`. With no
/// arguments it is `tr`.
#[macro_export]
macro_rules! t {
    ($s:literal) => {
        $crate::lang::tr($s)
    };
    ($s:expr) => {
        $crate::lang::tr($s)
    };
    ($s:literal, $($a:expr),+ $(,)?) => {
        $crate::lang::fill($crate::lang::tr($s), &[$($crate::lang::IntoArg::into_arg($a)),+])
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The English keys in the code: the literals in `t!(...)` and `tr(...)`.
    fn literal_keys() -> Vec<(String, String)> {
        fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            for e in std::fs::read_dir(dir).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().map(|x| x == "rs").unwrap_or(false) {
                    out.push(p);
                }
            }
        }
        let mut files = Vec::new();
        walk(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut files);
        let mut keys = Vec::new();
        for f in files {
            // This file's examples are not game text.
            if f.ends_with("lang.rs") {
                continue;
            }
            let src: String = std::fs::read_to_string(&f).unwrap().lines().filter(|l| !l.trim_start().starts_with("//")).collect::<Vec<_>>().join("\n");
            for opener in ["t!(\"", "tr(\""] {
                let mut rest = src.as_str();
                while let Some(k) = rest.find(opener) {
                    let body = &rest[k + opener.len()..];
                    // Not format!( or str(.
                    if rest[..k].ends_with(|c: char| c.is_alphanumeric() || c == '_') {
                        rest = body;
                        continue;
                    }
                    let mut s = String::new();
                    let mut chars = body.chars();
                    while let Some(c) = chars.next() {
                        match c {
                            '\\' => match chars.next() {
                                Some('n') => s.push('\n'),
                                Some(c) => s.push(c),
                                None => {}
                            },
                            '"' => break,
                            c => s.push(c),
                        }
                    }
                    keys.push((f.display().to_string(), s));
                    rest = body;
                }
            }
        }
        keys
    }

    /// Text that reaches the screen without a literal `t!` around it.
    fn other_keys() -> Vec<String> {
        use crate::game::magic::{ALL_SPELLS, RUNE_MEANINGS};
        let mut v: Vec<String> = Vec::new();
        v.extend([crate::ui::overlay::HELP_TEXT, crate::ui::overlay::MODERN_HELP_TEXT].map(String::from));
        v.extend(crate::ui::overlay::OPTIONS.map(String::from));
        v.extend(crate::game::dialogue::SCROLLS.iter().map(|s| s.to_string()));
        v.extend(crate::game::dialogue::CHRONICLE.iter().map(|s| s.to_string()));
        v.extend(RUNE_MEANINGS.map(String::from));
        v.extend(ALL_SPELLS.iter().map(|s| s.def().name.to_string()));
        v.extend(crate::ui::menus::story_texts().into_iter().map(String::from));
        let g = crate::game::Game::new(crate::game::Player::new("T", crate::game::PClass::Fighter, false), 1);
        for lv in &g.levels {
            v.push(lv.name.clone());
            v.extend(lv.writings.iter().map(|w| w.2.clone()));
            for t in &lv.triggers {
                if let crate::game::TriggerKind::Message { text } = &t.kind {
                    v.push(text.clone());
                }
            }
            v.extend(lv.npcs.iter().filter_map(|n| n.name.clone()));
        }
        v
    }

    fn placeholders(s: &str) -> (usize, bool) {
        let mut n = 0;
        let mut positional = false;
        let mut rest = s;
        while let Some(k) = rest.find('{') {
            let tail = &rest[k + 1..];
            let Some(end) = tail.find('}') else { break };
            let spec = &tail[..end];
            if !spec.contains('|') {
                n += 1;
                positional |= spec.starts_with(|c: char| c.is_ascii_digit());
            }
            rest = &tail[end + 1..];
        }
        (n, positional)
    }

    #[test]
    fn every_phrase_is_translated() {
        let t = polish();
        let mut missing: Vec<String> = Vec::new();
        for (file, k) in literal_keys() {
            if !t.phrases.contains_key(k.as_str()) && !missing.contains(&k) {
                missing.push(format!("{}: {}", file, k));
            }
        }
        for k in other_keys() {
            // Names of people are the same in Polish.
            if !t.phrases.contains_key(k.as_str()) && !t.nouns.contains_key(k.as_str()) {
                missing.push(k);
            }
        }
        missing.dedup();
        assert!(missing.is_empty(), "{} phrases have no Polish line:\n{}", missing.len(), missing.join("\n"));
        for (en, (pl, _)) in &t.phrases {
            let (ne, _) = placeholders(en);
            let (np, positional) = placeholders(pl);
            assert!(positional || ne == np, "'{}' has {} placeholders but its translation '{}' has {}", en, ne, pl, np);
        }
    }

    #[test]
    fn the_fonts_have_every_polish_letter() {
        let small = crate::gfx::font::Font::small();
        let t = polish();
        let mut texts: Vec<&str> = t.phrases.values().flat_map(|f| [f.0, f.1]).collect();
        for n in t.nouns.values() {
            texts.extend(n.sg);
            texts.extend(n.pl);
        }
        let dialogue = include_str!("../assets/dialogue.pl.txt");
        texts.push(dialogue);
        for s in texts {
            for ch in s.chars() {
                assert!(ch == ' ' || ch == '\n' || small.has_glyph(ch), "the small font has no '{}' (in '{}')", ch, s);
            }
        }
        let serif = crate::gfx::font::Font::serif();
        for ch in "ąćęłńóśźżĄĆĘŁŃÓŚŹŻ".chars() {
            assert!(serif.has_glyph(ch), "the serif font has no '{}'", ch);
        }
    }

    #[test]
    fn table_parses() {
        let t = polish();
        assert!(!t.phrases.is_empty());
        assert!(!t.nouns.is_empty());
    }

    #[test]
    fn english_is_unchanged() {
        set(Lang::En);
        let n = Noun::new("a worn longsword", "longsword").adj("worn");
        assert_eq!(t!("You see {}.", &n), "You see a worn longsword.");
        assert_eq!(fill("{1} and {0}", &[1.into_arg(), "two".into_arg()]), "two and 1");
    }

    #[test]
    fn adjectives_agree() {
        assert_eq!(inflect("mocno zużyty", Case::Acc, Gender::F), "mocno zużytą");
        assert_eq!(inflect("wrogi", Case::Acc, Gender::Ma), "wrogiego");
        assert_eq!(inflect("wrogi", Case::Nom, Gender::N), "wrogie");
        assert_eq!(inflect("zapalony", Case::Gen, Gender::P), "zapalonych");
        assert_eq!(inflect("nieco", Case::Acc, Gender::F), "nieco");
    }

    #[test]
    fn polish_nouns_decline() {
        set(Lang::Pl);
        let n = Noun::new("a torch", "torch").adj("lit");
        assert_eq!(fill("Widzisz {:acc}.", &[n.into_arg()]), "Widzisz zapaloną pochodnię.");
        let coins = |q| Noun::new("", "gold coin").qty(q).pl(Case::Acc);
        assert_eq!(coins(3), "3 złote monety");
        assert_eq!(coins(5), "5 złotych monet");
        assert_eq!(coins(22), "22 złote monety");
        assert_eq!(coins(12), "12 złotych monet");
        assert_eq!(gold(1), "1 sztuka złota");
        set_female(true);
        assert_eq!(fill("Zabił{eś|aś} go.", &[]), "Zabiłaś go.");
        set_female(false);
        set(Lang::En);
    }
}
