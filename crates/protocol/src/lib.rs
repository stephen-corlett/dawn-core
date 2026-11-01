//! What the client and server must agree on: the order network types are
//! registered in, and the tuning values both sides simulate with. Module types
//! and logic belong in their own crates.

mod forest;

pub use forest::forest;

use bevy::prelude::*;
use foundation::player::PlayerId;
use lightyear::prelude::*;
use movement::{MovementConfig, MovementProtocolPlugin};

/// Add this after lightyear's client or server plugins.
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.component::<PlayerId>().replicate();
        app.add_plugins(MovementProtocolPlugin);
    }
}

pub fn movement_config() -> MovementConfig {
    MovementConfig {
        sneak_speed: 1.5,
        run_speed: 5.4,
        sprint_speed: 8.2,
        dodge_lunge_speed: 9.0,
        dodge_lunge_secs: 0.4,
        dodge_rollout_speed: 2.2,
        dodge_rollout_secs: 0.6,
        dodge_recovery_secs: 0.25,
        dodge_recovery_speed: 1.5,
        jump_speed: 7.0,
        gravity: 18.0,
        ground_snap: 0.4,
        body_radius: 0.4,
        world_half_extent: 50.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdd::spec;

    #[spec]
    fn dodge_tuning() {
        given("this game's movement tuning", || {
            let config = movement_config();

            when("a roll is compared with running away", || {
                let (lunge, sprint) = (config.dodge_lunge_speed, config.sprint_speed);

                test("a lunge outruns a sprint", || {
                    assert!(lunge > sprint);
                });
            });

            when("the two halves of a roll are compared", || {
                let (lunge, rollout) = (config.dodge_lunge_speed, config.dodge_rollout_speed);

                test("the roll-out covers far less ground", || {
                    assert!(rollout < lunge / 2.0);
                });
            });

            when("recovery is compared with normal movement", || {
                let recovery = config.dodge_recovery_speed;

                test("recovery is much slower than running", || {
                    assert!(recovery < config.run_speed / 2.0);
                });
            });
        });
    }
}
