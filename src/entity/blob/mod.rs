use bevy::{
    ecs::system::{
        lifetimeless::{Read, SCommands, SQuery, SRes},
        StaticSystemParam, SystemParamItem,
    },
    prelude::*,
    render::Extract,
};
use fastrand::Rng;
use float_next_after::NextAfter;

use crate::{
    draw::{vertex::DrawVertex, Drawer, LineState, TriState},
    shape::vertex::{Request, Shaper},
    util::{
        curve, sin, vec_angle, FloatExt,
        Interp::{Linear, PowIn},
        Interpolation, RngExt, Vec3Ext,
    },
};

#[derive(Component, Copy, Clone)]
pub struct Blob {
    pub border_color: Color,
    pub eye_color: Color,
    pub cell_color: Color,
}

#[derive(Component, Copy, Clone)]
pub struct BlobShaper {
    pub id: u64,
    pub trns: GlobalTransform,
    pub time: f32,
    pub blob: Blob,
}

impl Shaper for BlobShaper {
    type ExtractParam = (
        SCommands,
        Extract<'static, 'static, SRes<Time<Virtual>>>,
        Extract<'static, 'static, SQuery<(Entity, Read<GlobalTransform>, Read<Blob>)>>,
    );
    type DrawParam = ();
    type Vertex = DrawVertex;

    fn extract(param: StaticSystemParam<Self::ExtractParam>) {
        let (mut commands, time, blobs) = param.into_inner();
        let time = time.elapsed_seconds_wrapped();

        for (e, &trns, &blob) in &blobs {
            commands.spawn(BlobShaper {
                id: e.to_bits(),
                trns,
                time,
                blob,
            });
        }
    }

    #[inline]
    fn draw(&mut self, _: &mut SystemParamItem<Self::DrawParam>, out: &mut Vec<Request<Self::Vertex>>) {
        let Self {
            id,
            trns,
            time,
            blob: Blob {
                border_color,
                eye_color,
                cell_color,
            },
        } = *self;

        let (pos, mut layer) = trns.translation().separate_z();
        let mut draw = Drawer::new(out);
        let mut rng = Rng::with_seed(id);

        let r = 200.0;
        for i in (0..32).map(|i| i as f32) {
            let origin = i / 32.0 * 360f32.to_radians();
            let deviate = rng.range_f32(0.0, 4.0).to_radians();
            let move_scl = rng.range_f32(32.0, 64.0).to_radians();
            let start = rng.range_f32(0.25, 0.4) * r;
            let end = rng.range_f32(0.6, 1.0) * r;
            let width = rng.range_f32(0.05, 0.08) * r;
            let alpha = rng.range_f32(0.1, 0.6);

            let angle = origin + sin((time * 120.0).to_radians() + i * move_scl, move_scl, deviate);
            let offset = vec_angle(angle, start, 0.0);

            draw.line_angle(
                LineState::default().stroke(width).color_tip(
                    PowIn(2).interp(cell_color, eye_color, curve(alpha, 0.1, 0.7)).with_a(0.0) * 0.4,
                    PowIn(2).interp(cell_color, eye_color, curve(alpha, 0.3, 0.6)).with_a(alpha),
                ),
                layer.next_swap(),
                pos.x + offset.x,
                pos.y + offset.y,
                angle,
                end - start,
            );
        }

        for i in (0..24).map(|i| i as f32) {
            let origin = rng.range_f32(0.0, 360.0.next_after(f32::NEG_INFINITY));
            let deviate = rng.range_f32(0.0, 60.0).to_radians();
            let move_scl = rng.range_f32(24.0, 48.0).to_radians();
            let len = rng.range_f32(0.2, 0.5) * r;
            let width = rng.range_f32(0.1, 0.18) * r;
            let alpha = rng.range_f32(0.3, 0.6);

            let angle = origin + sin((time * 60.0).to_radians() + i * move_scl, move_scl, deviate);
            let offset = vec_angle(angle, r, 0.0);
            let color = Linear.interp(border_color, cell_color, 0.5 - curve(alpha, 0.3, 0.6) * 0.5);

            draw.tri_angle(
                TriState::default()
                    .size(width, len)
                    .color_tip(color.with_a(0.0), color.with_a(alpha)),
                layer.next_swap(),
                pos.x + offset.x,
                pos.y + offset.y,
                angle - 180f32.to_radians(),
            );
        }

        /*
        float r = radius();
        Draw.color(cellColor, cellColor.a * 0.15f);
        Fill.light(pos.x, pos.y, Lines.circleVertices(r), r, Tmp.c1.set(eyeColor).a(0f), Tmp.c2.set(cellColor).mulA(0.2f));

        Lines.stroke(0.09f * r, borderColor);
        Lines.circle(pos.x, pos.y, r);

        float focus = lookRange * r;
        Tmp.v1.set(look).sub(pos).clampLength(0f, focus);
        float len = Interp.pow2Out.apply(Tmp.v1.len() / focus) * r * 3f / 4f;

        Tmp.v1.scl(1f / focus);
        float tiltLeft = Interp.smooth2.apply(
            Math.max(Tmp.v1.dot(Mathf.cosDeg(45f), Mathf.sinDeg(45f)), 0f) +
            Math.max(Tmp.v1.dot(Mathf.cosDeg(225f), Mathf.sinDeg(225f)), 0f)
        ) * 30f;
        float tiltRight = Interp.smooth2.apply(
            Math.max(Tmp.v1.dot(Mathf.cosDeg(135f), Mathf.sinDeg(135f)), 0f) +
            Math.max(Tmp.v1.dot(Mathf.cosDeg(-45f), Mathf.sinDeg(-45f)), 0f)
        ) * 30f;

        Draw.color(eyeColor, eyeColor.a * 0.8f);
        Tmp.v2.set(Tmp.v1).scl(Mathf.sqrt(len * len / Tmp.v1.len2()) * 0.75f);

        float tilt = tiltLeft - tiltRight;
        for(int sign : Mathf.signs) {
            Draws.tri(pos.x + Tmp.v2.x, pos.y + Tmp.v2.y, 0.3f * r, 0.6f * r, tilt + 90f * sign);
            Draws.tri(pos.x + Tmp.v2.x, pos.y + Tmp.v2.y, 0.6f * r, 0.25f * r, tilt + 90f + 90f * sign);
        }
         */
    }
}
