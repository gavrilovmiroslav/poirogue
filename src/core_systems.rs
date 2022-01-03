use bracket_color::prelude::{BLACK, WHITE};
use bracket_lib::prelude::{Algorithm2D, BTerm, field_of_view_set, VirtualKeyCode};
use shipyard::{IntoIter, View, ViewMut};
use bracket_color::prelude::RGB;
use crate::colors::named_color;
use crate::entity::*;
use crate::game::{GameSharedData, Store};
use crate::input::{InputSnapshot, InputSnapshots};
use crate::map::Map;
use crate::render_view::RenderView;

pub fn update_fov_system(data: &GameSharedData,
                         positions: View<HasPosition>, mut fovs: ViewMut<HasFieldOfView>, dirty: View<IsDirty>) {

    for (pos, mut fov, _) in (&positions, &mut fovs, &dirty).iter().filter(|(_, _, d)| d.is_dirty()) {
        fov.0 = field_of_view_set(pos.0, data.store.get("fov").unwrap_or(16), &data.map).into_iter().collect()
    }
}

pub fn update_revealed_fields_system(map: &mut Map, _: View<IsPlayer>, fovs: View<HasFieldOfView>, dirty: View<IsDirty>) {

    for (fov, _) in (&fovs, &dirty).iter().filter(|(_, d)| d.is_dirty()) {
        map.hide();
        map.show(&fov.0);
    }
}

pub fn handle_player_movement_system((input, store): (&InputSnapshots, &mut Store),
                                     _: View<IsPlayer>, mut positions: ViewMut<HasPosition>, mut dirty: ViewMut<IsDirty>) {

    for (mut has_pos, mut dirt) in (&mut positions, &mut dirty).iter() {
        let keyboard = &input.keyboard;
        let pos = has_pos.get_mut();

        if keyboard.is_pressed(VirtualKeyCode::Up) {
            pos.y -= 1; dirt.mark();
        }

        if keyboard.is_pressed(VirtualKeyCode::Down) {
            pos.y += 1; dirt.mark();
        }

        if keyboard.is_pressed(VirtualKeyCode::Left) {
            pos.x -= 1; dirt.mark();
        }

        if keyboard.is_pressed(VirtualKeyCode::Right) {
            pos.x += 1; dirt.mark();
        }

        if dirt.is_dirty() {
            store.set("player_position", &(pos.x, pos.y))
                .expect("Failed storing player position");
        }
    }
}

pub fn render_map_system((data, ctx): (&GameSharedData, &mut BTerm)) {

    fn render_map_layer(data: &GameSharedData, ctx: &mut BTerm) {
        let view = data.store.get::<RenderView>("view")
            .unwrap_or(RenderView::Game);

        ctx.set_active_console(0);
        ctx.cls();
        data.map.render(ctx, &view, &data.store);
    }

    fn render_fps_count(ctx: &mut BTerm) {
        ctx.set_active_console(2);
        ctx.cls();
        ctx.print_color(1, 1, named_color(WHITE), named_color(BLACK), format!("FPS: {}", ctx.fps));
    }

    render_map_layer(data, ctx);
    render_fps_count(ctx);
}

pub fn render_entities_system((data, ctx): (&GameSharedData, &mut BTerm),
                              positions: View<HasPosition>, glyphs: View<HasGlyph>) {

    for (has_pos, has_glyph) in (&positions, &glyphs).iter() {
        let glyph = has_glyph.0;
        let pos = has_pos.0;
        let index = data.map.point2d_to_index(pos);
        if data.map.is_tile_revealed(index) && data.map.is_tile_visible(index) {
            ctx.print_color(pos.x, pos.y, RGB::from(glyph.fg), RGB::from(glyph.bg), glyph.ch);
        }
    }
}

pub fn mark_clean_system(mut dirty: ViewMut<IsDirty>) {

    for (mut dirt) in (&mut dirty).iter().filter(|(d)| d.is_dirty()) {
        dirt.clean();
    }
}
