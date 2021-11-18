fn main() {
    #[cfg(feature = "app")]
    app::run();
}

#[cfg(feature = "sender")]
mod sender {}

#[cfg(feature = "app")]
mod app {
    use std::cell::Cell;

    use async_std::channel::{unbounded, Receiver};
    use iced::{
        canvas::Geometry, executor, Application, Canvas, Command, Rectangle, Settings, Subscription,
    };
    use monitor_tool::{spawn_draw, spawn_receive, CacheComplete, FigureProgram};

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
        redraw: Cell<Option<Receiver<(Rectangle, Vec<Geometry>)>>>,
        canvas: FigureProgram,
    }

    impl Application for Main {
        type Executor = executor::Default;
        type Message = (Rectangle, Vec<Geometry>);
        type Flags = Flags;

        fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
            let (sender, receiver) = unbounded();
            spawn_receive(flags.port, sender.clone());
            (
                Main {
                    title: flags.title,
                    port: flags.port,
                    redraw: Cell::new(Some(spawn_draw(receiver))),
                    canvas: FigureProgram::new(sender),
                },
                Command::none(),
            )
        }

        fn title(&self) -> String {
            format!("{}: {}", self.title, self.port)
        }

        fn subscription(&self) -> Subscription<Self::Message> {
            if let Some(r) = self.redraw.replace(None) {
                Subscription::from_recipe(CacheComplete(r))
            } else {
                Subscription::none()
            }
        }

        fn update(
            &mut self,
            message: Self::Message,
            _clipboard: &mut iced::Clipboard,
        ) -> Command<Self::Message> {
            self.canvas.state = message;
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
