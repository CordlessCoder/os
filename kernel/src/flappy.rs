use alloc::collections::VecDeque;
use alloc::format;
use core::num::NonZeroU8;
use core::pin::pin;
use futures::{FutureExt, StreamExt};
use kernel::prelude::{vga_color::*, *};
use kernel::task::{keyboard::KeypressStream, timer::Interval};
use kernel::vga::{BUFFER_HEIGHT, BUFFER_WIDTH, ScreenChar};
use pc_keyboard::{KeyCode, KeyEvent, KeyState};
use rand::{Rng, SeedableRng};
use x86_64::instructions::random::RdRand;

const WIDTH: i16 = BUFFER_WIDTH as i16;
const HEIGHT: i16 = BUFFER_HEIGHT as i16;

fn seed() -> u64 {
    RdRand::new()
        .and_then(|r| r.get_u64())
        .unwrap_or(1238109843)
}

pub async fn run() {
    const GAP_WIDTH: usize = 10;
    const OBSTACLE_EVERY: usize = 30;
    const GRAVITY: f32 = 0.14;

    fn new_obstacle_height(rng: &mut impl Rng) -> NonZeroU8 {
        NonZeroU8::new(rng.random_range(2..(BUFFER_HEIGHT - GAP_WIDTH) as u8)).unwrap()
    }

    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed());
    let bg = ColorCode::new(Black, Blue);
    let border = ColorCode::new(Black, LightGray);
    let bird_color = ColorCode::new(Black, Green);
    let obstacle_color = ColorCode::new(Black, Red);

    let mut bird: f32 = rng.random_range(3.0..=11.0);
    let mut velocity: f32 = 0.;
    let mut obstacles: [Option<NonZeroU8>; BUFFER_WIDTH] = [None; BUFFER_WIDTH];
    let mut to_next_obstacle = 1;

    let mut keypresses = KeypressStream::new();
    let mut timer = Interval::new(40);
    let mut score = 1;
    'game: loop {
        let mut out = VGA_OUT.lock();
        out.buf.map_framebuffer(|_| {
            let mut buf = [[ScreenChar {
                ascii: b' ',
                color: bg,
            }; BUFFER_WIDTH]; BUFFER_HEIGHT];
            obstacles
                .iter()
                .enumerate()
                .flat_map(|(i, h)| h.map(|h| (i, h)))
                .map(|(x, height)| (x, height.get()))
                .for_each(|(x, height)| {
                    let write_obstacle = |y: usize| {
                        buf[y][x.saturating_sub(1)] = ScreenChar {
                            ascii: b' ',
                            color: obstacle_color,
                        };
                        buf[y][x] = ScreenChar {
                            ascii: b' ',
                            color: obstacle_color,
                        };
                    };
                    (0..height as usize)
                        .chain(height as usize + GAP_WIDTH..BUFFER_HEIGHT)
                        .for_each(write_obstacle);
                });
            // (0..BUFFER_HEIGHT).for_each(|y| {
            //     buf[y][0] = border;
            //     buf[y][BUFFER_WIDTH - 1] = border;
            // });
            buf[bird as usize][0] = ScreenChar {
                color: bird_color,
                ascii: b' ',
            };
            buf[bird as usize][1] = ScreenChar {
                color: bird_color,
                ascii: b' ',
            };
            let text = format!("Score: {score:<4}");
            buf[0]
                .iter_mut()
                .rev()
                .take(text.len())
                .rev()
                .zip(text.bytes())
                .for_each(|(out, b)| {
                    out.ascii = b;
                });
            buf
        });
        out.unlock();
        let mut timer = pin!(timer.tick().fuse());
        loop {
            futures::select_biased!(
                _ = timer => break,
                key = keypresses.next() => {
                    let Some((KeyEvent { code, state: KeyState::Down }, _)) = key else {
                        continue
                    };
                    match code {
                        KeyCode::Spacebar => {
                            velocity = -1.2;
                        }
                        KeyCode::Q => {
                            break 'game;
                        },
                        _ => continue
                    };
                },
            );
        }
        bird = (bird as f32 + velocity).clamp(0., HEIGHT as f32);
        if bird >= HEIGHT as f32 {
            break 'game;
        }
        if let Some(height) = obstacles.iter().take(3).flatten().next() {
            let height = height.get() as f32;
            let below_top = bird >= height;
            let above_bottom = bird <= height + GAP_WIDTH as f32;
            if !(below_top && above_bottom) {
                break 'game;
            }
        }
        if obstacles[0].is_some() {
            score += 1;
        }
        obstacles[0] = None;
        to_next_obstacle -= 1;
        if to_next_obstacle == 0 {
            obstacles[0] = Some(
                NonZeroU8::new(rng.random_range(1u8..(BUFFER_HEIGHT - GAP_WIDTH) as u8)).unwrap(),
            );
            to_next_obstacle = OBSTACLE_EVERY;
        }
        obstacles.rotate_left(1);
        velocity += GRAVITY;
    }
    let mut out = VGA_OUT.lock();
    out.color.set_bg(Black);
    out.fill_screen(b' ');
    out.unlock();
    println!(fgcolor = LightBlue, "Your score: {score}!");
    println!(fgcolor = White, "Press space to continue.");
    loop {
        if let Some((_, Some(pc_keyboard::DecodedKey::Unicode(' ')))) = keypresses.next().await {
            break;
        };
    }
}
