//! Runs the real server and real clients together over UDP on this machine.
//! The clients have no window, so rendering isn't covered here.

use ::client::build_headless_app;
use ::client::network::ClientSettings;
use ::server::{ServerSettings, build_app};
use bdd::spec;
use bevy::prelude::*;
use foundation::player::PlayerId;
use lightyear::prelude::*;
use movement::PlayerPosition;
use test_support::{count, local_addr, prepare, run_until};

fn server_app(port: u16) -> App {
    prepare(build_app(ServerSettings {
        bind_addr: local_addr(port),
        ..default()
    }))
}

fn client_app(port: u16, client_id: u64) -> App {
    let mut app = build_headless_app(ClientSettings {
        client_id,
        server_addr: local_addr(port),
        ..default()
    });
    app.insert_resource(ButtonInput::<KeyCode>::default());
    prepare(app)
}

#[spec]
fn gameplay() {
    given("two connected clients that can see each other", || {
        let mut server = server_app(5101);
        let mut first = client_app(5101, 1);
        let mut second = client_app(5101, 2);
        run_until(
            &mut [&mut server, &mut first, &mut second],
            "each client seeing the other",
            |apps| {
                [1, 2].into_iter().all(|client| {
                    count::<(With<PlayerId>, With<Predicted>)>(apps[client]) == 1
                        && count::<(With<PlayerId>, With<Interpolated>)>(apps[client]) == 1
                })
            },
        );

        when("the first player holds W", || {
            first
                .world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyW);

            test("the second client sees them move", || {
                run_until(
                    &mut [&mut server, &mut first, &mut second],
                    "second client seeing the first client move",
                    |apps| {
                        apps[2]
                            .world_mut()
                            .query_filtered::<&PlayerPosition, With<Interpolated>>()
                            .iter(apps[2].world())
                            .any(|position| position.0.z < -1.0)
                    },
                );
            });
        });
    });
}
