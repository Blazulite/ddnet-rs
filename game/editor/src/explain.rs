pub const TEXT_QUAD_BRUSH: &str = "\
# Quad brush\n\
\n\
The quad brush allows to select a range of quads (`left click`) and paste the selection again (`left click`)  \n\
Clicking the corner or center points of a quad has special meanings  \n\
Center point:\n\
- `Left click` allows dragging the whole quad.\n\
- `Shift + Left click` allows dragging the center point itself.\n\
- `R + Left click` allows rotating the quad.\n\
- `Right click` opens the quad property panel.\n\n\
Corner points:\n\
- `Left click` allows dragging the quad's corner.\n\
- `Right click` opens the corner property panel.\n\n\
---\n\n\
Press `right click` to unset the selection.  \n\
If the grid is active, then hold `alt` while dragging to snap to the next grid point.\
";

pub const TEXT_QUAD_SELECTION: &str = "\
# Quad selection\n\
\n\
The quad selection is a specialized tool that focuses on making working with existing quads easier.\n\n\
You can change many shared properties at once to many quads. E.g. the color of one of the quad corners.\n\
Align many quads at once and so on similar to the `Quad brush`.  \n\
First select your quad(s) using `left click`:\n\
- `Right click` on a quad _point_ to open the property \
  window for all selected quads for the given _point_.\n\
- `Left click` on a center point to drag all quads.\n\
- `Shift + left click` on a center point to drag the center point of all quads.\n\
- `R + left click` on a center point to rotate all quads.\n\n\
Press `right click` on no quad to unset the selection.\n\n\
If the grid is active, holding `alt`-key will always snap the above actions to the nearest grid point.\n\n\
### Animations\n\
\n\
If one or more quads are selected with at least one shared \
pos/color animation and the `Animations`-panel is open you can \
control the animation properties using the quads directly. \
In other words: instead of changing the animation point values \
inside the `Animations`-panel you can \
simply drag the `Animations`-panel's time dragger to where you want to insert \
the next animation point -> change your quad properties -> insert the animation point at this position.\n\
> Keep in mind that moving the `Animations`-panel's time dragger resets the quad to the \
evaluated position/color etc.  \n\
The property's value is the interpreted as `value = base-value * animation-point-value` \
to calculate the value of the animation points.\
";

pub const TEXT_TILE_BRUSH: &str = "\
# Tile brush\n\
\n\
The tile brush allows to select a range of tiles (`left click`) and apply different actions on this selection:\n\
- `Left click` -> Draws this selection anywhere within a tile layer.\n\
- `Shift + left click selection` -> Creates a repeated pattern of the selected tiles by dragging.\n\n\
- `B + left click` (single tile only) -> Fills the empty area with the given tile.\
  (The fill tool is always destructive!)\n\n\
---\n\n\
Press `right click` to unset the selection.  \n\
Hold `shift` without any tile selected to delete a range of tiles.  \n\
Hold `space` to open the tile picker, which is basically an overview of all tiles within a tile layer image.\
";

pub const TEXT_TILE_SELECT: &str = "\
# Tile selection\n\
\n\
The tile selection allows to select a range of tiles (`left click`) and apply different actions on this selection \n\
such as flipping and rotating, which can be found in the tool bar below.  \n\
Press `right click` to unset the selection.\
";

pub const TEXT_SOUND_BRUSH: &str = "\
# Sound brush\n\
\n\
The sound brush allows to select a range of sounds (`left click`) and paste the selection again (`left click`)  \n\
Clicking the center point of a sound has special meanings:\n\
- `Left click` allows dragging the whole sound.\n\
- `Right click` opens the sound property panel.\n\n\
---\n\n\
Press `right click` to unset the selection.\
";

pub const TEXT_ADD_QUAD: &str = "\
# Add quad\n\
\n\
Adds a new quad to the active quad layer.\
";

pub const TEXT_ADD_SOUND: &str = "\
# Add sound\n\
\n\
Adds a new sound to the active sound layer.\
";

pub const TEXT_LAYER_PROPS_ANIM_COLOR: &str = "\
# Layer's animation color\n\
\n\
If the `Animations`-panel is open and this layer has a color \
animation active, you can change this property and \
insert a new animation point at the current `Animations`-panel's \
time value (move the time dragger) instead of changing the animation points inside \
the `Animations`-panel.\n\
> Keep in mind that moving the `Animations`-panel's time dragger resets the color to the \
evaluated color of the animation.  \n\
The rendered color value is the result of `color = base-color * animation-point-color`.\
";

pub const TEXT_ANIM_PANEL_OPEN: &str = "\
# Animations panel + properties\n\n\
To make animating easier, properties that are affected by animations like position, color & sound volume \
are entering a different mode when the `Animations`-panel is open.  \n\
The final product of animation values are applied like following:\n\
- position: `pos = base_pos + anim_pos`\n\
- color: `color = base_color * anim_color`\n\
- volume: `volume = base_volume * anim_volume`\n\n\
The conclusion of this is that if you insert a new animation key point this key point can automatically \
calculate the animation point values using the above equasion.\n\n\
> - You can opt-out of this animations handling in the global settings.\n\
> - Color values are always in the range [0-1] (or [0-255]), so e.g. if the base color for the red channel is 0 \
    the animation point cannot magically make the final color higher than 0.  \n\
    In other words that means that the base color reduces the color range (0.5 => final point is 0.5 at most, since anim point cannot go above 1.0).\
";

pub const TEXT_QUAD_PROP_COLOR: &str = "\
# Quad's animation color\n\
\n\
If the `Animations`-panel is open and this quad has a color \
animation active, then you can change this property and \
insert a new animation point at the current `Animations`-panel's \
time value (move the time dragger) instead of changing the animation points inside \
the `Animations`-panel.\n\
> Keep in mind that moving the `Animations`-panel's time dragger resets the color to the \
evaluated color of the animation.  \n\
The final rendered color value is the the product of `color = base-color * animation-point-color`.\
";

pub const TEXT_LAYERS_AND_GROUPS_OVERVIEW: &str = "\
# Layers & groups overview\n\n\
This gives an overview over all groups and their layers.\n\
\n\
Groups are separated into 3 main categories:\n\
- __Background__: the layers that will be rendered behind the Tees and other ingame objects.\n\
- __Physics__: the layers that control how the game works.\n\
- __Foreground__: the layers that will be rendered in front of ingame objects.\n\n\
All groups & layers can be hidden by pressing the eye-symbol. This will not affect how they are displayed in the client!\n\n\
To open the group & layer properties `right click` on their names. To select multiple groups __or__ layers hold `shift`.  \n\
This will allow to modify the overlapping properties of multiple groups/layers at once.  \n\
To activate a layer `left click` the name.\n\
";

pub const TEXT_IMAGES: &str = "\
# Images (quad layers)\n\n\
Quad layers support any kind of images without special requirements.\n\
> You can still use the same image for quad layers and tile layers, simply include it in both tabs, \
the client's implementation prevents loading the image twice.\
";

pub const TEXT_2D_IMAGE_ARRAY: &str = "\
# 2D image arrays (tile layers)\n\n\
Tile layers need special images. Their requirement is that the width and height must\n\
be divisible by 16 (e.g. `1024 / 16 = 64` without a rest => divisible by 16).  \n\
> You can still use the same image for quad layers and tile layers, simply include it in both tabs, \
the client's implementation prevents loading the image twice.\
";

pub const TEXT_SOUND_SOURCES: &str = "\
# Sound sources (sound layers)\n\n\
Sound sources are simply sound files that can be played by the client.\
";

pub const TEXT_ANIM_PANEL_AND_PROPS: &str = "\
# Animations panel + properties\n\n\
To make animations easier to use, you can use the properties of e.g. quads \
to animate your quads while the animations panel is open.  \n\
That however means that the _base_ properties will not be overwritten when \
changing the properties, because it will instead write to temporary properties\
that are influenced by the current animation time and allow to insert new animation points.  \n\
To opt-out of this behavior, enable this option.\
";

pub const TEXT_TILE_BRUSH_MIRROR: &str = "\
# Tile brush mirror\n\
\n\
Mirrors the tile brush horizontal or vertically.\
";

pub const TEXT_TILE_DESTRUCTIVE: &str = "\
# Destructive tiles\n\
\n\
If active, tiles that are non-air will be overwritten by the brush.\
";

pub const TEXT_TILE_ALLOW_UNUSED: &str = "\
# Allow unused tiles\n\
\n\
Allows to select and draw unused tiles, so those tiles are not known to the game \
(unused tiles can be used for modding purposes).\
";

pub const AUTO_MAPPER_CREATOR_EXPLAIN: &str = "\
# Auto mapper creator overview\
\n\
The auto mapper works by checking adjacent tiles.  \n\
The full process is as follows:\n\
- There are mutliple runs in a auto mapper rule.\n\
- Every run can change or spawn multiple tiles.\n\
- For every spawnable/changeable tile there are adjacent tile groups \
of boolean expressions, that check if a matches \
the given conditions (tile index & optional flags).\n\
- A condition can be negated or not. It can optionally be used as \
expression using an OR or NOT boolean operand\
and a second condition. (So it allows basic boolean algebra).\n\
- If the given condition fully evaluates to _true_, the tile is spawned/changed.\n\
- Optionally a randomness parameter can be used to skip the above \
calculation based on the random probability.\n\
";

pub const AUTO_MAPPER_CREATOR_EXPRESSION_LIST_EXPLAIN: &str = "\
# Overview over all expressions for a adjacent tile\
\n\
Every adjacent tile group is a list of expressions using\
the OR and NOT operator.  \n\
The order of evaluation is always bottom to top, which means \
that the most bottom expression will evaluate first, and the \
OR or NOT operator is that applied on the second most bottom \
expression, this whole expression is then evaluated with the operator \
of the third most bottom expression and so on.  \n\
If an expression is _negated_, then this negation operator is applied on the \
expression itself and not on expressions below or above.  \n\
";

pub const ANIMATION_PANEL: &str = "\
# Animations\
\n\
Animations are categorized into 3:\n\
- Position & rotation\n\
- Color\n\
- Sound\n\
\n\n\
Position & rotation affects sound sources and quads, while \
color affects tile layers & quads. Sound animations are \
for sound sources only.  \n\
To insert a new animation point press the `$ANIM_POINT_INSERT$` hotkey.  \n\
To delete a point simply `right click` the point in the time graph.\
";

pub const SERVER_COMMANDS_CONFIG_VAR: &str = "\
# Server commands & config variables\
\n\
First things first, the difference between config variables and \
commands is very blurry and depends on the game mod.  \n\
In ddrace usually config variables are the same as commands.  \n\
Config variables have the advantage that they can be parsed \
before the game is created, which allows to even modify \
config that changes the type of game (e.g. ctf vs dm and similar).  \n\
If unsure, prefer server commands.\
";
