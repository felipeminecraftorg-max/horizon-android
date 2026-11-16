#!/usr/bin/env python3
"""Bake original voxel meshes for native Old School screens (not Minecraft textures).

The carousel renders its meshes live. These 96px RGBA counterparts are used by
native controls/editor fallbacks; the existing native shader adds dynamic light.
Requires numpy, scipy and Pillow. No game data, copied models, or fonts are read.
"""
from pathlib import Path
import itertools
import math
import numpy as np
from PIL import Image
from scipy.spatial import ConvexHull

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'crates/horizon-client/src/launcher_icons/jade'
COOL = (.62, .76, .80)
WARM = (.64, .49, .29)
EDGE = (.30, .43, .47)
DARK = (.12, .19, .22)
LIGHT = (.77, .86, .86)


def rotation(axis, angle):
    c, s = math.cos(angle), math.sin(angle)
    return np.array([[c, -s, 0], [s, c, 0], [0, 0, 1]]) if axis == 'z' else np.array([[c, 0, s], [0, 1, 0], [-s, 0, c]])


def box(parts, position, size, color, turn=0):
    half = np.array(size) / 2
    bevel = min(.018, min(size) * .12)
    points = []
    for signs in itertools.product((-1, 1), repeat=3):
        for outer in range(3):
            p = (half - bevel) * signs
            p[outer] = half[outer] * signs[outer]
            points.append(p)
    points = np.array(points)
    hull = ConvexHull(points)
    normal = rotation('z', turn)
    vertices = points @ normal.T + position
    for tri in hull.simplices:
        parts.append((vertices[tri], color))


def prism(parts, center, size, color, turn=0):
    outline = np.array([[0, .62], [-.22, .25], [-.16, -.38], [.16, -.38], [.22, .25]])
    vertices = np.array([[x, y, z] for z in (-.125, .125) for x, y in outline]) * size
    hull = ConvexHull(vertices)
    vertices = vertices @ rotation('z', turn).T + center
    for tri in hull.simplices:
        parts.append((vertices[tri], color))


def model(name):
    parts = []
    def cube(x, y, z, w, h, d, color=COOL, turn=0):
        box(parts, (x, y, z), (w, h, d), color, turn)
    if name == 'play':
        r = rotation('z', -.67)
        for center, dims, color in [((0, -.12, 0), (.13, 1.02, .14), WARM), ((0, .34, 0), (.88, .17, .25), COOL), ((-.35, .18, 0), (.18, .28, .25), COOL), ((.35, .23, 0), (.16, .19, .25), COOL), ((0, -.52, .01), (.16, .14, .17), EDGE)]:
            box(parts, r @ center, dims, color, -.67)
    elif name == 'mods':
        cube(0, 0, 0, .82, .69, .65, WARM)
        cube(0, .37, .015, .92, .14, .74)
        for x in (-.25, 0, .25):
            for z in (-.22, .02, .26): cube(x, .448, z, .19, .012, .18, DARK)
        for x in (-.32, .32): cube(x, -.14, .34, .09, .46, .03, EDGE)
        cube(0, -.08, .342, .30, .12, .025)
        cube(0, -.25, .344, .12, .22, .028)
    elif name == 'packs':
        for k in range(3):
            cube((k-1)*.085, (k-1)*.25, (k-1)*-.04, .81, .22, .62, WARM if k == 2 else COOL)
            cube((k-1)*.085, (k-1)*.25-.07, (k-1)*-.04+.315, .72, .035, .018, EDGE)
        cube(.085, .405, -.04, .68, .08, .52, EDGE)
    elif name == 'shaders':
        for x, y, z, scale in [(-.28, -.10, 0, .66), (.04, .05, 0, 1), (.31, -.18, .08, .55)]:
            prism(parts, (x, y, z), scale, WARM if x > .2 else COOL, -x*.7)
        cube(0, -.44, 0, .72, .12, .48, EDGE)
    elif name in ('skins', 'account'):
        cube(0, .18, 0, .64, .64, .57)
        cube(0, -.35, -.01, .9, .28, .43, WARM if name == 'skins' else EDGE)
        cube(0, .43, .30, .64, .14, .035, EDGE)
        for x in (-.16, .16):
            cube(x, .18, .302, .11, .09, .025, DARK)
            cube(x-.02, .19, .322, .035, .04, .012, LIGHT)
        cube(0, -.02, .304, .18, .055, .03, EDGE)
    elif name == 'files':
        cube(0, -.06, 0, .94, .66, .65, WARM)
        cube(0, .31, 0, 1.00, .17, .71)
        for x in (-.39, .39): cube(x, -.1, .34, .08, .61, .03, EDGE)
        cube(0, -.03, .36, .18, .25, .06)
        cube(0, -.02, .401, .065, .10, .025, DARK)
    elif name == 'settings':
        cube(0, 0, 0, .67, .67, .23)
        for x, y in ((0, .43), (0, -.43), (-.43, 0), (.43, 0)):
            cube(x, y, 0, .24, .24, .23)
        cube(0, 0, .13, .31, .31, .03, DARK)
        cube(0, 0, .152, .16, .16, .03, WARM)
    return parts


def render(name):
    size = 384
    pixels = np.zeros((size, size, 4), dtype=np.float64)
    depth = np.full((size, size), -np.inf)
    angle = -.30
    view = rotation('y', angle)
    tilt = np.array([[1, 0, 0], [0, .97, -.243], [0, .243, .97]])
    view = tilt @ view
    light = np.array([-.55, .80, 1.6]); light /= np.linalg.norm(light)
    camera = np.array([0., 0., 1.]); halfway = light + camera; halfway /= np.linalg.norm(halfway)
    for triangle, color in model(name):
        world = triangle @ view.T
        normal = np.cross(world[1]-world[0], world[2]-world[0])
        length = np.linalg.norm(normal)
        if length < 1e-10: continue
        normal /= length
        if normal[2] < 0: normal = -normal
        p = world[:, :2] * np.array([245, -245]) + np.array([size/2, size/2])
        low = np.maximum(0, np.floor(p.min(axis=0)).astype(int)); high = np.minimum(size-1, np.ceil(p.max(axis=0)).astype(int))
        if np.any(high < low): continue
        xx, yy = np.meshgrid(np.arange(low[0], high[0]+1)+.5, np.arange(low[1], high[1]+1)+.5)
        denominator = (p[1,1]-p[2,1])*(p[0,0]-p[2,0])+(p[2,0]-p[1,0])*(p[0,1]-p[2,1])
        if abs(denominator) < 1e-10: continue
        a = ((p[1,1]-p[2,1])*(xx-p[2,0])+(p[2,0]-p[1,0])*(yy-p[2,1]))/denominator
        b = ((p[2,1]-p[0,1])*(xx-p[2,0])+(p[0,0]-p[2,0])*(yy-p[2,1]))/denominator
        c = 1-a-b
        z = a*world[0,2]+b*world[1,2]+c*world[2,2]
        ys = slice(low[1], high[1]+1); xs = slice(low[0], high[0]+1)
        mask = (a>=0)&(b>=0)&(c>=0)&(z>depth[ys,xs])
        diffuse = .38 + .62*max(0, np.dot(normal, light))
        highlight = max(0, np.dot(normal, halfway))**55 * .40
        sheen = np.exp(-((yy/size-.29)/.085)**2) * .08
        grain = np.sin(xx*1.71+yy*3.17)*.006
        rgb = np.clip(np.array(color)[None,None,:]*diffuse+highlight+sheen[:,:,None]+grain[:,:,None],0,1)
        target = pixels[ys,xs]; target[mask,:3] = rgb[mask]; target[mask,3] = 1;depth[ys,xs][mask] = z[mask]
    # Resampling premultiplied RGBA in Pillow retains clean transparent edges.
    image = Image.fromarray(np.round(pixels*255).astype(np.uint8)).resize((96,96),Image.Resampling.LANCZOS)
    OUT.mkdir(parents=True,exist_ok=True);image.save(OUT/(name+'.png'),optimize=True)
    return image


if __name__ == '__main__':
    names = ('play','mods','packs','shaders','skins','account','files','settings')
    images = [render(name) for name in names]
    print('Rendered original native voxel icons:', ', '.join(names))
