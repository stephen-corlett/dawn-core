use bdd::spec;

#[spec]
fn shared_setup() {
    given("two players", || {
        let mut players = [0.0, 0.0];

        when("player 1 walks", || {
            players[0] += 1.0;

            test("player 1 moves", || {
                assert_eq!(players[0], 1.0);
            });

            test("player 2 stays still", || {
                assert_eq!(players[1], 0.0);
            });
        });

        when("player 2 walks", || {
            players[1] += 1.0;

            test("each test gets fresh setup", || {
                assert_eq!(players, [0.0, 1.0]);
            });
        });
    });
}

#[spec]
fn descriptions() {
    given("punctuation, CAPITALS, and numbers: 1!", || {
        let value = true;

        when("it's used", || {
            test("the test still runs", || {
                assert!(value);
            });

            test("move", || {
                assert!(value);
            });

            test("5 players", || {
                assert!(value);
            });
        });
    });

    given("two scenarios with one description", || {
        let first = 1;

        when("the first adds a test", || {
            test("it runs", || {
                assert_eq!(first, 1);
            });
        });
    });

    given("two scenarios with one description", || {
        let second = 2;

        when("the second adds a test", || {
            test("it runs", || {
                assert_eq!(second, 2);
            });
        });
    });
}

#[spec]
fn setup_code() {
    given("setup containing blocks and method calls", || {
        let mut total = 0;
        for value in [1, 2, 3] {
            total += value;
        }
        let text = Some("test").map(|text| text.len());

        when("a test reads the results", || {
            let total = total * 2;

            test("all of the setup ran", || {
                assert_eq!(total, 12);
                assert_eq!(text, Some(4));
            });
        });
    });
}
