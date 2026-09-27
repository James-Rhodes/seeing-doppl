use core::f32;
use std::f32::consts::PI;

use macroquad::miniquad::conf::{Platform, WebGLVersion};
use macroquad::prelude::*;
use mqanim::{
    plot::{AxisStyle, Graph, GraphStyle},
    Animation,
};

const WINDOW_WIDTH: f32 = 640.0;
const WINDOW_HEIGHT: f32 = 360.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "Speed Camera Radar".to_owned(),
        sample_count: 4,
        window_width: WINDOW_WIDTH as i32,
        window_height: WINDOW_HEIGHT as i32,
        // WebGL1 has no multisampled render targets, which mqanim uses for
        // anti-aliasing. WebGL2 is supported by every browser that matters.
        platform: Platform {
            webgl_version: WebGLVersion::WebGL2,
            ..Default::default()
        },
        ..Default::default()
    }
}

const FRAME_TIME: f32 = 0.016;

struct Timer {
    start_time: f32,
    time_length: f32,
}
impl Timer {
    fn new(start_time: f32, time_length: f32) -> Self {
        Self {
            start_time,
            time_length,
        }
    }

    fn finished(&self, curr_time: f32) -> bool {
        curr_time - self.start_time > self.time_length
    }

    fn reset(&mut self, curr_time: f32) {
        self.start_time = curr_time;
    }
}

const NUM_SOUNDS: usize = 500;
const INIT_CAR_POS: Vec2 = Vec2::new(-350.0, -130.0);
struct Car {
    pos: Vec2,
    size: Vec2,
    speed: f32,
    texture: Texture2D,
}

impl Car {
    fn new(pos: Vec2, size: Vec2, speed: f32) -> Self {
        let car_texture: Texture2D = Texture2D::from_file_with_format(
            include_bytes!("../assets/car.png"),
            Some(ImageFormat::Png),
        );
        Self {
            pos,
            size,
            speed,
            texture: car_texture,
        }
    }
    fn update(&mut self) {
        self.pos += FRAME_TIME * Vec2::new(self.speed, 0.0);
    }

    fn draw(&self) {
        draw_texture_ex(
            &self.texture,
            self.pos.x - self.size.x / 2.0,
            self.pos.y - self.size.y / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(self.size),
                source: None,
                rotation: 0.,
                flip_x: false,
                flip_y: true,
                pivot: None,
            },
        );
    }
}

const SPEED_CAMERA_POS: Vec2 = Vec2::new(250.0, -130.0);
const SPEED_CAMERA_SIZE: Vec2 = Vec2::new(50.0, 60.0);

struct SpeedCamera {
    pos: Vec2,
    size: Vec2,
    plot_vals: Vec<Vec2>,
    texture: Texture2D,

    // Detecting stuff
    first_detect_time: Option<f32>,

    // Emitting stuff
    timer: Timer,
    num_sounds_to_emit: usize,
    emitted_values: Vec<Vec2>,
    first_emit_time: Option<f32>,
}

impl SpeedCamera {
    fn new(pos: Vec2, size: Vec2, num_sounds_to_emit: usize, sound_rate: f32) -> Self {
        let cam_texture: Texture2D = Texture2D::from_file_with_format(
            include_bytes!("../assets/speed_camera.png"),
            Some(ImageFormat::Png),
        );
        SpeedCamera {
            pos,
            size,
            plot_vals: vec![],
            first_detect_time: None,
            texture: cam_texture,
            timer: Timer::new(0., 1.0 / sound_rate),
            num_sounds_to_emit,
            emitted_values: vec![],
            first_emit_time: None,
        }
    }

    fn update(&mut self, time: f32) -> Option<Sound> {
        if !self.timer.finished(time) || self.num_sounds_to_emit == self.emitted_values.len() {
            return None;
        }

        self.timer.reset(time);
        let sound_value = compute_sound_value(time, 0.0);
        let first_time = self.first_emit_time.get_or_insert(time);
        self.emitted_values
            .push(Vec2::new(time - *first_time, sound_value));
        Some(Sound::new(
            self.pos,
            Vec2::new(-1000.0, INIT_CAR_POS.y),
            sound_value,
        )) // Just shoot to the left.
    }

    fn draw(&self) {
        draw_texture_ex(
            &self.texture,
            self.pos.x - self.size.x / 2.0,
            self.pos.y - self.size.y / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(self.size),
                source: None,
                rotation: 0.,
                flip_x: true,
                flip_y: true,
                pivot: None,
            },
        );
    }
}

struct Sound {
    prev_pos: Vec2,
    pos: Vec2,
    vel: Vec2,
    value: f32, // Must be in range -1, 1
    radius: f32,
    reflected: bool,
    total_distance: f32,
}

impl Sound {
    fn new(pos: Vec2, target: Vec2, value: f32) -> Self {
        const SOUND_VEL: f32 = 350.0;
        let dir = (target - pos).normalize();
        Self {
            prev_pos: pos,
            pos,
            vel: dir * SOUND_VEL,
            value,
            radius: 5.0,
            reflected: false,
            total_distance: 0.0,
        }
    }
    fn update(&mut self, time: f32) {
        self.total_distance += f32::abs(self.prev_pos.x - self.pos.x);

        self.prev_pos = self.pos;
        self.pos = self.pos + self.vel * FRAME_TIME;

        self.value = compute_sound_value(time, self.total_distance);
    }
    fn draw(&self) {
        // Lerp from red to green based on value
        let color_vec = Vec3::new(1.0, 0., 0.).lerp(Vec3::new(0., 1., 0.0), 0.5 * self.value + 0.5);
        let color = Color::new(color_vec.x, color_vec.y, color_vec.z, 1.0);
        draw_circle(self.pos.x, self.pos.y, self.radius, color);
    }
}

fn compute_sound_value(t: f32, x: f32) -> f32 {
    const K: f32 = 0.03;
    const SOUND_FREQ: f32 = 0.4;
    f32::sin(K * x - 2.0 * PI * SOUND_FREQ * t)
}

struct State {
    speed_camera: SpeedCamera,
    car: Car,
    sounds: Vec<Sound>,
    time: f32,
}

impl State {
    fn new() -> Self {
        let cam = SpeedCamera::new(SPEED_CAMERA_POS, SPEED_CAMERA_SIZE, NUM_SOUNDS, 200.0);

        let car = Car::new(INIT_CAR_POS, Vec2::new(150., 75.), 50.0);

        let sounds = vec![];
        Self {
            speed_camera: cam,
            car,
            sounds,
            time: 0.0,
        }
    }

    fn update(&mut self) {
        self.time += FRAME_TIME;
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut animation = Animation::new(WINDOW_WIDTH, WINDOW_HEIGHT, None);
    animation.enable_fxaa();

    let mut state = State::new();

    let mut loop_timer = Timer::new(state.time, 15.0);

    let graph_size = Vec2::new(200., 200.);
    let rx_graph_pos = Vec2::new(150., 50.);
    let graph_rx =
        Graph::new(rx_graph_pos, graph_size, 0.0..8.0 as f32, -1.1..1.1 as f32).style(GraphStyle {
            y_style: AxisStyle {
                end_point_style: mqanim::plot::GraphEndPointStyle::Nothing,
                ..Default::default()
            },
            ..Default::default()
        });

    let tx_graph_pos = Vec2::new(-150., 50.);
    let graph_tx =
        Graph::new(tx_graph_pos, graph_size, 0.0..8.0 as f32, -1.0..1.0 as f32).style(GraphStyle {
            y_style: AxisStyle {
                end_point_style: mqanim::plot::GraphEndPointStyle::Nothing,
                ..Default::default()
            },
            ..Default::default()
        });

    loop {
        state.update();
        let sound = state.speed_camera.update(state.time);
        if let Some(sound) = sound {
            state.sounds.push(sound);
        }

        for sound in state.sounds.iter_mut() {
            sound.update(state.time);
        }

        state.sounds.retain(|sound| {
            // If the sound bounces off then comes passed the speed cam again. Then remove it.
            if (sound.pos.x > state.speed_camera.pos.x) && sound.reflected {
                let first_time = state
                    .speed_camera
                    .first_detect_time
                    .get_or_insert(state.time);
                state
                    .speed_camera
                    .plot_vals
                    .push(Vec2::new(state.time - *first_time as f32, sound.value));
                return false;
            }
            true
        });

        state.sounds.iter_mut().for_each(|sound| {
            let car_edge = state.car.pos.x + (state.car.size.x / 2.0);
            if (sound.pos.x <= car_edge) && !sound.reflected {
                sound.vel.x *= -1.0; // Bounce off the car
                sound.pos.y -= 10.0;
                sound.pos.x = car_edge;
                sound.reflected = true;
            }
        });

        state.car.update();

        animation.set_camera();

        state.speed_camera.draw();
        for sound in state.sounds.iter() {
            sound.draw();
        }

        state.car.draw();

        let graph_indicator_radius = 10.;
        draw_circle(
            rx_graph_pos.x - graph_size.x / 2. - graph_indicator_radius - 5.,
            rx_graph_pos.y + graph_size.y / 2.,
            graph_indicator_radius,
            GREEN,
        );
        draw_circle(
            rx_graph_pos.x - graph_size.x / 2. - graph_indicator_radius - 5.,
            rx_graph_pos.y - graph_size.y / 2.,
            graph_indicator_radius,
            RED,
        );
        graph_rx.draw_axes();
        graph_rx.plot_line_vec(&state.speed_camera.plot_vals, 5.0, ORANGE);

        draw_circle(
            tx_graph_pos.x - graph_size.x / 2. - graph_indicator_radius - 5.,
            tx_graph_pos.y + graph_size.y / 2.,
            graph_indicator_radius,
            GREEN,
        );
        draw_circle(
            tx_graph_pos.x - graph_size.x / 2. - graph_indicator_radius - 5.,
            tx_graph_pos.y - graph_size.y / 2.,
            graph_indicator_radius,
            RED,
        );
        graph_tx.draw_axes();
        graph_tx.plot_line_vec(&state.speed_camera.emitted_values, 5.0, ORANGE);

        animation.set_default_camera();
        animation.draw_frame();

        if is_key_pressed(KeyCode::R) || loop_timer.finished(state.time) {
            loop_timer.reset(state.time);
            state = State::new();
        }

        next_frame().await;
    }
}
