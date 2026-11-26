"""The approved 0 A.D. hoplite rig as fitting targets, read from the delivered GLB.

Pure numpy; no bpy. Everything is in glTF bind space: Y up, metres, +Z forward,
the figure's left is +X. The skin node is a scene root, so bind-space vertex
positions are world positions at the bind pose.
"""
import sys
from pathlib import Path
import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from validate_glb import GLB

OAD_CREDIT = '0 A.D. art by Wildfire Games, CC BY-SA 3.0; adapted for Legatum'

# Chains used for pose fitting, proximal to distal. Each bone maps the segment
# from its joint to the next entry's joint ('*tip' = a mesh extremity).
CHAINS = {
    'torso': ['hip', 'spine', 'spine1', 'chest', 'neck', 'head', 'head_tip'],
    'arm_L': ['arm_L', 'forearm_L', 'hand_L', 'hand_tip_L'],
    'arm_R': ['arm_R', 'forearm_R', 'hand_R', 'hand_tip_R'],
    'leg_L': ['thigh_L', 'leg_L', 'foot_L', 'foot_tip_L'],
    'leg_R': ['thigh_R', 'leg_R', 'foot_R', 'foot_tip_R'],
}
RIGID = {'spear': 'weapon_R', 'aspis': 'shield_arm', 'helmet': 'helmet', 'crest': 'helmet'}


class Rig:
    def __init__(self, glb_path):
        self.g = g = GLB(glb_path); d = g.doc; self.path = Path(glb_path)
        skin = d['skins'][0]
        self.bones = [d['nodes'][j]['name'] for j in skin['joints']]
        parents = {c: i for i, n in enumerate(d['nodes']) for c in n.get('children', [])}
        self.parent = {d['nodes'][j]['name']: d['nodes'][parents[j]]['name'] if j in parents else None for j in skin['joints']}
        inv = np.array(g.accessor(skin['inverseBindMatrices'])).reshape(-1, 4, 4).transpose(0, 2, 1)
        self.bind = {b: np.linalg.inv(m) for b, m in zip(self.bones, inv)}
        self.joint = {b: m[:3, 3].copy() for b, m in self.bind.items()}
        mesh = d['meshes'][0]; prim = mesh['primitives'][0]; a = prim['attributes']
        self.P = np.array(g.accessor(a['POSITION']))
        self.J = np.array(g.accessor(a['JOINTS_0']), dtype=int)
        self.W = np.array(g.accessor(a['WEIGHTS_0']))
        self.F = np.array([i[0] for i in g.accessor(prim['indices'])]).reshape(-1, 3)
        self.regions = mesh['extras']['vertex_regions']
        self.part = np.empty(len(self.P), dtype=object); self.team = np.zeros(len(self.P), bool)
        for r in self.regions:
            self.part[r['start']:r['start'] + r['count']] = r['part']
            self.team[r['start']:r['start'] + r['count']] = bool(r['team'])
        cape = np.isin(self.J, [self.bones.index(b) for b in self.bones if b.startswith('cape')]) & (self.W > .001)
        self.cape_vertex = cape.any(1)
        fpart = self.part[self.F[:, 0]]
        fcape = self.cape_vertex[self.F].any(1)
        body = np.isin(fpart, ['body', 'Greek head'])
        self.body_faces = self.F[body & ~fcape]        # skin donor for the figure
        self.cloak_faces = self.F[body & fcape]        # donor for cloaks / capes
        self._tips()

    def _tips(self):
        """Mesh extremities that end the distal segments (fingertips, toes, crown)."""
        P = self.P; used = np.unique(self.body_faces)
        for side in 'LR':
            k = [self.bones.index(b) for b in (f'hand_{side}', f'finger_{side}', f'fingertip_{side}')]
            v = used[np.isin(self.J[used], k).any(1)]
            d = P[v] - self.joint[f'hand_{side}']
            self.joint[f'hand_tip_{side}'] = P[v[np.argmax(np.linalg.norm(d, axis=1))]]
            k = self.bones.index(f'foot_{side}')
            v = used[(self.J[used] == k).any(1)]
            toe = v[np.argmax(P[v, 2])]
            self.joint[f'foot_tip_{side}'] = np.array([self.joint[f'foot_{side}'][0], 0., P[toe, 2]])
        head = used[np.isin(self.part[used], ['Greek head'])]
        self.joint['head_tip'] = np.array([0., P[head, 1].max(), self.joint['head'][2]])

    # -- props in the bind pose -------------------------------------------------
    def region_points(self, *parts):
        return self.P[np.isin(self.part, parts)]

    def spear_frame(self):
        """Centre, unit axis toward the blade, blade/butt distances and grip distance."""
        p = self.region_points('spear'); c = p.mean(0)
        ax = np.linalg.svd(p - c)[2][0]; t = (p - c) @ ax
        def radius(end):
            q = p[np.abs(t - end) < .25] - c
            return np.linalg.norm(q - np.outer(q @ ax, ax), axis=1).max()
        if radius(t.min()) > radius(t.max()): ax = -ax; t = -t
        grip = (self.joint['weapon_R'] - c) @ ax
        return dict(centre=c, axis=ax, tip=t.max(), butt=t.min(), grip=grip)

    def shield_frame(self):
        """Rim centre, outward normal, in-plane up, radius and inner-face offset."""
        rim = self.P[(self.part == 'aspis') & self.team]; c = rim.mean(0)
        n = np.linalg.svd(rim - c)[2][2]
        front = self.region_points('aspis').mean(0); back = self.region_points('Aspis back', 'Aspis handgrip').mean(0)
        if (front - back) @ n < 0: n = -n
        up = np.array([0, 1., 0]) - n * n[1]; up /= np.linalg.norm(up)
        allp = self.region_points('aspis', 'Aspis back', 'Aspis handgrip')
        return dict(centre=c, normal=n, up=up, radius=np.linalg.norm(rim - c, axis=1).max(),
                    inner=((allp - c) @ n).min())

    def helmet_points(self):
        return self.region_points('helmet', 'crest', 'crest_base')

    def chain_points(self, chain):
        return np.array([self.joint[j] for j in CHAINS[chain]])
