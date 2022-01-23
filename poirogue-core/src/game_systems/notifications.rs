use std::collections::vec_deque::Iter;
use std::collections::VecDeque;
use std::sync::Arc;
use bracket_color::prelude::{BLACK, WHITE};
use bracket_lib::prelude::BTerm;
use shipyard::{UniqueView, UniqueViewMut};
use simple_ringbuf::RingBuffer;
use crate::colors::named_color;
use crate::entity::IsDirty;
use crate::game_systems::{BumpIntent, CollectIntent, UnlockIntent};
use crate::UI_CONSOLE_LAYER;

pub struct NotificationLog {
    pub log: VecDeque<String>,
    pub cap: u32,
    expire: u32,
    expire_time: u32,
}

impl NotificationLog {
    pub fn new(cap: usize, expire_time: u32) -> NotificationLog {
        NotificationLog{ log: VecDeque::new(), expire: 0, expire_time, cap: cap as u32 }
    }

    pub fn write(&mut self, str: String) {
        if let Some(back) = self.log.back() {
            if back == &str { return; }
        }

        if self.log.len() as u32 == self.cap {
            self.log.pop_front().unwrap();
        }
        self.log.push_back(str);
        self.expire = self.expire_time;
    }

    pub fn len(&self) -> usize {
        self.log.len()
    }

    pub fn tick(&mut self) {
        if !self.log.is_empty() {
            if self.expire > 0 {
                self.expire -= 1;
            } else {
                self.log.pop_front().unwrap();
                self.expire = self.expire_time;
            }
        }
    }
}

pub fn update_notification_log_expiry(mut log: UniqueViewMut<NotificationLog>) {
    log.tick();
}

pub fn render_notification_log(mut ctx: &mut BTerm,
                               notif_log: UniqueView<NotificationLog>,) {

    if notif_log.log.is_empty() { return; }

    ctx.set_active_console(UI_CONSOLE_LAYER);

    let height = ctx.get_char_size().1;
    let mut current_line = height - notif_log.cap - 1;

    for notif in &notif_log.log {
        ctx.print_color(2, current_line, named_color(WHITE), named_color(BLACK), notif);
        current_line += 1;
    }
}

pub fn render_fps(mut ctx: &mut BTerm, ) {
    ctx.set_active_console(UI_CONSOLE_LAYER);
    ctx.print_color(1, 1, named_color(WHITE), named_color(BLACK), format!("{} FPS", ctx.fps));
}