use async_std::{net::UdpSocket, sync::Arc, task};
use iced::{executor, Application, Canvas, Command, Settings, Subscription};
use monitor_tool::canvas2d::*;
use monitor_tool::{BorderMode, PolarAxis};

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
    socket: Arc<UdpSocket>,
}

impl Application for Main {
    type Executor = executor::Default;
    type Message = ();
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        task::block_on(async {
            let socket = Arc::new(
                UdpSocket::bind(format!("0.0.0.0:{}", flags.port))
                    .await
                    .unwrap(),
            );
            (
                Main {
                    title: flags.title,
                    socket,
                },
                Command::none(),
            )
        })
    }

    fn title(&self) -> String {
        format!(
            "{}: {}",
            self.title,
            self.socket.local_addr().unwrap().port()
        )
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    fn update(
        &mut self,
        _message: Self::Message,
        _clipboard: &mut iced::Clipboard,
    ) -> Command<Self::Message> {
        println!("update!");
        Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        use iced::Length::Fill;
        println!("new view!");
        Canvas::new(Figure::new(BorderMode::Polar(PolarAxis::Top)))
            .width(Fill)
            .height(Fill)
            .into()
    }
}
