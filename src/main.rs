use anyhow::{bail, Context, Result};
use chrono::{Datelike, Local, NaiveDateTime, TimeZone, Timelike};
use clap::{Args, Parser, Subcommand};
use image::{imageops::FilterType, AnimationDecoder, ImageFormat, RgbaImage};
use pixoo::canvas::{parse_color, Canvas, Rgb, SIZE};
use pixoo::device::{Channel, ClockOptions, DryRun, Pixoo, Transport};
use pixoo::protocol::{self, hex};
use pixoo::rfcomm::RfcommStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(version, about = "Control a Divoom Pixoo (16x16) over Bluetooth")]
struct Cli {
    /// Bluetooth address of the Pixoo, e.g. 11:22:33:44:55:66
    #[arg(short, long, env = "PIXOO_MAC", global = true)]
    device: Option<String>,

    /// RFCOMM channel (usually 1)
    #[arg(short, long, env = "PIXOO_CHANNEL", default_value_t = 1, global = true)]
    channel: u8,

    /// Do not connect, just print the bytes that would be sent
    #[arg(long, global = true)]
    dry_run: bool,

    /// Print all bytes sent and received
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Cmd,
}

#[derive(Args)]
struct StateArg {
    /// Keep the canvas in this file between invocations (768 raw RGB bytes)
    #[arg(long, env = "PIXOO_STATE")]
    state: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Set individual pixels: -p X,Y,COLOR (repeatable)
    Pixel {
        /// Pixel as X,Y,COLOR with X/Y in 0..15 and COLOR as #rrggbb or a name
        #[arg(short, long = "pixel", required = true, value_parser = parse_pixel)]
        pixels: Vec<(usize, usize, Rgb)>,
        /// Start from a canvas filled with this color (ignored if --state has content)
        #[arg(short, long, value_parser = parse_color)]
        background: Option<Rgb>,
        #[command(flatten)]
        state: StateArg,
    },
    /// Fill the whole display with one color
    Fill {
        #[arg(value_parser = parse_color)]
        color: Rgb,
        #[command(flatten)]
        state: StateArg,
    },
    /// Show the canvas stored in --state again, or a black one
    Show {
        #[command(flatten)]
        state: StateArg,
    },
    /// Show a PNG/JPEG/GIF (animated GIFs are played), scaled to 16x16
    Image {
        file: PathBuf,
        /// Override the frame time in milliseconds
        #[arg(long)]
        time: Option<u16>,
    },
    /// Set display brightness (0-100)
    Brightness { percent: u8 },
    /// Show the clock channel
    Clock {
        /// Clock face 0-15 (0 full screen, 1 rainbow, 2 boxed, 3 analog square, 4 negative, 5 analog round, 6 wide)
        #[arg(short, long, default_value_t = 0)]
        style: u8,
        #[arg(long)]
        h24: bool,
        #[arg(long)]
        weather: bool,
        #[arg(long)]
        temp: bool,
        #[arg(long)]
        calendar: bool,
        #[arg(long, value_parser = parse_color)]
        color: Option<Rgb>,
    },
    /// Show the light channel
    Light {
        #[arg(value_parser = parse_color)]
        color: Rgb,
        #[arg(short, long, default_value_t = 100)]
        brightness: u8,
        /// 0 = fixed color, 1-4 = animations
        #[arg(short, long, default_value_t = 0)]
        effect: u8,
    },
    /// Switch the display on (light channel, white, 100%)
    On,
    /// Switch the display off
    Off,
    /// Show a channel: clock, light, cloud, effects, visualization, design, scoreboard
    Channel { name: String },
    /// Effects channel, effect number
    Effect { number: u8 },
    /// Visualization channel, number
    Visualization { number: u8 },
    /// Design channel, optionally with a tab number
    Design { number: Option<u8> },
    /// Scoreboard (0-999 each)
    Scoreboard { red: u16, blue: u16 },
    /// Set the device clock (default: now, local time)
    SetTime {
        /// e.g. "2026-10-23T19:45:00"
        value: Option<String>,
    },
    /// Weather for the clock: temperature in °C and a weather code (1 clear, 3 cloudy, 5 thunderstorm, 6 rain, 8 snow, 9 fog)
    Weather {
        #[arg(allow_negative_numbers = true)]
        celsius: i8,
        #[arg(default_value_t = 1)]
        code: u8,
    },
    /// Timer tool: 0 stop, 1 start (2 is unverified)
    Timer { state: u8 },
    /// Countdown tool
    Countdown {
        minutes: u8,
        #[arg(default_value_t = 0)]
        seconds: u8,
        /// Only set the time, do not start
        #[arg(long)]
        no_start: bool,
    },
    /// Send a raw command: first byte is the command, rest are arguments (hex)
    Raw {
        #[arg(required = true, num_args = 1..)]
        hex: Vec<String>,
        /// Wait for and print replies
        #[arg(short, long)]
        read: bool,
    },
    /// Ask the device for its current view and print the reply
    GetView,
    /// Print everything the device sends for N seconds
    Listen {
        #[arg(default_value_t = 10)]
        seconds: u64,
    },
}

fn parse_pixel(s: &str) -> Result<(usize, usize, Rgb), String> {
    let mut it = s.splitn(3, ',');
    let (x, y, c) = match (it.next(), it.next(), it.next()) {
        (Some(x), Some(y), Some(c)) => (x, y, c),
        _ => return Err("expected X,Y,COLOR".into()),
    };
    let x: usize = x.trim().parse().map_err(|_| "X must be a number")?;
    let y: usize = y.trim().parse().map_err(|_| "Y must be a number")?;
    if x >= SIZE || y >= SIZE {
        return Err(format!("coordinates must be 0..{}", SIZE - 1));
    }
    Ok((x, y, parse_color(c.trim()).map_err(|e| e.to_string())?))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

fn open(cli: &Cli) -> Result<Pixoo<Box<dyn Transport>>> {
    let io: Box<dyn Transport> = if cli.dry_run {
        Box::new(DryRun::default())
    } else {
        let addr = cli
            .device
            .as_deref()
            .context("no device given: use --device AA:BB:CC:DD:EE:FF or set PIXOO_MAC")?;
        let s = RfcommStream::connect(addr, cli.channel, Duration::from_secs(10))?;
        // Replies are short; do not hang long when none arrive.
        s.set_read_timeout(Duration::from_millis(500))?;
        Box::new(s)
    };
    let mut p = Pixoo::new(io);
    p.verbose = cli.verbose || cli.dry_run;
    Ok(p)
}

fn load_state(state: &StateArg, background: Option<Rgb>) -> Result<Canvas> {
    if let Some(path) = &state.state {
        if path.exists() {
            let bytes =
                std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
            return Canvas::from_rgb_bytes(&bytes);
        }
    }
    Ok(background.map(Canvas::filled).unwrap_or_default())
}

fn save_state(state: &StateArg, canvas: &Canvas) -> Result<()> {
    if let Some(path) = &state.state {
        std::fs::write(path, canvas.to_rgb_bytes())
            .with_context(|| format!("writing {}", path.display()))?;
    }
    Ok(())
}

fn to_canvas(img: &RgbaImage) -> Canvas {
    let img = if img.width() as usize != SIZE || img.height() as usize != SIZE {
        image::imageops::resize(img, SIZE as u32, SIZE as u32, FilterType::Nearest)
    } else {
        img.clone()
    };
    let mut c = Canvas::default();
    for (x, y, px) in img.enumerate_pixels() {
        // Mostly transparent pixels become black, like the app does.
        let color = if px[3] > 32 {
            [px[0], px[1], px[2]]
        } else {
            [0, 0, 0]
        };
        c.set(x as usize, y as usize, color).expect("in range");
    }
    c
}

fn load_frames(path: &Path, time: Option<u16>) -> Result<Vec<(Canvas, u16)>> {
    let is_gif = image::ImageFormat::from_path(path).ok() == Some(ImageFormat::Gif);
    if is_gif {
        let file = std::io::BufReader::new(std::fs::File::open(path)?);
        let dec = image::codecs::gif::GifDecoder::new(file)?;
        let mut out = Vec::new();
        for frame in dec.into_frames() {
            let frame = frame?;
            let (n, d) = frame.delay().numer_denom_ms();
            let ms = (n / d.max(1)).min(u32::from(u16::MAX)) as u16;
            out.push((to_canvas(frame.buffer()), time.unwrap_or(ms)));
        }
        if out.is_empty() {
            bail!("GIF contains no frames");
        }
        Ok(out)
    } else {
        let img = image::open(path)?.to_rgba8();
        Ok(vec![(to_canvas(&img), time.unwrap_or(0))])
    }
}

fn parse_channel(name: &str) -> Result<Channel> {
    Ok(match name.to_ascii_lowercase().as_str() {
        "clock" | "0" => Channel::Clock,
        "light" | "1" => Channel::Light,
        "cloud" | "2" => Channel::Cloud,
        "effects" | "effect" | "3" => Channel::Effects,
        "visualization" | "4" => Channel::Visualization,
        "design" | "5" => Channel::Design,
        "scoreboard" | "6" => Channel::Scoreboard,
        _ => bail!("unknown channel '{name}'"),
    })
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let mut p = open(&cli)?;
    match &cli.command {
        Cmd::Pixel {
            pixels,
            background,
            state,
        } => {
            let mut canvas = load_state(state, *background)?;
            for &(x, y, c) in pixels {
                canvas.set(x, y, c)?;
            }
            p.set_image(&canvas)?;
            save_state(state, &canvas)?;
        }
        Cmd::Fill { color, state } => {
            let canvas = Canvas::filled(*color);
            p.set_image(&canvas)?;
            save_state(state, &canvas)?;
        }
        Cmd::Show { state } => p.set_image(&load_state(state, None)?)?,
        Cmd::Image { file, time } => {
            let frames = load_frames(file, *time)?;
            p.set_animation(&frames)?;
        }
        Cmd::Brightness { percent } => p.set_brightness(*percent)?,
        Cmd::Clock {
            style,
            h24,
            weather,
            temp,
            calendar,
            color,
        } => p.show_clock(&ClockOptions {
            style: *style,
            twenty_four_hour: *h24,
            weather: *weather,
            temperature: *temp,
            calendar: *calendar,
            color: *color,
        })?,
        Cmd::Light {
            color,
            brightness,
            effect,
        } => p.show_light(*color, *brightness, *effect, true)?,
        Cmd::On => {
            p.set_brightness(100)?;
            p.show_light([255, 255, 255], 100, 0, true)?;
        }
        Cmd::Off => p.show_light([1, 1, 1], 0, 0, false)?,
        Cmd::Channel { name } => p.set_channel(parse_channel(name)?)?,
        Cmd::Effect { number } => p.show_effect(*number)?,
        Cmd::Visualization { number } => p.show_visualization(*number)?,
        Cmd::Design { number } => p.show_design(*number)?,
        Cmd::Scoreboard { red, blue } => p.show_scoreboard(*red, *blue)?,
        Cmd::SetTime { value } => {
            let t = match value {
                Some(v) => {
                    let naive = NaiveDateTime::parse_from_str(v, "%Y-%m-%dT%H:%M:%S")
                        .or_else(|_| NaiveDateTime::parse_from_str(v, "%Y-%m-%d %H:%M:%S"))
                        .context("expected YYYY-MM-DDTHH:MM:SS")?;
                    Local
                        .from_local_datetime(&naive)
                        .single()
                        .context("ambiguous local time")?
                }
                None => Local::now(),
            };
            p.set_date_time(
                t.year() as u16,
                t.month() as u8,
                t.day() as u8,
                t.hour() as u8,
                t.minute() as u8,
                t.second() as u8,
                t.weekday().num_days_from_sunday() as u8,
            )?;
        }
        Cmd::Weather { celsius, code } => p.set_weather(*celsius, *code)?,
        Cmd::Timer { state } => p.timer(*state)?,
        Cmd::Countdown {
            minutes,
            seconds,
            no_start,
        } => p.countdown(!*no_start, *minutes, *seconds)?,
        Cmd::Raw { hex: parts, read } => {
            let bytes = protocol::parse_hex(&parts.join(" "))?;
            let (command, args) = bytes.split_first().context("empty command")?;
            p.send(*command, args)?;
            if *read {
                print_messages(&p.read_messages()?);
            }
        }
        Cmd::GetView => print_messages(&p.get_view()?),
        Cmd::Listen { seconds } => {
            let end = Instant::now() + Duration::from_secs(*seconds);
            p.verbose = true;
            while Instant::now() < end {
                print_messages(&p.read_messages()?);
            }
        }
    }
    Ok(())
}

fn print_messages(msgs: &[protocol::Message]) {
    for m in msgs {
        match m.command() {
            Some(c) => println!("reply to 0x{c:02x}: {}", hex(m.data())),
            None => println!("message: {}", hex(&m.payload)),
        }
    }
}
