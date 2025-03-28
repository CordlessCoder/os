use alloc::collections::VecDeque;
use core::pin::pin;
use futures::{FutureExt, StreamExt};
use kernel::prelude::{vga_color::*, *};
use kernel::task::{keyboard::KeypressStream, timer::Interval};
use kernel::vga::{BUFFER_HEIGHT, BUFFER_WIDTH, ScreenChar};
use pc_keyboard::{KeyCode, KeyEvent, KeyState};
use rand::{Rng, SeedableRng};
use x86_64::instructions::random::RdRand;

const WIDTH: i16 = BUFFER_WIDTH as i16 / 2;
const HEIGHT: i16 = BUFFER_HEIGHT as i16;

fn seed() -> u64 {
    RdRand::new()
        .and_then(|r| r.get_u64())
        .unwrap_or(1238109843)
}

fn random_position(mut rng: impl Rng) -> (i16, i16) {
    (rng.random_range(0..=WIDTH), rng.random_range(0..=HEIGHT))
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub const fn x_offset(&self) -> i16 {
        use Direction::*;
        match self {
            Right => 1,
            Left => -1,
            _ => 0,
        }
    }
    pub const fn y_offset(&self) -> i16 {
        use Direction::*;
        match self {
            Up => -1,
            Down => 1,
            _ => 0,
        }
    }
}

pub async fn run() {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed());
    let bg = ColorCode::new(Black, Black);
    let border = ColorCode::new(Black, LightGray);
    let snake_color = ColorCode::new(LightBlue, Blue);
    let apple_color = ColorCode::new(Black, Green);

    let mut keypresses = KeypressStream::new();
    let mut timer = Interval::new(100);
    let mut body: VecDeque<(i16, i16)> = VecDeque::from_iter([(0, 0)]);
    let mut apple = random_position(&mut rng);
    let mut direction = Direction::Down;
    let mut score = 1;
    'game: loop {
        let head = body[0];
        let head = (
            (WIDTH + head.0 + direction.x_offset()) % WIDTH,
            (HEIGHT + head.1 + direction.y_offset()) % HEIGHT,
        );
        if body.contains(&head) {
            break;
        }
        if head == apple {
            score += 1;
            apple = random_position(&mut rng);
        } else {
            body.pop_back();
        }
        body.push_front(head);
        let mut out = VGA_OUT.lock();
        out.buf.map_framebuffer(|_| {
            let mut buf = [[ScreenChar {
                ascii: b' ',
                color: bg,
            }; BUFFER_WIDTH]; BUFFER_HEIGHT];
            let border = ScreenChar {
                ascii: b' ',
                color: border,
            };
            buf[0] = [border; BUFFER_WIDTH];
            buf[BUFFER_HEIGHT - 1] = [border; BUFFER_WIDTH];
            for y in 1..BUFFER_HEIGHT - 1 {
                buf[y][0] = border;
                buf[y][BUFFER_WIDTH - 1] = border;
            }
            let mut paint_cell = |x: usize, y: usize, color, text: &[u8; 2]| {
                buf[y][x * 2] = ScreenChar {
                    ascii: text[0],
                    color,
                };
                buf[y][x * 2 + 1] = ScreenChar {
                    ascii: text[1],
                    color,
                };
            };
            paint_cell(apple.0 as usize, apple.1 as usize, apple_color, b"  ");
            for (x, y) in body.iter().skip(1) {
                paint_cell(*x as usize, *y as usize, snake_color, b"  ");
            }
            paint_cell(head.0 as usize, head.1 as usize, snake_color, b"()");

            buf
        });
        out.unlock();
        let mut timer = pin!(timer.tick().fuse());
        let old_dir = direction;
        loop {
            futures::select_biased!(
                _ = timer => break,
                key = keypresses.next() => {
                    let Some((KeyEvent { code, state: KeyState::Down }, _)) = key else {
                        continue
                    };
                    direction = match code {
                        KeyCode::ArrowRight | KeyCode::D if old_dir != Direction::Left => Direction::Right,
                        KeyCode::ArrowLeft | KeyCode::A if old_dir != Direction::Right => Direction::Left,
                        KeyCode::ArrowUp | KeyCode::W if old_dir != Direction::Down => Direction::Up,
                        KeyCode::ArrowDown | KeyCode::S if old_dir != Direction::Up => Direction::Down,
                        KeyCode::Q => {
                            break 'game;
                        },
                        _ => continue
                    };
                },
            );
        }
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
