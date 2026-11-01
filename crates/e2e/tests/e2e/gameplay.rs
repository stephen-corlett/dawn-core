//! Runs the real server and real clients together over UDP on this machine.
//! The clients have no window, so rendering isn't covered here.

use ::client::build_headless_app;
use ::client::config::ClientConfig;
use ::server::build_app;
use ::server::config::ServerConfig;
use bdd::spec;
use bevy::prelude::*;
use foundation::player::PlayerId;
use lightyear::prelude::*;
use movement::{Dodge, PlayerPosition, Sneaking};
use test_support::{count, local_addr, prepare, run_until};

fn server_app(port: u16) -> App {
    prepare(build_app(ServerConfig {
        bind_addr: local_addr(port),
        ..ServerConfig::dev_default()
    }))
}

fn client_app(port: u16, client_id: u64) -> App {
    let mut app = build_headless_app(ClientConfig {
        client_id,
        server_addr: local_addr(port),
        ..ClientConfig::dev_default()
    });
    app.insert_resource(ButtonInput::<KeyCode>::default());
    prepare(app)
}

/// A server and two clients that have each found the other's player.
fn connected_clients(port: u16) -> (App, App, App) {
    let mut server = server_app(port);
    let mut first = client_app(port, 1);
    let mut second = client_app(port, 2);
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
    (server, first, second)
}

fn hold_keys(app: &mut App, keys: &[KeyCode]) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    for key in keys {
        input.press(*key);
    }
}

/// Whether a client sees a remote player whose component passes a check.
fn remote_player_has<C: Component>(app: &mut App, check: impl Fn(&C) -> bool) -> bool {
    app.world_mut()
        .query_filtered::<&C, With<Interpolated>>()
        .iter(app.world())
        .any(check)
}

#[spec]
fn gameplay() {
    given("two connected clients that can see each other", || {
        let (mut server, mut first, mut second) = connected_clients(5101);

        when("the first player holds W", || {
            hold_keys(&mut first, &[KeyCode::KeyW]);

            test("the second client sees them move", || {
                run_until(
                    &mut [&mut server, &mut first, &mut second],
                    "second client seeing the first client move",
                    |apps| {
                        remote_player_has::<PlayerPosition>(apps[2], |position| position.0.z < -1.0)
                    },
                );
            });
        });
    });

    given("two clients, one about to dodge", || {
        let (mut server, mut first, mut second) = connected_clients(5102);

        when("the first player dodges while moving", || {
            hold_keys(&mut first, &[KeyCode::KeyW, KeyCode::ControlLeft]);

            test("the second client sees them roll", || {
                run_until(
                    &mut [&mut server, &mut first, &mut second],
                    "second client seeing the first client roll",
                    |apps| remote_player_has::<Dodge>(apps[2], Dodge::is_rolling),
                );
            });
        });
    });

    given("two clients, one about to sneak", || {
        let (mut server, mut first, mut second) = connected_clients(5103);

        when("the first player switches sneaking on", || {
            hold_keys(&mut first, &[KeyCode::KeyC]);
            first.update();
            first
                .world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .clear();

            test("the second client sees them sneak", || {
                run_until(
                    &mut [&mut server, &mut first, &mut second],
                    "second client seeing the first client sneak",
                    |apps| remote_player_has::<Sneaking>(apps[2], |sneaking| sneaking.0),
                );
            });
        });
    });
}
