use iced::{Canvas, Sandbox, Settings};
use monitor_tool::canvas2d::*;

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
        String::from("绘图工具")
    }

    fn update(&mut self, _: Self::Message) {}

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        use iced::Length::Fill;
        Canvas::new(DrawState::default())
            .width(Fill)
            .height(Fill)
            .into()
    }
}
