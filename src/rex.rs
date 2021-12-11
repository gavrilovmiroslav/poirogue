use bracket_lib::prelude::{BTerm, XpFile};
use caves::Cave;

fn get_rex_from_cave(cave: &dyn Cave, name: &str) -> XpFile {
    let buffer: Vec<u8> = cave.get(format!("{}.xp", name).as_str()).unwrap();
    XpFile::read(&mut &*buffer).unwrap()
}

pub fn draw_rex(data: &dyn Cave, ctx: &mut BTerm, name: &str, x: i32, y: i32) {
    let xp = get_rex_from_cave(data, name);
    ctx.render_xp_sprite(&xp, x, y);
}
