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

use crate::shape::vertex::{DrawLayer, Request, Vertex, VertexKey};

pub struct ShapePipeline<T: Vertex> {
    view_layout: BindGroupLayout,
    _marker: PhantomData<fn(T)>,
}

impl<T: Vertex> Resource for ShapePipeline<T> {}
impl<T: Vertex> FromWorld for ShapePipeline<T> {
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
pub struct ShapeCommonKey {
    pub hdr: bool,
    pub msaa: u8,
}

impl<T: Vertex> SpecializedRenderPipeline for ShapePipeline<T> {
    type Key = (ShapeCommonKey, T::Key);

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
                count: 1 << common.msaa,
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

pub fn queue_vertices<T: Vertex>(
    mut commands: Commands,
    msaa: Res<Msaa>,
    mut batch: ResMut<Batch<T>>,
    layer: Res<DrawLayer<T>>,
    mut requests: ResMut<Requests<T>>,
    draw_pipeline: Res<ShapePipeline<T>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<ShapePipeline<T>>>,
    pipeline_cache: Res<PipelineCache>,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    mut views: Query<(&mut RenderPhase<Transparent2d>, &ExtractedView)>,
) {
    let draw_function = draw_functions.read().id::<DrawShapes<T>>();
    let msaa = msaa.samples().trailing_zeros() as u8;

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
        let base_offset = indices.len() as u32;

        let new_key = request.key;
        match prev.take() {
            None => prev = Some((base_offset, new_key)),
            Some((prev_offset, prev_key)) => {
                if &prev_key != &new_key {
                    for (mut phase, view) in &mut views {
                        phase.add(Transparent2d {
                            sort_key: FloatOrd(layer.layer),
                            entity: commands
                                .spawn(BatchSection {
                                    start: prev_offset,
                                    end: base_offset,
                                })
                                .id(),
                            pipeline: pipelines.specialize(
                                &pipeline_cache,
                                &draw_pipeline,
                                (ShapeCommonKey { hdr: view.hdr, msaa }, prev_key.clone()),
                            ),
                            draw_function,
                            batch_range: 0..1,
                            dynamic_offset: None,
                        });
                    }
                }

                prev = Some((prev_offset, new_key));
            }
        }

        let base_index = vertices.len() as u32;
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
                    (ShapeCommonKey { hdr: view.hdr, msaa }, prev_key.clone()),
                ),
                draw_function,
                batch_range: 0..1,
                dynamic_offset: None,
            });
        }
    }
}

pub fn prepare_vertices_batch<T: Vertex>(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut batch: ResMut<Batch<T>>,
) {
    batch.vertices.write_buffer(&render_device, &render_queue);
    batch.indices.write_buffer(&render_device, &render_queue);
}

pub fn prepare_vertices_bind_group<T: Vertex>(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    view_uniforms: Res<ViewUniforms>,
    pipeline: Res<ShapePipeline<T>>,
    views: Query<Entity, With<ExtractedView>>,
) {
    let Some(view_binding) = view_uniforms.uniforms.binding() else {
        return
    };

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

pub type DrawShapes<T> = (SetItemPipeline, SetBatchBindGroup<T, 0>, DrawBatch<T>);

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
