use rambo_core::interactive::InteractiveSession;
use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    // 检查是否有参数
    if args.len() > 1 && args[1] == "b" {
        // 直接执行清理
        println!("🚀 RAM Booster v1.2.0 - 直接清理模式");
        let mut session = InteractiveSession::new();

        println!("💀 使用Killer模式进行清理...");
        if let Err(e) = session.handle_boost() {
            eprintln!("❌ 清理失败: {}", e);
            process::exit(1);
        }
        println!("✅ 清理完成！");
        return;
    }

    // 正常交互模式
    let mut session = InteractiveSession::new();

    if let Err(e) = session.start() {
        eprintln!("❌ 交互式会话发生错误: {}", e);
        process::exit(1);
    }
}