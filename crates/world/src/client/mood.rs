//! The light and air of the world. Any camera that appears takes on this mood.

use bevy::anti_alias::smaa::Smaa;
use bevy::camera::{Exposure, Hdr};
use bevy::light::atmosphere::ScatteringMedium;
use bevy::light::{Atmosphere, AtmosphereEnvironmentMapLight, CascadeShadowConfigBuilder, SunDisk};
use bevy::pbr::{AtmosphereSettings, ScreenSpaceAmbientOcclusion};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::view::{ColorGrading, ColorGradingGlobal};

/// The first shadow cascade stays this close to the camera, to keep nearby shadows sharp.
const NEAREST_SHADOW_CASCADE: f32 = 10.0;
/// How finely the atmosphere's scattering is stored, in samples along each axis.
const ATMOSPHERE_RESOLUTION: u32 = 256;

/// How the world feels. These are art direction, so the game supplies them.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct Mood {
    pub sun_color: Color,
    /// Sunlight strength in lux, before the atmosphere filters it.
    pub sun_illuminance: f32,
    /// A point the sun shines from, toward the world origin. The sky's color follows it.
    pub sun_position: Vec3,
    /// How much light the camera needs, where a higher value makes the image darker.
    pub exposure_ev100: f32,
    /// How strongly the sky lights the world, where 1 matches the visible sky.
    pub sky_light: f32,
    pub fog_color: Color,
    /// The distance fog starts at.
    pub fog_start: f32,
    /// The distance fog hides everything by.
    pub fog_end: f32,
    /// How far shadows reach from the camera.
    pub shadow_distance: f32,
    /// How colorful the image is, where 1 leaves it unchanged.
    pub saturation: f32,
    /// How much bright areas glow.
    pub glow: f32,
}

pub(super) struct MoodPlugin {
    pub mood: Mood,
}

impl Plugin for MoodPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.mood);
        // The sky lights the world now, so flat ambient light would only wash it out.
        app.insert_resource(GlobalAmbientLight::NONE);
        app.add_systems(Startup, (spawn_sky, spawn_sun));
        app.add_observer(grade_camera);
    }
}

fn spawn_sky(mut commands: Commands, mut media: ResMut<Assets<ScatteringMedium>>) {
    let medium = ScatteringMedium::earth(ATMOSPHERE_RESOLUTION, ATMOSPHERE_RESOLUTION);
    commands.spawn(Atmosphere::earth(media.add(medium)));
}

fn spawn_sun(mut commands: Commands, mood: Res<Mood>) {
    commands.spawn((
        DirectionalLight {
            color: mood.sun_color,
            illuminance: mood.sun_illuminance,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_translation(mood.sun_position).looking_at(Vec3::ZERO, Vec3::Y),
        SunDisk::EARTH,
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: NEAREST_SHADOW_CASCADE,
            maximum_distance: mood.shadow_distance,
            ..default()
        }
        .build(),
    ));
}

fn grade_camera(trigger: On<Add<Camera3d>>, mood: Res<Mood>, mut commands: Commands) {
    commands.entity(trigger.entity).insert((
        Hdr,
        Exposure {
            ev100: mood.exposure_ev100,
        },
        AtmosphereSettings::default(),
        AtmosphereEnvironmentMapLight {
            intensity: mood.sky_light,
            ..default()
        },
        DistanceFog {
            color: mood.fog_color,
            falloff: FogFalloff::Linear {
                start: mood.fog_start,
                end: mood.fog_end,
            },
            ..default()
        },
        ColorGrading {
            global: ColorGradingGlobal {
                post_saturation: mood.saturation,
                ..default()
            },
            ..default()
        },
        Bloom {
            intensity: mood.glow,
            ..Bloom::NATURAL
        },
        // Contact shadows can't run with multisampling, so edges smooth another way.
        Msaa::Off,
        ScreenSpaceAmbientOcclusion::default(),
        Smaa::default(),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdd::spec;

    const MOOD: Mood = Mood {
        sun_color: Color::srgb(1.0, 0.8, 0.6),
        sun_illuminance: 130_000.0,
        sun_position: Vec3::new(30.0, 20.0, 10.0),
        exposure_ev100: 13.0,
        sky_light: 1.5,
        fog_color: Color::srgb(0.2, 0.3, 0.3),
        fog_start: 10.0,
        fog_end: 80.0,
        shadow_distance: 60.0,
        saturation: 0.8,
        glow: 0.1,
    };

    fn mood_app() -> App {
        let mut app = App::new();
        app.add_plugins((TaskPoolPlugin::default(), AssetPlugin::default()));
        app.init_asset::<ScatteringMedium>();
        app.add_plugins(MoodPlugin { mood: MOOD });
        app
    }

    #[spec]
    fn world_mood() {
        given("an app with the mood plugin", || {
            let mut app = mood_app();

            when("it starts", || {
                app.update();
                let mut suns = app.world_mut().query::<(&DirectionalLight, &Transform)>();
                let (sun, transform) = suns.single(app.world()).expect("there is one sun");

                test("the sun has the mood's color and strength", || {
                    assert_eq!(sun.color, MOOD.sun_color);
                    assert_eq!(sun.illuminance, MOOD.sun_illuminance);
                });

                test("the sun shines toward the world origin", || {
                    let forward = transform.forward().as_vec3();
                    let toward_origin = -MOOD.sun_position.normalize();
                    assert!(forward.distance(toward_origin) < 0.001);
                });

                test("the world has one atmosphere", || {
                    let mut skies = app.world_mut().query::<&Atmosphere>();
                    assert_eq!(skies.iter(app.world()).count(), 1);
                });

                test("flat ambient light is off", || {
                    let ambient = app.world().resource::<GlobalAmbientLight>();
                    assert_eq!(ambient.brightness, 0.0);
                });
            });

            when("a camera appears", || {
                let camera = app.world_mut().spawn(Camera3d::default()).id();
                app.update();

                test("it gets the mood's fog", || {
                    let fog = app.world().get::<DistanceFog>(camera).unwrap();
                    assert_eq!(fog.color, MOOD.fog_color);
                });

                test("it gets the mood's color grading", || {
                    let grading = app.world().get::<ColorGrading>(camera).unwrap();
                    assert_eq!(grading.global.post_saturation, MOOD.saturation);
                });

                test("it gets the mood's glow", || {
                    let bloom = app.world().get::<Bloom>(camera).unwrap();
                    assert_eq!(bloom.intensity, MOOD.glow);
                });

                test("it gets contact shadows, with multisampling off", || {
                    let contact_shadows = app.world().get::<ScreenSpaceAmbientOcclusion>(camera);
                    assert!(contact_shadows.is_some());
                    assert_eq!(app.world().get::<Msaa>(camera), Some(&Msaa::Off));
                });

                test("it sees the sky and is lit by it", || {
                    assert!(app.world().get::<AtmosphereSettings>(camera).is_some());
                    let sky = app.world().get::<AtmosphereEnvironmentMapLight>(camera);
                    assert_eq!(sky.unwrap().intensity, MOOD.sky_light);
                });

                test("it gets the mood's exposure", || {
                    let exposure = app.world().get::<Exposure>(camera).unwrap();
                    assert_eq!(exposure.ev100, MOOD.exposure_ev100);
                });
            });
        });
    }
}
