use async_std::{
    channel::{unbounded, Receiver, Sender},
    net::UdpSocket,
    path::PathBuf,
    task,
};
use iced::{
    canvas::Geometry,
    executor,
    window::{self, Icon},
    Application, Canvas, Command,
    Length::Fill,
    Rectangle, Settings, Subscription,
};
use std::{cell::Cell, time::Instant};

mod cache_builder;
mod figure;
mod figure_program;

use cache_builder::spawn_background as spawn_draw;
use figure::FigureSnapshot;
use figure_program::{CacheComplete, FigureEvent, FigureProgram};

pub(crate) use figure::Figure;

#[derive(Debug)]
pub enum Flags {
    Resume(PathBuf),
    Realtime(String, u16),
}

pub fn run(flags: Flags) -> iced::Result {
    Main::run(Settings {
        antialiasing: true,
        flags,
        window: window::Settings {
            icon: load_icon("favicon.ico".into()),
            ..Default::default()
        },
        ..Default::default()
    })
}

type Painter = Cell<Option<Receiver<(Rectangle, Vec<Geometry>)>>>;

struct Main {
    title: String,
    painter: Painter,
    program: FigureProgram,
}

impl Application for Main {
    type Executor = executor::Default;
    type Message = (Rectangle, Vec<Geometry>);
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        match flags {
            Flags::Realtime(title, port) => {
                let (sender, receiver) = unbounded();
                spawn_udp(port, sender.clone());
                spawn_stdin(sender.clone());
                (
                    Main {
                        title: format!("{}: {}", title, port),
                        painter: Cell::new(Some(spawn_draw(receiver, None))),
                        program: FigureProgram::new(sender),
                    },
                    Command::none(),
                )
            }
            Flags::Resume(path) => {
                let title = path.as_os_str().to_string_lossy().into_owned();
                let snapshot = FigureSnapshot::load(path);
                let (sender, receiver) = unbounded();
                spawn_stdin(sender.clone());
                (
                    Main {
                        title,
                        painter: Cell::new(Some(spawn_draw(
                            receiver,
                            task::block_on(snapshot).ok(),
                        ))),
                        program: FigureProgram::new(sender),
                    },
                    Command::none(),
                )
            }
        }
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        self.painter
            .take()
            .map(|r| Subscription::from_recipe(CacheComplete(r)))
            .unwrap_or_else(Subscription::none)
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut iced::Clipboard,
    ) -> Command<Self::Message> {
        self.program.state = message;
        Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        Canvas::new(self.program.clone())
            .width(Fill)
            .height(Fill)
            .into()
    }
}

impl Default for Flags {
    fn default() -> Self {
        Self::Realtime("".into(), 0)
    }
}

/// 加载图标
fn load_icon(path: PathBuf) -> Option<Icon> {
    use image::GenericImageView;
    let img = image::open(path).ok()?;
    let (width, height) = img.dimensions();
    let rgba = img.into_bytes();
    Icon::from_rgba(rgba, width, height).ok()
}

/// 启动命令行解析
fn spawn_stdin(sender: Sender<FigureEvent>) {
    task::spawn(async move {
        loop {
            let mut line = String::new();
            let _ = match async_std::io::stdin().read_line(&mut line).await {
                Ok(0) | Err(_) => break,
                Ok(_) => sender.send(FigureEvent::Line(line)).await,
            };
        }
    });
}

/// 启动 UDP 接收
fn spawn_udp(port: u16, sender: Sender<FigureEvent>) {
    task::spawn(async move {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).await.unwrap();
        let mut buf = Box::new([0u8; 65536]);
        while let Ok((n, _)) = socket.recv_from(buf.as_mut()).await {
            let _ = sender
                .send(FigureEvent::Packet(Instant::now(), buf[..n].to_vec()))
                .await;
        }
    });
}
