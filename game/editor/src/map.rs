use std::{
    borrow::{Borrow, BorrowMut},
    collections::{HashMap, HashSet},
    fmt::Debug,
    path::PathBuf,
    rc::Rc,
    time::Duration,
};

use base::{hash::Hash, linked_hash_map_view::FxLinkedHashMap};
use base_io::runtime::IoRuntimeTask;
use camera::Camera;
use client_render_base::map::{
    map_buffered::{PhysicsTileLayerVisuals, QuadLayerVisuals, SoundLayerSounds, TileLayerVisuals},
    render_pipe::GameTimeInfo,
};
use egui_file_dialog::FileDialog;
use egui_timeline::timeline::Timeline;
use graphics::handles::texture::texture::{TextureContainer, TextureContainer2dArray};
use hiarc::Hiarc;
use map::{
    map::{
        animations::{
            AnimPointColor, AnimPointPos, AnimPointSound, ColorAnimation, PosAnimation,
            SoundAnimation,
        },
        command_value::CommandValue,
        groups::{
            MapGroupAttr, MapGroupPhysicsAttr,
            layers::{
                design::{MapLayerQuadsAttrs, MapLayerSoundAttrs},
                tiles::MapTileLayerAttr,
            },
        },
    },
    skeleton::{
        MapSkeleton,
        animations::{
            AnimationsSkeleton, ColorAnimationSkeleton, PosAnimationSkeleton,
            SoundAnimationSkeleton,
        },
        config::ConfigSkeleton,
        groups::{
            MapGroupPhysicsSkeleton, MapGroupSkeleton, MapGroupsSkeleton,
            layers::{
                design::{
                    MapLayerArbitrarySkeleton, MapLayerQuadSkeleton, MapLayerSkeleton,
                    MapLayerSoundSkeleton, MapLayerTileSkeleton,
                },
                physics::MapLayerPhysicsSkeleton,
            },
        },
        metadata::MetadataSkeleton,
        resources::{MapResourceRefSkeleton, MapResourcesSkeleton},
    },
    types::NonZeroU16MinusOne,
};
use math::math::vector::{ffixed, fvec2, vec2};
use sound::{scene_object::SceneObject, sound_listener::SoundListener, sound_object::SoundObject};

use crate::event::EditorEventLayerIndex;

pub trait EditorCommonLayerOrGroupAttrInterface {
    fn editor_attr(&self) -> &EditorCommonGroupOrLayerAttr;
    fn editor_attr_mut(&mut self) -> &mut EditorCommonGroupOrLayerAttr;
}

pub trait EditorDesignLayerInterface {
    fn is_selected(&self) -> bool;
}

pub trait EditorPhysicsLayerInterface {
    fn is_selected(&self) -> bool;
}

#[derive(Debug, Default, Clone)]
pub struct EditorCommonGroupOrLayerAttr {
    pub hidden: bool,
    // active layer/group, e.g. a brush on an active tile layer would have effect
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct ResourceSelection {
    /// the resource the resource selector currently hovers over
    pub hovered_resource: Option<Option<usize>>,
}

#[derive(Debug, Clone)]
pub struct EditorTileLayerPropsSelection {
    pub attr: MapTileLayerAttr,
    pub name: String,
    pub image_2d_array_selection_open: Option<ResourceSelection>,
}

#[derive(Debug, Clone)]
pub struct EditorTileLayerProps {
    pub visuals: TileLayerVisuals,
    pub attr: EditorCommonGroupOrLayerAttr,
    // selected e.g. by a right-click or by a SHIFT/CTRL + left-click in a multi select
    pub selected: Option<EditorTileLayerPropsSelection>,

    pub auto_mapper_rule: Option<String>,
    pub auto_mapper_seed: Option<u64>,

    /// This field is also used by the server, care!
    pub live_edit: Option<(u64, (String, String, Hash))>,
}

impl Borrow<TileLayerVisuals> for EditorTileLayerProps {
    fn borrow(&self) -> &TileLayerVisuals {
        &self.visuals
    }
}

impl BorrowMut<TileLayerVisuals> for EditorTileLayerProps {
    fn borrow_mut(&mut self) -> &mut TileLayerVisuals {
        &mut self.visuals
    }
}

#[derive(Debug, Clone)]
pub struct EditorQuadLayerPropsPropsSelection {
    pub attr: MapLayerQuadsAttrs,
    pub name: String,
    pub image_selection_open: Option<ResourceSelection>,
}

#[derive(Debug, Clone)]
pub struct EditorQuadLayerProps {
    pub visuals: QuadLayerVisuals,
    pub attr: EditorCommonGroupOrLayerAttr,
    // selected e.g. by a right-click or by a SHIFT/CTRL + left-click in a multi select
    pub selected: Option<EditorQuadLayerPropsPropsSelection>,
}

impl Borrow<QuadLayerVisuals> for EditorQuadLayerProps {
    fn borrow(&self) -> &QuadLayerVisuals {
        &self.visuals
    }
}

impl BorrowMut<QuadLayerVisuals> for EditorQuadLayerProps {
    fn borrow_mut(&mut self) -> &mut QuadLayerVisuals {
        &mut self.visuals
    }
}

#[derive(Debug, Clone)]
pub struct EditorArbitraryLayerProps {
    pub attr: EditorCommonGroupOrLayerAttr,
}

#[derive(Debug, Clone)]
pub struct EditorSoundLayerPropsPropsSelection {
    pub attr: MapLayerSoundAttrs,
    pub name: String,
    pub sound_selection_open: Option<ResourceSelection>,
}

#[derive(Debug, Clone)]
pub struct EditorSoundLayerProps {
    pub sounds: SoundLayerSounds,
    pub attr: EditorCommonGroupOrLayerAttr,
    // selected e.g. by a right-click or by a SHIFT/CTRL + left-click in a multi select
    pub selected: Option<EditorSoundLayerPropsPropsSelection>,
}

impl Borrow<SoundLayerSounds> for EditorSoundLayerProps {
    fn borrow(&self) -> &SoundLayerSounds {
        &self.sounds
    }
}

impl BorrowMut<SoundLayerSounds> for EditorSoundLayerProps {
    fn borrow_mut(&mut self) -> &mut SoundLayerSounds {
        &mut self.sounds
    }
}

#[derive(Debug, Default, Clone)]
pub struct EditorPhysicsLayerNumberExtra {
    pub name: String,
    pub extra: FxLinkedHashMap<String, CommandValue>,
    pub enter_extra: Option<String>,
    pub leave_extra: Option<String>,
}

#[derive(Debug, Hiarc, Default, Clone, PartialEq, Eq)]
pub struct TuneZoneEdit {
    pub name: Option<String>,
    pub tunes: FxLinkedHashMap<String, CommandValue>,

    /// Message a server/client _can_ display, if the tee enters this tune zone.
    pub enter_msg: Option<String>,
    /// Message a server/client _can_ display, if the tee leaves this tune zone.
    pub leave_msg: Option<String>,
}

impl TuneZoneEdit {
    pub fn in_use(&self) -> bool {
        self.name.is_some()
            || !self.tunes.is_empty()
            || self.enter_msg.is_some()
            || self.leave_msg.is_some()
    }
}

pub type TuneOverviewExtra = FxLinkedHashMap<u8, TuneZoneEdit>;

#[derive(Debug, Clone)]
pub struct EditorPhysicsLayerProps {
    pub visuals: PhysicsTileLayerVisuals,
    pub attr: EditorCommonGroupOrLayerAttr,
    // selected e.g. by a right-click or by a SHIFT/CTRL + left-click in a multi select
    pub selected: Option<()>,
    /// for physics layers that have numbers that reference other stuff
    /// e.g. tele, switch & tune zone layer
    pub number_extra: FxLinkedHashMap<u8, EditorPhysicsLayerNumberExtra>,
    pub number_extra_text: String,
    pub enter_extra_text: String,
    pub leave_extra_text: String,

    // TODO: clean this up into own props for tune layers etc.
    pub tune_overview_extra: TuneOverviewExtra,
    pub number_extra_zone: u8,

    pub switch_delay: u8,

    pub speedup_force: u8,
    pub speedup_angle: i16,
    pub speedup_max_speed: u8,

    pub context_menu_open: bool,
    pub context_menu_extra_open: bool,
}

impl Borrow<PhysicsTileLayerVisuals> for EditorPhysicsLayerProps {
    fn borrow(&self) -> &PhysicsTileLayerVisuals {
        &self.visuals
    }
}

impl BorrowMut<PhysicsTileLayerVisuals> for EditorPhysicsLayerProps {
    fn borrow_mut(&mut self) -> &mut PhysicsTileLayerVisuals {
        &mut self.visuals
    }
}

#[derive(Debug, Clone)]
pub struct EditorGroupPropsSelection {
    pub attr: MapGroupAttr,
    pub name: String,
}

#[derive(Debug, Default, Clone)]
pub struct EditorGroupProps {
    pub attr: EditorCommonGroupOrLayerAttr,
    // selected e.g. by a right-click or by a SHIFT/CTRL + left-click in a multi select
    pub selected: Option<EditorGroupPropsSelection>,
}

#[derive(Debug, Default, Clone)]
pub struct EditorPhysicsGroupProps {
    pub attr: EditorCommonGroupOrLayerAttr,
    // selected e.g. by a right-click or by a SHIFT/CTRL + left-click in a multi select
    pub selected: Option<MapGroupPhysicsAttr>,

    /// currently active tele, e.g. to draw tele outcomes
    pub active_tele: u8,
    /// currently active switch, e.g. to draw a switch trigger
    pub active_switch: u8,
    /// currently active tune zone, e.g. to draw a tune tile
    /// referencing the active tune zone
    pub active_tune_zone: u8,
    /// when the tele is selected, the client checks if the tele
    /// was already used and caches it here
    pub active_tele_in_use: Option<bool>,
    /// when the switch is selected, the client checks if the switch
    /// was already used and caches it here
    pub active_switch_in_use: Option<bool>,
    /// when the tune zone is selected, the client checks if the tune zone
    /// was already used and caches it here
    pub active_tune_zone_in_use: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct EditorGroupsProps {
    pub pos: vec2,
    pub zoom: f32,
    pub parallax_aware_zoom: bool,
}

#[derive(Debug, Hiarc, Clone)]
pub struct EditorResource<U, P> {
    pub file: Rc<Vec<u8>>,
    pub user: U,
    pub props: P,
    pub hq: Option<(Rc<Vec<u8>>, U)>,
}

impl<U, P> Borrow<U> for EditorResource<U, P> {
    fn borrow(&self) -> &U {
        &self.user
    }
}

#[derive(Debug, Hiarc, Clone)]
pub struct EditorResourceTexture2dArray {
    pub tile_non_fully_transparent_percentage: [u8; 256],
}

impl EditorResourceTexture2dArray {
    pub fn new(img: &[u8], single_width: usize, single_height: usize) -> Self {
        let mut tile_non_fully_transparent_percentage = vec![0u8; 256];
        let single_pitch = single_width * 4;
        let single_size = single_pitch * single_height;
        for y in 0..16 {
            for x in 0..16 {
                let i = y * 16 + x;
                let x_off = x * single_size;
                let y_off = y * single_size * 16;
                let mut non_transparent_counter = 0;
                for y in 0..single_height {
                    for x in 0..single_width {
                        let index = y_off + (y * single_pitch) + x_off + x * 4;
                        let alpha = img[index + 3];
                        if alpha > 0 {
                            non_transparent_counter += 1;
                        }
                    }
                }

                tile_non_fully_transparent_percentage[i] =
                    ((non_transparent_counter * 100) / (single_width * single_height)) as u8;
            }
        }
        Self {
            tile_non_fully_transparent_percentage: tile_non_fully_transparent_percentage
                .try_into()
                .unwrap(),
        }
    }
}

pub type EditorImage = MapResourceRefSkeleton<EditorResource<TextureContainer, ()>>;
pub type EditorImage2dArray =
    MapResourceRefSkeleton<EditorResource<TextureContainer2dArray, EditorResourceTexture2dArray>>;
pub type EditorSound = MapResourceRefSkeleton<EditorResource<SoundObject, ()>>;

pub type EditorResources = MapResourcesSkeleton<
    (),
    EditorResource<TextureContainer, ()>,
    EditorResource<TextureContainer2dArray, EditorResourceTexture2dArray>,
    EditorResource<SoundObject, ()>,
>;

#[derive(Debug, Hiarc, Default, Clone)]
pub struct EditorActiveAnimationProps {
    // timeline graph
    pub selected_points: HashSet<usize>,
    pub hovered_point: Option<usize>,
    // value graph
    pub selected_point_channels: HashMap<usize, HashSet<usize>>,
    pub hovered_point_channels: HashMap<usize, HashSet<usize>>,
    pub selected_point_channel_beziers: HashMap<usize, HashSet<(usize, bool)>>,
    pub hovered_point_channel_beziers: HashMap<usize, HashSet<(usize, bool)>>,
}

#[derive(Debug, Hiarc, Default, Clone)]
pub struct EditorActiveAnim {
    pub pos: Option<(usize, PosAnimation, EditorActiveAnimationProps)>,
    pub color: Option<(usize, ColorAnimation, EditorActiveAnimationProps)>,
    pub sound: Option<(usize, SoundAnimation, EditorActiveAnimationProps)>,
}

#[derive(Debug, Hiarc, Default, Clone)]
pub struct EditorActiveAnimPoint {
    pub pos: Option<AnimPointPos>,
    pub color: Option<AnimPointColor>,
    pub sound: Option<AnimPointSound>,
}

#[derive(Debug, Hiarc, Default, Clone)]
pub struct EditorAnimationsProps {
    pub selected_pos_anim: Option<usize>,
    pub selected_color_anim: Option<usize>,
    pub selected_sound_anim: Option<usize>,

    // current selected anim points to fake
    pub active_anims: EditorActiveAnim,
    pub active_anim_points: EditorActiveAnimPoint,
}

pub type EditorAnimationProps = ();

pub type EditorAnimations = AnimationsSkeleton<EditorAnimationsProps, EditorAnimationProps>;
pub type EditorPosAnimation = PosAnimationSkeleton<EditorAnimationProps>;
pub type EditorColorAnimation = ColorAnimationSkeleton<EditorAnimationProps>;
pub type EditorSoundAnimation = SoundAnimationSkeleton<EditorAnimationProps>;

pub type EditorGroups = MapGroupsSkeleton<
    EditorGroupsProps,
    EditorPhysicsGroupProps,
    EditorPhysicsLayerProps,
    EditorGroupProps,
    EditorTileLayerProps,
    EditorQuadLayerProps,
    EditorSoundLayerProps,
    EditorArbitraryLayerProps,
>;
pub type EditorGroup = MapGroupSkeleton<
    EditorGroupProps,
    EditorTileLayerProps,
    EditorQuadLayerProps,
    EditorSoundLayerProps,
    EditorArbitraryLayerProps,
>;
pub type EditorLayerArbitrary = MapLayerArbitrarySkeleton<EditorArbitraryLayerProps>;
pub type EditorLayerTile = MapLayerTileSkeleton<EditorTileLayerProps>;
pub type EditorLayerQuad = MapLayerQuadSkeleton<EditorQuadLayerProps>;
pub type EditorLayerSound = MapLayerSoundSkeleton<EditorSoundLayerProps>;
pub type EditorLayer = MapLayerSkeleton<
    EditorTileLayerProps,
    EditorQuadLayerProps,
    EditorSoundLayerProps,
    EditorArbitraryLayerProps,
>;
pub type EditorGroupPhysics =
    MapGroupPhysicsSkeleton<EditorPhysicsGroupProps, EditorPhysicsLayerProps>;
pub type EditorPhysicsLayer = MapLayerPhysicsSkeleton<EditorPhysicsLayerProps>;

impl EditorCommonLayerOrGroupAttrInterface for EditorGroup {
    fn editor_attr(&self) -> &EditorCommonGroupOrLayerAttr {
        &self.user.attr
    }

    fn editor_attr_mut(&mut self) -> &mut EditorCommonGroupOrLayerAttr {
        &mut self.user.attr
    }
}

impl EditorCommonLayerOrGroupAttrInterface for EditorGroupPhysics {
    fn editor_attr(&self) -> &EditorCommonGroupOrLayerAttr {
        &self.user.attr
    }

    fn editor_attr_mut(&mut self) -> &mut EditorCommonGroupOrLayerAttr {
        &mut self.user.attr
    }
}

impl EditorCommonLayerOrGroupAttrInterface for EditorLayer {
    fn editor_attr(&self) -> &EditorCommonGroupOrLayerAttr {
        match self {
            MapLayerSkeleton::Abritrary(layer) => &layer.user.attr,
            MapLayerSkeleton::Tile(layer) => &layer.user.attr,
            MapLayerSkeleton::Quad(layer) => &layer.user.attr,
            MapLayerSkeleton::Sound(layer) => &layer.user.attr,
        }
    }
    fn editor_attr_mut(&mut self) -> &mut EditorCommonGroupOrLayerAttr {
        match self {
            MapLayerSkeleton::Abritrary(layer) => &mut layer.user.attr,
            MapLayerSkeleton::Tile(layer) => &mut layer.user.attr,
            MapLayerSkeleton::Quad(layer) => &mut layer.user.attr,
            MapLayerSkeleton::Sound(layer) => &mut layer.user.attr,
        }
    }
}

impl EditorDesignLayerInterface for EditorLayer {
    fn is_selected(&self) -> bool {
        match self {
            MapLayerSkeleton::Abritrary(_) => false,
            MapLayerSkeleton::Tile(layer) => layer.user.selected.is_some(),
            MapLayerSkeleton::Quad(layer) => layer.user.selected.is_some(),
            MapLayerSkeleton::Sound(layer) => layer.user.selected.is_some(),
        }
    }
}

impl EditorCommonLayerOrGroupAttrInterface for EditorPhysicsLayer {
    fn editor_attr(&self) -> &EditorCommonGroupOrLayerAttr {
        &self.user().attr
    }

    fn editor_attr_mut(&mut self) -> &mut EditorCommonGroupOrLayerAttr {
        &mut self.user_mut().attr
    }
}

impl EditorPhysicsLayerInterface for EditorPhysicsLayer {
    fn is_selected(&self) -> bool {
        self.user().selected.is_some()
    }
}

pub enum EditorLayerUnionRef<'a> {
    Physics {
        layer: &'a EditorPhysicsLayer,
        group_attr: &'a MapGroupPhysicsAttr,
        layer_index: usize,
    },
    Design {
        layer: &'a EditorLayer,
        group: &'a EditorGroup,
        group_index: usize,
        layer_index: usize,
        is_background: bool,
    },
}

pub enum EditorLayerUnionRefMut<'a> {
    Physics {
        layer: &'a mut EditorPhysicsLayer,
        layer_index: usize,
    },
    Design {
        layer: &'a mut EditorLayer,
        group_index: usize,
        layer_index: usize,
        is_background: bool,
    },
}

impl EditorLayerUnionRef<'_> {
    pub fn get_width_and_height(&self) -> (NonZeroU16MinusOne, NonZeroU16MinusOne) {
        match self {
            EditorLayerUnionRef::Physics { group_attr, .. } => {
                (group_attr.width, group_attr.height)
            }
            EditorLayerUnionRef::Design { layer, .. } => {
                if let EditorLayer::Tile(layer) = layer {
                    (layer.layer.attr.width, layer.layer.attr.height)
                } else {
                    panic!("this is not a tile layer")
                }
            }
        }
    }

    pub fn get_offset_and_parallax(&self) -> (vec2, vec2) {
        match self {
            EditorLayerUnionRef::Physics { .. } => (vec2::default(), vec2::new(100.0, 100.0)),
            EditorLayerUnionRef::Design { group, .. } => (
                vec2::new(group.attr.offset.x.to_num(), group.attr.offset.y.to_num()),
                vec2::new(
                    group.attr.parallax.x.to_num(),
                    group.attr.parallax.y.to_num(),
                ),
            ),
        }
    }

    pub fn get_or_fake_group_attr(&self) -> MapGroupAttr {
        match self {
            EditorLayerUnionRef::Physics { .. } => MapGroupAttr {
                offset: Default::default(),
                parallax: fvec2::new(ffixed::from_num(100), ffixed::from_num(100)),
                clipping: None,
            },
            EditorLayerUnionRef::Design { group, .. } => group.attr,
        }
    }

    pub fn is_tile_layer(&self) -> bool {
        match self {
            EditorLayerUnionRef::Physics { .. } => true,
            EditorLayerUnionRef::Design { layer, .. } => {
                matches!(layer, EditorLayer::Tile(_))
            }
        }
    }

    pub fn is_quad_layer(&self) -> bool {
        match self {
            EditorLayerUnionRef::Physics { .. } => false,
            EditorLayerUnionRef::Design { layer, .. } => {
                matches!(layer, EditorLayer::Quad(_))
            }
        }
    }

    pub fn is_sound_layer(&self) -> bool {
        match self {
            EditorLayerUnionRef::Physics { .. } => false,
            EditorLayerUnionRef::Design { layer, .. } => {
                matches!(layer, EditorLayer::Sound(_))
            }
        }
    }

    pub fn color_anim(&self) -> &Option<usize> {
        match self {
            EditorLayerUnionRef::Design {
                layer: EditorLayer::Tile(layer),
                ..
            } => &layer.layer.attr.color_anim,
            EditorLayerUnionRef::Physics { .. } | EditorLayerUnionRef::Design { .. } => &None,
        }
    }

    pub fn is_active(&self) -> bool {
        match self {
            EditorLayerUnionRef::Physics { layer, .. } => layer.editor_attr().active,
            EditorLayerUnionRef::Design { layer, .. } => layer.editor_attr().active,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EditorMapSetLayer {
    Physics { layer: usize },
    Background { group: usize, layer: usize },
    Foreground { group: usize, layer: usize },
}

#[derive(Debug, Clone, Copy)]
pub enum EditorMapSetGroup {
    Physics,
    Background { group: usize },
    Foreground { group: usize },
}

pub trait EditorMapInterface {
    fn active_layer(&'_ self) -> Option<EditorLayerUnionRef<'_>>;
    fn active_layer_mut(&'_ mut self) -> Option<EditorLayerUnionRefMut<'_>>;

    fn set_active_layer(&mut self, layer: EditorMapSetLayer);

    fn unselect_all(&mut self, unselect_groups: bool, unselect_layers: bool);
    fn toggle_selected_layer(&mut self, layer: EditorMapSetLayer, try_multiselect: bool);
    fn toggle_selected_group(&mut self, group: EditorMapSetGroup, try_multiselect: bool);

    fn game_time_info(&self) -> GameTimeInfo;
    fn game_camera(&self) -> Camera;

    fn active_animations(&self) -> &EditorAnimations;
}

pub trait EditorMapGroupsInterface {
    fn active_layer(&'_ self) -> Option<EditorLayerUnionRef<'_>>;
    fn active_layer_mut(&'_ mut self) -> Option<EditorLayerUnionRefMut<'_>>;

    fn selected_layers(&'_ self) -> Vec<EditorLayerUnionRef<'_>>;

    fn live_edited_layers(&self) -> Vec<EditorEventLayerIndex>;
}

#[derive(Debug, Clone, Default)]
pub struct EditorMapConfig {
    pub cmd_string: String,
    pub selected_cmd: Option<usize>,

    pub conf_var_string: String,
    pub selected_conf_var: Option<usize>,
}
pub type EditorConfig = ConfigSkeleton<EditorMapConfig>;
pub type EditorMetadata = MetadataSkeleton<()>;

#[derive(Default)]
pub struct EditorGroupPanelResources {
    pub file_dialog: FileDialog,
    pub loading_tasks: HashMap<PathBuf, IoRuntimeTask<Vec<u8>>>,
}

impl Debug for EditorGroupPanelResources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditorGroupPanelResources").finish()
    }
}

impl Clone for EditorGroupPanelResources {
    fn clone(&self) -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
pub enum EditorGroupPanelTab {
    GroupsAndLayers,
    Images(EditorGroupPanelResources),
    ArrayImages(EditorGroupPanelResources),
    Sounds(EditorGroupPanelResources),
}

#[derive(Debug, Clone, Default)]
pub struct EditorChatState {
    pub msg: String,
}

#[derive(Debug, Clone)]
pub struct EditorMapPropsUiValues {
    pub group_panel_active_tab: EditorGroupPanelTab,
    pub animations_panel_open: bool,
    pub server_commands_open: bool,
    pub server_config_variables_open: bool,
    pub chat_panel_open: Option<EditorChatState>,
    pub timeline: Timeline,
}

impl Default for EditorMapPropsUiValues {
    fn default() -> Self {
        Self {
            group_panel_active_tab: EditorGroupPanelTab::GroupsAndLayers,
            animations_panel_open: false,
            server_commands_open: false,
            server_config_variables_open: false,
            chat_panel_open: None,
            timeline: Timeline::default(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct EditorGlobalOptions {
    /// don't allow properties to be influenced by the animation panel
    /// the animation panel will act like a completely separated system
    pub no_animations_with_properties: bool,
    /// show tile numbers for the current active tile layer
    pub show_tile_numbers: bool,
    /// Whether to render a grid for aligning quads & sounds.
    pub render_grid: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct EditorMapProps {
    pub options: EditorGlobalOptions,
    pub ui_values: EditorMapPropsUiValues,

    pub sound_scene: SceneObject,
    pub global_sound_listener: SoundListener,

    // current global time of the map (used for animation etc.)
    pub time: Duration,
    // the scale how much the time should be progress, 0 = paused, 1 = normal speed etc.
    pub time_scale: u32,

    /// these animations are for if the animations panel is open and
    /// fake anim points have to be inserted
    pub animations: EditorAnimations,
}

impl EditorMapProps {
    /// If animation panel is open, it uses that time, otherwise the editor time.
    pub fn render_time(&self) -> Duration {
        if self.ui_values.animations_panel_open {
            self.ui_values.timeline.time()
        } else {
            self.time
        }
    }

    /// If animations are paused and animation pannel is open,
    /// then it's nice for the animator to see the last animation point too.
    pub fn include_last_anim_point(&self) -> bool {
        self.ui_values.animations_panel_open && self.ui_values.timeline.is_paused()
    }

    /// If animation panel is open and the user wants easier animation handling in the layer/quad/sound attributes,
    /// then this returns `true`.
    pub fn change_animations(&self) -> bool {
        self.ui_values.animations_panel_open && !self.options.no_animations_with_properties
    }
}

pub type EditorMap = MapSkeleton<
    EditorMapProps,
    (),
    EditorResource<TextureContainer, ()>,
    EditorResource<TextureContainer2dArray, EditorResourceTexture2dArray>,
    EditorResource<SoundObject, ()>,
    EditorGroupsProps,
    EditorPhysicsGroupProps,
    EditorPhysicsLayerProps,
    EditorGroupProps,
    EditorTileLayerProps,
    EditorQuadLayerProps,
    EditorSoundLayerProps,
    EditorArbitraryLayerProps,
    EditorAnimationsProps,
    EditorAnimationProps,
    EditorMapConfig,
    (),
>;

impl EditorMapGroupsInterface for EditorGroups {
    fn active_layer(&self) -> Option<EditorLayerUnionRef<'_>> {
        fn find_layer(
            is_background: bool,
            (group_index, group): (usize, &EditorGroup),
        ) -> Option<EditorLayerUnionRef<'_>> {
            group
                .layers
                .iter()
                .enumerate()
                .find_map(|(layer_index, layer)| {
                    if layer.editor_attr().active {
                        Some(EditorLayerUnionRef::Design {
                            layer,
                            group,
                            group_index,
                            layer_index,
                            is_background,
                        })
                    } else {
                        None
                    }
                })
        }
        let layer = self
            .background
            .iter()
            .enumerate()
            .find_map(|g| find_layer(true, g));
        if layer.is_some() {
            return layer;
        }
        let layer = self
            .physics
            .layers
            .iter()
            .enumerate()
            .find_map(|(layer_index, layer)| {
                if layer.editor_attr().active {
                    Some(EditorLayerUnionRef::Physics {
                        layer,
                        group_attr: &self.physics.attr,
                        layer_index,
                    })
                } else {
                    None
                }
            });
        if layer.is_some() {
            return layer;
        }
        let layer = self
            .foreground
            .iter()
            .enumerate()
            .find_map(|g| find_layer(false, g));
        if layer.is_some() {
            return layer;
        }
        None
    }

    fn active_layer_mut(&mut self) -> Option<EditorLayerUnionRefMut<'_>> {
        fn find_layer(
            is_background: bool,
            (group_index, group): (usize, &'_ mut EditorGroup),
        ) -> Option<EditorLayerUnionRefMut<'_>> {
            group
                .layers
                .iter_mut()
                .enumerate()
                .find_map(|(layer_index, layer)| {
                    if layer.editor_attr().active {
                        Some(EditorLayerUnionRefMut::Design {
                            layer,
                            group_index,
                            layer_index,
                            is_background,
                        })
                    } else {
                        None
                    }
                })
        }
        let layer = self
            .background
            .iter_mut()
            .enumerate()
            .find_map(|g| find_layer(true, g));
        if layer.is_some() {
            return layer;
        }
        let layer = self
            .physics
            .layers
            .iter_mut()
            .enumerate()
            .find_map(|(layer_index, layer)| {
                if layer.editor_attr().active {
                    Some(EditorLayerUnionRefMut::Physics { layer, layer_index })
                } else {
                    None
                }
            });
        if layer.is_some() {
            return layer;
        }
        let layer = self
            .foreground
            .iter_mut()
            .enumerate()
            .find_map(|g| find_layer(false, g));
        if layer.is_some() {
            return layer;
        }
        None
    }

    fn selected_layers(&self) -> Vec<EditorLayerUnionRef<'_>> {
        fn collect_group(
            group: &[EditorGroup],
            is_background: bool,
        ) -> Vec<EditorLayerUnionRef<'_>> {
            group
                .iter()
                .enumerate()
                .flat_map(|(group_index, g)| {
                    g.layers
                        .iter()
                        .enumerate()
                        .filter_map(|(layer_index, l)| {
                            l.is_selected().then_some(EditorLayerUnionRef::Design {
                                layer: l,
                                group: g,
                                group_index,
                                layer_index,
                                is_background,
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        }
        collect_group(&self.background, true)
            .into_iter()
            .chain(collect_group(&self.foreground, false))
            .chain(
                self.physics
                    .layers
                    .iter()
                    .enumerate()
                    .filter_map(|(layer_index, l)| {
                        l.is_selected().then_some(EditorLayerUnionRef::Physics {
                            layer: l,
                            group_attr: &self.physics.attr,
                            layer_index,
                        })
                    })
                    .collect::<Vec<_>>(),
            )
            .collect()
    }

    fn live_edited_layers(&self) -> Vec<EditorEventLayerIndex> {
        self.background
            .iter()
            .enumerate()
            .map(|(group_index, g)| (true, group_index, g))
            .chain(
                self.foreground
                    .iter()
                    .enumerate()
                    .map(|(group_index, g)| (false, group_index, g)),
            )
            .flat_map(|(is_background, group_index, group)| {
                group
                    .layers
                    .iter()
                    .enumerate()
                    .filter_map(|(layer_index, layer)| {
                        if let EditorLayer::Tile(layer) = layer {
                            if layer.user.live_edit.is_some() {
                                Some(EditorEventLayerIndex {
                                    is_background,
                                    group_index,
                                    layer_index,
                                })
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}

impl EditorMapInterface for EditorMap {
    fn active_layer(&self) -> Option<EditorLayerUnionRef<'_>> {
        self.groups.active_layer()
    }

    fn active_layer_mut(&mut self) -> Option<EditorLayerUnionRefMut<'_>> {
        self.groups.active_layer_mut()
    }

    fn unselect_all(&mut self, unselect_groups: bool, unselect_layers: bool) {
        self.groups
            .background
            .iter_mut()
            .chain(self.groups.foreground.iter_mut())
            .for_each(|g| {
                if unselect_groups {
                    g.user.selected = None;
                }
                if unselect_layers {
                    g.layers.iter_mut().for_each(|layer| match layer {
                        MapLayerSkeleton::Abritrary(_) => {}
                        MapLayerSkeleton::Tile(layer) => layer.user.selected = None,
                        MapLayerSkeleton::Quad(layer) => layer.user.selected = None,
                        MapLayerSkeleton::Sound(layer) => layer.user.selected = None,
                    });
                }
            });

        if unselect_groups {
            self.groups.physics.user.selected = None;
        }

        if unselect_layers {
            self.groups
                .physics
                .layers
                .iter_mut()
                .for_each(|layer| layer.user_mut().selected = None);
        }
    }

    fn set_active_layer(&mut self, layer: EditorMapSetLayer) {
        self.groups
            .physics
            .layers
            .iter_mut()
            .for_each(|layer| layer.user_mut().attr.active = false);
        self.groups.background.iter_mut().for_each(|group| {
            group
                .layers
                .iter_mut()
                .for_each(|layer| layer.editor_attr_mut().active = false)
        });
        self.groups.foreground.iter_mut().for_each(|group| {
            group
                .layers
                .iter_mut()
                .for_each(|layer| layer.editor_attr_mut().active = false)
        });

        match layer {
            EditorMapSetLayer::Physics { layer } => {
                self.groups.physics.layers[layer].user_mut().attr.active = true;
            }
            EditorMapSetLayer::Background { group, layer } => {
                self.groups.background[group].layers[layer]
                    .editor_attr_mut()
                    .active = true;
            }
            EditorMapSetLayer::Foreground { group, layer } => {
                self.groups.foreground[group].layers[layer]
                    .editor_attr_mut()
                    .active = true;
            }
        }
    }

    fn toggle_selected_layer(&mut self, set_layer: EditorMapSetLayer, try_multiselect: bool) {
        if !try_multiselect {
            self.unselect_all(true, true);
        } else {
            self.unselect_all(true, false);
        }

        match set_layer {
            EditorMapSetLayer::Physics { layer } => {
                let layer = &mut self.groups.physics.layers[layer];
                if layer.user().selected.is_none() {
                    layer.user_mut().selected = Some(());
                } else {
                    layer.user_mut().selected = None;
                }
            }
            EditorMapSetLayer::Background { group, layer }
            | EditorMapSetLayer::Foreground { group, layer } => {
                let layer = &mut if matches!(set_layer, EditorMapSetLayer::Background { .. }) {
                    &mut self.groups.background
                } else {
                    &mut self.groups.foreground
                }[group]
                    .layers[layer];

                match layer {
                    EditorLayer::Abritrary(_) => {}
                    EditorLayer::Tile(layer) => {
                        if layer.user.selected.is_none() {
                            layer.user.selected = Some(EditorTileLayerPropsSelection {
                                attr: layer.layer.attr,
                                name: layer.layer.name.clone(),
                                image_2d_array_selection_open: None,
                            });
                        } else {
                            layer.user.selected = None;
                        }
                    }
                    EditorLayer::Quad(layer) => {
                        if layer.user.selected.is_none() {
                            layer.user.selected = Some(EditorQuadLayerPropsPropsSelection {
                                attr: layer.layer.attr,
                                name: layer.layer.name.clone(),
                                image_selection_open: None,
                            });
                        } else {
                            layer.user.selected = None;
                        }
                    }
                    EditorLayer::Sound(layer) => {
                        if layer.user.selected.is_none() {
                            layer.user.selected = Some(EditorSoundLayerPropsPropsSelection {
                                attr: layer.layer.attr,
                                name: layer.layer.name.clone(),
                                sound_selection_open: None,
                            });
                        } else {
                            layer.user.selected = None;
                        }
                    }
                }
            }
        }
    }

    fn toggle_selected_group(&mut self, set_group: EditorMapSetGroup, try_multiselect: bool) {
        if !try_multiselect {
            self.unselect_all(true, true);
        } else {
            self.unselect_all(false, true);
        }

        match set_group {
            EditorMapSetGroup::Physics => {
                if self.groups.physics.user.selected.is_none() {
                    self.groups.physics.user.selected = Some(self.groups.physics.attr);
                } else {
                    self.groups.physics.user.selected = None;
                }
            }
            EditorMapSetGroup::Background { group } | EditorMapSetGroup::Foreground { group } => {
                let group = &mut if matches!(set_group, EditorMapSetGroup::Background { .. }) {
                    &mut self.groups.background
                } else {
                    &mut self.groups.foreground
                }[group];
                if group.user.selected.is_none() {
                    group.user.selected = Some(EditorGroupPropsSelection {
                        attr: group.attr,
                        name: group.name.clone(),
                    });
                } else {
                    group.user.selected = None;
                }
            }
        }
    }

    fn game_time_info(&self) -> GameTimeInfo {
        let time = self.user.render_time();
        GameTimeInfo {
            ticks_per_second: 50.try_into().unwrap(),
            intra_tick_time: Duration::from_nanos(
                (time.as_nanos() % (Duration::from_secs(1).as_nanos() / 50)) as u64,
            ),
        }
    }

    fn game_camera(&self) -> Camera {
        Camera {
            pos: self.groups.user.pos,
            zoom: self.groups.user.zoom,
            parallax_aware_zoom: self.groups.user.parallax_aware_zoom,
            forced_aspect_ratio: None,
        }
    }

    fn active_animations(&self) -> &EditorAnimations {
        if self.user.ui_values.animations_panel_open {
            &self.user.animations
        } else {
            &self.animations
        }
    }
}
