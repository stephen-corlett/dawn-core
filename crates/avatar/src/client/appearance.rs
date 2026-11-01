//! Gives each displayed player a body, built from separate glTF parts on one skeleton.
//!
//! The models hold no animations, and the clips live in their own file so that every
//! body can share them. glTF only wires bones to an animation player when a file has
//! both, so this does that wiring itself.

use super::AvatarSystems;
use super::animation::{AvatarClips, Rig};
use super::motion::Stride;
use bevy::animation::{AnimatedBy, AnimationTargetId};
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;
use lightyear::prelude::*;
use movement::PlayerPosition;

/// What a player looks like.
#[derive(Clone, Debug, PartialEq)]
pub struct AppearanceConfig {
    /// glTF files to wear together, relative to the asset folder. They must share a
    /// skeleton with the animation clips.
    pub parts: Vec<String>,
    /// Turns the models to face negative Z, as radians around the up axis.
    pub model_yaw: f32,
}

/// Marks a player that has been given a body.
#[derive(Component)]
pub(super) struct Avatar;

pub(super) struct AppearancePlugin {
    pub config: AppearanceConfig,
}

impl Plugin for AppearancePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Appearance(self.config.clone()));
        app.add_systems(Update, add_avatars.in_set(AvatarSystems::Appear));
    }
}

#[derive(Resource)]
struct Appearance(AppearanceConfig);

/// Filtering on `Predicted` and `Interpolated` skips the replicated data before it's
/// ready to display.
fn add_avatars(
    mut commands: Commands,
    assets: Res<AssetServer>,
    appearance: Res<Appearance>,
    clips: Res<AvatarClips>,
    players: Query<
        (Entity, &PlayerPosition),
        (Or<(With<Predicted>, With<Interpolated>)>, Without<Avatar>),
    >,
) {
    for (player, position) in &players {
        commands.entity(player).insert((
            Avatar,
            Stride::default(),
            Transform::from_translation(position.0),
            Visibility::default(),
        ));
        let rig = commands
            .spawn((
                ChildOf(player),
                Rig::default(),
                AnimationPlayer::default(),
                AnimationTransitions::new(),
                AnimationGraphHandle(clips.graph.clone()),
                Transform::from_rotation(Quat::from_rotation_y(appearance.0.model_yaw)),
                Visibility::default(),
            ))
            .id();
        for part in &appearance.0.parts {
            let scene = assets.load(GltfAssetLabel::Scene(0).from_asset(part.clone()));
            commands
                .spawn((ChildOf(rig), WorldAssetRoot(scene)))
                .observe(connect_bones);
        }
    }
}

/// Connects a finished part's bones to the animation player of the rig it belongs to.
/// Bevy wraps each glTF scene in an entity of its own, which clips don't count as part
/// of a bone's path, so the walk starts on the wrapper's children.
fn connect_bones(
    ready: On<WorldInstanceReady>,
    parents: Query<&ChildOf>,
    children: Query<&Children>,
    names: Query<&Name>,
    mut commands: Commands,
) {
    connect_part(ready.entity, &parents, &children, &names, &mut commands);
}

fn connect_part(
    part: Entity,
    parents: &Query<&ChildOf>,
    children: &Query<&Children>,
    names: &Query<&Name>,
    commands: &mut Commands,
) {
    let Ok(rig) = parents.get(part).map(ChildOf::parent) else {
        return;
    };
    let wrappers = children.get(part).into_iter().flatten();
    for root in wrappers.flat_map(|&wrapper| children.get(wrapper).into_iter().flatten()) {
        connect_node(*root, rig, &mut Vec::new(), children, names, commands);
    }
}

/// Clips find a bone by the chain of names leading to it, starting at the glTF root.
fn connect_node(
    node: Entity,
    rig: Entity,
    path: &mut Vec<Name>,
    children: &Query<&Children>,
    names: &Query<&Name>,
    commands: &mut Commands,
) {
    let Ok(name) = names.get(node) else {
        return;
    };
    path.push(name.clone());
    commands
        .entity(node)
        .insert((AnimationTargetId::from_names(path.iter()), AnimatedBy(rig)));
    for child in children.get(node).into_iter().flatten() {
        connect_node(*child, rig, path, children, names, commands);
    }
    path.pop();
}

#[cfg(test)]
mod tests {
    use super::super::animation::AnimationPlugin;
    use super::super::{AnimationConfig, ClipIndices};
    use super::*;
    use bdd::spec;
    use bevy::ecs::system::RunSystemOnce;
    use core::f32::consts::PI;
    use movement::MovementConfig;

    fn appearance_app() -> App {
        let mut app = App::new();
        app.add_plugins((TaskPoolPlugin::default(), AssetPlugin::default()));
        app.init_asset::<AnimationClip>();
        app.init_asset::<AnimationGraph>();
        app.init_asset::<WorldAsset>();
        app.insert_resource(MovementConfig {
            sneak_speed: 2.0,
            run_speed: 6.0,
            sprint_speed: 10.0,
            dodge_speed: 7.0,
            dodge_secs: 0.7,
            dodge_cooldown_secs: 0.4,
            jump_speed: 7.0,
            gravity: 18.0,
            ground_snap: 0.4,
            body_radius: 0.4,
            world_half_extent: 30.0,
        });
        app.add_plugins(AnimationPlugin {
            config: AnimationConfig {
                library: "clips.glb".into(),
                clips: ClipIndices {
                    idle: 0,
                    sneak_idle: 7,
                    sneak: 1,
                    run: 2,
                    sprint: 3,
                    roll: 8,
                    jump_start: 4,
                    falling: 5,
                    landing: 6,
                },
                sneak_clip_speed: 1.0,
                run_clip_speed: 6.0,
                sprint_clip_speed: 8.0,
                jump_rate: 2.0,
                roll_rate: 2.5,
                fade_secs: 0.1,
                moving_speed: 0.3,
            },
        });
        app.add_plugins(AppearancePlugin {
            config: AppearanceConfig {
                parts: vec!["body.gltf".into(), "head.gltf".into()],
                model_yaw: PI,
            },
        });
        app
    }

    fn rig_count(app: &mut App) -> usize {
        let mut rigs = app.world_mut().query::<&Rig>();
        rigs.iter(app.world()).count()
    }

    fn named(app: &mut App, name: &str, parent: Entity) -> Entity {
        app.world_mut()
            .spawn((Name::new(name.to_owned()), ChildOf(parent)))
            .id()
    }

    fn target_of(app: &App, bone: Entity) -> Option<AnimationTargetId> {
        app.world().get::<AnimationTargetId>(bone).copied()
    }

    fn id_for(names: &[&str]) -> Option<AnimationTargetId> {
        let names: Vec<Name> = names
            .iter()
            .map(|name| Name::new((*name).to_owned()))
            .collect();
        Some(AnimationTargetId::from_names(names.iter()))
    }

    fn finish_loading(app: &mut App, part: Entity) {
        app.world_mut()
            .run_system_once(
                move |parents: Query<&ChildOf>,
                      children: Query<&Children>,
                      names: Query<&Name>,
                      mut commands: Commands| {
                    connect_part(part, &parents, &children, &names, &mut commands);
                },
            )
            .expect("the system runs");
    }

    #[spec]
    fn giving_players_bodies() {
        given("a predicted player", || {
            let mut app = appearance_app();
            let start = Vec3::new(1.0, 2.0, 3.0);
            let player = app
                .world_mut()
                .spawn((PlayerPosition(start), Predicted))
                .id();

            when("a frame runs", || {
                app.world_mut().run_schedule(Update);
                let (rig, parent) = app
                    .world_mut()
                    .query_filtered::<(Entity, &ChildOf), With<Rig>>()
                    .single(app.world())
                    .map(|(rig, parent)| (rig, parent.parent()))
                    .expect("there is one rig");

                test("the rig belongs to the player", || {
                    assert_eq!(parent, player);
                });

                test("the body starts where the player is", || {
                    let transform = app.world().get::<Transform>(player).unwrap();
                    assert_eq!(transform.translation, start);
                });

                test("the rig turns the model to face forward", || {
                    let transform = app.world().get::<Transform>(rig).unwrap();
                    assert_eq!(transform.rotation, Quat::from_rotation_y(PI));
                });

                test("the rig wears every part", || {
                    assert_eq!(app.world().get::<Children>(rig).unwrap().len(), 2);
                });
            });

            when("two frames run", || {
                app.world_mut().run_schedule(Update);
                app.world_mut().run_schedule(Update);

                test("the player only gets one body", || {
                    assert_eq!(rig_count(&mut app), 1);
                });
            });
        });

        given("an interpolated player", || {
            let mut app = appearance_app();
            app.world_mut()
                .spawn((PlayerPosition::default(), Interpolated));

            when("a frame runs", || {
                app.world_mut().run_schedule(Update);

                test("they get a body too", || {
                    assert_eq!(rig_count(&mut app), 1);
                });
            });
        });

        given("replicated data that isn't ready to display", || {
            let mut app = appearance_app();
            app.world_mut().spawn(PlayerPosition::default());

            when("a frame runs", || {
                app.world_mut().run_schedule(Update);

                test("it gets no body", || {
                    assert_eq!(rig_count(&mut app), 0);
                });
            });
        });
    }

    #[spec]
    fn connecting_bones() {
        given("a part that has finished loading", || {
            let mut app = App::new();
            let rig = app.world_mut().spawn_empty().id();
            let part = app.world_mut().spawn(ChildOf(rig)).id();
            let wrapper = named(&mut app, "Scene", part);
            let armature = named(&mut app, "Armature", wrapper);
            let root = named(&mut app, "root", armature);
            let left = named(&mut app, "thigh_l", root);
            let right = named(&mut app, "thigh_r", root);
            let unnamed = app.world_mut().spawn(ChildOf(armature)).id();

            when("it's ready", || {
                finish_loading(&mut app, part);

                test("bones are found by their path from the glTF root", || {
                    assert_eq!(target_of(&app, armature), id_for(&["Armature"]));
                    let path = id_for(&["Armature", "root", "thigh_l"]);
                    assert_eq!(target_of(&app, left), path);
                });

                test("sibling bones keep separate paths", || {
                    let path = id_for(&["Armature", "root", "thigh_r"]);
                    assert_eq!(target_of(&app, right), path);
                });

                test("the wrapper isn't treated as a bone", || {
                    assert_eq!(target_of(&app, wrapper), None);
                });

                test("nodes without names are skipped", || {
                    assert_eq!(target_of(&app, unnamed), None);
                });

                test("bones answer to the rig's animation player", || {
                    assert_eq!(app.world().get::<AnimatedBy>(left).unwrap().0, rig);
                });
            });
        });

        given("a part with no rig above it", || {
            let mut app = App::new();
            let part = app.world_mut().spawn_empty().id();
            let wrapper = named(&mut app, "Scene", part);
            let armature = named(&mut app, "Armature", wrapper);

            when("it's ready", || {
                finish_loading(&mut app, part);

                test("nothing is connected", || {
                    assert_eq!(target_of(&app, armature), None);
                });
            });
        });
    }
}
