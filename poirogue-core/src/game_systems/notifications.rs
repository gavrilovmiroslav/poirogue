use std::collections::vec_deque::Iter;
use std::collections::VecDeque;
use std::sync::Arc;
use bracket_color::prelude::{BLACK, WHITE};
use bracket_lib::prelude::BTerm;
use shipyard::{UniqueView, UniqueViewMut};
use simple_ringbuf::RingBuffer;
use crate::colors::named_color;

pub struct NotificationLog {
    log: VecDeque<String>,
    cap: usize,
}

impl NotificationLog {
    pub fn new(cap: usize) -> NotificationLog {
        NotificationLog{ log: VecDeque::new(), cap }
    }

    pub fn write(&mut self, str: String) {
        if self.log.len() == self.cap {
            self.log.pop_front();
        }
        self.log.push_back(str);
    }

    pub fn len(&self) -> usize {
        self.log.len()
    }

    pub fn get(&self) -> Iter<String> {
        self.log.iter()
    }
}

pub fn render_notification_log(mut ctx: &mut BTerm,
                               log: UniqueView<NotificationLog>) {

    let cap = log.len() as u32;
    let height = ctx.get_char_size().1;

    let mut current_line = height - cap - 1;
    for notif in log.get() {
        ctx.print_color(2, current_line, named_color(WHITE), named_color(BLACK), notif);
        current_line += 1;
    }
}