fn main() {
    #[cfg(feature = "app")]
    {
        use monitor_tool::{Application, Flags, Main, Settings};
        let _ = Main::run(Settings {
            antialiasing: true,
            flags: Flags {
                title: "Figure1".into(),
                port: 12345,
            },
            ..Default::default()
        });
    }
}
