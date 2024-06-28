use std::{marker::PhantomData, sync::Mutex};

use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        query::ROQueryItem,
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem, SystemState,
        },
    },
    prelude::*,
    render::{
        mesh::PrimitiveTopology,
        render_phase::{
            DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult, RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            binding_types::uniform_buffer, BindGroup, BindGroupEntries, BindGroupLayout, BlendState, BufferAddress,
            BufferUsages, BufferVec, ColorTargetState, ColorWrites, FragmentState, FrontFace, IndexFormat, MultisampleState,
            PipelineCache, PolygonMode, PrimitiveState, RenderPipelineDescriptor, ShaderStages, SpecializedRenderPipeline,
            SpecializedRenderPipelines, TextureFormat, VertexBufferLayout, VertexState, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::{ExtractedView, ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms},
    },
    utils::FloatOrd,
};

use crate::draw::vertex::{DrawLayer, Drawer, Request, Vertex, VertexKey};

pub struct DrawPipeline<T: Vertex> {
    view_layout: BindGroupLayout,
    _marker: PhantomData<fn(T)>,
}

impl<T: Vertex> Resource for DrawPipeline<T> {}
impl<T: Vertex> FromWorld for DrawPipeline<T> {
    fn from_world(world: &mut World) -> Self {
        let device = SystemState::<Res<RenderDevice>>::new(world).get_mut(world);
        Self {
            view_layout: device.create_bind_group_layout("draw_view_layout", &[
                uniform_buffer::<ViewUniform>(true).build(0, ShaderStages::VERTEX)
            ]),
            _marker: PhantomData,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct DrawKey {
    hdr: bool,
}

impl<T: Vertex> SpecializedRenderPipeline for DrawPipeline<T> {
    type Key = (DrawKey, T::Key);

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let (common, key) = key;
        let mut desc = RenderPipelineDescriptor {
            label: Some("draw_pipeline".into()),
            layout: [self.view_layout.clone()].into(),
            push_constant_ranges: Vec::new(),
            vertex: VertexState {
                shader: T::SHADER,
                shader_defs: Vec::new(),
                entry_point: "vertex_main".into(),
                buffers: [VertexBufferLayout {
                    array_stride: size_of::<T>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: T::LAYOUT.into(),
                }]
                .into(),
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                shader: T::SHADER,
                shader_defs: Vec::new(),
                entry_point: "fragment_main".into(),
                targets: [Some(ColorTargetState {
                    format: match common.hdr {
                        true => ViewTarget::TEXTURE_FORMAT_HDR,
                        false => TextureFormat::bevy_default(),
                    },
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })]
                .into(),
            }),
        };

        key.specialize(&mut desc);
        desc
    }
}

#[derive(Component)]
pub struct BatchBindGroup<T: Vertex> {
    pub group: BindGroup,
    _marker: PhantomData<fn(T)>,
}

#[derive(Resource)]
pub struct Requests<T: Vertex> {
    pub values: Mutex<Vec<Request<T>>>,
}

impl<T: Vertex> Default for Requests<T> {
    #[inline]
    fn default() -> Self {
        Self { values: default() }
    }
}

#[derive(Resource)]
pub struct Batch<T: Vertex> {
    pub vertices: BufferVec<T>,
    pub indices: BufferVec<u32>,
}

impl<T: Vertex> Default for Batch<T> {
    #[inline]
    fn default() -> Self {
        Self {
            vertices: BufferVec::new(BufferUsages::VERTEX),
            indices: BufferVec::new(BufferUsages::INDEX),
        }
    }
}

#[derive(Component, Copy, Clone)]
pub struct BatchSection {
    start: u32,
    end: u32,
}

pub fn queue_drawers<T: Drawer>(
    param: SystemParamItem<T::Param>,
    mut query: Query<(&mut T, T::Entity)>,
    requests: Res<Requests<T::Vertex>>,
    mut vertices: Local<Vec<Request<T::Vertex>>>,
) {
    for (mut drawer, e) in &mut query {
        drawer.draw(&param, e, &mut vertices);
    }

    requests.values.lock().unwrap().append(&mut vertices);
}

pub fn queue_vertices<T: Vertex>(
    mut commands: Commands,
    mut batch: ResMut<Batch<T>>,
    layer: Res<DrawLayer<T>>,
    mut requests: ResMut<Requests<T>>,
    draw_pipeline: Res<DrawPipeline<T>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<DrawPipeline<T>>>,
    pipeline_cache: Res<PipelineCache>,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    mut views: Query<(&mut RenderPhase<Transparent2d>, &ExtractedView)>,
) {
    let draw_function = draw_functions.read().id::<DrawFunction<T>>();

    let requests = requests.values.get_mut().unwrap();
    radsort::sort_by_key(requests, |req| req.layer);

    let Batch {
        ref mut vertices,
        ref mut indices,
    } = *batch;
    vertices.clear();
    indices.clear();

    let mut prev = None;
    for mut request in requests.drain(..) {
        let base_index = indices.len() as u32;

        let new_key = request.key;
        match prev.take() {
            None => prev = Some((base_index, new_key)),
            Some((prev_index, prev_key)) => {
                if &prev_key != &new_key {
                    for (mut phase, view) in &mut views {
                        phase.add(Transparent2d {
                            sort_key: FloatOrd(layer.layer),
                            entity: commands
                                .spawn(BatchSection {
                                    start: prev_index,
                                    end: base_index,
                                })
                                .id(),
                            pipeline: pipelines.specialize(
                                &pipeline_cache,
                                &draw_pipeline,
                                (DrawKey { hdr: view.hdr }, prev_key.clone()),
                            ),
                            draw_function,
                            batch_range: 0..0,
                            dynamic_offset: None,
                        });
                    }
                } else {
                    prev = Some((prev_index, new_key));
                }
            }
        }

        vertices.values_mut().append(&mut request.vertices);
        indices
            .values_mut()
            .extend(request.indices.into_iter().map(|i| base_index + i));
    }

    if let Some((prev_index, prev_key)) = prev.take() {
        for (mut phase, view) in &mut views {
            phase.add(Transparent2d {
                sort_key: FloatOrd(layer.layer),
                entity: commands
                    .spawn(BatchSection {
                        start: prev_index,
                        end: indices.len() as u32,
                    })
                    .id(),
                pipeline: pipelines.specialize(
                    &pipeline_cache,
                    &draw_pipeline,
                    (DrawKey { hdr: view.hdr }, prev_key.clone()),
                ),
                draw_function,
                batch_range: 0..0,
                dynamic_offset: None,
            });
        }
    }
}

pub fn prepare_vertices<T: Vertex>(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut batch: ResMut<Batch<T>>,
    view_uniforms: Res<ViewUniforms>,
    pipeline: Res<DrawPipeline<T>>,
    views: Query<Entity, With<ExtractedView>>,
) {
    let Some(view_binding) = view_uniforms.uniforms.binding() else {
        return;
    };

    batch.vertices.write_buffer(&render_device, &render_queue);
    batch.indices.write_buffer(&render_device, &render_queue);

    for view in &views {
        commands.entity(view).insert(BatchBindGroup::<T> {
            group: render_device.create_bind_group(
                "draw_view_group",
                &pipeline.view_layout,
                &BindGroupEntries::single(view_binding.clone()),
            ),
            _marker: PhantomData,
        });
    }
}

type DrawFunction<T> = (SetItemPipeline, SetBatchBindGroup<T, 0>);

pub struct SetBatchBindGroup<T: Vertex, const I: usize> {
    _marker: PhantomData<fn(T)>,
}

impl<T: Vertex, P: PhaseItem, const I: usize> RenderCommand<P> for SetBatchBindGroup<T, I> {
    type Param = ();
    type ViewQuery = (Read<ViewUniformOffset>, Read<BatchBindGroup<T>>);
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        _: &P,
        (view_uniform, bind_group): ROQueryItem<'w, Self::ViewQuery>,
        _: Option<ROQueryItem<'w, Self::ItemQuery>>,
        _: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &bind_group.group, &[view_uniform.offset]);
        RenderCommandResult::Success
    }
}

pub struct DrawBatch<T: Vertex> {
    _marker: PhantomData<fn(T)>,
}

impl<T: Vertex, P: PhaseItem> RenderCommand<P> for DrawBatch<T> {
    type Param = SRes<Batch<T>>;
    type ViewQuery = ();
    type ItemQuery = Read<BatchSection>;

    fn render<'w>(
        _: &P,
        _: ROQueryItem<'w, Self::ViewQuery>,
        section: Option<ROQueryItem<'w, Self::ItemQuery>>,
        batch: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(section) = section else {
            return RenderCommandResult::Failure
        };

        let Batch {
            ref vertices,
            ref indices,
        } = batch.into_inner();

        let (Some(vertices), Some(indices)) = (vertices.buffer(), indices.buffer()) else {
            return RenderCommandResult::Failure
        };

        pass.set_vertex_buffer(0, vertices.slice(..));
        pass.set_index_buffer(indices.slice(..), 0, IndexFormat::Uint32);
        pass.draw_indexed(section.start..section.end, 0, 0..1);

        RenderCommandResult::Success
    }
}
