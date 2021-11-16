use iced::{executor, Application, Canvas, Command, Settings, Subscription};
use monitor_tool::{FigureProgram, Server};
use std::{net::SocketAddr, time::Instant};

fn main() -> iced::Result {
    Main::run(Settings {
        antialiasing: true,
        flags: Flags {
            title: "Figure1".into(),
            port: 12345,
        },
        ..Default::default()
    })
}

#[derive(Default, Debug)]
struct Flags {
    title: String,
    port: u16,
}

struct Main {
    title: String,
    port: u16,
    canvas: FigureProgram,
}

impl Application for Main {
    type Executor = executor::Default;
    type Message = (Instant, SocketAddr, Vec<u8>);
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Main {
                title: flags.title,
                port: flags.port,
                // server: ,
                canvas: FigureProgram::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        format!("{}: {}", self.title, self.port)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::from_recipe(Server::new(self.port))
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut iced::Clipboard,
    ) -> Command<Self::Message> {
        println!("Received! len = {}", message.2.len());
        Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        use iced::Length::Fill;
        Canvas::new(self.canvas.clone())
            .width(Fill)
            .height(Fill)
            .into()
    }
}
