use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub version: u32,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    #[serde(default)]
    pub checksums: BTreeMap<String, String>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Material {
    pub id: String,
    pub source: Option<String>,
    pub license: Option<String>,
    pub albedo: String,
    pub roughness: Option<String>,
    pub normal: Option<String>,
    pub height: Option<String>,
    pub ao: Option<String>,
    #[serde(default = "default_normal_y")]
    pub normal_y: String,
}
fn default_normal_y() -> String {
    "directx".into()
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Texture {
    pub name: String,
    pub material: String,
    pub recipe: String,
    pub tint: [u8; 3],
    #[serde(default = "default_f0")]
    pub f0: u8,
    #[serde(default)]
    pub porosity: u8,
    #[serde(default = "default_emissive")]
    pub emissive: u8,
}
fn default_f0() -> u8 {
    10
}
fn default_emissive() -> u8 {
    255
}

pub fn read(path: &Path) -> Result<Manifest, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn safe_file(root: &Path, input: &str) -> Result<PathBuf, String> {
    let path = Path::new(input);
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|p| !matches!(p, Component::Normal(_)))
    {
        return Err(format!("invalid relative asset path: {input:?}"));
    }
    let root = root
        .canonicalize()
        .map_err(|e| format!("{}: {e}", root.display()))?;
    let path = root
        .join(path)
        .canonicalize()
        .map_err(|e| format!("{input}: {e}"))?;
    if !path.starts_with(&root) || !path.is_file() {
        return Err(format!(
            "asset is not a file inside the input directory: {input}"
        ));
    }
    Ok(path)
}

pub fn validate(m: &Manifest, root: &Path) -> Result<(), String> {
    if m.version != 1 {
        return Err("manifest version must be 1".into());
    }
    if m.materials.len() > 1024 || m.textures.len() > 1024 {
        return Err("at most 1024 materials and textures are supported".into());
    }
    if m.materials.is_empty() || m.textures.is_empty() {
        return Err("materials and textures must not be empty".into());
    }
    let mut ids = HashSet::new();
    for material in &m.materials {
        if !valid_name(&material.id) || !ids.insert(&material.id) {
            return Err(format!("invalid or duplicate material ID: {}", material.id));
        }
        for path in std::iter::once(&material.albedo)
            .chain(material.roughness.iter())
            .chain(material.normal.iter())
            .chain(material.height.iter())
            .chain(material.ao.iter())
        {
            safe_file(root, path)?;
        }
        if material.normal_y != "directx" && material.normal_y != "opengl" {
            return Err(format!(
                "{}: normal_y must be directx or opengl",
                material.id
            ));
        }
    }
    let mut names = HashSet::new();
    for t in &m.textures {
        if !valid_name(&t.name)
            || t.name.ends_with("_n")
            || t.name.ends_with("_s")
            || !names.insert(&t.name)
        {
            return Err(format!("invalid or duplicate texture name: {}", t.name));
        }
        if !ids.contains(&t.material) {
            return Err(format!("{}: unknown material {}", t.name, t.material));
        }
        if ![
            "stone", "brick", "plank", "moss", "glass", "metal", "gem", "leaves", "material",
        ]
        .contains(&t.recipe.as_str())
        {
            return Err(format!("{}: unknown recipe {}", t.name, t.recipe));
        }
        if t.porosity > 64 {
            return Err(format!("{}: porosity must be 0..64", t.name));
        }
        if (238..=254).contains(&t.f0) {
            return Err(format!("{}: reserved F0/metal ID", t.name));
        }
    }
    Ok(())
}
/// Stream file content through SHA-256 without loading it all into memory.
pub fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = file
            .read(&mut buffer)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Hash all declared maps and validate any supplied SHA-256 checksums before
/// creating output. A sorted map makes the report stable across thread counts.
pub fn input_hashes(m: &Manifest, root: &Path) -> Result<BTreeMap<String, String>, String> {
    let mut hashes = BTreeMap::new();
    for material in &m.materials {
        for path in std::iter::once(&material.albedo)
            .chain(material.roughness.iter())
            .chain(material.normal.iter())
            .chain(material.height.iter())
            .chain(material.ao.iter())
        {
            if hashes.contains_key(path) {
                continue;
            }
            let full = safe_file(root, path)?;
            let hash = sha256_file(&full)?;
            if let Some(expected) = m.checksums.get(path) {
                if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) {
                    return Err(format!(
                        "{path}: expected SHA-256 must contain 64 hexadecimal characters"
                    ));
                }
                if !hash.eq_ignore_ascii_case(expected) {
                    return Err(format!("{path}: SHA-256 mismatch (actual {hash})"));
                }
            }
            hashes.insert(path.clone(), hash);
        }
    }
    for key in m.checksums.keys() {
        if !hashes.contains_key(key) {
            return Err(format!("checksum refers to undeclared input: {key}"));
        }
    }
    Ok(hashes)
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
}

pub fn by_id(m: &Manifest) -> HashMap<&str, &Material> {
    m.materials
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect()
}

/// Check target paths against the user's local Minecraft client JAR. This does
/// not redistribute vanilla assets, infer the JAR's version, or validate color
/// targets. Sidecar metadata is rejected until animation handling is supported.
pub fn validate_against_jar(manifest: &Manifest, jar: &Path) -> Result<(), String> {
    let file = fs::File::open(jar).map_err(|e| format!("{}: {e}", jar.display()))?;
    let mut archive = ::zip::ZipArchive::new(file)
        .map_err(|e| format!("{}: invalid ZIP/JAR: {e}", jar.display()))?;
    let mut problems = Vec::new();
    for texture in &manifest.textures {
        let name = format!("assets/minecraft/textures/block/{}.png", texture.name);
        match archive.by_name(&name) {
            Ok(file) if !file.is_dir() => (),
            Ok(_) => problems.push(format!("{name}: is a directory")),
            Err(::zip::result::ZipError::FileNotFound) => {
                problems.push(format!("{name}: not found in client JAR"))
            }
            Err(e) => return Err(format!("{name}: {e}")),
        }
        let meta = format!("{name}.mcmeta");
        match archive.by_name(&meta) {
            Ok(_) => problems.push(format!("{meta}: metadata/animation is not supported yet")),
            Err(::zip::result::ZipError::FileNotFound) => (),
            Err(e) => return Err(format!("{meta}: {e}")),
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Minecraft target validation failed:\n{}",
            problems.join("\n")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn names_and_paths() {
        assert!(valid_name("oak_planks"));
        assert!(!valid_name("../stone"));
        assert!(safe_file(Path::new("."), "../Cargo.toml").is_err());
    }
}
