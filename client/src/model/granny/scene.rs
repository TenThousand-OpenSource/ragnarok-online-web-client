//! What a Granny file holds, read out of its own description: the textures, the skeleton and the meshes of
//! its first model (the only one the client takes: `C3dGrannyModelRes::Load`), and its animation.
//!
//! The names of the members are the file's own (Granny 2.1's `granny_file_info` and what hangs on it).

use super::types::Object;
use super::*;

pub(super) fn read(image: &file::Image) -> Result<Granny, String> {
    let info = Object { image, kind: image.root_type, at: image.root };
    let textures = info.objects("Textures")?;
    let Some(model) = info.objects("Models")?.into_iter().next() else {
        return Ok(Granny { textures: Vec::new(), bones: Vec::new(), meshes: Vec::new(), animation: animation(&info, "")? });
    };
    let bones = match model.object("Skeleton")? {
        Some(skeleton) => skeleton.objects("Bones")?.iter().map(bone).collect::<Result<Vec<_>, _>>()?,
        None => Vec::new(),
    };
    let mut meshes = Vec::new();
    for binding in model.objects("MeshBindings")? {
        if let Some(found) = binding.object("Mesh")? {
            meshes.push(mesh(&found, &bones, &textures)?);
        }
    }
    let textures = textures.iter().map(texture).collect::<Result<Vec<_>, _>>()?;
    Ok(Granny { textures, bones, meshes, animation: animation(&info, &model.text("Name")?)? })
}

/// A texture: the client takes only those of one image (`textureType == 0 && imageCount == 1`) and their
/// largest level; another kind is left empty, as the client leaves it with no texture.
fn texture(texture: &Object) -> Result<Texture, String> {
    let (width, height) = (texture.number("Width")? as usize, texture.number("Height")? as usize);
    let empty = Texture { width: 0, height: 0, pixels: Vec::new() };
    let images = texture.objects("Images")?;
    if texture.number("TextureType")? != 0.0 || images.len() != 1 {
        return Ok(empty);
    }
    let Some(level) = images[0].objects("MIPLevels")?.into_iter().next() else { return Ok(empty) };
    let packed: Vec<u8> = level.numbers("Pixels")?.into_iter().map(|byte| byte as u8).collect();
    let alpha = match texture.object("Layout")? {
        Some(layout) => layout.numbers("BitsForComponent")?.get(3).is_some_and(|&bits| bits != 0.0),
        None => false,
    };
    let pixels = match texture.number("Encoding")? as u32 {
        RAW if alpha => packed.get(..width * height * 4).ok_or("a texture with too few pixels")?.to_vec(),
        BINK => bink::unpack(&packed, width, height, alpha)?,
        other => return Err(format!("a texture kept in a way not read here ({other})")),
    };
    Ok(Texture { width, height, pixels })
}

/// How a texture's pixels are kept (Granny's `texture_encoding`): as they are, or packed as Bink packs.
/// (2 is S3TC; none of the client's files has it or pixels as they are without an alpha, and neither is
/// read here.)
const RAW: u32 = 1;
const BINK: u32 = 3;

fn bone(bone: &Object) -> Result<Bone, String> {
    let inverse: Vec<f32> = bone.numbers("InverseWorldTransform")?.into_iter().map(|number| number as f32).collect();
    Ok(Bone {
        name: bone.text("Name")?,
        parent: usize::try_from(bone.number("ParentIndex")? as i64).ok(),
        rest: bone.transform("Transform")?,
        from_world: inverse.try_into().map_err(|_| "a bone with no way back from the world")?,
    })
}

fn mesh(mesh: &Object, bones: &[Bone], textures: &[Object]) -> Result<Mesh, String> {
    let vertices = match mesh.object("PrimaryVertexData")? {
        Some(data) => data.objects("Vertices")?.iter().map(vertex).collect::<Result<Vec<_>, _>>()?,
        None => Vec::new(),
    };
    let (mut indices, mut material) = (Vec::new(), 0);
    if let Some(topology) = mesh.object("PrimaryTopology")? {
        indices = topology.numbers("Indices")?;
        if indices.is_empty() {
            indices = topology.numbers("Indices16")?;
        }
        // The client draws every triangle with the material of the first group (`GetMeshRenderData`).
        if let Some(group) = topology.objects("Groups")?.first() {
            material = group.number("MaterialIndex")? as usize;
        }
    }
    let indices: Vec<u32> = indices.into_iter().map(|index| index as u32).collect();
    if indices.iter().any(|&index| index as usize >= vertices.len()) {
        return Err("a triangle of a corner that is none".to_owned());
    }
    let bound = mesh.objects("BoneBindings")?;
    let named = |binding: &Object| Ok::<_, String>(bones.iter().position(|bone| Ok(&bone.name) == binding.text("BoneName").as_ref()).unwrap_or(0));
    let texture = match mesh.objects("MaterialBindings")?.get(material).map(|binding| binding.object("Material")).transpose()?.flatten() {
        Some(material) => diffuse(&material, 0)?.and_then(|texture| textures.iter().position(|other| other.at == texture.at)),
        None => None,
    };
    Ok(Mesh {
        name: mesh.text("Name")?,
        rigid: !vertices.iter().any(|vertex| vertex.weights != [0; 4]),
        vertices,
        indices,
        texture,
        bones: bound.iter().map(named).collect::<Result<_, _>>()?,
    })
}

/// The texture of a material's colour (the client asks `GrannyGetMaterialTextureByType` for the diffuse
/// one): its own, or that of the map it uses as "Diffuse Color".
fn diffuse<'a>(material: &Object<'a>, depth: usize) -> Result<Option<Object<'a>>, String> {
    if let Some(texture) = material.object("Texture")? {
        return Ok(Some(texture));
    }
    for map in material.objects("Maps")? {
        if depth < 4 && map.text("Usage")? == "Diffuse Color" {
            if let Some(inner) = map.object("Map")? {
                return diffuse(&inner, depth + 1);
            }
        }
    }
    Ok(None)
}

fn vertex(vertex: &Object) -> Result<Vertex, String> {
    let floats = |name: &str| Ok::<Vec<f32>, String>(vertex.numbers(name)?.into_iter().map(|number| number as f32).collect());
    let bytes = |name: &str| -> Result<[u8; 4], String> {
        let mut bytes = [0; 4];
        for (byte, number) in bytes.iter_mut().zip(vertex.numbers(name)?) {
            *byte = number as u8;
        }
        Ok(bytes)
    };
    let three = |numbers: Vec<f32>| <[f32; 3]>::try_from(numbers).map_err(|_| "a corner with no place".to_owned());
    let uv = floats("TextureCoordinates0")?;
    Ok(Vertex {
        position: three(floats("Position")?)?,
        normal: three(floats("Normal")?)?,
        uv: [uv.first().copied().unwrap_or(0.0), uv.get(1).copied().unwrap_or(0.0)],
        weights: bytes("BoneWeights")?,
        bones: bytes("BoneIndices")?,
    })
}

/// The file's animation: the client takes a file of exactly one (`GrannyAnimationLibrary::Load`), and of
/// it the tracks of the group named as the model (Granny binds a group to a model by its name); a file
/// of one group gives that one.
fn animation(info: &Object, model: &str) -> Result<Option<Animation>, String> {
    let animations = info.objects("Animations")?;
    let [animation] = &animations[..] else { return Ok(None) };
    let groups = animation.objects("TrackGroups")?;
    let named = groups.iter().find(|group| group.text("Name").is_ok_and(|name| name == model));
    let Some(group) = named.or(groups.first()) else { return Ok(None) };
    let tracks = group.objects("TransformTracks")?.iter().map(track).collect::<Result<_, _>>()?;
    Ok(Some(Animation { duration: animation.number("Duration")? as f32, tracks }))
}

fn track(track: &Object) -> Result<Track, String> {
    let curve = |name: &str, width: usize| -> Result<Curve, String> {
        let Some(curve) = track.object(name)? else { return Ok(Curve::default()) };
        let floats = |name: &str| Ok::<Vec<f32>, String>(curve.numbers(name)?.into_iter().map(|number| number as f32).collect());
        let (knots, controls) = (floats("Knots")?, floats("Controls")?);
        if controls.len() != knots.len() * width {
            return Err("a curve whose numbers do not fit its knots".to_owned());
        }
        Ok(Curve { degree: curve.number("Degree")? as usize, knots, controls, width })
    };
    let (position, orientation, scale) = (curve("PositionCurve", 3)?, curve("OrientationCurve", 4)?, curve("ScaleShearCurve", 9)?);
    Ok(Track { bone: track.text("Name")?, position, orientation, scale })
}
