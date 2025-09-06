use std::{cell::Cell, collections::HashSet};

use camera::CameraInterface;
use client_render_base::map::{
    map::{QuadAnimEvalResult, RenderMap},
    map_buffered::QuadLayerVisuals,
    map_pipeline::{MapGraphics, QuadRenderInfo},
};
use graphics::{
    graphics_mt::GraphicsMultiThreaded,
    handles::{
        backend::backend::GraphicsBackendHandle,
        buffer_object::buffer_object::{BufferObject, GraphicsBufferObjectHandle},
        canvas::canvas::GraphicsCanvasHandle,
        stream::stream::{GraphicsStreamHandle, StreamedUniforms},
        texture::texture::TextureContainer,
    },
};
use graphics_types::rendering::State;
use hiarc::{Hiarc, hi_closure};
use map::map::groups::layers::design::Quad;
use math::math::vector::{dvec2, ffixed, fvec3, nfvec4, ubvec4, vec2};
use pool::pool::Pool;
use rustc_hash::FxHashMap;

use crate::{
    actions::actions::{
        ActChangeQuadAttr, ActQuadLayerAddQuads, ActQuadLayerAddRemQuads, EditorAction,
    },
    client::EditorClient,
    map::{EditorLayer, EditorLayerUnionRef, EditorMap, EditorMapInterface},
    map_tools::{finish_design_quad_layer_buffer, upload_design_quad_layer_buffer},
    tools::{
        quad_layer::shared::QUAD_POINT_RADIUS_FACTOR,
        shared::{align_pos, in_radius, rotate},
        utils::render_rect,
    },
    utils::{UiCanvasSize, ui_pos_to_world_pos, ui_pos_to_world_pos_and_world_height},
};

use super::shared::{QuadPointerDownPoint, QuadSelectionQuads, render_quad_points};

#[derive(Debug, Hiarc)]
pub struct QuadBrushQuads {
    pub quads: Vec<Quad>,
    pub w: f32,
    pub h: f32,

    pub render: QuadLayerVisuals,
    pub map_render: MapGraphics,
    pub texture: TextureContainer,
}

#[derive(Debug, Hiarc)]
pub struct QuadSelection {
    pub is_background: bool,
    pub group: usize,
    pub layer: usize,
    pub quad_index: usize,
    pub quad: Quad,
    pub point: QuadPointerDownPoint,
    pub cursor_in_world_pos: vec2,
    pub cursor_corner_offset: vec2,
}

#[derive(Debug, Hiarc)]
pub enum QuadPointerDownState {
    None,
    /// quad corner/center point
    Point(QuadPointerDownPoint),
    /// selection of quads
    Selection(vec2),
}

impl QuadPointerDownState {
    pub fn is_selection(&self) -> bool {
        matches!(self, Self::Selection(_))
    }
}

/// quad brushes are relative to where the mouse selected them
#[derive(Debug, Hiarc)]
pub struct QuadBrush {
    pub brush: Option<QuadBrushQuads>,

    /// this is the last quad selected (clicked on the corner selectors), this can be used
    /// for the animation to know the current quad
    pub last_popup: Option<QuadSelectionQuads>,
    /// The quad point last moved/rotated etc.
    pub last_translation: Option<QuadSelection>,
    /// The quad point last selected, moved etc.
    pub last_selection: Option<QuadSelectionQuads>,

    pub pointer_down_state: QuadPointerDownState,

    pub pos_offset: dvec2,
}

impl Default for QuadBrush {
    fn default() -> Self {
        Self::new()
    }
}

impl QuadBrush {
    pub fn new() -> Self {
        Self {
            brush: Default::default(),
            last_popup: None,
            last_translation: None,
            last_selection: None,
            pointer_down_state: QuadPointerDownState::None,

            pos_offset: dvec2::default(),
        }
    }

    fn handle_brush_select(
        &mut self,
        ui_canvas: &UiCanvasSize,
        graphics_mt: &GraphicsMultiThreaded,
        buffer_object_handle: &GraphicsBufferObjectHandle,
        backend_handle: &GraphicsBackendHandle,
        canvas_handle: &GraphicsCanvasHandle,
        map: &mut EditorMap,
        fake_texture: &TextureContainer,
        latest_pointer: &egui::PointerState,
        current_pointer_pos: &egui::Pos2,
        latest_modifiers: &egui::Modifiers,
        latest_keys_down: &HashSet<egui::Key>,
        client: &mut EditorClient,
    ) {
        let layer = map.active_layer();
        let (offset, parallax) = if let Some(layer) = &layer {
            layer.get_offset_and_parallax()
        } else {
            Default::default()
        };
        let Some(EditorLayerUnionRef::Design {
            layer: EditorLayer::Quad(layer),
            group_index,
            is_background,
            layer_index,
            ..
        }) = layer
        else {
            return;
        };

        let parallax_aware_zoom = map.groups.user.parallax_aware_zoom;

        let pointer_cur = vec2::new(current_pointer_pos.x, current_pointer_pos.y);

        let is_primary_allowed_down = !latest_modifiers.ctrl && latest_pointer.primary_down();
        let is_primary_allowed_pressed = !latest_modifiers.ctrl && latest_pointer.primary_pressed();

        let vec2 {
            x: mut x1,
            y: mut y1,
        } = ui_pos_to_world_pos(
            canvas_handle,
            ui_canvas,
            map.groups.user.zoom,
            vec2::new(pointer_cur.x, pointer_cur.y),
            map.groups.user.pos.x,
            map.groups.user.pos.y,
            offset.x,
            offset.y,
            parallax.x,
            parallax.y,
            parallax_aware_zoom,
        );

        // if pointer was already down
        if let QuadPointerDownState::Selection(pointer_down) = &self.pointer_down_state {
            // find current layer
            let &vec2 {
                x: mut x0,
                y: mut y0,
            } = pointer_down;

            if x0 > x1 {
                std::mem::swap(&mut x0, &mut x1);
            }
            if y0 > y1 {
                std::mem::swap(&mut y0, &mut y1);
            }

            // check if any quads are in the selection
            let mut quads: Vec<Quad> = Default::default();

            for quad in &layer.layer.quads {
                let points =
                    super::shared::get_quad_points_animated(quad, map, map.user.render_time());

                if super::shared::in_box(&points[0], x0, y0, x1, y1)
                    || super::shared::in_box(&points[1], x0, y0, x1, y1)
                    || super::shared::in_box(&points[2], x0, y0, x1, y1)
                    || super::shared::in_box(&points[3], x0, y0, x1, y1)
                    || super::shared::in_box(&points[4], x0, y0, x1, y1)
                {
                    quads.push(*quad);
                }
            }

            // if there is an selection, apply that
            if !quads.is_empty() {
                let pointer_down = vec2::new(x0, y0);

                let x = -pointer_down.x;
                let y = -pointer_down.y;

                for quad in &mut quads {
                    for point in &mut quad.points {
                        point.x += ffixed::from_num(x);
                        point.y += ffixed::from_num(y);
                    }
                }

                let buffer =
                    upload_design_quad_layer_buffer(graphics_mt, &layer.layer.attr, &quads);
                let render =
                    finish_design_quad_layer_buffer(buffer_object_handle, backend_handle, buffer);
                self.brush = Some(QuadBrushQuads {
                    quads,
                    w: x1 - x0,
                    h: y1 - y0,
                    render,
                    map_render: MapGraphics::new(backend_handle),
                    texture: layer
                        .layer
                        .attr
                        .image
                        .map(|img| map.resources.images[img].user.user.clone())
                        .unwrap_or_else(|| fake_texture.clone()),
                });
            } else {
                // else unset
                self.brush = None;
            }

            if !is_primary_allowed_down {
                self.pointer_down_state = QuadPointerDownState::None;
            }
        } else {
            let align_pos = |pos: vec2| align_pos(map, latest_modifiers, pos);

            // check if the pointer clicked on one of the quad corner/center points
            let mut clicked_quad_point = false;
            if is_primary_allowed_pressed || latest_pointer.secondary_pressed() {
                for (q, quad) in layer.layer.quads.iter().enumerate() {
                    let points =
                        super::shared::get_quad_points_animated(quad, map, map.user.render_time());

                    let pointer_cur = vec2::new(current_pointer_pos.x, current_pointer_pos.y);

                    let (pointer_cur, h) = ui_pos_to_world_pos_and_world_height(
                        canvas_handle,
                        ui_canvas,
                        map.groups.user.zoom,
                        vec2::new(pointer_cur.x, pointer_cur.y),
                        map.groups.user.pos.x,
                        map.groups.user.pos.y,
                        offset.x,
                        offset.y,
                        parallax.x,
                        parallax.y,
                        parallax_aware_zoom,
                    );
                    let h = h / canvas_handle.canvas_height() as f32;
                    let radius = QUAD_POINT_RADIUS_FACTOR * h;
                    let mut p = [false; 5];
                    p.iter_mut().enumerate().for_each(|(index, p)| {
                        *p = in_radius(&points[index], &pointer_cur, radius)
                    });
                    if let Some((index, _)) = p.iter().enumerate().rev().find(|&(_, &p)| p) {
                        // pointer is in a drag mode
                        clicked_quad_point = true;
                        let down_point = if index == 4 {
                            QuadPointerDownPoint::Center
                        } else {
                            QuadPointerDownPoint::Corner(index)
                        };
                        let quad_pos =
                            vec2::new(points[index].x.to_num(), points[index].y.to_num());
                        let cursor = vec2::new(x1, y1);
                        self.pointer_down_state = QuadPointerDownState::Point(down_point);
                        if is_primary_allowed_pressed {
                            self.last_translation = Some(QuadSelection {
                                is_background,
                                group: group_index,
                                layer: layer_index,
                                quad_index: q,
                                quad: *quad,
                                point: down_point,
                                cursor_in_world_pos: cursor,
                                cursor_corner_offset: cursor - quad_pos,
                            });
                        } else {
                            self.last_popup = Some(QuadSelectionQuads {
                                quads: vec![(q, *quad)].into_iter().collect(),
                                x: 0.0,
                                y: 0.0,
                                w: 0.0,
                                h: 0.0,
                                point: Some(down_point),
                            });
                        }
                        self.last_selection = Some(QuadSelectionQuads {
                            quads: vec![(q, *quad)].into_iter().collect(),
                            x: 0.0,
                            y: 0.0,
                            w: 0.0,
                            h: 0.0,
                            point: Some(down_point),
                        });

                        break;
                    }
                }
            }
            // else check if the pointer is down now
            if !clicked_quad_point && is_primary_allowed_pressed && self.last_translation.is_none()
            {
                let pointer_cur = vec2::new(current_pointer_pos.x, current_pointer_pos.y);
                let pos = ui_pos_to_world_pos(
                    canvas_handle,
                    ui_canvas,
                    map.groups.user.zoom,
                    vec2::new(pointer_cur.x, pointer_cur.y),
                    map.groups.user.pos.x,
                    map.groups.user.pos.y,
                    offset.x,
                    offset.y,
                    parallax.x,
                    parallax.y,
                    parallax_aware_zoom,
                );
                self.pointer_down_state = QuadPointerDownState::Selection(pos);
            }
            if !clicked_quad_point && is_primary_allowed_pressed {
                self.last_translation = None;
                self.last_selection = None;
            }
            if is_primary_allowed_down && self.last_translation.is_some() {
                let last_active = self.last_translation.as_mut().unwrap();
                let new_pos = vec2::new(x1, y1);
                let aligned_pos = align_pos(new_pos);
                let new_pos = if let Some(aligned_pos) = aligned_pos {
                    aligned_pos + last_active.cursor_corner_offset
                } else {
                    new_pos
                };
                if let Some(edit_quad) = layer.layer.quads.get(last_active.quad_index).copied() {
                    let p = match last_active.point {
                        QuadPointerDownPoint::Center => 4,
                        QuadPointerDownPoint::Corner(index) => index,
                    };
                    let cursor_pos = last_active.cursor_in_world_pos;

                    let quad = &mut last_active.quad;

                    let pos_anim = edit_quad.pos_anim;
                    let alter_anim_point = map.user.change_animations() && pos_anim.is_some();

                    if matches!(last_active.point, QuadPointerDownPoint::Center)
                        && latest_keys_down.contains(&egui::Key::R)
                    {
                        // handle rotation
                        let diff = new_pos - vec2::new(cursor_pos.x, cursor_pos.y);
                        let diff = diff.x;

                        let (points, center) = quad.points.split_at_mut(4);

                        if alter_anim_point {
                            if let Some(pos) = &mut map.animations.user.active_anim_points.pos {
                                pos.value.z += ffixed::from_num(diff);
                            }
                        } else {
                            rotate(&center[0], ffixed::from_num(diff), points);
                        }
                    } else {
                        // handle position
                        let diff_x = ffixed::from_num(new_pos.x - cursor_pos.x);
                        let diff_y = ffixed::from_num(new_pos.y - cursor_pos.y);

                        if alter_anim_point {
                            if let Some(pos) = &mut map.animations.user.active_anim_points.pos {
                                pos.value.x += diff_x;
                                pos.value.y += diff_y;
                            }
                        } else {
                            quad.points[p].x += diff_x;
                            quad.points[p].y += diff_y;

                            if matches!(last_active.point, QuadPointerDownPoint::Center)
                                && !latest_modifiers.shift
                            {
                                // move other points too (because shift is not pressed to only move center)
                                for i in 0..4 {
                                    quad.points[i].x += diff_x;
                                    quad.points[i].y += diff_y;
                                }
                            }
                        }
                    }

                    if *quad != edit_quad {
                        let index = last_active.quad_index;
                        client.execute(
                            EditorAction::ChangeQuadAttr(Box::new(ActChangeQuadAttr {
                                is_background,
                                group_index,
                                layer_index,
                                old_attr: edit_quad,
                                new_attr: *quad,

                                index,
                            })),
                            Some(&format!(
                                "change-quad-attr-{is_background}-{group_index}-{layer_index}-{index}"
                            )),
                        );
                    }
                }

                last_active.cursor_in_world_pos = new_pos;
            }
        }
    }

    pub fn handle_brush_draw(
        &mut self,
        ui_canvas: &UiCanvasSize,
        canvas_handle: &GraphicsCanvasHandle,
        map: &EditorMap,
        latest_pointer: &egui::PointerState,
        latest_modifiers: &egui::Modifiers,
        current_pointer_pos: &egui::Pos2,
        client: &mut EditorClient,
    ) {
        let layer = map.active_layer().unwrap();
        let (offset, parallax) = layer.get_offset_and_parallax();

        let is_primary_allowed_pressed = !latest_modifiers.ctrl && latest_pointer.primary_pressed();

        // reset brush
        if latest_pointer.secondary_pressed() {
            self.brush = None;
        }
        // apply brush
        else {
            let brush = self.brush.as_ref().unwrap();

            if is_primary_allowed_pressed {
                let pos = current_pointer_pos;

                let pos = vec2::new(pos.x, pos.y);

                let vec2 { x, y } = ui_pos_to_world_pos(
                    canvas_handle,
                    ui_canvas,
                    map.groups.user.zoom,
                    vec2::new(pos.x, pos.y),
                    map.groups.user.pos.x,
                    map.groups.user.pos.y,
                    offset.x,
                    offset.y,
                    parallax.x,
                    parallax.y,
                    map.groups.user.parallax_aware_zoom,
                );

                let mut quads = brush.quads.clone();
                for quad in &mut quads {
                    for point in &mut quad.points {
                        point.x += ffixed::from_num(x);
                        point.y += ffixed::from_num(y);
                    }
                }

                if let Some((action, group_indentifier)) = if let EditorLayerUnionRef::Design {
                    layer: EditorLayer::Quad(layer),
                    layer_index,
                    is_background,
                    group_index,
                    ..
                } = layer
                {
                    Some((
                        EditorAction::QuadLayerAddQuads(ActQuadLayerAddQuads {
                            base: ActQuadLayerAddRemQuads {
                                is_background,
                                group_index,
                                layer_index,
                                index: layer.layer.quads.len(),
                                quads,
                            },
                        }),
                        format!("quad-brush design {layer_index}"),
                    ))
                } else {
                    None
                } {
                    client.execute(action, Some(&group_indentifier));
                }
            }
        }
    }

    fn render_selection(
        &self,
        ui_canvas: &UiCanvasSize,
        canvas_handle: &GraphicsCanvasHandle,
        stream_handle: &GraphicsStreamHandle,
        map: &EditorMap,
        latest_pointer: &egui::PointerState,
        latest_modifiers: &egui::Modifiers,
        current_pointer_pos: &egui::Pos2,
    ) {
        let layer = map.active_layer();
        let (offset, parallax) = if let Some(layer) = &layer {
            layer.get_offset_and_parallax()
        } else {
            Default::default()
        };
        let is_primary_allowed_down = !latest_modifiers.ctrl && latest_pointer.primary_down();
        // if pointer was already down
        if let QuadPointerDownState::Selection(pointer_down) = &self.pointer_down_state
            && is_primary_allowed_down
        {
            let pos = current_pointer_pos;
            let pos = ui_pos_to_world_pos(
                canvas_handle,
                ui_canvas,
                map.groups.user.zoom,
                vec2::new(pos.x, pos.y),
                map.groups.user.pos.x,
                map.groups.user.pos.y,
                offset.x,
                offset.y,
                parallax.x,
                parallax.y,
                map.groups.user.parallax_aware_zoom,
            );
            let pos = egui::pos2(pos.x, pos.y);

            let down_pos = pointer_down;
            let down_pos = egui::pos2(down_pos.x, down_pos.y);

            let rect = egui::Rect::from_two_pos(pos, down_pos);

            render_rect(
                canvas_handle,
                stream_handle,
                map,
                rect,
                ubvec4::new(255, 0, 0, 255),
                &parallax,
                &offset,
            );
        }
    }

    fn render_brush(
        &self,
        ui_canvas: &UiCanvasSize,
        canvas_handle: &GraphicsCanvasHandle,
        stream_handle: &GraphicsStreamHandle,
        map: &EditorMap,
        current_pointer_pos: &egui::Pos2,
    ) {
        let layer = map.active_layer();
        let (offset, parallax) = if let Some(layer) = &layer {
            layer.get_offset_and_parallax()
        } else {
            Default::default()
        };

        let brush = self.brush.as_ref().unwrap();

        let pos = current_pointer_pos;
        let pos_on_map = ui_pos_to_world_pos(
            canvas_handle,
            ui_canvas,
            map.groups.user.zoom,
            vec2::new(pos.x, pos.y),
            map.groups.user.pos.x,
            map.groups.user.pos.y,
            offset.x,
            offset.y,
            parallax.x,
            parallax.y,
            map.groups.user.parallax_aware_zoom,
        );
        let pos = pos_on_map;
        let pos = egui::pos2(pos.x, pos.y);

        let mut state = State::new();
        map.game_camera().project(
            canvas_handle,
            &mut state,
            layer.map(|layer| layer.get_or_fake_group_attr()).as_ref(),
        );

        let center = -pos_on_map;
        state.canvas_br.x += center.x;
        state.canvas_br.y += center.y;
        state.canvas_tl.x += center.x;
        state.canvas_tl.y += center.y;
        if let Some(buffer_object_index) = &brush.render.buffer_object_index {
            let quads = &brush.quads;
            let cur_time = &map.user.render_time();
            let cur_anim_time = cur_time;
            let cur_quad_offset_cell = Cell::new(0);
            let cur_quad_offset = &cur_quad_offset_cell;
            let animations = map.active_animations();
            let include_last_anim_point = map.user.include_last_anim_point();

            let QuadAnimEvalResult {
                pos_anims_values,
                color_anims_values,
            } = RenderMap::prepare_quad_anims(
                &Pool::with_capacity(8),
                &Pool::with_capacity(8),
                cur_time,
                cur_anim_time,
                include_last_anim_point,
                &brush.render,
                animations,
            );

            let pos_anims_values = &*pos_anims_values;
            let color_anims_values = &*color_anims_values;

            stream_handle.fill_uniform_instance(
                hi_closure!(
                    [
                        pos_anims_values: &FxHashMap<(usize, time::Duration), fvec3>,
                        color_anims_values: &FxHashMap<(usize, time::Duration), nfvec4>,
                        cur_quad_offset: &Cell<usize>,
                        quads: &Vec<Quad>,
                    ],
                    |stream_handle: StreamedUniforms<
                        '_,
                        QuadRenderInfo,
                    >|
                    -> () {
                        RenderMap::prepare_quad_rendering(
                            stream_handle,
                            color_anims_values,
                            pos_anims_values,
                            cur_quad_offset,
                            quads,
                            0
                        );
                    }
                ),
                hi_closure!([
                    brush: &QuadBrushQuads,
                    state: State,
                    buffer_object_index: &BufferObject,
                    cur_quad_offset: &Cell<usize>,
                ], |instance: usize, count: usize| -> () {
                    brush.map_render.render_quad_layer(
                        &state,
                        (&brush.texture).into(),
                        buffer_object_index,
                        instance,
                        count,
                        cur_quad_offset.get(),
                    );
                    cur_quad_offset.set(cur_quad_offset.get() + count);
                }),
            );
        }

        let brush_size = vec2::new(brush.w, brush.h);
        let rect =
            egui::Rect::from_min_max(pos, egui::pos2(pos.x + brush_size.x, pos.y + brush_size.y));

        render_rect(
            canvas_handle,
            stream_handle,
            map,
            rect,
            ubvec4::new(255, 0, 0, 255),
            &parallax,
            &offset,
        );
    }

    pub fn update(
        &mut self,
        ui_canvas: &UiCanvasSize,
        graphics_mt: &GraphicsMultiThreaded,
        buffer_object_handle: &GraphicsBufferObjectHandle,
        backend_handle: &GraphicsBackendHandle,
        canvas_handle: &GraphicsCanvasHandle,
        map: &mut EditorMap,
        fake_texture: &TextureContainer,
        latest_pointer: &egui::PointerState,
        current_pointer_pos: &egui::Pos2,
        latest_modifiers: &egui::Modifiers,
        latest_keys_down: &HashSet<egui::Key>,
        client: &mut EditorClient,
    ) {
        let layer = map.active_layer();
        if !layer.as_ref().is_some_and(|layer| layer.is_quad_layer()) {
            return;
        }

        if self.brush.is_none() || self.pointer_down_state.is_selection() {
            self.handle_brush_select(
                ui_canvas,
                graphics_mt,
                buffer_object_handle,
                backend_handle,
                canvas_handle,
                map,
                fake_texture,
                latest_pointer,
                current_pointer_pos,
                latest_modifiers,
                latest_keys_down,
                client,
            );
        } else {
            self.handle_brush_draw(
                ui_canvas,
                canvas_handle,
                map,
                latest_pointer,
                latest_modifiers,
                current_pointer_pos,
                client,
            );
        }
    }

    pub fn render(
        &mut self,
        ui_canvas: &UiCanvasSize,
        stream_handle: &GraphicsStreamHandle,
        canvas_handle: &GraphicsCanvasHandle,
        map: &EditorMap,
        latest_pointer: &egui::PointerState,
        latest_modifiers: &egui::Modifiers,
        current_pointer_pos: &egui::Pos2,
    ) {
        let layer = map.active_layer();
        if !layer.as_ref().is_some_and(|layer| layer.is_quad_layer()) {
            return;
        }

        render_quad_points(
            ui_canvas,
            layer,
            current_pointer_pos,
            stream_handle,
            canvas_handle,
            map,
            true,
        );

        if self.brush.is_none() || self.pointer_down_state.is_selection() {
            self.render_selection(
                ui_canvas,
                canvas_handle,
                stream_handle,
                map,
                latest_pointer,
                latest_modifiers,
                current_pointer_pos,
            );
        } else {
            self.render_brush(
                ui_canvas,
                canvas_handle,
                stream_handle,
                map,
                current_pointer_pos,
            );
        }
    }
}
