use std::borrow::Borrow;
use std::sync::Mutex;
use bracket_lib::prelude::{BTerm, xp_to_draw_batch, XpFile};
use caves::Cave;
use lru::{DefaultHasher, LruCache};
use lazy_static::*;
use object_pool::Reusable;
use crate::game::{Game, GameSharedData};
use bracket_lib::prelude::DrawBatch;

lazy_static! {
    static ref LRU: Mutex<LruCache<&'static str, XpFile>> = Mutex::new(LruCache::with_hasher(2, DefaultHasher::default()));
}

fn dig_from_cave(data: &dyn Cave, name: &'static str) -> XpFile {
    let buffer: Vec<u8> = data.get(format!("{}.xp", name).as_str()).unwrap();
    XpFile::read(&mut &*buffer).unwrap()
}

pub fn draw_rex(game: &mut GameSharedData, ctx: &mut BTerm, name: &'static str, x: i32, y: i32) {
    let mut lru = LRU.lock().unwrap();
    if !lru.contains(&name) {
        let rex = dig_from_cave(game.data.borrow(), name);
        ctx.render_xp_sprite(&rex, x, y);
        lru.put(name, rex);
    } else {
        ctx.render_xp_sprite(lru.get(&name).unwrap(), x, y);
    }
}
