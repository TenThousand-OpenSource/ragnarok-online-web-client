//! The scripts of a homunculus and a mercenary: the rules of the client's functions, a script of this test's
//! own run through them, and the client's own `AI.lua` and `AI_M.lua` when the client's folder is there.

use super::*;

fn seen(id: u32, cell: (i32, i32)) -> Seen {
    Seen { id, cell, ..Seen::default() }
}

/// The player (100) at 50, 50, its Lif (7) three cells east, a monster (9) that struck the player, and an NPC.
fn sight() -> Sight {
    Sight {
        myself: 7,
        owner: seen(100, (50, 50)),
        owner_life: Life { hp: 900, max_hp: 1000, sp: 40, max_sp: 50 },
        actors: vec![
            Seen { job: 6001, ..seen(7, (53, 50)) },
            Seen { job: 1002, target: 100, enemy: true, motion: motion::ATTACK, ..seen(9, (58, 54)) },
            Seen { job: 46, ..seen(11, (40, 40)) },
        ],
        life: Life { hp: 150, max_hp: 200, sp: 20, max_sp: 30 },
        range: 1,
        attack_delay: 700,
        skills: vec![(8013, 3, 9)],
        friends: Vec::new(),
        tick: 10_000.0,
        walkable: None,
    }
}

#[test]
fn answers_what_the_script_asks() {
    let sight = sight();
    let ask = |what: i32, id: u32| sight.value(Kind::Homunculus, what, id, 0);
    assert_eq!(ask(0, 7), [100.0]);
    assert_eq!((ask(1, 7), ask(1, 100), ask(1, 12)), (vec![53.0, 50.0], vec![50.0, 50.0], vec![-1.0, -1.0]));
    assert_eq!((ask(3, 9), ask(3, 12), ask(4, 7), ask(5, 9), ask(5, 12)), (vec![2.0], vec![-1.0], vec![1.0], vec![100.0], vec![-1.0]));
    assert_eq!(
        (ask(7, 7), ask(8, 7), ask(8, 100), ask(8, 9), ask(11, 100), ask(11, 7)),
        (vec![1.0], vec![150.0], vec![900.0], vec![-1.0], vec![50.0], vec![30.0])
    );
    // A skill it has reaches as far as the server said; one it has not, one cell.
    assert_eq!((sight.value(Kind::Homunculus, 6, 7, 8013), sight.value(Kind::Homunculus, 14, 7, 8001)), (vec![9.0], vec![1.0]));
    // A mercenary's script is told which mercenary it is, and no target of the dead.
    let mut hired = sight.clone();
    hired.actors[0].job = 6018;
    hired.actors[1].motion = motion::DEAD;
    assert_eq!(
        (hired.value(Kind::Mercenary, 12, 7, 0), hired.value(Kind::Mercenary, 5, 9, 0), hired.value(Kind::Mercenary, 7, 7, 0)),
        (vec![2.0], vec![-1.0], vec![])
    );
    assert!(sight.is_monster(9) && !sight.is_monster(11) && !sight.is_monster(100));
}

#[test]
fn hides_the_hidden() {
    let mut sight = sight();
    sight.actors[1].hidden = true;
    assert_eq!(sight.position(9, 0), (-1, -1));
    assert!(!sight.is_monster(9));
    assert_eq!(sight.attacks(7, 9, 0.0), None);
    assert!(hidden(0x02) && hidden(0x04) && hidden(0x40) && !hidden(0x08));
}

#[test]
fn moves_a_place_told_by_the_reach_of_a_skill() {
    let mut sight = sight();
    sight.actors[1].cell = (60, 50);
    // The monster is seven cells east of the Lif: with a skill that reaches nine, the cell told is nine cells
    // towards the Lif along the row (the angle is a half turn: its cosine -1, its sine 0).
    assert_eq!(sight.position(9, 8013), (51, 50));
    assert_eq!(sight.position(9, 0), (60, 50));
    // The player and the player's other companions are told where they are.
    sight.friends = vec![11];
    assert_eq!((sight.position(100, 8013), sight.position(11, 8013)), ((50, 50), (40, 40)));
    // A cell that cannot be walked on is put right: the first walkable one about it, in the client's order.
    sight.walkable = Some(Rc::new(|x, _| x != 51));
    assert_eq!(sight.position(9, 8013), (52, 50));
}

#[test]
fn lets_through_what_the_client_lets_through() {
    let sight = sight();
    assert_eq!(sight.moves(7, (60, 60)), Some(Order::Move(7, (60, 60))));
    // Twenty-five cells and more, counted along both ways, is too far; and only its own one is moved.
    assert_eq!((sight.moves(7, (66, 62)), sight.moves(9, (58, 55))), (None, None));
    // A blow is not asked for sooner than its delay after the last.
    assert_eq!((sight.attacks(7, 9, 9_000.0), sight.attacks(7, 9, 9_400.0), sight.attacks(7, 12, 0.0)), (Some(Order::Attack(7, 9)), None, None));
    assert_eq!(sight.skill_on(7, 3, 8013, 9), (Some(Order::SkillOn { level: 3, skill: 8013, target: 9 }), false));
    assert_eq!(sight.skill_on(7, 3, 8001, 9), (None, false));
    assert_eq!(sight.skill_at(7, 2, 8013, (55, 55)), Some(Order::SkillAt { level: 2, skill: 8013, cell: (55, 55) }));
    // In the middle of a blow it uses no skill.
    let mut striking = sight.clone();
    striking.actors[0].motion = motion::ATTACK;
    assert_eq!((striking.skill_on(7, 3, 8013, 9).0, striking.skill_at(7, 2, 8013, (55, 55))), (None, None));
}

#[test]
fn finds_the_walkable_cell_towards_the_player() {
    let mut sight = sight();
    // A wall four cells thick east of the player: from beyond it the first cell before it is found.
    sight.walkable = Some(Rc::new(|x, _| !(54..=57).contains(&x)));
    assert_eq!(sight.closest_walkable((56, 50)), Some((53, 50)));
    assert_eq!(sight.moves(7, (56, 50)), Some(Order::Move(7, (53, 50))));
    // From the player's own cell there is no line to walk.
    assert_eq!(sight.closest_walkable((50, 50)), None);
}

#[test]
fn writes_a_command_as_the_script_reads_it() {
    assert_eq!(Command { kind: command::MOVE, cell: (10, 20), ..Command::default() }.row(), [1.0, 10.0, 20.0]);
    assert_eq!(Command { kind: command::ATTACK_OBJECT, target: 9, ..Command::default() }.row(), [3.0, 9.0]);
    assert_eq!(Command { kind: command::SKILL_OBJECT, level: 2, skill: 8013, target: 9, ..Command::default() }.row(), [7.0, 2.0, 8013.0, 9.0]);
    assert_eq!(Command { kind: command::SKILL_AREA, level: 2, skill: 8013, cell: (5, 6), ..Command::default() }.row(), [8.0, 2.0, 8013.0, 5.0, 6.0]);
    assert_eq!((Command { kind: command::FOLLOW, ..Command::default() }.row(), Command::default().row()), (vec![9.0], vec![0.0]));
    assert_eq!((host::path(b"AI\\Const"), host::path(b"./AI/USER_AI/Const_.lua")), ("ai/const.lua".to_owned(), "ai/user_ai/const_.lua".to_owned()));
}

/// A script of this test's own that uses what the client's scripts use.
fn files() -> Files {
    let script = br#"
        require "AI\\Util"
        require "AI\\Util"
        Calls = 0
        function AI(id)
            Calls = Calls + 1
            local msg = GetMsg(id)
            local reserved = GetResMsg(id)
            if msg[1] == 1 then Move(id, msg[2], msg[3]) end
            if msg[1] == 9 then MoveToOwner(id) end
            if reserved[1] == 3 then Attack(id, reserved[2]) end
            for i, v in ipairs(GetActors()) do
                if IsMonster(v) == 1 and GetV(5, v) == GetV(0, id) then
                    local x, y = GetV(1, v)
                    TraceValue(Loaded .. " " .. v .. " " .. x .. "," .. y .. " " .. GetTick())
                end
            end
            TraceAI("called")
        end
    "#;
    Files::from([("ai/ai.lua".to_owned(), script.to_vec()), ("ai/util.lua".to_owned(), b"Loaded = (Loaded or 0) + 1".to_vec())])
}

#[test]
fn runs_a_script_through_the_clients_functions() {
    let mut ai = Ai::load(Kind::Homunculus, false, files()).unwrap();
    // `require` ran its file once; the monster that struck the player is told of with its place.
    assert_eq!(ai.call(sight()).unwrap(), [Order::Say("1 9 58,54 10000".to_owned())]);
    // A command is read once.
    ai.tell(Command { kind: command::MOVE, cell: (55, 51), ..Command::default() }, false);
    ai.tell(Command { kind: command::ATTACK_OBJECT, target: 9, ..Command::default() }, true);
    let orders = ai.call(sight()).unwrap();
    assert_eq!(&orders[..2], [Order::Move(7, (55, 51)), Order::Attack(7, 9)]);
    assert_eq!(ai.call(sight()).unwrap().len(), 1);
    ai.tell(Command { kind: command::FOLLOW, ..Command::default() }, false);
    assert_eq!(ai.call(sight()).unwrap()[0], Order::ToOwner(7));
}

#[test]
fn is_called_every_140_milliseconds() {
    let mut ai = Ai::load(Kind::Homunculus, false, files()).unwrap();
    assert_eq!((ai.due(1000.0), ai.due(1100.0), ai.due(1139.0), ai.due(1140.0), ai.due(1200.0)), (true, false, false, true, false));
}

#[test]
fn tells_what_is_wrong_with_a_script() {
    let broken = |script: &[u8]| Files::from([("ai/ai.lua".to_owned(), script.to_vec())]);
    let wrong = |kind: Kind, files: Files| Ai::load(kind, false, files).err().unwrap_or_default();
    assert!(!wrong(Kind::Homunculus, broken(b"function AI(")).is_empty());
    assert!(wrong(Kind::Homunculus, broken(b"require 'AI\\\\Gone'")).contains("ai/gone.lua"));
    assert!(wrong(Kind::Mercenary, files()).contains("ai/ai_m.lua"));
    // A script without `AI`, one that fails, and one that never ends are errors of the call, not a hang.
    assert!(Ai::load(Kind::Homunculus, false, broken(b"x = 1")).unwrap().call(sight()).is_err());
    assert!(Ai::load(Kind::Homunculus, false, broken(b"function AI(id) local t = nil; return t.x end")).unwrap().call(sight()).is_err());
    assert!(
        Ai::load(Kind::Homunculus, false, broken(b"function AI(id) while true do end end")).unwrap().call(sight()).unwrap_err().contains(
            "does not come to an end"
        )
    );
}

#[test]
fn gives_a_players_script_what_lua_5_1_has() {
    // A script in the manner of the players' own: Lua 5.1's names, a file of the folder read, notes written
    // and read back, the table of globals.
    let script = br#"
        dofile("./AI/USER_AI/Config.lua")
        local notes = io.open("./AI/USER_AI/data/notes.txt", "w")
        notes:write("seen ", table.getn({ 1, 2, 3 }), "\n", math.mod(7, 4))
        notes:close()
        function AI(id)
            local first, second = unpack({ Greeting, _G.Greeting })
            local file = io.open("AI/USER_AI/Config.lua", "r")
            local size = file:seek("end")
            file:close()
            local lines = {}
            for line in io.lines("./AI/USER_AI/data/notes.txt") do table.insert(lines, line) end
            local missing = io.open("./AI/USER_AI/nothing.txt", "r")
            TraceValue(first .. second .. " " .. size .. " " .. lines[1] .. "|" .. lines[2] .. " " .. tostring(missing) .. " " .. string.gfind("a1b22", "%d+")())
        end
    "#;
    let files = Files::from([("ai/user_ai/ai.lua".to_owned(), script.to_vec()), ("ai/user_ai/config.lua".to_owned(), b"Greeting = 'hi'".to_vec())]);
    let mut ai = Ai::load(Kind::Homunculus, true, files).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(ai.call(sight()).unwrap(), [Order::Say("hihi 15 seen 3|3 nil 1".to_owned())]);
}

/// The client's own scripts, read from the client's folder beside this project (they are not in the
/// repository); none when the folder is not there.
fn clients_files() -> Option<Files> {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Feb-Client-CLEAN/AI");
    let mut files = Files::new();
    for name in ["AI.lua", "AI_M.lua", "Const.lua", "Util.lua"] {
        files.insert(format!("ai/{}", name.to_lowercase()), std::fs::read(folder.join(name)).ok()?);
    }
    Some(files)
}

#[test]
fn the_clients_own_script_chases_who_strikes_its_owner() {
    let Some(files) = clients_files() else { return };
    let mut ai = Ai::load(Kind::Homunculus, false, files).unwrap();
    let mut sight = sight();
    // Idle: it sees the monster that struck its owner. Chasing: it walks to it.
    assert_eq!(ai.call(sight.clone()).unwrap(), []);
    assert_eq!(ai.call(sight.clone()).unwrap(), [Order::Move(7, (58, 54))]);
    // Beside it, it stops chasing and strikes: once, then not again before its delay is over.
    sight.actors[0].cell = (57, 54);
    assert_eq!(ai.call(sight.clone()).unwrap(), []);
    assert_eq!(ai.call(sight.clone()).unwrap(), [Order::Attack(7, 9)]);
    sight.tick += 140.0;
    assert_eq!(ai.call(sight.clone()).unwrap(), []);
    sight.tick += 700.0;
    assert_eq!(ai.call(sight.clone()).unwrap(), [Order::Attack(7, 9)]);
    // The monster dead, it idles; far from its owner, it follows.
    sight.actors.remove(1);
    assert_eq!(ai.call(sight.clone()).unwrap(), []);
    assert_eq!(ai.call(sight.clone()).unwrap(), []);
    assert_eq!(ai.call(sight.clone()).unwrap(), [Order::ToOwner(7)]);
}

#[test]
fn the_clients_own_script_does_as_it_is_told() {
    let Some(files) = clients_files() else { return };
    let mut ai = Ai::load(Kind::Mercenary, false, files).unwrap();
    let mut sight = sight();
    sight.actors[0].job = 6017;
    sight.actors.remove(1);
    // Alt+T: it comes to its owner and stands by; told to walk, it walks.
    ai.tell(Command { kind: command::FOLLOW, ..Command::default() }, false);
    assert_eq!(ai.call(sight.clone()).unwrap(), [Order::ToOwner(7)]);
    ai.tell(Command { kind: command::MOVE, cell: (56, 52), ..Command::default() }, false);
    assert_eq!(ai.call(sight.clone()).unwrap(), [Order::Move(7, (56, 52))]);
    // Told to use a skill on someone in its reach, it uses it.
    sight.skills = vec![(8207, 5, 9)];
    sight.actors.push(Seen { job: 1002, enemy: true, ..seen(9, (55, 50)) });
    ai.tell(Command { kind: command::SKILL_OBJECT, level: 5, skill: 8207, target: 9, ..Command::default() }, false);
    assert_eq!(ai.call(sight.clone()).unwrap(), []);
    assert_eq!(ai.call(sight.clone()).unwrap(), [Order::SkillOn { level: 5, skill: 8207, target: 9 }]);
}

/// Every Lua file under the client's `AI` folder, the players' own among them, keyed as the gateway keys them.
fn every_file(folder: &std::path::Path, under: &str, files: &mut Files) {
    for entry in std::fs::read_dir(folder).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if entry.path().is_dir() {
            every_file(&entry.path(), &format!("{under}/{name}"), files);
        } else if name.ends_with(".lua") {
            files.insert(format!("{under}/{name}"), std::fs::read(entry.path()).unwrap_or_default());
        }
    }
}

#[test]
fn a_players_own_script_beside_the_client_loads_and_is_called() {
    // The folder this client is developed against has a script of a player's under `USER_AI` (AzzyAI): three
    // thousand lines that read their settings, keep notes in files and use Lua 5.1's older names.
    let mut files = Files::new();
    every_file(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Feb-Client-CLEAN/AI"), "ai", &mut files);
    if !files.contains_key("ai/user_ai/ai.lua") {
        return;
    }
    let mut ai = Ai::load(Kind::Homunculus, true, files).unwrap_or_else(|error| panic!("{error}"));
    let mut sight = sight();
    for _ in 0..20 {
        sight.tick += 140.0;
        if let Err(error) = ai.call(sight.clone()) {
            panic!("{error}");
        }
    }
}
