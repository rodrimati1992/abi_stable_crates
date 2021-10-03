use aho_corasick::{AhoCorasick, Match};

#[allow(dead_code)]
#[derive(Copy, Clone)]
struct MatchMine {
    pati: usize,
    start: usize,
    end: usize,
}

impl From<Match> for MatchMine {
    fn from(this: Match) -> Self {
        MatchMine {
            pati: this.pattern() as _,
            start: this.start() as _,
            end: this.end() as _,
        }
    }
}

pub fn replace_text(text: &str, map: &[(String, String)], buffer: &mut String) {
    let find_automata = AhoCorasick::new(map.iter().map(|(k, _)| k.as_bytes()));

    buffer.clear();
    buffer.reserve(text.len() * 130 / 100);

    let mut last_m = MatchMine {
        pati: 0,
        start: 0,
        end: 0,
    };
    for m in find_automata
        .find_iter(text.as_bytes())
        .map(MatchMine::from)
    {
        buffer.push_str(&text[last_m.end..m.start]);
        buffer.push_str(&map[m.pati].1);

        last_m = m;
    }
    buffer.push_str(&text[last_m.end..]);
}
