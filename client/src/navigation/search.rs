//! What a search of the navigation finds (the client's `CNaviData::Search` with `CNaviInfo::Search…`), and how
//! what was found is put into the groups its window lists (`CNavigationManager::BuildNodePaths`).

use super::{Atlas, Target, What};

/// The words a search is told apart by, as the client's message table has them (its lines 2322 to 2325):
/// what every such word begins with, the word for a level or a range of levels, the sign between the two
/// levels, and the word for everything.
#[derive(Debug, Clone)]
pub struct Words {
    pub special: String,
    pub level: String,
    pub range: String,
    pub all: String,
}

impl Default for Words {
    fn default() -> Self {
        Self { special: "$$".to_owned(), level: "$$lv".to_owned(), range: "~".to_owned(), all: "$$all".to_owned() }
    }
}

/// What is asked for (`CNaviData::ParseSearchKind`).
#[derive(Debug, PartialEq)]
enum Asked {
    /// Plain words: every one of them is in the name (1), or a special word that is none of those below (0).
    Named,
    /// `$$all`: everything (2).
    All,
    /// `$$lv30` or `$$lv20~30`: the monsters of those levels (3).
    Levels(i32, i32),
    /// `$$DB NAME`: the monsters called so in the database (4).
    Titled,
}

/// A whole number at the beginning of a text, as C's `atoi` reads one: nought for none.
pub(super) fn atoi(text: &str) -> i32 {
    let text = text.trim_start();
    let (sign, digits) = match text.strip_prefix('-') {
        Some(rest) => (-1, rest),
        None => (1, text.strip_prefix('+').unwrap_or(text)),
    };
    let end = digits.find(|digit: char| !digit.is_ascii_digit()).unwrap_or(digits.len());
    digits[..end].parse::<i64>().map_or(0, |number| (sign * number).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32)
}

fn asked(first: &str, words: &Words) -> Asked {
    if !first.starts_with(&words.special) {
        return Asked::Named;
    }
    if first.len() > words.level.len() && first.starts_with(&words.level) {
        let levels = &first[words.level.len()..];
        let (low, high) = match levels.find(&words.range) {
            None => (levels, levels),
            // Nothing after the sign is "and upwards": to 500.
            Some(at) => (&levels[..at], Some(&levels[at + words.range.len()..]).filter(|high| !high.is_empty()).unwrap_or("500")),
        };
        let (low, high) = (atoi(low), atoi(high));
        return if low < 1 || high < 1 || low > high { Asked::Named } else { Asked::Levels(low, high) };
    }
    if first.starts_with(&words.all) {
        return Asked::All;
    }
    if first.starts_with("$$DB") { Asked::Titled } else { Asked::Named }
}

/// Whether every word is in the text, whatever the size of its letters (`StrStrIW`).
fn has_all(text: &str, words: &[String]) -> bool {
    let text = text.to_lowercase();
    words.iter().all(|word| text.contains(&word.to_lowercase()))
}

impl Atlas {
    /// What a text typed into the navigation's line finds, map by map; none for a text of no words. `kind`
    /// narrows it as the client's older window did (0 for everything, as its window of now always asks).
    pub fn search(&self, query: &str, kind: i32, words: &Words) -> Option<Vec<Target>> {
        let tokens: Vec<String> = query.split(' ').filter(|token| !token.is_empty()).map(str::to_owned).collect();
        let asked = asked(tokens.first()?, words);
        let mut found = Vec::new();
        for map in 0..self.maps.len() {
            match asked {
                Asked::All => self.everything(map, kind, &mut found),
                Asked::Levels(low, high) => {
                    let within = |mob: &usize| (low..=high).contains(&self.mobs[*mob].x);
                    found.extend(self.maps[map].mobs.iter().copied().filter(within).map(|mob| self.listed(What::Mob(mob))));
                }
                Asked::Titled => {
                    if let [_, title] = tokens.as_slice() {
                        let titled = |mob: &usize| self.mobs[*mob].title == *title;
                        found.extend(self.maps[map].mobs.iter().copied().filter(titled).map(|mob| self.listed(What::Mob(mob))));
                    }
                }
                Asked::Named => self.named(map, kind, &tokens, &mut found),
            }
        }
        Some(found)
    }

    /// An NPC or a monster as a search lists it: with its two numbers and its name.
    fn listed(&self, what: What) -> Target {
        let spot = match what {
            What::Npc(npc) => &self.npcs[npc],
            What::Mob(mob) => &self.mobs[mob],
            What::Place(map) | What::Map(map) => return Target { what, x: -1, y: -1, name: self.maps[map].name.clone() },
            What::Node(link) => return Target { what, x: -1, y: -1, name: self.links[link].name.clone() },
        };
        Target { what, x: spot.x, y: spot.y, name: spot.name.clone() }
    }

    /// Everything of a map (`CNaviInfo::CollectTargets`): the map itself unless it is indoors, its NPCs, its
    /// monsters.
    fn everything(&self, map: usize, kind: i32, found: &mut Vec<Target>) {
        let of = &self.maps[map];
        if !of.name.contains("_in") && (kind == 1 || kind == 0) {
            found.push(self.listed(What::Map(map)));
        }
        if kind == 2 || kind == 0 {
            found.extend(of.npcs.iter().map(|npc| self.listed(What::Npc(*npc))));
        }
        if kind == 3 || kind == 0 {
            found.extend(of.mobs.iter().map(|mob| self.listed(What::Mob(*mob))));
        }
    }

    /// What of a map has every word in its name (`CNaviInfo::Search`): the map itself, by its name and what
    /// the client calls it, and then each of its monsters listed under it (`SearchMap`); its monsters by their
    /// names (`SearchMobs`); its NPCs by theirs (`SearchNpcs`). (As the client has it, a search narrowed to
    /// NPCs, kind 2, looks through the monsters, and one narrowed to monsters through the NPCs.)
    fn named(&self, map: usize, kind: i32, words: &[String], found: &mut Vec<Target>) {
        let of = &self.maps[map];
        if of.searchable() && has_all(&format!("{} {} ", of.name, of.title), words) {
            found.push(self.listed(What::Map(map)));
            found.extend(of.mobs.iter().map(|mob| Target { what: What::Mob(*mob), x: -1000, y: -1000, name: self.mobs[*mob].name.clone() }));
        }
        if kind == 2 || kind == 0 {
            found.extend(of.mobs.iter().filter(|mob| has_all(&self.mobs[**mob].name, words)).map(|mob| self.listed(What::Mob(*mob))));
        }
        if kind == 3 || kind == 0 {
            found.extend(of.npcs.iter().filter(|npc| has_all(&self.npcs[**npc].name, words)).map(|npc| self.listed(What::Npc(*npc))));
        }
    }

    /// The name of what a target is (`UINavigationV4Wnd::ObjectName`): an NPC's or a monster's own, a map's
    /// for a map and a place.
    pub fn name_of(&self, what: What) -> &str {
        match what {
            What::Npc(npc) => &self.npcs[npc].name,
            What::Mob(mob) => &self.mobs[mob].name,
            What::Place(map) | What::Map(map) => &self.maps[map].name,
            What::Node(link) => &self.links[link].name,
        }
    }

    /// The group a target found belongs to among those made so far, if any (`FindNodePath`): a monster listed
    /// under its map goes with that map; an NPC or a monster with the group whose first is another of its
    /// name. A map and a place go with none.
    fn group_of(&self, found: &[Target], groups: &[Vec<usize>], index: usize) -> Option<usize> {
        let target = &found[index];
        if matches!(target.what, What::Map(_) | What::Place(_)) {
            return None;
        }
        for (place, group) in groups.iter().enumerate() {
            let first = found.get(*group.first()?)?;
            if let (What::Mob(mob), true) = (target.what, target.under_its_map()) {
                let What::Map(map) = first.what else { continue };
                if self.maps[map].name == self.maps[self.mobs[mob].map].name {
                    return Some(place);
                }
            }
            if first.what.kind() == target.what.kind() && self.name_of(first.what) == self.name_of(target.what) {
                return Some(place);
            }
        }
        None
    }

    /// What was found, in the groups the window lists (`BuildNodePaths`), each a list of places in `found`:
    /// every map a group of its own first (when everything or the maps are listed), then every other target
    /// of the kind listed (0 for all; 2 NPCs, 3 monsters) with its group, or as a new one.
    pub fn groups(&self, found: &[Target], kind: i32) -> Vec<Vec<usize>> {
        let mut groups: Vec<Vec<usize>> = Vec::new();
        if kind == 0 || kind == 1 {
            groups.extend(found.iter().enumerate().filter(|(_, target)| matches!(target.what, What::Map(_))).map(|(index, _)| vec![index]));
        }
        for (index, target) in found.iter().enumerate() {
            if (kind != 0 && target.what.kind() != kind) || matches!(target.what, What::Map(_)) {
                continue;
            }
            match self.group_of(found, &groups, index) {
                Some(group) => groups[group].push(index),
                None => groups.push(vec![index]),
            }
        }
        groups
    }
}

/// What was found, ordered as the client orders it (`UpdateSearchResults`: by name, the later first).
pub fn ordered(found: &mut [Target]) {
    found.sort_by(|left, right| right.name.as_bytes().cmp(left.name.as_bytes()));
}

/// The group whose first target has the name sought changes places with the first group (`SelectNodePath`).
pub fn first_named(found: &[Target], groups: &mut [Vec<usize>], name: &str) {
    if name.is_empty() || groups.len() < 2 {
        return;
    }
    if let Some(place) = groups.iter().position(|group| group.first().and_then(|first| found.get(*first)).is_some_and(|first| first.name == name)) {
        groups.swap(0, place);
    }
}
