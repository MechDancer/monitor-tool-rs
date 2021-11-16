use async_std::{net::UdpSocket, sync::Arc, task};
use iced::{executor, Application, Canvas, Command, Settings, Subscription};
use monitor_tool::FigureCanvas;
use std::time::Duration;

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
            (
                Main {
                    title: flags.title,
                    socket: Arc::new(
                        UdpSocket::bind(format!("0.0.0.0:{}", flags.port))
                            .await
                            .unwrap(),
                    ),
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
        iced::time::every(Duration::from_millis(30)).map(|_| ())
    }

    fn update(
        &mut self,
        _message: Self::Message,
        _clipboard: &mut iced::Clipboard,
    ) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        use iced::Length::Fill;
        Canvas::new(FigureCanvas::new())
            .width(Fill)
            .height(Fill)
            .into()
    }
}
