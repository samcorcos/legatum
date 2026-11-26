"""MPFB anatomy, native weight-preserving rigs and authored rest-space correctives.

Rigify is opt-in for new units. Never migrate a working motion rig implicitly.
No automatic weights and no invented corrective displacement; callers supply
sculpted rest-space deltas after the macro mix is frozen.
"""
import bpy


def create_body(macro_overrides, name):
    bpy.ops.preferences.addon_enable(module='bl_ext.user_default.mpfb')
    from bl_ext.user_default.mpfb.services.humanservice import HumanService
    from bl_ext.user_default.mpfb.services.targetservice import TargetService
    macro = TargetService.get_default_macro_info_dict()
    macro.update(macro_overrides)
    body = HumanService.create_human(macro_detail_dict=macro)
    body.name = name
    return body


def freeze_macro_mix(body):
    """Bake MPFB's active macro/muscle shape keys into the mesh of a WORKING COPY.

    MPFB keeps its macro targets as live shape keys. Editing `data.vertices`
    while they are active does not change the rendered shape (Spearman round 6:
    25 active keys silently discarded the foot fitting). Call this on the
    disposable projection copy before any vertex fitting; keep the morphable
    original in its own file. Topology, weights and the rig are unchanged.
    Add new corrective keys only after this.
    """
    count = len(body.data.vertices)
    if body.data.shape_keys:
        bpy.context.view_layer.objects.active = body
        bpy.ops.object.shape_key_remove(all=True, apply_mix=True)
    assert len(body.data.vertices) == count and body.data.shape_keys is None
    return body


def add_rig(body, name, rigify=False):
    from bl_ext.user_default.mpfb.services.humanservice import HumanService
    if rigify:
        bpy.ops.preferences.addon_enable(module='rigify')
        from bl_ext.user_default.mpfb.services.rigservice import RigService
        meta = HumanService.add_builtin_rig(body, 'rigify.human', import_weights=True)
        bpy.context.view_layer.update()
        rig = RigService.generate_rigify_rig(meta, name=name, meta_rig_action='hide')
    else:
        rig = HumanService.add_builtin_rig(body, 'game_engine')
        rig.name = name
    assert rig is not None
    mods = [m for m in body.modifiers if m.type == 'ARMATURE']
    assert mods and all(m.object == rig for m in mods)
    return rig


def add_hinge_corrective(body, rig, name, bone, deltas, axis='ROT_X', sign=1,
                         start=.785398, full=1.832596):
    """Apply authored rest-space deltas and an isolated local hinge driver.

    Not a posed-to-rest inversion solver. Do not use for swing/twist shoulders.
    Validate volume and combined poses before enabling in a production unit.
    """
    assert full > start >= 0 and sign in (-1, 1)
    assert bone in rig.pose.bones and axis in ('ROT_X', 'ROT_Y', 'ROT_Z')
    if body.data.shape_keys is None:
        body.shape_key_add(name='Basis')
    assert body.data.shape_keys.key_blocks.get(name) is None
    key = body.shape_key_add(name=name)
    for index, delta in deltas.items():
        assert 0 <= index < len(key.data) and len(delta) == 3
        for i in range(3):
            key.data[index].co[i] += delta[i]
    driver = key.driver_add('value').driver
    var = driver.variables.new(); var.name = 'bend'; var.type = 'TRANSFORMS'
    target = var.targets[0]; target.id = rig; target.bone_target = bone
    target.transform_type = axis; target.transform_space = 'LOCAL_SPACE'
    driver.expression = f'min(1,max(0,({sign}*bend-{start})/{full-start}))'
    return key
