use std::io::{self, Write};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use crate::config::Config;
use crate::release::{boost, BoostResult};
use crate::{read_mem_stats, MemStats};
use crate::processes::{get_all_processes, sort_and_take_processes};
use crate::hotkey::GlobalHotkey;
use crate::version::{check_for_updates, perform_update};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoostLevel {
    Low,
    Medium,
    High,
}

impl BoostLevel {
    pub fn description(&self) -> &'static str {
        match self {
            BoostLevel::Low => "轻度清理 - 基础内存优化",
            BoostLevel::Medium => "中度清理 - 标准内存释放",
            BoostLevel::High => "强力清理 - 深度内存优化",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            BoostLevel::Low => "🟡",
            BoostLevel::Medium => "🟠",
            BoostLevel::High => "🔴",
        }
    }

    pub fn next(&self) -> BoostLevel {
        match self {
            BoostLevel::Low => BoostLevel::Medium,
            BoostLevel::Medium => BoostLevel::High,
            BoostLevel::High => BoostLevel::Low,
        }
    }

    pub fn prev(&self) -> BoostLevel {
        match self {
            BoostLevel::High => BoostLevel::Medium,
            BoostLevel::Medium => BoostLevel::Low,
            BoostLevel::Low => BoostLevel::High,
        }
    }
}

pub struct InteractiveTerminal {
    config: Config,
    current_level: BoostLevel,
    running: bool,
    input_buffer: String,
}

impl InteractiveTerminal {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            current_level: BoostLevel::Medium,
            running: true,
            input_buffer: String::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), terminal::EnterAlternateScreen)?;

        self.show_welcome_screen()?;

        while self.running {
            self.show_prompt()?;
            self.handle_input()?;
        }

        execute!(io::stdout(), terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    fn show_welcome_screen(&self) -> Result<(), Box<dyn std::error::Error>> {
        execute!(
            io::stdout(),
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            SetForegroundColor(Color::Cyan),
            Print("╔══════════════════════════════════════════════════════════════╗\n"),
            Print("║                     🦀 RAM Booster 交互模式                    ║\n"),
            Print("║                        v1.2.0 - Rust Edition                ║\n"),
            Print("╚══════════════════════════════════════════════════════════════╝\n"),
            ResetColor,
            Print("\n"),
            SetForegroundColor(Color::Yellow),
            Print("💡 可用命令:\n"),
            ResetColor,
            Print("   /boost    - 执行内存清理\n"),
            Print("   /lv       - 切换清理强度 (上下键选择)\n"),
            Print("   /status   - 显示内存状态\n"),
            Print("   /hotkey   - 快捷键管理\n"),
            Print("   /daemon   - 后台服务管理\n"),
            Print("   /update   - 检查和更新版本\n"),
            Print("   /help     - 显示帮助\n"),
            Print("   /exit     - 退出 (或按 Ctrl+C)\n"),
            Print("\n"),
            SetForegroundColor(Color::Green),
            Print("🎯 当前清理强度: "),
            SetForegroundColor(Color::White),
            Print(format!("{} {}\n", self.current_level.icon(), self.current_level.description())),
            ResetColor,
            Print("\n"),
        )?;
        Ok(())
    }

    fn show_prompt(&self) -> Result<(), Box<dyn std::error::Error>> {
        execute!(
            io::stdout(),
            SetForegroundColor(Color::Blue),
            Print("rambo> "),
            Print(&self.input_buffer),
            ResetColor,
        )?;
        Ok(())
    }

    fn handle_input(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Event::Key(key_event) = event::read()? {
            match key_event {
                KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => {
                    self.running = false;
                    println!("\n👋 再见！");
                }
                KeyEvent {
                    code: KeyCode::Enter,
                    ..
                } => {
                    println!();
                    if !self.input_buffer.is_empty() {
                        self.execute_command(&self.input_buffer.clone())?;
                        self.input_buffer.clear();
                    }
                }
                KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                } => {
                    self.input_buffer.push(c);
                }
                KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                } => {
                    self.input_buffer.pop();
                    execute!(
                        io::stdout(),
                        cursor::MoveLeft(1),
                        Print(" "),
                        cursor::MoveLeft(1)
                    )?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn execute_command(&mut self, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        match command {
            "/boost" => self.execute_boost()?,
            "/lv" => self.show_level_selector()?,
            "/status" => self.show_status()?,
            "/hotkey" => self.show_hotkey_info()?,
            "/daemon" => self.show_daemon_info()?,
            "/update" => self.show_update_interface()?,
            "/help" => self.show_help()?,
            "/exit" => {
                self.running = false;
                println!("👋 再见！");
            }
            _ => {
                println!("❌ 未知命令: {}", command);
                println!("💡 输入 /help 查看可用命令");
            }
        }
        Ok(())
    }

    fn execute_boost(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 开始执行 {} 内存清理...", self.current_level.description());

        match boost() {
            Ok(result) => {
                self.print_boost_result(&result)?;
            }
            Err(e) => {
                println!("❌ 内存清理失败: {:?}", e);
            }
        }
        Ok(())
    }

    fn show_level_selector(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n📊 选择清理强度 (上下键切换，Enter确认，Esc取消):");

        let levels = [BoostLevel::Low, BoostLevel::Medium, BoostLevel::High];
        let mut selected_index = levels.iter().position(|&l| l == self.current_level).unwrap_or(1);

        loop {
            // 清除之前的选择显示
            execute!(
                io::stdout(),
                cursor::MoveUp(3),
                terminal::Clear(ClearType::FromCursorDown)
            )?;

            for (i, level) in levels.iter().enumerate() {
                let prefix = if i == selected_index { "→ " } else { "  " };
                let color = if i == selected_index { Color::Green } else { Color::White };

                execute!(
                    io::stdout(),
                    SetForegroundColor(color),
                    Print(format!("{}{} {}\n", prefix, level.icon(), level.description())),
                    ResetColor,
                )?;
            }

            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Up => {
                        selected_index = if selected_index == 0 { levels.len() - 1 } else { selected_index - 1 };
                    }
                    KeyCode::Down => {
                        selected_index = (selected_index + 1) % levels.len();
                    }
                    KeyCode::Enter => {
                        self.current_level = levels[selected_index];
                        println!("✅ 已切换到: {} {}", self.current_level.icon(), self.current_level.description());
                        break;
                    }
                    KeyCode::Esc => {
                        println!("❌ 已取消");
                        break;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn show_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 系统内存状态:");

        match read_mem_stats() {
            Ok(mem_stats) => {
                self.print_memory_stats(&mem_stats)?;

                // 显示进程信息
                let processes = get_all_processes();
                let top_processes = sort_and_take_processes(processes, 5);

                println!("\n🔝 内存占用前5的进程:");
                println!("{:<8} {:<25} {:>12}", "PID", "名称", "内存(MB)");
                println!("{:-<8} {:-<25} {:->12}", "", "", "");

                for p in &top_processes {
                    let name = if p.name.len() > 23 {
                        format!("{}...", &p.name[..23])
                    } else {
                        p.name.clone()
                    };
                    println!("{:<8} {:<25} {:>12}", p.pid, name, p.rss_mb);
                }
            }
            Err(e) => {
                println!("❌ 获取内存状态失败: {}", e);
            }
        }
        Ok(())
    }

    fn show_hotkey_info(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("⌨️  全局快捷键状态:");
        println!("   启用状态: {}", if self.config.hotkey.enabled { "✅ 已启用" } else { "❌ 已禁用" });
        println!("   快捷键: {}", self.config.hotkey.key_combination);
        println!("   显示通知: {}", if self.config.hotkey.show_notification { "是" } else { "否" });

        if !self.config.hotkey.enabled {
            println!("💡 使用 'rambo hotkey enable' 启用快捷键功能");
        }
        Ok(())
    }

    fn show_daemon_info(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🤖 后台服务信息:");
        println!("   配置文件: ~/.config/rambo/config.toml");
        println!("   日志文件: ~/.local/share/rambo/logs/");
        println!("💡 使用 'rambo daemon --install' 安装后台服务");
        Ok(())
    }

    fn show_help(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📖 RAM Booster 交互模式帮助:");
        println!();
        println!("🎯 可用命令:");
        println!("   /boost    - 执行内存清理");
        println!("   /lv       - 切换清理强度");
        println!("   /status   - 显示内存状态");
        println!("   /hotkey   - 快捷键管理");
        println!("   /daemon   - 后台服务管理");
        println!("   /update   - 检查和更新版本");
        println!("   /help     - 显示此帮助");
        println!("   /exit     - 退出程序");
        println!();
        println!("🎮 交互操作:");
        println!("   上下键    - 在选择界面中切换选项");
        println!("   Enter     - 确认选择");
        println!("   Esc       - 取消当前操作");
        println!("   Ctrl+C    - 退出程序");
        Ok(())
    }

    fn print_boost_result(&self, result: &BoostResult) -> Result<(), Box<dyn std::error::Error>> {
        println!("✅ 内存清理完成!");
        println!("   用时: {:.2}秒", result.duration.as_secs_f32());

        if result.delta_mb >= 0 {
            println!("   释放内存: {} MB", result.delta_mb);
        } else {
            println!("   内存增加: {} MB", -result.delta_mb);
        }

        println!("   清理前: {} MB 可用", result.before.free_mb);
        println!("   清理后: {} MB 可用", result.after.free_mb);
        Ok(())
    }

    fn print_memory_stats(&self, stats: &MemStats) -> Result<(), Box<dyn std::error::Error>> {
        println!("   总内存: {} MB", stats.total_mb);
        println!("   可用内存: {} MB", stats.free_mb);
        println!("   活跃内存: {} MB", stats.active_mb);
        println!("   非活跃内存: {} MB", stats.inactive_mb);
        println!("   固定内存: {} MB", stats.wired_mb);
        println!("   压缩内存: {} MB", stats.compressed_mb);
        println!("   内存压力: {:?}", stats.pressure);
        Ok(())
    }

    fn show_update_interface(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔄 版本更新管理:");
        println!("   [1] 检查更新");
        println!("   [2] 执行更新");
        println!("   [ESC] 返回");
        println!();
        print!("请选择操作 (1-2): ");

        loop {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('1') => {
                        println!("1\n");
                        self.check_version_status()?;
                        break;
                    }
                    KeyCode::Char('2') => {
                        println!("2\n");
                        self.execute_update()?;
                        break;
                    }
                    KeyCode::Esc => {
                        println!("已取消");
                        break;
                    }
                    _ => {
                        // 忽略其他按键
                    }
                }
            }
        }
        Ok(())
    }

    fn check_version_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔍 正在检查版本信息...");

        match check_for_updates() {
            Ok(version_info) => {
                println!("📊 版本信息:");
                println!("   当前版本: {}", version_info.current);

                if let Some(latest) = &version_info.latest {
                    println!("   最新版本: {}", latest);

                    if version_info.update_available {
                        println!("✨ 发现新版本可用！");
                        println!("💡 使用 /update 选择选项2进行更新");
                    } else {
                        println!("✅ 您已经是最新版本！");
                    }
                } else {
                    println!("❌ 无法检查远程版本（可能是网络问题）");
                }
            }
            Err(e) => {
                println!("❌ 检查更新失败: {}", e);
            }
        }
        Ok(())
    }

    fn execute_update(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 开始执行更新...");
        println!("⚠️  更新将替换当前程序文件");
        println!("   [Y] 确认更新");
        println!("   [N] 取消更新");
        print!("是否继续？(Y/N): ");

        loop {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        println!("Y\n");
                        println!("🔄 正在执行更新...");

                        match perform_update(false) {
                            Ok(()) => {
                                println!("🎉 更新完成！");
                                println!("💡 您可能需要重新启动终端或重新加载路径");
                                println!("🔄 建议退出当前会话并重新启动 RAM Booster");
                            }
                            Err(e) => {
                                println!("❌ 更新失败: {}", e);
                                println!("💡 您可以尝试手动运行更新脚本或从 GitHub 下载最新版本");
                            }
                        }
                        break;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        println!("N");
                        println!("❌ 更新已取消");
                        break;
                    }
                    _ => {
                        // 忽略其他按键
                    }
                }
            }
        }
        Ok(())
    }
}

// 简化模式 - 用于兼容原有的 rb b 命令
pub fn run_direct_boost() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 执行中等强度内存清理...");
    match boost() {
        Ok(result) => {
            println!("✅ 内存清理完成!");
            println!("   用时: {:.2}秒", result.duration.as_secs_f32());
            if result.delta_mb >= 0 {
                println!("   释放内存: {} MB", result.delta_mb);
            } else {
                println!("   内存变化: {} MB", result.delta_mb);
            }
            println!("   清理前: {} MB 可用", result.before.free_mb);
            println!("   清理后: {} MB 可用", result.after.free_mb);
        }
        Err(e) => {
            println!("❌ 内存清理失败: {:?}", e);
        }
    }
    Ok(())
}