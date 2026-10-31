use crate::connected_client;
use bdd::spec;
use bevy::prelude::*;
use foundation::player::PlayerId;
use lightyear::prelude::*;
use movement::PlayerPosition;
use test_support::stub_server::{move_remote_player, spawn_remote_player};
use test_support::{count, run_until};

fn remote_position(app: &mut App) -> Option<Vec3> {
    app.world_mut()
        .query_filtered::<&PlayerPosition, With<Interpolated>>()
        .iter(app.world())
        .next()
        .map(|position| position.0)
}

#[spec]
fn players() {
    given("a connected client", || {
        let (mut server, mut client) = connected_client(5321);

        when("another player joins the world", || {
            spawn_remote_player(&mut server, Vec3::new(3.0, 0.0, 0.0));

            test("the client shows them as interpolated", || {
                run_until(&mut [&mut server, &mut client], "remote player", |apps| {
                    count::<(With<PlayerId>, With<Interpolated>)>(apps[1]) == 1
                });
            });
        });
    });

    given("a client that can see another player", || {
        let (mut server, mut client) = connected_client(5322);
        let player = spawn_remote_player(&mut server, Vec3::ZERO);
        run_until(&mut [&mut server, &mut client], "remote player", |apps| {
            remote_position(apps[1]).is_some()
        });

        when("the server moves that player", || {
            let destination = Vec3::new(5.0, 0.0, 0.0);
            move_remote_player(&mut server, player, destination);

            test("the client follows them to the new position", || {
                run_until(&mut [&mut server, &mut client], "remote movement", |apps| {
                    remote_position(apps[1])
                        .is_some_and(|position| position.distance(destination) < 0.1)
                });
            });
        });
    });
}
