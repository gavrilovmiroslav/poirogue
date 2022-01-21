use std::borrow::Borrow;
use std::sync::Mutex;
use bracket_lib::prelude::{BTerm, xp_to_draw_batch, XpFile};
use caves::Cave;
use object_pool::Reusable;
use crate::game::{Game};
use bracket_lib::prelude::DrawBatch;
use lazy_static::*;
use lru::{DefaultHasher, LruCache};

lazy_static! {
    static ref XP_LRU: Mutex<LruCache<&'static str, XpFile>> = Mutex::new(LruCache::with_hasher(2, DefaultHasher::default()));
}

pub fn draw_rex(data: &mut Box<dyn Cave>, ctx: &mut BTerm, name: &'static str, x: i32, y: i32) {
    let mut lru = XP_LRU.lock().unwrap();
    if !lru.contains(&name) {
        let buffer: Vec<u8> = data.get(format!("{}.xp", name).as_str()).unwrap();
        let rex = XpFile::read(&mut &*buffer).unwrap();
        ctx.render_xp_sprite(&rex, x, y);
        lru.put(name, rex);
    } else {
        ctx.render_xp_sprite(lru.get(&name).unwrap(), x, y);
    }
}
