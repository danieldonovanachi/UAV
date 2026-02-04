use bevy::camera::RenderTarget;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task};
use bevy::{math::VectorSpace, prelude::*};

mod model;

mod physical_link;

#[derive(Resource)]
struct PhysicalLink {
    channel: std::sync::mpsc::Sender<()>,
}

#[derive(Component)]
struct PointSize(f32);

fn apply_velocity(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity), With<Drone>>) {
    let delta = time.delta().as_millis() as f32;
    for (mut position, velocity) in query.iter_mut() {
        position.translation += (velocity.0 * delta).extend(0.0);
        //println!("position: {position:?}, velocity: {velocity:?}");
    }
}

#[derive(Component)]
struct Drone {
    mission: Option<Entity>,
    next: usize,
}

fn control_drone(
    mut q_drone: Query<(&Drone, &Transform, &mut Velocity)>,
    q_mission: Query<(&Mission, &Children)>,
    q_mission_points: Query<(&Transform, &PointSize)>,
) {
    for (drone, position, mut velocity) in q_drone.iter_mut() {
        //if drone.mission.is_none() {
        //    continue;
        //}

        //let mission_entity = drone.mission.unwrap();
        //println!("Drone on mission: {mission_entity:?}");
        //let mission = q_mission.get(mission);

        let mission_opt = q_mission.iter().next();
        if mission_opt.is_none() {
            continue;
        }

        let (mission, children) = mission_opt.unwrap();
        //println!("mission children: {children:?}");

        // From that mission, we get the next point
        let point_id = drone.next;
        let target = children[point_id];

        let speed = 0.03f32;
        let (target_transform, point_size) = q_mission_points.get(target).unwrap();

        let diff = (target_transform.translation - position.translation).truncate();
        let length = diff.length();
        let normalized = diff / length;
        *velocity = Velocity(normalized * speed);
    }
}

fn drone_mission_control(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    materials: Res<Assets<ColorMaterial>>,
    link: ResMut<PhysicalLink>,
    // q_camera: Query<&bevy::camera::Camera>,
    mut q_drone: Query<(&mut Drone, &Transform, &Velocity)>,
    q_mission: Query<(&Mission, &Children)>,
    mut q_mission_points: Query<(&Transform, &PointSize, &mut MeshMaterial2d<ColorMaterial>)>,
) {
    for (mut drone, position, velocity) in q_drone.iter_mut() {
        //if drone.mission.is_none() {
        //    continue;
        //}

        //let mission_entity = drone.mission.unwrap();
        //println!("Drone on mission: {mission_entity:?}");
        //let mission = q_mission.get(mission);

        let mission_opt = q_mission.iter().next();
        if mission_opt.is_none() {
            continue;
        }

        let (mission, children) = mission_opt.unwrap();
        //println!("mission children: {children:?}");

        // From that mission, we get the next point
        let point_id = drone.next;
        let target = children[point_id];

        let radius = 2.0f32;
        let (target_transform, point_size, mut material) =
            q_mission_points.get_mut(target).unwrap();

        let diff = (target_transform.translation - position.translation).truncate();
        //println!("diff: {diff:?}, children count: {}", children.len());

        // We activate that dot
        if diff.length() <= radius {
            material.0 = mission.mat_reached.clone();
            drone.next = (drone.next + 1) % children.len();

            {
                commands.spawn((
                    AudioPlayer::new(asset_server.load("beep.ogg")),
                    PlaybackSettings::REMOVE,
                ));

                link.channel.send(());
            }

            if drone.next == 0 {
                drone.mission = None;
            }
        }
    }
}

#[derive(Component, Clone, Debug)]
struct Velocity(pub Vec2);

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }
}

#[derive(Component, Clone, Debug)]
struct Mission {
    mat_unreached: Handle<ColorMaterial>,
    mat_reached: Handle<ColorMaterial>,
}

const X_EXTENT: f32 = 900.;
fn spawn_base(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // This will control the view
    let camera_entity = commands
        .spawn((
            Camera2d,
            Camera {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..Default::default()
            },
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: bevy::camera::ScalingMode::WindowSize,
                scale: 1.,
                ..OrthographicProjection::default_2d()
            }),
        ))
        .id();

    // This is camera (or viewport) positioned, it's the crosshair
    commands.spawn((
        CameraPositionned {
            camera: camera_entity,
            pos: Vec2::new(0.0, 0.0),
            z: 1.0,
        },
        Transform::default(),
        WindowLocated { active: true },
        children![
            // Y Axis
            (
                Mesh2d(meshes.add(Segment2d::new(
                    Vec2::new(0., -10000.),
                    Vec2::new(0., 10000.)
                ))),
                MeshMaterial2d(materials.add(Color::WHITE)),
            ),
            // X Axis
            (
                Mesh2d(meshes.add(Segment2d::new(
                    Vec2::new(-10000., 0.),
                    Vec2::new(10000., 0.)
                ))),
                MeshMaterial2d(materials.add(Color::WHITE)),
            )
        ],
    ));

    // The drone
    commands.spawn((
        Drone {
            mission: None,
            next: 0,
        },
        Mesh2d(meshes.add(Circle::new(10. / 2.))),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Velocity::new(0.00, 0.00),
        Transform::from_xyz(0.0, 0.0, 0.1),
        children![
            // Y Axis
            (
                Mesh2d(meshes.add(Segment2d::new(
                    Vec2::new(0., -10000.),
                    Vec2::new(0., 10000.)
                ))),
                MeshMaterial2d(materials.add(Color::WHITE)),
            ),
            // X Axis
            (
                Mesh2d(meshes.add(Segment2d::new(
                    Vec2::new(-10000., 0.),
                    Vec2::new(10000., 0.)
                ))),
                MeshMaterial2d(materials.add(Color::WHITE)),
            )
        ],
    ));
}

static TESTDATA_DIR_PATH: std::sync::LazyLock<std::path::PathBuf> =
    std::sync::LazyLock::new(|| {
        let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("testdata");
        path
    });

#[derive(Component)]
struct MissionPreparation(Task<bevy::ecs::world::CommandQueue>);

fn move_camera(
    mut camera: Single<(&Camera2d, &mut Transform)>,
    time: Res<Time>,
    kb_input: Res<ButtonInput<KeyCode>>,
) {
    let mut direction = Vec2::ZERO;

    if kb_input.pressed(KeyCode::KeyW) {
        direction.y += 1.;
    }

    if kb_input.pressed(KeyCode::KeyS) {
        direction.y -= 1.;
    }

    if kb_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.;
    }

    if kb_input.pressed(KeyCode::KeyD) {
        direction.x += 1.;
    }

    const MOVE_SPEED: f32 = 175.;
    let move_delta = direction.normalize_or_zero() * MOVE_SPEED * time.delta_secs();
    camera.1.translation += move_delta.extend(0.);
}

fn scroll_events(
    mut camera: Query<(&Camera, &mut Projection)>,
    mut evr_scroll: MessageReader<bevy::input::mouse::MouseWheel>,
    q_primary_window: Option<Single<Entity, (With<Window>, With<bevy::window::PrimaryWindow>)>>,
) {
    use bevy::input::mouse::MouseScrollUnit;

    for ev in evr_scroll.read() {
        if ev.y == 0.0 {
            continue;
        }

        let window = ev.window;
        for (camera, mut projection) in camera.iter_mut() {
            let camera_window = match camera.target {
                bevy::camera::RenderTarget::Window(camera_window) => camera_window,
                _ => continue,
            };

            let camera_window_entity = match (camera_window, &q_primary_window) {
                (bevy::window::WindowRef::Entity(e), _) => e,
                (bevy::window::WindowRef::Primary, Some(e)) => **e,
                _ => continue,
            };

            if camera_window_entity != window {
                continue;
            };

            match *projection {
                Projection::Orthographic(OrthographicProjection { ref mut scale, .. }) => {
                    *scale += ev.y
                }
                _ => continue,
            }
        }
    }
}

fn spawn_mission(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    let col = meshes.add(Circle::new(1.0));
    let mat_unreached = materials.add(Color::linear_rgb(0.005, 0.0, 0.0));
    let mat_reached = materials.add(Color::linear_rgb(0.0, 0.15, 0.0));
    // Spawn new task on the AsyncComputeTaskPool; the task will be
    // executed in the background, and the Task future returned by
    // spawn() can be used to poll for the result
    let entity = commands.spawn_empty().id();
    let task = thread_pool.spawn(async move {
        use rand::Rng;
        let duration = std::time::Duration::from_secs_f32(rand::rng().random_range(0.05..5.0));

        let mut command_queue = bevy::ecs::world::CommandQueue::default();

        const MAIN_IMAGE_FILENAME: &'static str = "Surveillance.png";

        let image_path = TESTDATA_DIR_PATH.join(MAIN_IMAGE_FILENAME);

        const SUPPORTED_FORMATS: &'static [&'static str] = &[
            "png", "jpg", "jpeg", "avif", "bmp", "dds", "gif", "exr", "ff", "hdr", "ico", "pnm",
            "qoi", "tga", "tiff", "webp",
        ];

        let files = rfd::AsyncFileDialog::new()
            .add_filter("Supported Images", SUPPORTED_FORMATS)
            .set_directory(TESTDATA_DIR_PATH.as_os_str())
            .pick_file()
            .await;

        let file_path = files.unwrap();

        let reader = image::ImageReader::open(file_path.path())
            .expect(&format!("image loading failed: {:?}", &image_path));
        let decoded = reader.decode().expect("image decoding failed");

        // Now, we grayscale it
        let grayscale = decoded.to_luma32f();
        let image = plot_planner::generation::hdp_common::memory::Cube::from_image(grayscale);

        const PPU: f32 = 0.3;
        let image_in_world = plot_planner::ImageWorldPlacement::from_image(
            &grayscale,
            nalgebra::Point2::default(),
            PPU,
        );
        let size = image_in_world.size();
        let grid = plot_planner::generation::ScreeningGrid {
            resolution: 16.0,
            point_size: 24.0,
            ..std::default::Default::default()
        };
        use rand_xoshiro::rand_core::SeedableRng;
        let mut rng = rand_xoshiro::Xoroshiro64Star::seed_from_u64(0);

        let mut path_recorder = plot_planner::generation::GenerationBufferVec::new();
        /*plot_planner::screening::screen_fm(&grayscale, &image_in_world, &grid, &mut rng, |p| {
            path_recorder.push_point(plot_planner::path::Point::new(p, grid.point_size))
        });*/
        use plot_planner::generation::Generator;
        let mut process =
            plot_planner::generation::FMScreeningGenerator::start(&image_in_world, (grid, rng));

        use plot_planner::generation::GenerationProcess;
        let image_ref = image.as_ref();
        process.generate(&image_ref, &mut path_recorder, 10);

        // OPTIMIZE

        let characteristics = plot_planner::optimization::SpecificEnergyCost {
            up: 1.5,
            down: 0.1,
            forward: 1.0,
        };
        let optimizer = plot_planner::optimization::OptimizationSettings {
            specific_energy: characteristics,
            start: nalgebra::Point2::new(0., 0.0),
            include_start: true,
        };

        let order = optimizer.optimize_points(&path_recorder.points);

        // we use a raw command queue to pass a FnOnce(&mut World) back to be
        // applied in a deferred manner.
        command_queue.push(move |world: &mut World| {
            world
                .entity_mut(entity)
                // Add our new `Mesh3d` and `MeshMaterial3d` to our tagged entity
                .insert((
                    Mission {
                        mat_unreached: mat_unreached.clone(),
                        mat_reached,
                    },
                    Transform::from_xyz(0.0, 0.0, 0.0),
                    Children::spawn(SpawnWith(
                        move |parent: &mut bevy::ecs::relationship::RelatedSpawner<ChildOf>| {
                            for item in order {
                                let point = path_recorder.points[item];
                                let pos = point.position;

                                parent.spawn((
                                    Mesh2d(col.clone()),
                                    MeshMaterial2d(mat_unreached.clone()),
                                    // Bevy's 2D coordinates are x right, y up, rhs
                                    Transform::from_xyz(pos.x, size.y - pos.y, 0.0).with_scale(
                                        Vec3::new(
                                            point.size / 2.,
                                            point.size / 2.,
                                            point.size / 2.,
                                        ),
                                    ),
                                    PointSize(point.size),
                                ));
                            }
                        },
                    )),
                ));
        });

        command_queue
    });

    // Add our new task as a component
    commands.entity(entity).insert(MissionPreparation(task));
}

/// This system queries for entities that have our Task<Transform> component. It polls the
/// tasks to see if they're complete. If the task is complete it takes the result, adds a
/// new [`Mesh3d`] and [`MeshMaterial3d`] to the entity using the result from the task's work, and
/// removes the task component from the entity.
fn handle_tasks(
    mut commands: Commands,
    mut transform_tasks: Query<(Entity, &mut MissionPreparation)>,
) {
    for (entity, mut task) in &mut transform_tasks {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            // append the returned command queue to have it execute later
            commands.append(&mut commands_queue);
            // Task is complete, so remove task component from entity
            commands.entity(entity).remove::<MissionPreparation>();
        }
    }
}

#[derive(Component, Clone, Debug)]
struct CameraPositionned {
    camera: Entity,
    pos: Vec2,
    z: f32,
}

#[derive(Component, Clone, Debug)]
struct WindowLocated {
    active: bool,
}

/// Positionnes a camera-positionned cursor
fn cursor_position(
    q_window: Query<&Window>,
    q_primary_window: Option<Single<&Window, With<bevy::window::PrimaryWindow>>>,
    q_cursor: Query<(&mut WindowLocated, &mut CameraPositionned)>,
    q_camera: Query<&bevy::camera::Camera>,
) {
    for (_window_located, mut camera_positionned) in q_cursor {
        let camera = q_camera.get(camera_positionned.camera).unwrap();
        let camera_window = match camera.target {
            bevy::camera::RenderTarget::Window(camera_window) => camera_window,
            _ => continue,
        };

        let window = match camera_window {
            bevy::window::WindowRef::Primary if q_primary_window.is_some() => {
                let unwrapped: &Single<'_, '_, &Window, With<bevy::window::PrimaryWindow>> =
                    q_primary_window.as_ref().unwrap();
                unwrapped
            }
            bevy::window::WindowRef::Entity(entity) => {
                let unwrapped = q_window.get(entity).unwrap();
                unwrapped
            }
            _ => continue,
        };

        let pos = match window.cursor_position() {
            Some(pos) => pos,
            None => continue,
        };

        camera_positionned.pos = pos;
    }
}

/// Applies a camera-position to an element with a transform
fn apply_camera_positionned(
    mut q_wp: Query<(&CameraPositionned, &mut Transform)>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    q_window: Query<&Window>,
) {
    for (pos, mut transform) in q_wp.iter_mut() {
        let (camera, camera_transform) = q_camera.get(pos.camera).unwrap();
        let world_pos = camera.viewport_to_world_2d(camera_transform, pos.pos);
        match world_pos {
            Ok(world_pos) => transform.translation = world_pos.extend(pos.z),
            Err(_) => continue,
        }
    }
}

fn main() {
    let (send, recv) = std::sync::mpsc::channel();

    let _link_thread = std::thread::spawn(move || {
        let mut link = physical_link::Link::new().unwrap();

        loop {
            let r = recv.recv();
            let _msg = match r {
                Err(e) => {
                    println!("{:?}", e);
                    link.signal(false);
                    return;
                }
                Ok(msg) => msg,
            };

            println!("STARTING...");
            link.signal(true);
            std::thread::sleep(physical_link::PRESS_DURATION);
            println!("STOPPING");
            link.signal(false);
        }
    });

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "UAS Simulation".into(),
                name: Some("uas.sim".into()),
                present_mode: bevy::window::PresentMode::AutoVsync,
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                window_theme: Some(bevy::window::WindowTheme::Dark),
                mode: bevy::window::WindowMode::Windowed, /*BorderlessFullscreen(
                                                              bevy::window::MonitorSelection::Current,
                                                          )*/
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(PhysicalLink { channel: send })
        .add_systems(Startup, (spawn_base, spawn_mission))
        .add_systems(
            Update,
            (
                scroll_events,
                move_camera,
                handle_tasks,
                (control_drone, apply_velocity, drone_mission_control).chain(),
                (cursor_position, apply_camera_positionned).chain(),
            ),
        )
        .run();
}
