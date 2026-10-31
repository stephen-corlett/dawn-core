use crate::client_app;
use bdd::spec;
use bevy::prelude::*;
use core::time::Duration;
use lightyear::connection::client::Connected;
use test_support::stub_server::stub_server_app;
use test_support::{count, prepare, run_for, run_until};

#[spec]
fn connection() {
    given("a running server and a client", || {
        let mut server = prepare(stub_server_app(5301));
        let mut client = client_app(5301);

        when("the client runs", || {
            run_until(&mut [&mut server, &mut client], "connection", |apps| {
                count::<With<Connected>>(apps[1]) == 1
            });

            test("it's connected", || {
                assert_eq!(count::<With<Connected>>(&mut client), 1);
            });
        });
    });

    given("a client with no server running", || {
        let mut client = client_app(5302);

        when("it runs for a second", || {
            run_for(&mut [&mut client], Duration::from_secs(1));

            test("it isn't connected", || {
                assert_eq!(count::<With<Connected>>(&mut client), 0);
            });
        });
    });
}
