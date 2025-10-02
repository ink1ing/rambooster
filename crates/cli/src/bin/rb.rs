use rambo_core::interactive::{InteractiveTerminal, run_direct_boost};
use rambo_core::config::load_config;
use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    // 加载配置
    let config = match load_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("❌ 加载配置失败: {}", e);
            process::exit(1);
        }
    };

    // 检查是否有参数
    if args.len() > 1 && args[1] == "b" {
        // 直接执行清理
        if let Err(e) = run_direct_boost() {
            eprintln!("❌ 清理失败: {:?}", e);
            process::exit(1);
        }
        return;
    }

    // 正常交互模式
    let mut terminal = InteractiveTerminal::new(config);

    if let Err(e) = terminal.run() {
        eprintln!("❌ 交互式终端发生错误: {:?}", e);
        process::exit(1);
    }
}