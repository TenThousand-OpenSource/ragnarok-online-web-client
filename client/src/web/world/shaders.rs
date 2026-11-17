//! The WGSL programs the world is drawn with: the ground, the models, the water, the sprites and the effects.

pub(super) const GROUND_SHADER: &str = "
struct Camera { view_projection: mat4x4<f32> }
@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var ground: texture_2d<f32>;
@group(1) @binding(1) var ground_sampler: sampler;
@group(1) @binding(2) var light: texture_2d<f32>;

struct In { @location(0) position: vec3<f32>, @location(1) uv: vec2<f32>, @location(2) light_uv: vec2<f32>, @location(3) colour: vec3<f32> }
struct Out { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32>, @location(1) light_uv: vec2<f32>, @location(2) colour: vec3<f32> }

@vertex fn vs(in: In) -> Out {
    var out: Out;
    out.position = camera.view_projection * vec4<f32>(in.position, 1.0);
    out.uv = in.uv;
    out.light_uv = in.light_uv;
    out.colour = in.colour;
    return out;
}

@fragment fn fs(in: Out) -> @location(0) vec4<f32> {
    let texel = textureSample(ground, ground_sampler, in.uv);
    let baked = textureSample(light, ground_sampler, in.light_uv);
    if (texel.a < 0.5) { discard; }
    // The texture, tinted, darkened by the baked shadow, and lit by the baked light's colour.
    return vec4<f32>(texel.rgb * in.colour * baked.a + baked.rgb, 1.0);
}
";

pub(super) const MODEL_SHADER: &str = "
struct Camera { view_projection: mat4x4<f32> }
@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var picture: texture_2d<f32>;
@group(1) @binding(1) var picture_sampler: sampler;

struct Out { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32>, @location(1) colour: vec3<f32> }

@vertex fn vs(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>, @location(2) colour: vec3<f32>) -> Out {
    var out: Out;
    out.position = camera.view_projection * vec4<f32>(position, 1.0);
    out.uv = uv;
    out.colour = colour;
    return out;
}

@fragment fn fs(in: Out) -> @location(0) vec4<f32> {
    let texel = textureSample(picture, picture_sampler, in.uv);
    if (texel.a < 0.5) { discard; }
    return vec4<f32>(texel.rgb * in.colour, 1.0);
}
";

/// The water lets the ground under it show through.
pub(super) const WATER_SHADER: &str = "
struct Camera { view_projection: mat4x4<f32> }
@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var picture: texture_2d<f32>;
@group(1) @binding(1) var picture_sampler: sampler;

struct Out { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> }

@vertex fn vs(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>) -> Out {
    var out: Out;
    out.position = camera.view_projection * vec4<f32>(position, 1.0);
    out.uv = uv;
    return out;
}

@fragment fn fs(in: Out) -> @location(0) vec4<f32> {
    return vec4<f32>(textureSample(picture, picture_sampler, in.uv).rgb, 0.56);
}
";

pub(super) const SPRITE_SHADER: &str = "
struct Camera { view_projection: mat4x4<f32> }
@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var picture: texture_2d<f32>;
@group(1) @binding(1) var picture_sampler: sampler;

struct Out { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> }

@vertex fn vs(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>) -> Out {
    var out: Out;
    out.position = camera.view_projection * vec4<f32>(position, 1.0);
    out.uv = uv;
    return out;
}

// How see-through a quad is rides in its v coordinate: every 2 added to it is a sixteenth less of the picture.
@fragment fn fs(in: Out) -> @location(0) vec4<f32> {
    let faded = floor(in.uv.y / 2.0);
    var texel = textureSample(picture, picture_sampler, vec2<f32>(in.uv.x, in.uv.y - faded * 2.0));
    texel.a = texel.a * (1.0 - faded / 16.0);
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
