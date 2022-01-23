use std::collections::VecDeque;
use std::rc::Rc;
use bracket_color::prelude::{BLACK, WHITE};
use bracket_lib::prelude::{Point, Algorithm2D, BTerm, field_of_view_set};
use crate::tiles::TileIndex;
use serde::{Serialize, Deserialize};
use shipyard::{AddEntity, IntoIter, ViewMut, IntoWithId, UniqueViewMut, UniqueView, EntitiesViewMut, Storage, EntityId, Get, Remove};
use simple_ringbuf::RingBuffer;
use crate::colors::named_color;
use crate::entity::{HasPosition, HasSight, IsDirty, IsDoor};
use crate::game_systems::{NotificationLog, ResolvedIntents};
use crate::map::Map;

pub struct NotifyDirective {
    pub entity_alive: Option<u64>,
    pub notification: String,
}

pub fn notify_if_alive(id: u64, msg: &str) -> NotifyDirective {
    NotifyDirective { entity_alive: Some(id), notification: msg.to_string(), }
}

pub fn notify(msg: &str) -> NotifyDirective {
    NotifyDirective { entity_alive: None, notification: msg.to_string(), }
}

pub struct MoveDirective(pub EntityId, pub Point);
pub struct UnlockDirective(pub EntityId);

pub fn resolve_notify_if_entity_alive_directives(mut notif_dirs: UniqueViewMut<VecDeque<NotifyDirective>>,
                                                 mut log: UniqueViewMut<NotificationLog>,
                                                 mut handled: UniqueViewMut<ResolvedIntents>) {

    while let Some(notif) = notif_dirs.pop_back() {
        if notif.entity_alive.is_none() || !handled.0.contains(&notif.entity_alive.unwrap()) {
            log.write(notif.notification);
        }
    }
}

pub fn resolve_move_directives(mut move_dirs: UniqueViewMut<VecDeque<MoveDirective>>,
                               mut positions: ViewMut<HasPosition>,
                               mut dirty: UniqueViewMut<IsDirty>,) {

    while let Some(mov) = move_dirs.pop_back() {
        if let Ok(mut pos) = (&mut positions).get(mov.0) {
            pos.0 = mov.1;

            dirty.0 = true;
        }
    }
}

pub fn resolve_unlock_directive(mut unlock_dirs: UniqueViewMut<VecDeque<UnlockDirective>>,
                                mut doors: ViewMut<IsDoor>,
                                mut dirty: UniqueViewMut<IsDirty>,) {

    while let Some(dir) = unlock_dirs.pop_back() {
        (&mut doors).get(dir.0).unwrap().is_locked = None;
        dirty.0 = true;
    }
}