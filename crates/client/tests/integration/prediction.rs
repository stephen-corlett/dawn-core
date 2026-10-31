use crate::connected_client;
use bdd::spec;
use bevy::prelude::*;
use lightyear::prelude::*;
use movement::PlayerPosition;
use test_support::run_until;
use test_support::stub_server::set_controlled_positions;

fn predicted_position(app: &mut App) -> Vec3 {
    app.world_mut()
        .query_filtered::<&PlayerPosition, With<Predicted>>()
        .single(app.world())
        .expect("the client has one predicted player")
        .0
}

#[spec]
fn prediction() {
    given(
        "a connected client and a server that never moves this player",
        || {
            let (mut server, mut client) = connected_client(5311);

            when("the player holds W", || {
                client
                    .world_mut()
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .press(KeyCode::KeyW);

                test("the client moves its own player anyway", || {
                    run_until(
                        &mut [&mut server, &mut client],
                        "local prediction",
                        |apps| predicted_position(apps[1]).z < 0.0,
                    );
                });
            });
        },
    );

    given("a connected client at the start position", || {
        let (mut server, mut client) = connected_client(5312);
        let corrected = Vec3::new(10.0, 0.0, 10.0);

        when("the server moves the player somewhere else", || {
            set_controlled_positions(&mut server, corrected);

            test("the client adopts the server's position", || {
                run_until(
                    &mut [&mut server, &mut client],
                    "corrected position",
                    |apps| predicted_position(apps[1]).distance(corrected) < 0.5,
                );
            });
        });
    });
}
