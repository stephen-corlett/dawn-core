//! Chooses what each body's animation should be, and blends between them.

use super::AvatarSystems;
use super::motion::Stride;
use bevy::prelude::*;
use core::time::Duration;
use movement::MovementConfig;

/// How a looping movement clip follows a body's speed. Its rate stretches to keep the feet
/// planted, but only so far, so that a sprint under a huge speed boost still looks like a
/// sprint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GaitClip {
    /// How fast the clip's feet travel at normal rate, in world units per second.
    pub speed: f32,
    pub slowest_rate: f32,
    pub fastest_rate: f32,
}

impl GaitClip {
    fn rate(&self, speed: f32) -> f32 {
        (speed / self.speed).clamp(self.slowest_rate, self.fastest_rate)
    }
}

/// Where each pose's clip sits in the animation library, as its glTF animation index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClipIndices {
    pub idle: usize,
    pub sneak_idle: usize,
    pub sneak: usize,
    pub walk: usize,
    pub run: usize,
    pub sprint: usize,
    pub roll: usize,
    pub jump_start: usize,
    pub falling: usize,
    pub landing: usize,
}

/// The clips bodies use, and how to play them.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct AnimationConfig {
    /// The glTF file holding the clips, relative to the asset folder.
    pub library: String,
    pub clips: ClipIndices,
    pub sneak_clip: GaitClip,
    pub walk_clip: GaitClip,
    pub run_clip: GaitClip,
    pub sprint_clip: GaitClip,
    /// A body upright and slower than this share of normal run speed walks.
    pub walk_below: f32,
    /// A body upright and faster than this share of normal run speed sprints. It sits a
    /// little over 1, so that a body at run speed doesn't flicker between the two.
    pub sprint_above: f32,
    /// Plays the jump start and landing clips this much faster than normal.
    pub jump_rate: f32,
    /// How fast the roll clip plays during a dodge's lunge. It should reach the frame
    /// where the body lands on its back just as the lunge ends.
    pub lunge_rate: f32,
    /// How fast the roll clip plays during the roll-out. The roll should end where the
    /// clip's feet stop travelling, or the body glides through the clip's stand-up.
    pub rollout_rate: f32,
    /// How long one pose takes to blend into the next.
    pub fade_secs: f32,
    /// A body moving slower than this stands idle.
    pub moving_speed: f32,
}

/// What a body is doing, in terms of which clip to show.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Pose {
    #[default]
    Idle,
    SneakIdle,
    Sneak,
    Walk,
    Run,
    Sprint,
    Roll,
    JumpStart,
    Falling,
    Landing,
}

impl Pose {
    fn loops(self) -> bool {
        !matches!(self, Pose::Roll | Pose::JumpStart | Pose::Landing)
    }
}

/// The body that plays animations, holding the pose it's showing. `None` means it
/// hasn't started yet.
#[derive(Component, Default)]
pub(super) struct Rig {
    pose: Option<Pose>,
}

/// The shared animation graph, and where each pose's clip sits in it.
#[derive(Resource)]
pub(super) struct AvatarClips {
    pub graph: Handle<AnimationGraph>,
    nodes: Nodes,
}

#[derive(Clone, Copy)]
struct Nodes {
    idle: AnimationNodeIndex,
    sneak_idle: AnimationNodeIndex,
    sneak: AnimationNodeIndex,
    walk: AnimationNodeIndex,
    run: AnimationNodeIndex,
    sprint: AnimationNodeIndex,
    roll: AnimationNodeIndex,
    jump_start: AnimationNodeIndex,
    falling: AnimationNodeIndex,
    landing: AnimationNodeIndex,
}

impl Nodes {
    fn of(&self, pose: Pose) -> AnimationNodeIndex {
        match pose {
            Pose::Idle => self.idle,
            Pose::SneakIdle => self.sneak_idle,
            Pose::Sneak => self.sneak,
            Pose::Walk => self.walk,
            Pose::Run => self.run,
            Pose::Sprint => self.sprint,
            Pose::Roll => self.roll,
            Pose::JumpStart => self.jump_start,
            Pose::Falling => self.falling,
            Pose::Landing => self.landing,
        }
    }
}

pub(super) struct AnimationPlugin {
    pub config: AnimationConfig,
}

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        let clips = load_clips(app, &self.config);
        app.insert_resource(self.config.clone());
        app.insert_resource(clips);
        app.add_systems(Update, drive_rigs.in_set(AvatarSystems::Present));
    }
}

fn load_clips(app: &mut App, config: &AnimationConfig) -> AvatarClips {
    let assets = app.world().resource::<AssetServer>();
    let load = |index| {
        assets.load::<AnimationClip>(
            GltfAssetLabel::Animation(index).from_asset(config.library.clone()),
        )
    };
    let indices = config.clips;
    let (graph, nodes) = AnimationGraph::from_clips([
        load(indices.idle),
        load(indices.sneak_idle),
        load(indices.sneak),
        load(indices.walk),
        load(indices.run),
        load(indices.sprint),
        load(indices.roll),
        load(indices.jump_start),
        load(indices.falling),
        load(indices.landing),
    ]);
    let nodes = Nodes {
        idle: nodes[0],
        sneak_idle: nodes[1],
        sneak: nodes[2],
        walk: nodes[3],
        run: nodes[4],
        sprint: nodes[5],
        roll: nodes[6],
        jump_start: nodes[7],
        falling: nodes[8],
        landing: nodes[9],
    };
    let graph = app
        .world_mut()
        .resource_mut::<Assets<AnimationGraph>>()
        .add(graph);
    AvatarClips { graph, nodes }
}

fn drive_rigs(
    clips: Res<AvatarClips>,
    config: Res<AnimationConfig>,
    movement: Res<MovementConfig>,
    strides: Query<&Stride>,
    mut rigs: Query<(
        &ChildOf,
        &mut Rig,
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>,
) {
    let fade = Duration::from_secs_f32(config.fade_secs);
    for (parent, mut rig, mut player, mut transitions) in &mut rigs {
        let Ok(stride) = strides.get(parent.parent()) else {
            continue;
        };
        let clip_finished = rig.pose.is_some_and(|pose| {
            player
                .animation(clips.nodes.of(pose))
                .is_none_or(|active| active.is_finished())
        });
        let current = rig.pose.unwrap_or_default();
        let next = next_pose(current, stride, clip_finished, &config, &movement);

        if rig.pose != Some(next) {
            let active = transitions.play(&mut player, clips.nodes.of(next), fade);
            if next.loops() {
                active.repeat();
            }
            rig.pose = Some(next);
        }
        if let Some(active) = player.animation_mut(clips.nodes.of(next)) {
            active.set_speed(playback_rate(next, stride, &config));
        }
    }
}

/// Decides the next pose. A roll beats everything else, and recovery comes next, so that
/// being caught off balance never looks like choosing to sneak. Jumps play through start,
/// falling, and landing in order, and a body that starts moving cuts its landing short.
fn next_pose(
    current: Pose,
    stride: &Stride,
    clip_finished: bool,
    config: &AnimationConfig,
    movement: &MovementConfig,
) -> Pose {
    if stride.rolling {
        return Pose::Roll;
    }
    let on_foot = gait_pose(stride, config, movement);
    let moving = stride.speed >= config.moving_speed;
    if stride.recovering && !stride.airborne {
        return if moving { Pose::Walk } else { Pose::Landing };
    }
    match (current, stride.airborne) {
        (Pose::JumpStart, true) if !clip_finished => Pose::JumpStart,
        (Pose::JumpStart | Pose::Falling, true) => Pose::Falling,
        (_, true) => Pose::JumpStart,
        (Pose::JumpStart | Pose::Falling, false) => Pose::Landing,
        (Pose::Landing, false) if !clip_finished && !moving => Pose::Landing,
        (_, false) => on_foot,
    }
}

/// Picks the pose for a body on its feet from its speed as a share of normal run speed,
/// so that anything that changes a body's speed changes its gait too.
fn gait_pose(stride: &Stride, config: &AnimationConfig, movement: &MovementConfig) -> Pose {
    let moving = stride.speed >= config.moving_speed;
    let share_of_run = stride.speed / movement.run_speed;
    match (stride.sneaking, moving) {
        (true, true) => Pose::Sneak,
        (true, false) => Pose::SneakIdle,
        (false, false) => Pose::Idle,
        (false, true) if share_of_run < config.walk_below => Pose::Walk,
        (false, true) if share_of_run <= config.sprint_above => Pose::Run,
        (false, true) => Pose::Sprint,
    }
}

fn playback_rate(pose: Pose, stride: &Stride, config: &AnimationConfig) -> f32 {
    match pose {
        Pose::Sneak => config.sneak_clip.rate(stride.speed),
        Pose::Walk => config.walk_clip.rate(stride.speed),
        Pose::Run => config.run_clip.rate(stride.speed),
        Pose::Sprint => config.sprint_clip.rate(stride.speed),
        Pose::Roll if stride.rolling_out => config.rollout_rate,
        Pose::Roll => config.lunge_rate,
        Pose::JumpStart | Pose::Landing => config.jump_rate,
        Pose::Idle | Pose::SneakIdle | Pose::Falling => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdd::spec;

    const MOVEMENT: MovementConfig = MovementConfig {
        sneak_speed: 2.0,
        run_speed: 6.0,
        sprint_speed: 10.0,
        dodge_lunge_speed: 7.0,
        dodge_lunge_secs: 0.4,
        dodge_rollout_speed: 2.0,
        dodge_rollout_secs: 0.3,
        dodge_recovery_secs: 0.6,
        dodge_recovery_speed: 1.5,
        jump_speed: 7.0,
        gravity: 18.0,
        ground_snap: 0.4,
        body_radius: 0.4,
        world_half_extent: 30.0,
    };

    const IN_AIR: Stride = Stride {
        speed: 0.0,
        airborne: true,
        rolling: false,
        rolling_out: false,
        recovering: false,
        sneaking: false,
    };

    fn config() -> AnimationConfig {
        AnimationConfig {
            library: "clips.glb".into(),
            clips: ClipIndices {
                idle: 0,
                sneak_idle: 7,
                sneak: 1,
                walk: 9,
                run: 2,
                sprint: 3,
                roll: 8,
                jump_start: 4,
                falling: 5,
                landing: 6,
            },
            sneak_clip: GaitClip {
                speed: 1.0,
                slowest_rate: 0.5,
                fastest_rate: 2.0,
            },
            walk_clip: GaitClip {
                speed: 1.0,
                slowest_rate: 0.5,
                fastest_rate: 2.0,
            },
            run_clip: GaitClip {
                speed: 6.0,
                slowest_rate: 0.5,
                fastest_rate: 1.2,
            },
            sprint_clip: GaitClip {
                speed: 8.0,
                slowest_rate: 0.8,
                fastest_rate: 1.25,
            },
            walk_below: 0.5,
            sprint_above: 1.1,
            jump_rate: 2.0,
            lunge_rate: 2.5,
            rollout_rate: 1.5,
            fade_secs: 0.1,
            moving_speed: 0.3,
        }
    }

    fn on_ground(speed: f32) -> Stride {
        Stride { speed, ..default() }
    }

    fn rate(pose: Pose, speed: f32) -> f32 {
        playback_rate(pose, &on_ground(speed), &config())
    }

    fn recovering(speed: f32) -> Stride {
        Stride {
            speed,
            recovering: true,
            ..default()
        }
    }

    fn sneaking(speed: f32) -> Stride {
        Stride {
            speed,
            sneaking: true,
            ..default()
        }
    }

    const ROLLING: Stride = Stride {
        speed: 7.0,
        airborne: false,
        rolling: true,
        rolling_out: false,
        recovering: false,
        sneaking: false,
    };

    fn next(current: Pose, stride: &Stride, clip_finished: bool) -> Pose {
        next_pose(current, stride, clip_finished, &config(), &MOVEMENT)
    }

    fn animation_app() -> App {
        let mut app = App::new();
        app.add_plugins((TaskPoolPlugin::default(), AssetPlugin::default()));
        app.init_asset::<AnimationClip>();
        app.init_asset::<AnimationGraph>();
        app.insert_resource(MOVEMENT);
        app.add_plugins(AnimationPlugin { config: config() });
        app
    }

    fn spawn_rig(app: &mut App, parent: Entity) -> Entity {
        app.world_mut()
            .spawn((
                ChildOf(parent),
                Rig::default(),
                AnimationPlayer::default(),
                AnimationTransitions::new(),
            ))
            .id()
    }

    fn spawn_body(app: &mut App, stride: Stride) -> (Entity, Entity) {
        let body = app.world_mut().spawn(stride).id();
        let rig = spawn_rig(app, body);
        (body, rig)
    }

    fn pose_of(app: &App, rig: Entity) -> Option<Pose> {
        app.world().get::<Rig>(rig).unwrap().pose
    }

    #[spec]
    fn pose_rules() {
        given("a body on the ground", || {
            when("it stands still", || {
                let pose = next(Pose::Idle, &on_ground(0.0), false);

                test("it idles", || {
                    assert_eq!(pose, Pose::Idle);
                });
            });

            when("it moves under half of run speed", || {
                let pose = next(Pose::Idle, &on_ground(2.0), false);

                test("it walks", || {
                    assert_eq!(pose, Pose::Walk);
                });
            });

            when("it moves over half of run speed", || {
                let pose = next(Pose::Walk, &on_ground(3.5), false);

                test("it runs", || {
                    assert_eq!(pose, Pose::Run);
                });
            });

            when("it wobbles just over run speed", || {
                let pose = next(Pose::Run, &on_ground(6.2), false);

                test("it keeps running", || {
                    assert_eq!(pose, Pose::Run);
                });
            });

            when("it moves near running speed", || {
                let pose = next(Pose::Idle, &on_ground(6.0), false);

                test("it runs", || {
                    assert_eq!(pose, Pose::Run);
                });
            });

            when("it moves near sprinting speed", || {
                let pose = next(Pose::Run, &on_ground(10.0), false);

                test("it sprints", || {
                    assert_eq!(pose, Pose::Sprint);
                });
            });

            when("it leaves the ground", || {
                let pose = next(Pose::Run, &IN_AIR, false);

                test("it starts a jump", || {
                    assert_eq!(pose, Pose::JumpStart);
                });
            });
        });

        given("a sneaking body", || {
            when("it stands still", || {
                let pose = next(Pose::Idle, &sneaking(0.0), false);

                test("it crouches in place", || {
                    assert_eq!(pose, Pose::SneakIdle);
                });
            });

            when("it moves", || {
                let pose = next(Pose::SneakIdle, &sneaking(1.5), false);

                test("it sneaks", || {
                    assert_eq!(pose, Pose::Sneak);
                });
            });
        });

        given("a body starting a roll", || {
            when("it was running", || {
                let pose = next(Pose::Run, &ROLLING, false);

                test("it rolls", || {
                    assert_eq!(pose, Pose::Roll);
                });
            });

            when("it was in the middle of landing", || {
                let pose = next(Pose::Landing, &ROLLING, false);

                test("the roll takes over", || {
                    assert_eq!(pose, Pose::Roll);
                });
            });

            when("it was sneaking", || {
                let pose = next(Pose::Sneak, &ROLLING, false);

                test("the roll takes over", || {
                    assert_eq!(pose, Pose::Roll);
                });
            });
        });

        given("a body that has finished rolling", || {
            when("it keeps running", || {
                let pose = next(Pose::Roll, &on_ground(6.0), true);

                test("it runs", || {
                    assert_eq!(pose, Pose::Run);
                });
            });

            when("it was sneaking", || {
                let pose = next(Pose::Roll, &sneaking(0.0), true);

                test("it goes back to crouching", || {
                    assert_eq!(pose, Pose::SneakIdle);
                });
            });
        });

        given("a body recovering from a roll", || {
            when("it keeps moving", || {
                let pose = next(Pose::Roll, &recovering(1.5), false);

                test("it walks", || {
                    assert_eq!(pose, Pose::Walk);
                });
            });

            when("it stands still", || {
                let pose = next(Pose::Roll, &recovering(0.0), false);

                test("it gets its balance back", || {
                    assert_eq!(pose, Pose::Landing);
                });
            });

            when("its player has sneaking switched on", || {
                let stride = Stride {
                    sneaking: true,
                    ..recovering(1.5)
                };
                let pose = next(Pose::Roll, &stride, false);

                test("it still looks off balance, not sneaky", || {
                    assert_eq!(pose, Pose::Walk);
                });
            });

            when("it runs off a ledge", || {
                let stride = Stride {
                    airborne: true,
                    ..recovering(1.5)
                };
                let pose = next(Pose::Walk, &stride, false);

                test("it starts to fall", || {
                    assert_eq!(pose, Pose::JumpStart);
                });
            });
        });

        given("a body whose recovery has just ended", || {
            when("its player has sneaking off", || {
                let pose = next(Pose::Walk, &on_ground(6.0), false);

                test("it runs again", || {
                    assert_eq!(pose, Pose::Run);
                });
            });

            when("its player has sneaking on", || {
                let pose = next(Pose::Walk, &sneaking(1.5), false);

                test("it goes back to sneaking", || {
                    assert_eq!(pose, Pose::Sneak);
                });
            });
        });

        given("a body starting a jump", || {
            when("the clip is still playing", || {
                let pose = next(Pose::JumpStart, &IN_AIR, false);

                test("it stays in the jump start", || {
                    assert_eq!(pose, Pose::JumpStart);
                });
            });

            when("the clip finishes in the air", || {
                let pose = next(Pose::JumpStart, &IN_AIR, true);

                test("it starts falling", || {
                    assert_eq!(pose, Pose::Falling);
                });
            });

            when("it reaches the ground early", || {
                let pose = next(Pose::JumpStart, &on_ground(0.0), false);

                test("it lands", || {
                    assert_eq!(pose, Pose::Landing);
                });
            });
        });

        given("a body falling", || {
            when("it stays in the air", || {
                let pose = next(Pose::Falling, &IN_AIR, false);

                test("it keeps falling", || {
                    assert_eq!(pose, Pose::Falling);
                });
            });

            when("it reaches the ground", || {
                let pose = next(Pose::Falling, &on_ground(0.0), false);

                test("it lands", || {
                    assert_eq!(pose, Pose::Landing);
                });
            });
        });

        given("a body landing", || {
            when("it stands still with the clip playing", || {
                let pose = next(Pose::Landing, &on_ground(0.0), false);

                test("it finishes landing", || {
                    assert_eq!(pose, Pose::Landing);
                });
            });

            when("the clip finishes", || {
                let pose = next(Pose::Landing, &on_ground(0.0), true);

                test("it goes back to idling", || {
                    assert_eq!(pose, Pose::Idle);
                });
            });

            when("it starts to run", || {
                let pose = next(Pose::Landing, &on_ground(6.0), false);

                test("it cuts the landing short", || {
                    assert_eq!(pose, Pose::Run);
                });
            });
        });
    }

    #[spec]
    fn playback_rates() {
        given("a run clip whose feet travel at 6 units per second", || {
            let config = config();

            when("a body runs at that speed", || {
                let rate = rate(Pose::Run, 6.0);

                test("the clip plays at normal speed", || {
                    assert_eq!(rate, 1.0);
                });
            });

            when("a body runs at half that speed", || {
                let rate = rate(Pose::Run, 3.0);

                test("the clip slows down to match", || {
                    assert_eq!(rate, 0.5);
                });
            });

            when("a body runs far too fast", || {
                let rate = rate(Pose::Run, 60.0);

                test("the clip stops speeding up", || {
                    assert_eq!(rate, config.run_clip.fastest_rate);
                });
            });

            when("a speed boost pushes a sprint far past normal", || {
                let rate = rate(Pose::Sprint, 80.0);

                test("it still plays like a sprint", || {
                    assert_eq!(rate, config.sprint_clip.fastest_rate);
                });
            });

            when("a sprint is barely moving", || {
                let rate = rate(Pose::Sprint, 1.0);

                test("the clip stops slowing down", || {
                    assert_eq!(rate, config.sprint_clip.slowest_rate);
                });
            });

            when("a sneaking or sprinting body is measured", || {
                let sneaking = rate(Pose::Sneak, 1.5);
                let sprinting = rate(Pose::Sprint, 8.0);

                test("each uses its own clip's speed", || {
                    assert_eq!(sneaking, 1.5);
                    assert_eq!(sprinting, 1.0);
                });
            });

            when("a walking body is measured", || {
                let rate = rate(Pose::Walk, 1.5);

                test("it uses its own clip's speed", || {
                    assert_eq!(rate, 1.5);
                });
            });

            when("a roll is in its lunge", || {
                let rate = playback_rate(Pose::Roll, &ROLLING, &config);

                test("it uses the lunge rate", || {
                    assert_eq!(rate, config.lunge_rate);
                });
            });

            when("a roll is rolling out", || {
                let stride = Stride {
                    rolling_out: true,
                    ..ROLLING
                };
                let rate = playback_rate(Pose::Roll, &stride, &config);

                test("the animation slows down", || {
                    assert_eq!(rate, config.rollout_rate);
                });
            });

            when("a jump is played", || {
                let rate = rate(Pose::JumpStart, 0.0);

                test("it uses the jump rate", || {
                    assert_eq!(rate, config.jump_rate);
                });
            });

            when("a body stands idle", || {
                let rate = rate(Pose::Idle, 0.0);

                test("the clip plays at normal speed", || {
                    assert_eq!(rate, 1.0);
                });
            });
        });
    }

    #[spec]
    fn driving_rigs() {
        given("a new rig on a running body", || {
            let mut app = animation_app();
            let (_, rig) = spawn_body(&mut app, on_ground(6.0));

            when("a frame runs", || {
                app.world_mut().run_schedule(Update);

                test("it starts running", || {
                    assert_eq!(pose_of(&app, rig), Some(Pose::Run));
                });

                test("its player is showing the run clip", || {
                    let run = app.world().resource::<AvatarClips>().nodes.run;
                    let player = app.world().get::<AnimationPlayer>(rig).unwrap();
                    assert!(player.is_playing_animation(run));
                });
            });
        });

        given("a rig that has started running", || {
            let mut app = animation_app();
            let (body, rig) = spawn_body(&mut app, on_ground(6.0));
            app.world_mut().run_schedule(Update);

            when("its body stops", || {
                *app.world_mut().get_mut::<Stride>(body).unwrap() = on_ground(0.0);
                app.world_mut().run_schedule(Update);

                test("it changes pose", || {
                    assert_eq!(pose_of(&app, rig), Some(Pose::Idle));
                });
            });

            when("its body slows a little", || {
                *app.world_mut().get_mut::<Stride>(body).unwrap() = on_ground(5.0);
                app.world_mut().run_schedule(Update);

                test("its clip slows down to match", || {
                    let run = app.world().resource::<AvatarClips>().nodes.run;
                    let player = app.world().get::<AnimationPlayer>(rig).unwrap();
                    assert_eq!(player.animation(run).unwrap().speed(), 5.0 / 6.0);
                });
            });
        });

        given("a rig with nothing above it that has a stride", || {
            let mut app = animation_app();
            let parent = app.world_mut().spawn_empty().id();
            let rig = spawn_rig(&mut app, parent);

            when("a frame runs", || {
                app.world_mut().run_schedule(Update);

                test("it's left alone", || {
                    assert_eq!(pose_of(&app, rig), None);
                });
            });
        });
    }
}
