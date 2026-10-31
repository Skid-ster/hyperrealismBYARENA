#!/usr/bin/env python3
"""Generate original 26.3 adult temperate farm-mob entity maps (no game pixels).

The atlas spans preserve the vanilla UV/alpha footprints; procedural color and
LabPBR maps are authored here and are not derived from Minecraft RGB pixels.
"""
from __future__ import annotations

import argparse
import json
import math
import shutil
import struct
import zipfile
import zlib
from pathlib import Path

S = 8
# UV occupancy spans are inclusive native texels, recorded per scan row.
COW = [
 [(6,21),(23,24),(53,60)],[(6,25),(52,57)],[(6,25),(52,57)],[(6,25),(52,57)],[(6,21),(28,57)],[(6,21),(28,57)],
 [(0,57)],[(0,51)],[(0,51)],[(0,51)],[(0,51)],[(0,51)],[(0,51)],[(0,51)],[(18,61)],[(18,61)],[(4,11),(18,61)],[(4,11),(18,61)],
 [(4,11),(18,61)],[(4,11),(18,61)],[(0,15),(18,61)],[(0,15),(18,61)],[(0,15),(18,61)],[(0,15),(18,61)],[(0,15),(18,61)],[(0,15),(18,61)],
 [(0,15),(18,61)],[(0,15),(18,61)],[(0,15),(18,61)],[(0,15),(18,61)],[(0,15),(18,61)],[(0,15),(18,61)],[(2,13)],[(2,13)],[(0,15)],[(0,15)],[(0,15)],
] + [[] for _ in range(27)]
PIG = [[(8,23)] for _ in range(8)] + [
 [(0,31),(36,55)],[(0,31),(36,55)],[(0,31),(36,55)],[(0,31),(36,55)],[(0,31),(36,55)],[(0,31),(36,55)],[(0,31),(36,55)],[(0,31),(36,55)],
 [(4,11),(17,24),(28,63)],[(4,11),(16,21),(28,63)],[(4,11),(16,21),(28,63)],[(4,11),(16,21),(28,63)],
 [(0,15),(28,63)],[(0,15),(28,63)],[(0,15),(28,63)],[(0,15),(28,63)],[(0,15),(28,63)],[(0,15),(28,63)],
 [(28,63)],[(28,63)],[(28,63)],[(28,63)],[(28,63)],[(28,63)],
] + [[] for _ in range(32)]
SHEEP = [[(8,19)] for _ in range(8)] + [
 [(0,27),(34,49)],[(0,27),(34,49)],[(0,27),(34,49)],[(0,27),(34,49)],[(0,27),(34,49)],[(0,27),(34,49)],[(28,55)],[(28,55)],
 [(2,11),(14,15),(28,55)],[(0,2),(4,14),(28,55)],[(0,0),(2,11),(13,15),(28,55)],[(2,2),(4,13),(28,55)],
 [(0,15),(28,55)],[(0,15),(28,55)],[(0,15),(28,55)],[(0,15),(28,55)],[(0,15),(28,55)],[(0,15),(28,55)],[(0,15),(28,55)],[(0,15),(28,55)],
 [(0,15),(28,55)],[(0,15),(28,55)],
 [(0,15)],[(0,15)]
]
SHEEP_WOOL = [[(6,17)] for _ in range(6)] + [[(0,23)] for _ in range(2)] + [
 [(0,23),(34,49)],[(0,23),(34,49)],[(0,23),(34,49)],[(0,23),(34,49)],[(34,49)],[(34,49)],[(28,55)],[(28,55)],
 [(4,11),(28,55)],[(4,11),(28,55)],[(4,11),(28,55)],[(4,11),(28,55)],[(0,15),(28,55)],[(0,15),(28,55)],[(0,15),(28,55)],[(0,15),(28,55)],
 [(0,15),(28,55)],[(0,15),(28,55)],[(28,55)],[(28,55)],[(28,55)],[(28,55)],[],[]
]
SPECS = {
 "cow/cow_temperate": (64,64,COW,"cow"),
 "pig/pig_temperate": (64,64,PIG,"pig"),
 "sheep/sheep": (64,32,SHEEP,"sheep_skin"),
 "sheep/sheep_wool": (64,32,SHEEP_WOOL,"wool"),
}


def noise(x:int,y:int,seed:int)->float:
 n=(x*0x1F123BB5)^(y*0x5F356495)^(seed*0x6C8E9CF5)
 n=(n^(n>>13))*0x85EBCA6B
 return ((n^(n>>16))&0xffffffff)/0xffffffff


def smooth(v:float)->float:
 v=min(1.0,max(0.0,v)); return v*v*(3-2*v)


def value_noise(x:float,y:float,scale:float,seed:int)->float:
 gx,gy=x/scale,y/scale; ix,iy=math.floor(gx),math.floor(gy); fx,fy=smooth(gx-ix),smooth(gy-iy)
 a,b,c,d=noise(ix,iy,seed),noise(ix+1,iy,seed),noise(ix,iy+1,seed),noise(ix+1,iy+1,seed)
 return (a*(1-fx)+b*fx)*(1-fy)+(c*(1-fx)+d*fx)*fy


def byte(v:float)->int: return max(0,min(255,round(v*255)))


def png(path:Path,w:int,h:int,rgba:bytes)->None:
 rows=b''.join(b'\0'+rgba[y*w*4:(y+1)*w*4] for y in range(h))
 def chunk(name:bytes,data:bytes)->bytes:
  body=name+data; return struct.pack('>I',len(data))+body+struct.pack('>I',zlib.crc32(body)&0xffffffff)
 data=b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(rows,9))+chunk(b'IEND',b'')
 path.parent.mkdir(parents=True,exist_ok=True); path.write_bytes(data)


def surface(kind:str,x:int,y:int)->tuple[tuple[int,int,int],float,float,float]:
 if kind=="cow":
  spot=value_noise(x,y,73,170)+0.42*value_noise(x,y,29,171)
  patch=smooth((spot-0.61)/0.075)
  # The adult temperate cow keeps a warm ivory coat with irregular chestnut patches.
  grain=(noise(x//3,y//4,172)-0.5)*0.025
  base=(0.91+grain,0.87+grain,0.78+grain); dark=(0.20+grain,0.105+grain,0.055+grain)
  color=tuple(round(255*(base[i]*(1-patch)+dark[i]*patch)) for i in range(3))
  h=0.515+0.012*value_noise(x,y,9,173)+0.018*patch
  return color,h,0.97,0.86
 if kind=="pig":
  pore=value_noise(x,y,31,210)-0.5; fine=noise(x//2,y//3,211)-0.5
  shade=0.045*pore+0.02*fine
  # A localized snout is placed on the head's front-facing atlas patch.
  snout=(8*S<=x<24*S and 0<=y<7*S)
  if snout:
   ring=0.5+0.5*math.sin(math.hypot((x-16*S)/S,(y-3.5*S)/S)*2.3)
   color=(round(202+18*ring),round(101+24*ring),round(108+22*ring)); h=0.555+0.018*ring; rough=0.68
  else:
   color=(round(221+255*shade),round(139+220*shade),round(145+210*shade)); h=0.515+0.012*pore; rough=0.76
  return color,h,0.97,rough
 if kind=="sheep_skin":
  face=(8*S<=x<20*S and 0<=y<8*S)
  leg=(y>=16*S and x<16*S)
  n=value_noise(x,y,23,301)-0.5
  if face:
   color=(round(180+16*n),round(137+14*n),round(104+12*n)); rough=0.82; h=0.51+0.008*n
  elif leg:
   color=(round(157+13*n),round(114+11*n),round(86+9*n)); rough=0.84; h=0.51+0.01*n
  else:
   color=(239,233,220); rough=0.9; h=0.505+0.004*n
  return color,h,0.97,rough
 # Sheep's tintable outer coat: bright neutral wool with tightly curled locks.
 curl=(0.5+0.5*math.sin(x*0.39+math.sin(y*0.22)*1.4)*math.cos(y*0.34))
 macro=value_noise(x,y,39,411)-0.5; fleck=noise(x//3,y//3,412)-0.5
 relief=0.025*curl+0.02*macro+0.01*fleck
 base=0.945+0.022*curl+0.018*macro+0.008*fleck
 return (round(255*base),round(255*(base*0.994)),round(255*(base*0.975))),0.515+relief,0.94,0.89


def make_texture(name:str,spec:tuple[int,int,list,str],root:Path)->None:
 uw,uh,spans,kind=spec; w,h=uw*S,uh*S; n=w*h
 albedo=bytearray(n*4); heights=[0.5]*n; aos=[1.0]*n; roughs=[0.8]*n
 for y in range(h):
  v=y//S
  for x in range(w):
   u=x//S; i=y*w+x
   color,ht,ao,rough=surface(kind,x,y); heights[i]=ht; aos[i]=ao; roughs[i]=rough
   opaque=any(a<=u<=b for a,b in spans[v])
   albedo[i*4:i*4+4]=bytes((*color,255 if opaque else 0))
 normals=bytearray(n*4); specs=bytearray(n*4)
 for y in range(h):
  for x in range(w):
   i=y*w+x; l=heights[y*w+max(0,x-1)]; r=heights[y*w+min(w-1,x+1)]
   u=heights[max(0,y-1)*w+x]; d=heights[min(h-1,y+1)*w+x]
   nx=-(r-l)*0.9; ny=-(d-u)*0.9; inv=1/math.sqrt(nx*nx+ny*ny+1)
   normals[i*4:i*4+4]=bytes((byte(nx*inv*0.5+0.5),byte(ny*inv*0.5+0.5),byte(aos[i]),max(1,byte(heights[i]))))
   # LabPBR 1.3 dielectric surfaces, no metal, porosity/SSS or emission.
   specs[i*4:i*4+4]=bytes((byte((1-math.sqrt(roughs[i]))*0.2),10,0,255))
 out=root/(name+'.png')
 png(out,w,h,bytes(albedo)); png(out.with_name(out.stem+'_n.png'),w,h,bytes(normals)); png(out.with_name(out.stem+'_s.png'),w,h,bytes(specs))


def create_pack(source:Path,pack:Path,zip_path:Path)->None:
 if pack.exists(): shutil.rmtree(pack)
 pack.mkdir(parents=True)
 (pack/'pack.mcmeta').write_text(json.dumps({'pack':{'description':'HyperRealism 512 — common farm-mob PBR pilot','min_format':[97,1],'max_format':97}},indent=2)+'\n')
 prop=pack/'assets/minecraft/optifine/texture.properties'; prop.parent.mkdir(parents=True,exist_ok=True); prop.write_text('format=lab-pbr/1.3\n')
 for entry in source.rglob('*.png'):
  rel=entry.relative_to(source); dst=pack/'assets/minecraft/textures/entity'/rel; dst.parent.mkdir(parents=True,exist_ok=True); shutil.copyfile(entry,dst)
 (pack/'README.txt').write_text('Minecraft Java 26.3 experimental adult temperate farm-mob textures.\nIncludes chicken, cow, pig, and both sheep base/outer-wool layers. Other biome, baby, coat, dye, model, and equipment variants remain vanilla.\nOriginal procedural maps; vanilla color pixels are not included. LabPBR entity rendering depends on shader support.\nNot yet visually validated in-game.\n')
 zip_path.parent.mkdir(parents=True,exist_ok=True)
 with zipfile.ZipFile(zip_path,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=9) as z:
  for p in sorted(x for x in pack.rglob('*') if x.is_file()):
   info=zipfile.ZipInfo(p.relative_to(pack).as_posix(),date_time=(2024,1,1,0,0,0)); info.compress_type=zipfile.ZIP_DEFLATED; info.external_attr=0o100644<<16
   z.writestr(info,p.read_bytes(),compress_type=zipfile.ZIP_DEFLATED,compresslevel=9)


def main()->int:
 ap=argparse.ArgumentParser(description=__doc__)
 ap.add_argument('--sources',type=Path,default=Path('work/entity_textures'))
 ap.add_argument('--pack',type=Path,default=Path('experiments/common_farm_mobs/pack'))
 ap.add_argument('--zip',type=Path,default=Path('experiments/common_farm_mobs/HyperRealism-Common-Farm-Mobs-Pilot.zip'))
 args=ap.parse_args()
 args.sources.mkdir(parents=True,exist_ok=True)
 for name,spec in SPECS.items(): make_texture(name,spec,args.sources)
 create_pack(args.sources,args.pack,args.zip)
 print('Built original adult temperate cow/pig and sheep base/wool LabPBR maps at 512x512 or 512x256.')
 return 0

if __name__=='__main__': raise SystemExit(main())
