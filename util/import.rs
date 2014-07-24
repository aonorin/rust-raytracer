use std::io::{BufferedReader, File};
use material::materials::CookTorranceMaterial;
use geometry::{Mesh, Prim};
use geometry::prims::Triangle;
use raytracer::compositor::{Surface, ColorRGBA};
use vec3::Vec3;

/// This is limited to only CookTorranceMaterials, as I couldn't get a Box<Material> to clone
/// a new material for each triangle primitive in the object model.
#[allow(dead_code)]
pub fn from_obj(position: Vec3, scale: f64, material: CookTorranceMaterial /*Box<Material>*/, flip_normals: bool, filename: &str) -> Mesh {
    let path = Path::new(filename);
    let mut file = BufferedReader::new(File::open(&path));

    let mut vertices:   Vec<Vec3> = Vec::new();
    let mut tex_coords: Vec<Vec3> = Vec::new();
    let mut normals:    Vec<Vec3> = Vec::new();
    let mut triangles:  Vec<Box<Prim+Send+Share>> = Vec::new();

    let start_time = ::time::get_time();
    let print_progress_every = 1024u;
    let mut current_line = 0;
    let mut processed_bytes = 0;
    let total_bytes = match path.stat() {
        Ok(stat) => stat.size,
        Err(e) => fail!(e)
    };

    for line_iter in file.lines() {
        let line: String = match line_iter {
            Ok(x) => x,
            Err(e) => fail!(e)
        };

        let tokens: Vec<&str> = line.as_slice().words().collect();
        if tokens.len() < 4 { continue }

        match tokens[0].as_slice() {
            "v" => {
                vertices.push(Vec3 {
                    x: parse_coord_str(tokens[1].as_slice(), scale, line.as_slice()),
                    y: parse_coord_str(tokens[2].as_slice(), scale, line.as_slice()),
                    z: parse_coord_str(tokens[3].as_slice(), scale, line.as_slice()),
                });
            },
            "vt" => {
                tex_coords.push(Vec3 {
                    x: parse_coord_str(tokens[1].as_slice(), scale, line.as_slice()),
                    y: parse_coord_str(tokens[2].as_slice(), scale, line.as_slice()),
                    z: 0.0
                });
            },
            "vn" => {
                let normals_flip_scale = if flip_normals { -1.0 } else { 1.0 } * scale;
                normals.push(Vec3 {
                    x: parse_coord_str(tokens[1].as_slice(), scale * normals_flip_scale, line.as_slice()),
                    y: parse_coord_str(tokens[2].as_slice(), scale * normals_flip_scale, line.as_slice()),
                    z: parse_coord_str(tokens[3].as_slice(), scale * normals_flip_scale, line.as_slice()),
                });
            },
            "f" => {
                // let pairs: Vec<&str> = tokens.tail().iter().map( |token| {
                //     token.to_string().as_slice().split('/').collect()
                // }).collect();

                let v0: Vec<&str> = tokens[1].split('/').collect();
                let v1: Vec<&str> = tokens[2].split('/').collect();
                let v2: Vec<&str> = tokens[3].split('/').collect();

                // If no texture coordinates were supplied, default to zero.
                let mut u = Vec3::zero();
                let mut v = Vec3::zero();

                if v0[1].len() > 0 {
                    u = Vec3 {
                        x: get_tex_coord(v0[1], &tex_coords),
                        y: get_tex_coord(v1[1], &tex_coords),
                        z: get_tex_coord(v2[1], &tex_coords)
                    };

                    v = Vec3 {
                        x: get_tex_coord(v0[2], &tex_coords),
                        y: get_tex_coord(v1[2], &tex_coords),
                        z: get_tex_coord(v2[2], &tex_coords)
                    };
                }

                triangles.push(box Triangle {
                    v0: get_string_index(v0[0], &vertices),
                    v1: get_string_index(v1[0], &vertices),
                    v2: get_string_index(v2[0], &vertices),

                    n0: get_string_index(v0[2], &normals),
                    n1: get_string_index(v1[2], &normals),
                    n2: get_string_index(v2[2], &normals),

                    u: u,
                    v: v,
                    material: box material.clone()
                });
            },
            _ => {}
        }

        current_line += 1;
        processed_bytes += line.as_bytes().len();
        if current_line % print_progress_every == 0 {
            ::util::print_progress("Bytes", start_time, processed_bytes, total_bytes as uint);
        }
    }

    // Cheat the progress meter
    ::util::print_progress("Bytes", start_time, total_bytes as uint, total_bytes as uint);

    Mesh {
        position: position,
        scale: scale,
        triangles: triangles
    }
}

fn parse_coord_str(coord: &str, scale: f64, line: &str) -> f64 {
    match from_str::<f64>(coord.as_slice()) {
        Some(f) => f * scale,
        None => fail!(format!("Bad vertex or texture coordinate in file. `{}`", line))
    }
}

fn get_tex_coord(i: &str, tex_coords: &Vec<Vec3>) -> f64 {
    match from_str::<uint>(i) {
        Some(x) => (tex_coords[x-1u as uint]).x,
        None => fail!("Bad texture")
    }
}

fn get_string_index(i: &str, vec: &Vec<Vec3>) -> Vec3 {
    match from_str::<uint>(i) {
        Some(x) => vec[x-1u as uint],
        None => fail!("Bad index")
    }
}


#[allow(dead_code)]
pub fn from_ppm(filename: &str) -> Surface {
    let path = Path::new(filename);
    let mut file = BufferedReader::new(File::open(&path));

    let tex = match file.read_to_string() { Ok(f) => f, Err(e) => fail!(e) };
    let mut tokens: Vec<&str> = tex.as_slice().words().collect();

    tokens.shift(); // PPM type
    let width  = match tokens.shift() { Some(x) => uint_from_string(x), None => fail!("Bad PPM") };
    let height = match tokens.shift() { Some(x) => uint_from_string(x), None => fail!("Bad PPM") };
    tokens.shift(); // Max color value

    print!("Importing image texture {}", filename);
    println!(" {}x{}", width, height);

    let mut surface = Surface::new(width, height, ColorRGBA::new_rgb(0, 0, 0));

    let mut i = 0u;

    for chunk in tokens.as_slice().chunks(3) {
        let x = i % width;
        let y = i / width;
        i += 1;

        if x >= width || y >= height { break };

        *surface.get_mut(x, y) = ColorRGBA::new_rgb(
            uint_from_string(chunk[0]) as u8,
            uint_from_string(chunk[1]) as u8,
            uint_from_string(chunk[2]) as u8
        );
    }

    surface
}

fn uint_from_string(s: &str) -> uint {
     match from_str::<uint>(s) {
        Some(x) => x,
        None => fail!("Bad uint string {}", s)
    }
}