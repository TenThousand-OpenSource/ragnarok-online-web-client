//! The WGSL programs the world is drawn with: the ground, the models, the water, the sprites and the effects.

/// What the programs of the map (the ground, the models, the water) are told every frame besides the camera's
/// matrix (`gpu/land.rs` writes it; the sprites' and the effects' programs read the matrix alone): where the
/// eye is, the fog (where it begins and ends, whether it shows; its colour), how much of the map's light the
/// night leaves, and the water's waves (how far their turn is, the degrees between a cube and the next, their
/// height; the water's colour and how much it hides). `fogged` is a colour as the fog leaves it at a depth
/// of the view: even from the fog's start to its end (the client's `DepthAlpha`, by which it fogs every
/// corner it draws; a corner's depth is the last of its place as the camera's matrix makes it).
macro_rules! frame {
    () => {
        "
struct Camera {
    view_projection: mat4x4<f32>,
    eye: vec4<f32>,
    fog: vec4<f32>,
    fog_colour: vec4<f32>,
    light: vec4<f32>,
    water: vec4<f32>,
    water_colour: vec4<f32>,
}
@group(0) @binding(0) var<uniform> camera: Camera;

fn fogged(colour: vec3<f32>, depth: f32) -> vec3<f32> {
    let clear = clamp((camera.fog.y - depth) / max(camera.fog.y - camera.fog.x, 0.001), 0.0, 1.0);
    return mix(colour, mix(camera.fog_colour.rgb, colour, clear), camera.fog.z);
}
"
    };
}

pub(super) const GROUND_SHADER: &str = concat!(
    frame!(),
    "
@group(1) @binding(0) var ground: texture_2d<f32>;
@group(1) @binding(1) var ground_sampler: sampler;
@group(1) @binding(2) var light: texture_2d<f32>;

struct In { @location(0) position: vec3<f32>, @location(1) uv: vec2<f32>, @location(2) light_uv: vec2<f32>, @location(3) colour: vec3<f32> }
struct Out { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32>, @location(1) light_uv: vec2<f32>, @location(2) colour: vec3<f32>, @location(3) depth: f32 }

@vertex fn vs(in: In) -> Out {
    var out: Out;
    out.position = camera.view_projection * vec4<f32>(in.position, 1.0);
    out.uv = in.uv;
    out.light_uv = in.light_uv;
    out.colour = in.colour;
    out.depth = out.position.w;
    return out;
}

@fragment fn fs(in: Out) -> @location(0) vec4<f32> {
    let texel = textureSample(ground, ground_sampler, in.uv);
    let baked = textureSample(light, ground_sampler, in.light_uv);
    if (texel.a < 0.5) { discard; }
    // The texture, tinted, darkened by the baked shadow, and lit by the baked light's colour; then what the
    // night leaves of it, and the fog over it.
    return vec4<f32>(fogged((texel.rgb * in.colour * baked.a + baked.rgb) * camera.light.rgb, in.depth), 1.0);
}
"
);

/// The models' program. A face that shows from one side alone (the client's `twoSided` not set) is not drawn
/// seen from behind: its corners carry the way it looks out, nothing for one that shows from both. `$cut` is
/// the alpha under which a point of the picture is left out: half for a picture cut out by its colour, next
/// to nothing for one mixed in by its alpha.
macro_rules! model_shader {
    ($cut:literal) => {
        concat!(
            frame!(),
            "
@group(1) @binding(0) var picture: texture_2d<f32>;
@group(1) @binding(1) var picture_sampler: sampler;

struct Out { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32>, @location(1) colour: vec3<f32>, @location(2) outward: vec3<f32>, @location(3) world: vec3<f32>, @location(4) depth: f32 }

@vertex fn vs(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>, @location(2) colour: vec3<f32>, @location(3) outward: vec3<f32>) -> Out {
    var out: Out;
    out.position = camera.view_projection * vec4<f32>(position, 1.0);
    out.uv = uv;
    out.colour = colour;
    out.outward = outward;
    out.world = position;
    out.depth = out.position.w;
    return out;
}

@fragment fn fs(in: Out) -> @location(0) vec4<f32> {
    let texel = textureSample(picture, picture_sampler, in.uv);
    if (texel.a < ",
            $cut,
            ") { discard; }
    if (dot(in.outward, in.outward) > 0.25 && dot(in.outward, camera.eye.xyz - in.world) < 0.0) { discard; }
    return vec4<f32>(fogged(texel.rgb * in.colour * camera.light.rgb, in.depth), texel.a);
}
"
        )
    };
}

pub(super) const MODEL_SHADER: &str = model_shader!("0.5");
pub(super) const MODEL_BLENDING_SHADER: &str = model_shader!("0.02");

/// The water: each corner is lifted by its wave (`GroundGeometry::DrawWaterCell`: the sine of whole degrees,
/// the corner's number times the pitch and the turn so far), and lets the ground under it show through.
pub(super) const WATER_SHADER: &str = concat!(
    frame!(),
    "
@group(1) @binding(0) var picture: texture_2d<f32>;
@group(1) @binding(1) var picture_sampler: sampler;

struct Out { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32>, @location(1) depth: f32 }

@vertex fn vs(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>, @location(2) wave: f32) -> Out {
    var out: Out;
    let turn = radians(trunc(wave * camera.water.y + camera.water.x));
    let lifted = vec3<f32>(position.x, position.y - sin(turn) * camera.water.z, position.z);
    out.position = camera.view_projection * vec4<f32>(lifted, 1.0);
    out.uv = uv;
    out.depth = out.position.w;
    return out;
}

@fragment fn fs(in: Out) -> @location(0) vec4<f32> {
    return vec4<f32>(fogged(textureSample(picture, picture_sampler, in.uv).rgb * camera.water_colour.rgb, in.depth), camera.water_colour.a);
}
"
);

/// The sprites' program: a picture, its colours multiplied by the colour of its corners (white for a picture
/// as it is; the colour a state or an effect gives a body, and how see-through it is: `crate::body`).
pub(super) const SPRITE_SHADER: &str = "
struct Camera { view_projection: mat4x4<f32> }
@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var picture: texture_2d<f32>;
@group(1) @binding(1) var picture_sampler: sampler;

struct Out { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32>, @location(1) colour: vec4<f32> }

@vertex fn vs(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>, @location(2) colour: vec4<f32>) -> Out {
    var out: Out;
    out.position = camera.view_projection * vec4<f32>(position, 1.0);
    out.uv = uv;
    out.colour = colour;
    return out;
}

@fragment fn fs(in: Out) -> @location(0) vec4<f32> {
    let texel = textureSample(picture, picture_sampler, in.uv) * in.colour;
    if (texel.a < 0.02) { discard; }
    return texel;
}
";

/// The client's effects (`.str`): textured quads, each tinted and as see-through as its corners say.
pub(super) const EFFECT_SHADER: &str = "
struct Camera { view_projection: mat4x4<f32> }
@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var picture: texture_2d<f32>;
@group(1) @binding(1) var picture_sampler: sampler;

struct Out { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32>, @location(1) colour: vec4<f32> }

@vertex fn vs(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>, @location(2) colour: vec4<f32>) -> Out {
    var out: Out;
    out.position = camera.view_projection * vec4<f32>(position, 1.0);
    out.uv = uv;
    out.colour = colour;
    return out;
}

@fragment fn fs(in: Out) -> @location(0) vec4<f32> {
    let texel = textureSample(picture, picture_sampler, in.uv) * in.colour;
    if (texel.a < 0.004) { discard; }
    return texel;
}
";
