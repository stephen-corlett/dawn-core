use crate::server_app;
use bdd::spec;
use bevy::prelude::*;
use foundation::player::PlayerId;
use lightyear::prelude::*;
use movement::{MoveInput, PlayerPosition};
use test_support::fake_client::{FakeClientSettings, ScriptedInput, fake_client_app};
use test_support::{count, prepare, run_until};

fn connected_pair(port: u16) -> (App, App) {
    let mut server = server_app(port);
    let mut client = prepare(fake_client_app(FakeClientSettings::new(port, 1)));
    run_until(&mut [&mut server, &mut client], "client's player", |apps| {
        count::<(With<PlayerId>, With<Controlled>)>(apps[1]) == 1
    });
    (server, client)
}

fn has_position(app: &mut App, matches: impl Fn(Vec3) -> bool) -> bool {
    app.world_mut()
        .query::<&PlayerPosition>()
        .iter(app.world())
        .any(|position| matches(position.0))
}

#[spec]
fn networked_movement() {
    given("a connected client", || {
        let (mut server, mut client) = connected_pair(5211);

        when("the client holds forward", || {
            client.insert_resource(ScriptedInput(MoveInput {
                forward: true,
                ..default()
            }));

            test("the server moves the player", || {
                run_until(&mut [&mut server, &mut client], "server movement", |apps| {
                    has_position(apps[0], |position| position.z < -1.0)
                });
            });
        });
    });

    given("a connected client", || {
        let (mut server, mut client) = connected_pair(5212);

        when("the client holds right", || {
            client.insert_resource(ScriptedInput(MoveInput {
                right: true,
                ..default()
            }));

            test("the new position reaches the client", || {
                run_until(&mut [&mut server, &mut client], "replicated move", |apps| {
                    has_position(apps[1], |position| position.x > 1.0)
                });
            });
        });
    });
}
