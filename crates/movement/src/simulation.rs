//! The simulation step both sides share, so client prediction runs exactly what the
//! server runs.

use crate::{
    Body, Dodge, MoveInput, MovementConfig, PlayerPosition, Sneaking, VerticalSpeed, WalkSurface,
    next_body,
};
use bevy::ecs::query::QueryData;
use bevy::prelude::*;

/// Every component of a player that movement simulates.
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct SimulatedBody {
    position: &'static mut PlayerPosition,
    vertical_speed: &'static mut VerticalSpeed,
    dodge: &'static mut Dodge,
    sneaking: &'static mut Sneaking,
}

impl SimulatedBodyItem<'_, '_> {
    /// Runs one step. Writing only on change keeps idle players out of network updates.
    pub(crate) fn advance(
        &mut self,
        input: &MoveInput,
        delta_secs: f32,
        config: &MovementConfig,
        surface: &dyn WalkSurface,
    ) {
        let body = Body {
            position: self.position.0,
            vertical_speed: self.vertical_speed.0,
            dodge: *self.dodge,
            sneaking: self.sneaking.0,
        };
        let next = next_body(body, input, delta_secs, config, surface);
        self.position.set_if_neq(PlayerPosition(next.position));
        self.vertical_speed
            .set_if_neq(VerticalSpeed(next.vertical_speed));
        self.dodge.set_if_neq(next.dodge);
        self.sneaking.set_if_neq(Sneaking(next.sneaking));
    }
}
