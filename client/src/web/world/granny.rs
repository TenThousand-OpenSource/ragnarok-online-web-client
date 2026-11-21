//! Those on the map that the client draws as Granny models, not as sprites (`C3dGrannyGameActor`): the guild
//! flags, the Emperium, the guardians and the treasure boxes of a castle. Their files are read as they are
//! first wanted; every frame each one's skin is moved by its bones for the action it is in, stood on its
//! place, lit, and handed to the GPU (`gpu/granny.rs`). The rules are `crate::model::granny::shown`'s.

use super::*;
use crate::vector::add;
use crate::model::granny::shown::{self, Action, Seen};
use crate::model::granny::{Animation, Granny};

/// The models read so far and what each one in sight is doing.
#[derive(Default)]
pub(super) struct Models {
    /// By name, as the table of jobs has it: asked for and not come yet (or not to be had), or read.
    read: HashMap<String, Option<Rc<Model>>>,
    /// The animations of the other actions, by the model's name: none for one the client has no file of.
    actions: HashMap<(String, Action), Option<Animation>>,
    /// By the server's ID: the action one is in, and when it began.
    doing: HashMap<u32, (Action, f64)>,
}

pub(super) struct Model {
    pub(super) granny: Granny,
}

/// A picture a mesh is drawn with: one of a model's own, or a guild's emblem.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) enum Picture {
    Own(String, usize),
    Emblem(u32, u32),
}

/// This frame's corners (a place, where of the picture, a colour: nine numbers each), and which of them are
/// drawn with which picture.
#[derive(Default)]
pub(super) struct Corners {
    pub(super) numbers: Vec<f32>,
    pub(super) batches: Vec<(Picture, std::ops::Range<u32>)>,
}

pub(super) use shown::FLAG;

/// The look of one whose name in the table of jobs is a Granny model's: it is drawn as that model
/// (`CGameMode::NewActor`), which is asked for.
pub(super) fn look(models: &mut Models, name: String) -> Look {
    wanted(models, &name);
    Look::Model(name)
}

/// A model is wanted: its file is asked for the first time, and with it the files of its other actions.
fn wanted(models: &mut Models, name: &str) {
    if models.read.contains_key(name) {
        return;
    }
    models.read.insert(name.to_owned(), None);
    let asked = name.to_owned();
    wasm_bindgen_futures::spawn_local(async move {
        let read = fetch_asset(&shown::file(&asked)).await.and_then(|bytes| Granny::read(&bytes));
        match read {
            Ok(granny) => STATE.with_borrow_mut(|state| state.granny.read.insert(asked.clone(), Some(Rc::new(Model { granny })))),
            Err(error) => return web_sys::console::warn_1(&format!("model {asked}: {error}").into()),
        };
        for action in Action::OTHERS {
            let Some(file) = shown::action_file(&asked, action) else { continue };
            // (One the client has no file of plays nothing: `PlayAnimation` gives no control.)
            let animation = fetch_asset(&file).await.ok().and_then(|bytes| Granny::read(&bytes).ok()).and_then(|granny| granny.animation);
            STATE.with_borrow_mut(|state| state.granny.actions.insert((asked.clone(), action), animation));
        }
    });
}

impl Models {
    /// How long a model's action takes on the screen, in milliseconds, once its file is read.
    pub(super) fn lasts(&self, name: &str, action: Action) -> Option<f64> {
        self.actions.get(&(name.to_owned(), action))?.as_ref().map(|animation| shown::lasts(animation.duration))
    }

    pub(super) fn model(&self, name: &str) -> Option<&Rc<Model>> {
        self.read.get(name)?.as_ref()
    }

    /// The bones of a model for the action one is in: its own animation while it stands, the action's
    /// file otherwise; as it rests for an action the client has no file of.
    fn pose(&self, name: &str, model: &Model, (action, since): (Action, f64), time: f64) -> crate::model::granny::Pose {
        let animation = match action {
            Action::Stand => model.granny.animation.as_ref(),
            other => self.actions.get(&(name.to_owned(), other)).and_then(Option::as_ref),
        };
        model.granny.pose(animation.map(|animation| (animation, animation.time(shown::played(time - since), action.looped()))))
    }
}

/// The action a model is in, by what the world has of the one it shows (the client's `SetAction`, by the
/// states of its actor): fallen, struck or striking, walking, or standing.
fn action_of(unit: &Mover, time: f64) -> Action {
    match unit.motion.filter(|(_, began, lasts)| *began <= time && time < began + lasts) {
        _ if unit.dying.is_some() => Action::Dead,
        Some((16, ..)) => Action::Attack,
        Some((24, ..)) => Action::Damage,
        _ if unit.walk.is_some() => Action::Move,
        _ => Action::Stand,
    }
}

/// Every model in sight as it is now, put on the GPU for this frame's drawing.
pub(super) fn drawn(state: &mut State, view: &View, time: f64) {
    let (Some(gpu), Some(scene)) = (&mut state.gpu, &state.scene) else { return };
    let middle = [scene.gat.width as f32 * CELL / 2.0, 0.0, scene.gat.height as f32 * CELL / 2.0];
    let seen = Seen { eye: view.eye, right: view.right, up: view.up, forward: view.forward, middle };
    let models = &mut state.granny;
    models.doing.retain(|id, _| scene.units.contains_key(id));
    let mut corners = Corners::default();
    for (id, unit) in &scene.units {
        let Look::Model(name) = &unit.look else { continue };
        let Some(model) = models.model(name).cloned() else { continue };
        if unit.hidden() {
            continue;
        }
        // An action begins when the one before it ends; one fallen began to fall so long before it lay.
        let action = action_of(unit, time);
        let began = match (action, unit.dying) {
            (Action::Dead, Some(fallen)) => fallen - models.lasts(name, Action::Dead).unwrap_or(0.0),
            _ => time,
        };
        // (A blow is played to its end before the model stands again: `ProcessMotion`.)
        let blow = models.lasts(name, Action::Attack).unwrap_or(0.0);
        let striking = |doing: &(Action, f64)| doing.0 == Action::Attack && action == Action::Stand && time - doing.1 < blow;
        let doing = match models.doing.get(id).filter(|doing| doing.0 == action || striking(doing)) {
            Some(doing) => *doing,
            None => (action, began),
        };
        models.doing.insert(*id, doing);
        let pose = models.pose(name, &model, doing, time);
        let emblem = Some(unit.guild).filter(|guild| state.guilds.emblem(*guild).is_some());
        let (feet, light) = (on_ground(&scene.gat, unit.at), shown::light(unit.job, &seen));
        let stand = Standing { name, unit, feet, light, left: left_of(unit, false, time), emblem };
        stand.meshes(&model.granny, &pose, &mut corners);
    }
    gpu.granny_loaded(&state.granny, &state.guilds, corners);
}

/// One model to be drawn: whose it is, where its feet are, the light on it and how much of it is left to see.
struct Standing<'a> {
    name: &'a str,
    unit: &'a Mover,
    feet: Vector,
    light: Vector,
    left: f32,
    emblem: Option<(u32, u32)>,
}

impl Standing<'_> {
    /// The corners of its meshes. The flag's second mesh shows its guild's emblem, and is not drawn
    /// without one; another mesh with no picture is not drawn at all (`RenderShadowed`, `RenderLit`).
    fn meshes(&self, granny: &Granny, pose: &crate::model::granny::Pose, corners: &mut Corners) {
        let base = add(self.feet, shown::offset(self.unit.job));
        let direction = crate::pose::direction_of(self.unit.facing);
        for (index, mesh) in granny.meshes.iter().enumerate() {
            let picture = match (self.unit.job == shown::FLAG && index == 1, mesh.texture) {
                (true, _) => self.emblem.map(|(guild, version)| Picture::Emblem(guild, version)),
                (false, Some(texture)) => Some(Picture::Own(self.name.to_owned(), texture)),
                (false, None) => None,
            };
            let Some(picture) = picture else { continue };
            let skin = granny.skin(mesh, pose);
            let first = (corners.numbers.len() / 9) as u32;
            for &corner in &mesh.indices {
                let ((position, normal), vertex) = (skin[corner as usize], &mesh.vertices[corner as usize]);
                let grey = f32::from(shown::grey(self.unit.job, self.light, normal)) / 255.0;
                corners.numbers.extend(add(base, shown::stood(self.name, direction, position)));
                corners.numbers.extend(vertex.uv);
                corners.numbers.extend([grey, grey, grey, self.left]);
            }
            corners.batches.push((picture, first..(corners.numbers.len() / 9) as u32));
        }
    }
}

/// Where on the page the feet of a model are that a point of the page is on: the client's rectangle for one
/// is a square about its feet (`shown::REACH`), not the model's own outline.
pub(super) fn touched(scene: &Scene, view: &View, unit: &Mover, (width, height): (f32, f32), pointer: (f32, f32)) -> Option<(f32, f32)> {
    let feet = on_ground(&scene.gat, unit.at);
    let corner = |across: f32, up: f32| view.project(along(along(feet, view.right, across * shown::REACH), view.up, up * shown::REACH));
    let (low, high, under) = (corner(-1.0, 1.0)?, corner(1.0, -1.0)?, view.project(feet)?);
    let inside = (low.0 * width..=high.0 * width).contains(&pointer.0) && (low.1 * height..=high.1 * height).contains(&pointer.1);
    inside.then_some((under.0 * width, under.1 * height))
}
