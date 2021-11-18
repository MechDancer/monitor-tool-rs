fn main() {
    #[cfg(feature = "app")]
    app::run();
}

#[cfg(feature = "sender")]
mod sender {}

#[cfg(feature = "app")]
mod app {
    use async_std::channel::{unbounded, Receiver};
    use iced::{
        canvas::Geometry, executor, Application, Canvas, Command, Rectangle, Settings, Subscription,
    };
    use monitor_tool::{spawn_background, FigureProgram, Message};

    pub fn run() {
        let _ = Main::run(Settings {
            antialiasing: true,
            flags: Flags {
                title: "Figure1".into(),
                port: 12345,
            },
            ..Default::default()
        });
    }

    #[derive(Default, Debug)]
    struct Flags {
        pub title: String,
        pub port: u16,
    }

    struct Main {
        title: String,
        port: u16,
        redraw: Receiver<(Rectangle, Vec<Geometry>)>,
        canvas: FigureProgram,
    }

    impl Application for Main {
        type Executor = executor::Default;
        type Message = Message;
        type Flags = Flags;

        fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
            let (sender, receiver) = unbounded();
            (
                Main {
                    title: flags.title,
                    port: flags.port,
                    redraw: spawn_background(receiver),
                    canvas: FigureProgram(sender, Default::default(), vec![]),
                },
                Command::none(),
            )
        }

        fn title(&self) -> String {
            format!("{}: {}", self.title, self.port)
        }

        fn subscription(&self) -> Subscription<Self::Message> {
            // Subscription::from_recipe(UdpReceiver::new(self.port))
            Subscription::none()
        }

        fn update(
            &mut self,
            message: Self::Message,
            _clipboard: &mut iced::Clipboard,
        ) -> Command<Self::Message> {
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
}
