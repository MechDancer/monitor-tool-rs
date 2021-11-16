use iced::{executor, Application, Canvas, Command, Settings, Subscription};
use monitor_tool::{FigureProgram, Message, Server};
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
    type Message = Message;
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Main {
                title: flags.title,
                port: flags.port,
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
        match message {
            Message::MessageReceived(_, _, buf) => println!("Received! len = {}", buf.len()),
            Message::ViewUpdated => println!("View Updated!"),
        };
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
