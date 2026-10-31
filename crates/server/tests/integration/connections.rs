use crate::server_app;
use bdd::spec;
use bevy::prelude::*;
use core::time::Duration;
use foundation::net::PROTOCOL_ID;
use foundation::player::PlayerId;
use lightyear::connection::client::Connected;
use lightyear::prelude::*;
use test_support::fake_client::{FakeClientSettings, disconnect, fake_client_app};
use test_support::{count, prepare, run_for, run_until};

const REFUSAL_WAIT: Duration = Duration::from_secs(3);

fn client_app(settings: FakeClientSettings) -> App {
    prepare(fake_client_app(settings))
}

#[spec]
fn connections() {
    given("a running server and a client", || {
        let mut server = server_app(5201);
        let mut client = client_app(FakeClientSettings::new(5201, 1));

        when("the client connects", || {
            run_until(
                &mut [&mut server, &mut client],
                "the client's player to arrive",
                |apps| count::<(With<PlayerId>, With<Controlled>)>(apps[1]) == 1,
            );

            test("the server holds one player", || {
                assert_eq!(count::<With<PlayerId>>(&mut server), 1);
            });
        });
    });

    given("a running server and two clients", || {
        let mut server = server_app(5202);
        let mut first = client_app(FakeClientSettings::new(5202, 1));
        let mut second = client_app(FakeClientSettings::new(5202, 2));

        when("both clients connect", || {
            run_until(
                &mut [&mut server, &mut first, &mut second],
                "both clients to see both players",
                |apps| {
                    count::<With<PlayerId>>(apps[1]) == 2 && count::<With<PlayerId>>(apps[2]) == 2
                },
            );

            test("the server holds two players", || {
                assert_eq!(count::<With<PlayerId>>(&mut server), 2);
            });
        });
    });

    given("a connected client with a player", || {
        let mut server = server_app(5203);
        let mut client = client_app(FakeClientSettings::new(5203, 1));
        run_until(&mut [&mut server, &mut client], "player to spawn", |apps| {
            count::<With<PlayerId>>(apps[0]) == 1
        });

        when("the client disconnects", || {
            disconnect(&mut client);

            test("the server removes the player", || {
                run_until(
                    &mut [&mut server, &mut client],
                    "player to despawn",
                    |apps| count::<With<PlayerId>>(apps[0]) == 0,
                );
            });
        });
    });

    given("a client with the wrong protocol version", || {
        let mut server = server_app(5204);
        let mut client = client_app(FakeClientSettings {
            protocol_id: PROTOCOL_ID + 1,
            ..FakeClientSettings::new(5204, 1)
        });

        when("it tries to connect", || {
            run_for(&mut [&mut server, &mut client], REFUSAL_WAIT);

            test("it isn't connected and has no player", || {
                assert_eq!(count::<With<Connected>>(&mut client), 0);
                assert_eq!(count::<With<PlayerId>>(&mut server), 0);
            });
        });
    });

    given("a client with the wrong private key", || {
        let mut server = server_app(5205);
        let mut client = client_app(FakeClientSettings {
            private_key: [7; 32],
            ..FakeClientSettings::new(5205, 1)
        });

        when("it tries to connect", || {
            run_for(&mut [&mut server, &mut client], REFUSAL_WAIT);

            test("it isn't connected and has no player", || {
                assert_eq!(count::<With<Connected>>(&mut client), 0);
                assert_eq!(count::<With<PlayerId>>(&mut server), 0);
            });
        });
    });
}
