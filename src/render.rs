
use object_pool::Reusable;
use bracket_lib::prelude::*;
use caves::Cave;
use crate::game::{Entity, Game, GameSharedData};
use crate::tiles::MapTile;
use crate::views::View;

pub struct RenderView<'a> {
    pub tile_render: &'a dyn View<MapTile>,
    pub entity_render: &'a dyn View<Entity>,
}

#[derive(Default)]
pub struct RenderViewGroup<'a> {
    pub views: Vec<RenderView<'a>>,
    current_view: usize,
}

impl<'a> RenderViewGroup<'a> {
    pub fn cycle(&mut self) {
        self.current_view += 1;
        if self.current_view >= self.views.len() {
            self.current_view = 0;
        }
    }

    pub fn push(&mut self, render_view: RenderView<'a>) {
        self.views.push(render_view);
    }

    pub fn get_current_view(&self) -> &RenderView<'a> {
        self.views.get(self.current_view).unwrap()
    }
}

#[macro_export]
macro_rules! render_view {
    ($t: expr) => { RenderView{ tile_render: &$t as &dyn View<MapTile>, entity_render: &$t as &dyn View<Entity> } }
}

pub type RenderingPassFn = Box<dyn Fn(&mut GameSharedData, &mut BTerm)>;
