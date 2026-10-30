#!/usr/bin/env python3
"""Create original procedural construction masks, no game image inputs.

RGBA = relative shade, signed relief around 128, hardware mask, opacity.
Coordinates are independently drawn vector shapes in a 16-unit square. Real
photographic material grain comes from the separate licensed material scans.
Requires Pillow. Run from the repository root.
"""
from pathlib import Path
from PIL import Image, ImageDraw
S=32
OUT=Path('work/materials/shaped_layouts')
OUT.mkdir(parents=True, exist_ok=True)

class Layout:
 def __init__(self, opaque=True):
  self.im=Image.new('RGBA',(512,512),(235,128,0,255) if opaque else (0,128,0,0));self.d=ImageDraw.Draw(self.im)
 def rect(self,box,shade=220,relief=140,metal=False,alpha=255):
  x1,y1,x2,y2=box; self.d.rectangle((round(x1*S),round(y1*S),round(x2*S)-1,round(y2*S)-1),fill=(shade,relief,255 if metal else 0,alpha))
 def ellipse(self,box,shade=220,relief=140,metal=False,alpha=255):
  self.d.ellipse(tuple(round(v*S) for v in box),fill=(shade,relief,255 if metal else 0,alpha))
 def line(self,coords,width,shade=220,relief=140,metal=False,alpha=255):
  self.d.line([(round(x*S),round(y*S)) for x,y in coords],fill=(shade,relief,255 if metal else 0,alpha),width=round(width*S),joint='curve')
 def hole(self,box): self.rect(box,0,128,False,0)
 def panel(self,box,open=False):
  # Soft, supersampled molding uses low shade/relief contrast to reduce harsh
  # panel bands and POM-like depth. Open-panel holes retain exact coordinates.
  x1,y1,x2,y2=box
  self.rect(box,211,137)
  self.rect((x1+.16,y1+.16,x2-.16,y2-.16),239,143)
  self.rect((x1+.34,y1+.34,x2-.34,y2-.34),220,119)
  self.rect((x1+.5,y1+.5,x2-.5,y2-.5),248,124)
  if open:self.hole((x1+.55,y1+.55,x2-.55,y2-.55))
 def handle(self):
  # Recessed escutcheon, raised knob and two small fasteners. All details
  # stay within the existing opaque hardware area and remain a metal ID mask.
  self.rect((2.4,9.6,3.9,11.1),122,144,True)
  self.rect((2.55,9.74,3.75,10.96),196,162,True)
  self.ellipse((2.7,9.8,3.6,10.7),242,181,True)
  self.ellipse((2.86,9.96,3.44,10.54),151,137,True)
  self.ellipse((2.58,9.7,2.82,9.94),222,170,True)
  self.ellipse((3.48,10.76,3.72,11.0),222,170,True)
 def frame(self):
  self.rect((0,0,1.4,16),182,153);self.rect((14.6,0,16,16),182,153)
  self.rect((1.2,0,1.4,16),219,161);self.rect((14.6,0,14.8,16),219,161)
 def save(self,name):self.im.save(OUT/(name+'.png'),optimize=True)

def door(wood,part):
 a=Layout(); a.frame(); top=part=='top'
 if wood in ('oak','iron'):
  if top:
   for y in (2,8.1):
    for x in (2.2,8.2): a.panel((x,y,x+5.5,y+5.5),open=True)
  else:
   for y in (1.3,8.3):
    for x in (2,8): a.panel((x,y,x+5.7,y+6),open=False)
   a.handle()
 elif wood=='copper':
  if top:
   a.panel((2,2,14,14),open=True)
   a.line([(3.0,3),(13,13)],1.2,205,169)
   a.line([(13,3),(3,13)],1.2,205,169)
  else:
   a.panel((1.8,1.5,14.2,14.7));a.rect((10.5,1.8,13,4.5),165,169,True);a.rect((11.2,2.1,12.4,3.7),235,180,True)
 elif wood=='acacia':
  a.rect((1.3,2,14.7,12.5),175,148)
  for x in (3,6.5,10): a.hole((x,2.6,x+2.1,11.7))
  a.rect((0,13.7,16,16),188,156)
  if not top:a.handle()
 elif wood=='bamboo':
  a.panel((2,2,14,14),open=top)
  if top:
   for x in (4.1,6.1,8.1,10.1,12.1):a.rect((x,2.3,x+.3,13.7),195,152)
   for y in (4,6,8,10,12):a.rect((2.3,y,13.7,y+.3),195,152)
  else:a.handle()
 elif wood=='cherry':
  a.panel((2,1.8,14,14),open=False)
  if top:
   for cx,cy in [(5,5),(10.8,5),(7.9,8.2),(5,11),(10.8,11)]:
    a.hole((cx-.8,cy-.3,cx+.8,cy+.3));a.hole((cx-.3,cy-.8,cx+.3,cy+.8))
  else:a.handle()
 elif wood=='jungle':
  a.panel((2,2,14,13.8),open=False)
  if top:
   a.ellipse((4,2.4,12,10.5),0,128,False,0);a.hole((4,6,12,9.5))
   a.rect((7.6,2.5,8.4,10),186,155);a.rect((3.5,9.4,12.5,10.2),186,155)
  else:
   a.line([(5,2),(5,13)],.25,172,135);a.line([(10.5,2),(10.5,13)],.25,172,135);a.handle()
 elif wood=='poplar':
  a.panel((1.6,1.5,14.4,14.8))
  if top:
   a.d.polygon([(8*S,3*S),(13*S,8*S),(8*S,13*S),(3*S,8*S)],fill=(0,128,0,0))
  else:
   for x in range(-6,15,3):a.line([(x,2),(x+11,13)],.3,183,138)
   a.handle()
 elif wood=='spruce':
  for x in (4,8,12):a.rect((x,0,x+.2,16),165,120)
  for y in (1.5,12.7):
   a.rect((0,y,16,y+1.1),160,160,True)
   for x in (3,8,13):a.ellipse((x-.2,y+.35,x+.2,y+.75),245,180,True)
  if not top:a.handle()
 elif wood in ('crimson','warped'):
  for y in (1.8,6.9,12):a.panel((1.7,y,14.3,min(y+3.2,15.4)))
  if not top:a.handle()
 elif wood in ('dark_oak','pale_oak','birch','mangrove'):
  if wood=='dark_oak':
   for x in (2,8):a.panel((x,1.5,x+5.7,14.6))
  elif wood=='mangrove':
   a.panel((2,1.3,14,14.8));a.ellipse((4,3.5,12,12.5),185,117);a.ellipse((4.5,4,11.5,12),232,130)
  else:a.panel((2,1.4,14,14.6))
  if not top:a.handle()
  if wood=='pale_oak':a.rect((0,2.3,11.5,3.4),165,164,True)
 else:raise ValueError(wood)
 return a

def trap(wood):
 a=Layout();
 if wood in ('oak','iron','dark_oak'):
  for y in (1.5,8.1):
   for x in (1.5,8.1): a.panel((x,y,x+6,y+6),open=wood!='dark_oak')
 elif wood=='copper':
  a.panel((1,1,15,15),open=True);a.line([(2,2),(14,14)],1.2,205,165);a.line([(14,2),(2,14)],1.2,205,165)
 elif wood=='acacia':
  a.panel((1,1,15,15))
  for x in (3,6.5,10):a.hole((x,2.5,x+2.2,13.4))
 elif wood in ('crimson','warped'):
  a.panel((1,1,15,15))
  for y in (3,6.5,10):a.hole((2.5,y,13.5,y+2))
 elif wood=='bamboo':
  a.panel((1,1,15,15),open=True)
  for x in (3,5,7,9,11,13):a.rect((x,1.5,x+.55,14.5),215,150)
  for y in (3,5,7,9,11,13):a.rect((1.5,y,14.5,y+.55),215,150)
 elif wood=='cherry':
  a.panel((1.5,1.5,14.5,14.5))
  for y in (3,6.7,10.4):
   for x in (3,6.7,10.4):a.hole((x,y,x+2.3,y+2.3))
 elif wood=='jungle':
  a.panel((1.3,1.3,14.7,14.7),open=True)
  for x in (4.7,8,11.3):a.rect((x,1.5,x+.6,14.5),205,145)
  a.rect((1.5,7.5,14.5,8.5),205,155)
 elif wood=='mangrove':
  a.panel((1,1,15,15));a.ellipse((3.5,3.5,12.5,12.5),0,128,False,0)
 elif wood=='poplar':
  a.panel((1,1,15,15));a.d.polygon([(8*S,3*S),(13*S,8*S),(8*S,13*S),(3*S,8*S)],fill=(0,128,0,0))
 elif wood=='spruce':
  for x in (4,8,12):a.rect((x,0,x+.25,16),170,120)
  for y in (3.3,11):a.rect((0,y,16,y+.8),165,160,True)
 elif wood in ('birch','pale_oak'):
  a.panel((1,1,15,15));a.rect((7,13,9,14),160,170,True)
 else:raise ValueError(wood)
 return a

wood=['acacia','bamboo','birch','cherry','copper','crimson','dark_oak','iron','jungle','mangrove','oak','pale_oak','poplar','spruce','warped']
for w in wood:
 for part in ('top','bottom'):door(w,part).save(w+'_door_'+part)
 trap(w).save(w+'_trapdoor')
# Ladder is an independently drawn transparent two-rail construction.
a=Layout(False)
for x in (1.4,13):a.rect((x,0,x+1.6,16),210,151)
for y in (1.5,5,8.5,12):a.rect((0,y,16,y+1.4),225,153)
a.save('ladder')
# Bars use transparent space between slender posts; colors depend on material.
a=Layout(False)
for x in (2,7.5,13):a.rect((x,0,x+1.1,16),220,153)
for x1,y,x2 in [(2,3,8.6),(7.5,12,14.1),(0,7,3.1),(13,7,16)]:a.rect((x1,y,x2,y+1.1),208,151)
a.save('bars')
# Vanilla's lever texture has exactly this opaque 2x10 UV island; keep the
# original cutout/UV footprint while adding aligned wood and iron PBR detail.
a=Layout(False)
a.rect((7,6,9,9),205,151)
a.rect((7.2,6.2,8.8,8.8),232,164)
a.line([(7.42,6.35),(7.42,8.65)],.1,172,142)
a.line([(8.58,6.35),(8.58,8.65)],.1,172,142)
a.rect((7,9,9,9.8),132,145,True)
a.rect((7.2,9.15,8.8,9.65),210,164,True)
a.rect((7,9.8,9,15.2),156,147)
a.rect((7.25,10,8.75,15),205,155)
a.rect((7,15.2,9,16),118,143,True)
a.save('lever')
print('wrote',len(list(OUT.glob('*.png'))),'construction masks')
