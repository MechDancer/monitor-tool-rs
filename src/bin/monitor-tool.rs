fn main() {
    #[cfg(feature = "app")]
    {
        use monitor_tool::{run, Flags};
        let args = std::env::args().skip(1).collect::<Vec<_>>();
        match args.len() {
            0 => {
                let _ = run(Flags::Realtime("Figure1".into(), 12345));
            }
            1 => {
                let _ = run(Flags::Resume(args[0].clone().into()));
            }
            2 => {
                if let Ok(port) = args[1].parse() {
                    let _ = run(Flags::Realtime(args[0].clone(), port));
                } else {
                    eprintln!("参数格式：标题 端口号");
                }
            }
            _ => eprintln!("参数必须是 0 个、1 个或 2 个"),
        };
    }
}
