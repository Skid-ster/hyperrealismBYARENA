/*
    Copyright (c) 2024 Josh Britain (jbritain)
    Licensed under the MIT license

      _____   __   _                          
     / ___/  / /  (_)  __ _   __ _  ___   ____
    / (_ /  / /  / /  / ' \ /  ' \/ -_) / __/
    \___/  /_/  /_/  /_/_/_//_/_/_/\__/ /_/   
    
    By jbritain
    https://jbritain.net
                                            
*/

#ifndef MATERIAL_GLSL
#define MATERIAL_GLSL

// enums for metal IDs
#define NO_METAL 0
#define IRON 1
#define GOLD 2
#define ALUMINIUM 3
#define CHROME 4
#define COPPER 5
#define LEAD 6
#define PLATINUM 7
#define SILVER 8
#define OTHER_METAL 9

// The user-selected LabPBR mode is authoritative. Iris loads _n/_s sidecars
// independently of the global texture.properties macro, and that macro can be
// absent/stale when another pack is above HyperRealism or resources were
// reloaded without a shader rebuild. Missing sidecars fall back to Iris'
// default flat normal / zero specular bytes, which decode safely to a rough
// dielectric. MC_TEXTURE_FORMAT_LAB_PBR is retained for diagnostics only.
#if PBR_MODE == 1
#define HYPERREALISM_LAB_PBR 1
#else
#define HYPERREALISM_LAB_PBR 0
#endif

mat3x2 getMetalN_K(uint metalID) {
  switch (metalID) {
    case IRON:
      return mat3x2(vec3(2.9114, 2.9497, 2.5845), vec3(3.0893, 2.9318, 2.767));
    case GOLD:
      return mat3x2(
        vec3(0.18299, 0.42108, 1.3734),
        vec3(3.4242, 2.3459, 1.7704)
      );
    case ALUMINIUM:
      return mat3x2(
        vec3(1.3456, 0.96521, 0.61722),
        vec3(7.4746, 6.3995, 5.3031)
      );
    case CHROME:
      return mat3x2(vec3(3.1071, 3.1812, 2.323), vec3(3.3314, 3.3291, 3.135));
    case COPPER:
      return mat3x2(
        vec3(0.27105, 0.67693, 1.3164),
        vec3(3.6092, 2.6248, 2.2921)
      );
    case LEAD:
      return mat3x2(vec3(1.91, 1.83, 1.44), vec3(3.51, 3.4, 3.18));
    case PLATINUM:
      return mat3x2(vec3(2.3757, 2.0847, 1.8453), vec3(4.2655, 3.7153, 3.1365));
    case SILVER:
      return mat3x2(
        vec3(0.15943, 0.14512, 0.13547),
        vec3(3.9291, 3.19, 2.3808)
      );
  }
  return mat3x2(0.0);
}

// Normal-incidence conductor reflectance for the predefined LabPBR metals.
// Keeping this separate from albedo prevents predefined metals from being
// double-tinted by the block texture.
vec3 getMetalF0(uint metalID) {
  mat3x2 N_K = getMetalN_K(metalID);
  vec3 n = vec3(N_K[0][0], N_K[1][0], N_K[2][0]);
  vec3 k = vec3(N_K[0][1], N_K[1][1], N_K[2][1]);
  vec3 nm1 = n - 1.0;
  vec3 np1 = n + 1.0;
  return (nm1 * nm1 + k * k) / (np1 * np1 + k * k);
}

struct Material {
  vec3 albedo;
  float emission;
  vec3 f0;
  mat3x2 N_K;
  float roughness;
  float sss;
  float porosity;
  uint metalID;
  float ao;
};

const Material waterMaterial = Material(
  vec3(0.0),
  0.0,
  vec3(0.02),
  mat3x2(0.0),
  0.0,
  0.0,
  0.0,
  NO_METAL,
  0.0
);

int labPBRByte(float value) {
  return clamp(int(value * 255.0 + 0.5), 0, 255);
}

// LabPBR normal map: RG = tangent-space XY, B = ambient occlusion,
// A = height. Z is reconstructed, never read from the blue channel.
vec3 labPBRNormal(vec4 normalData, mat3 tbnMatrix) {
  #if HYPERREALISM_LAB_PBR
  vec2 xy = normalData.rg * 2.0 - 1.0;
  xy *= PBR_NORMAL_STRENGTH;

  // Strength above 1 can push XY outside the unit circle. Clamp the slope
  // before sqrt so filtering/quantisation cannot produce NaNs.
  float xy2 = dot(xy, xy);
  if (xy2 > 1.0) {
    xy *= inversesqrt(xy2);
    xy2 = 1.0;
  }

  vec3 tangentNormal = vec3(xy, sqrt(max(1.0 - xy2, 0.0)));
  return normalize(tbnMatrix * tangentNormal);
  #else
  return normalize(tbnMatrix[2]);
  #endif
}

Material materialFromLabPBR(vec3 albedo, vec4 normalData, vec4 specularData) {
  Material material;
  material.albedo = albedo;
  material.N_K = mat3x2(0.0);
  material.roughness = 1.0;
  material.f0 = vec3(0.04);
  material.metalID = NO_METAL;
  material.porosity = 0.0;
  material.sss = 0.0;
  material.emission = 0.0;
  material.ao = 1.0;

  #if HYPERREALISM_LAB_PBR
  // Normal blue is AO, not normal Z.
  material.ao = mix(1.0, clamp01(normalData.b), PBR_AO_STRENGTH);

  // Specular red is perceptual smoothness: roughness = (1 - smoothness)^2.
  float smoothness = float(labPBRByte(specularData.r)) / 255.0;
  material.roughness = clamp(pow2(1.0 - smoothness), 0.002, 1.0);

  // Specular green: 0..229 dielectric F0, 230..237 predefined metals,
  // 238..255 albedo-tinted/other metal. Do not let reserved/custom IDs index
  // the predefined metal table and accidentally become perfect mirrors.
  int f0Byte = labPBRByte(specularData.g);
  if (f0Byte < 230) {
    material.f0 = vec3(float(f0Byte) / 255.0);
    material.metalID = NO_METAL;
  } else if (f0Byte < 238) {
    material.metalID = uint(f0Byte - 229);
    material.N_K = getMetalN_K(material.metalID);
    material.f0 = getMetalF0(material.metalID);
  } else {
    material.metalID = OTHER_METAL;
    material.f0 = albedo;
  }

  // Specular blue multiplexes porosity (0..64) and subsurface scattering
  // (65..255). They are mutually exclusive; SSS must not invent porosity.
  int blueByte = labPBRByte(specularData.b);
  if (blueByte <= 64) {
    material.porosity = float(blueByte) / 64.0;
    material.sss = 0.0;
  } else {
    material.porosity = 0.0;
    material.sss = float(blueByte - 64) / 191.0;
  }

  // Specular alpha: 255 is the no-emission sentinel, 0..254 is emission.
  int emissionByte = labPBRByte(specularData.a);
  material.emission = emissionByte == 255 ? 0.0 : float(emissionByte) / 255.0;
  #endif

  return material;
}

// Compatibility wrapper for programs without a LabPBR normal/specular pair
// (for example weather). It still respects PBR_MODE and the detected format.
Material materialFromSpecularMap(vec3 albedo, vec4 specularData) {
  return materialFromLabPBR(albedo, vec4(0.5, 0.5, 1.0, 1.0), specularData);
}

// Diagnostic views for verifying that the resource-pack channels reach the
// shader. These are development views, not a physically based lighting mode.
vec3 pbrDebugColor(int mode, vec4 normalData, vec4 specularData, Material material) {
  if (mode == 1) {
    vec2 xy = normalData.rg * 2.0 - 1.0;
    float z = sqrt(max(1.0 - dot(xy, xy), 0.0));
    return vec3(xy, z) * 0.5 + 0.5;
  }
  if (mode == 2) return vec3(material.roughness);
  if (mode == 3) return material.metalID == NO_METAL ? material.f0 : material.albedo;
  if (mode == 4) return vec3(material.emission);
  if (mode == 5) return vec3(material.ao);
  if (mode == 6) return vec3(normalData.a);
  if (mode == 7) return vec3(material.porosity);
  if (mode == 8) return vec3(material.sss);
  if (mode == 9) {
    // Green = both sidecars present, yellow = one sidecar, red = neither.
    // Blue channel indicates Iris detected the LabPBR texture.properties flag.
    bool hasNormal = distance(normalData, vec4(0.5, 0.5, 1.0, 1.0)) > 0.02;
    bool hasSpecular = distance(specularData, vec4(0.0)) > 0.02;
    float formatFlag = 0.0;
    #ifdef MC_TEXTURE_FORMAT_LAB_PBR
    formatFlag = 1.0;
    #endif
    if (hasNormal && hasSpecular) return vec3(0.0, 1.0, formatFlag);
    if (hasNormal || hasSpecular) return vec3(1.0, 0.75, formatFlag);
    return vec3(1.0, 0.0, formatFlag);
  }
  return material.albedo;
}

#endif // MATERIAL_GLSL
