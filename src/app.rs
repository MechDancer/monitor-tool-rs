use async_std::channel::{unbounded, Receiver};
use iced::{canvas::Geometry, executor, Canvas, Command, Length::Fill, Rectangle, Subscription};
use std::cell::Cell;

mod cache_builder;
mod command_receiver;
mod figure;
mod figure_program;

use cache_builder::spawn_background as spawn_draw;
use command_receiver::spawn_background as spawn_receive;
use figure_program::{CacheComplete, FigureProgram};

pub(crate) use figure::Figure;
pub use iced::{Application, Settings};

#[derive(Default, Debug)]
pub struct Flags {
    pub title: String,
    pub port: u16,
}

pub struct Main {
    title: String,
    painter: Cell<Option<Receiver<(Rectangle, Vec<Geometry>)>>>,
    program: FigureProgram,
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
                title: format!("{}: {}", flags.title, flags.port),
                painter: Cell::new(Some(spawn_draw(receiver))),
                program: FigureProgram::new(sender),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        self.painter
            .take()
            .map(|r| Subscription::from_recipe(CacheComplete(r)))
            .unwrap_or(Subscription::none())
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
