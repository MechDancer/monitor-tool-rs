use iced::{Canvas, Sandbox, Settings};
use monitor_tool::canvas2d::*;
use monitor_tool::{BorderMode, PolarAxis};

fn main() -> iced::Result {
    Main::run(Settings {
        antialiasing: true,
        ..Default::default()
    })
}

struct Main;

impl Sandbox for Main {
    type Message = ();

    fn new() -> Self {
        Main {}
    }

    fn title(&self) -> String {
        String::from("Plot1")
    }

    fn update(&mut self, _: Self::Message) {}

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        use iced::Length::Fill;
        Canvas::new(DrawState::new("Plot1", BorderMode::Polar(PolarAxis::Top)))
            .width(Fill)
            .height(Fill)
            .into()
    }
}
