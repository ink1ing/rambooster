use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use crate::config::HotkeyConfig;

pub struct GlobalHotkey {
    config: HotkeyConfig,
    sender: Option<Sender<()>>,
    _receiver: Option<Receiver<()>>,
}

impl GlobalHotkey {
    pub fn new(config: HotkeyConfig) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            config,
            sender: Some(sender),
            _receiver: Some(receiver),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn check_accessibility_permission() -> bool {
        // 检查辅助功能权限
        unsafe {
            // 尝试创建一个事件tap来测试权限
            use std::ptr;
            use libc::c_void;

            // CGEventTapCreate需要辅助功能权限
            extern "C" {
                fn CGEventTapCreate(
                    tap: u32,
                    place: u32,
                    options: u32,
                    events_of_interest: u64,
                    callback: *const c_void,
                    refcon: *mut c_void,
                ) -> *mut c_void;
            }

            let tap = CGEventTapCreate(
                0, // kCGSessionEventTap
                0, // kCGHeadInsertEventTap
                0, // kCGEventTapOptionDefault
                1 << 10, // kCGEventMaskForAllEvents simplified
                ptr::null(),
                ptr::null_mut(),
            );

            let has_permission = !tap.is_null();

            // 清理资源
            if !tap.is_null() {
                extern "C" {
                    fn CFRelease(cf: *const c_void);
                }
                CFRelease(tap);
            }

            has_permission
        }
    }

    pub fn request_accessibility_permission() -> Result<(), Box<dyn std::error::Error>> {
        println!("🔒 RAM Booster 需要辅助功能权限来监听全局快捷键");
        println!("📋 请按以下步骤操作：");
        println!("   1. 系统设置 > 隐私与安全性 > 辅助功能");
        println!("   2. 点击 + 添加 RAM Booster 或终端应用");
        println!("   3. 勾选启用权限");
        println!("💡 权限授权后，按 Control+R 即可快速清理内存");
        Ok(())
    }

    pub fn start_monitoring(&self, callback: impl Fn() + Send + 'static) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.enabled {
            return Ok(());
        }

        if !Self::check_accessibility_permission() {
            Self::request_accessibility_permission()?;
            return Err("需要辅助功能权限".into());
        }

        println!("🎹 全局快捷键已启用: {}", self.config.key_combination);

        // 启动后台监听线程

        thread::spawn(move || {
            unsafe {
                use std::ptr;
                use libc::c_void;

                // 设置事件监听回调
                extern "C" fn event_tap_callback(
                    _proxy: *mut c_void,
                    event_type: u32,
                    event: *mut c_void,
                    refcon: *mut c_void,
                ) -> *mut c_void {
                    const CG_EVENT_KEY_DOWN: u32 = 10;

                    if event_type == CG_EVENT_KEY_DOWN {
                        extern "C" {
                            fn CGEventGetIntegerValueField(event: *mut c_void, field: u32) -> i64;
                        }

                        let keycode = unsafe { CGEventGetIntegerValueField(event, 9) }; // kCGKeyboardEventKeycode
                        let flags = unsafe { CGEventGetIntegerValueField(event, 1) }; // kCGEventSourceFlagsField

                        // 检查是否为 Control+R (keycode 15, Control flag 0x40000)
                        if keycode == 15 && (flags & 0x40000) != 0 {
                            if !refcon.is_null() {
                                unsafe {
                                    let callback = &*(refcon as *const Box<dyn Fn() + Send>);
                                    callback();
                                }
                            }
                        }
                    }

                    event // 返回原始事件，不拦截
                }

                // 创建事件tap
                extern "C" {
                    fn CGEventTapCreate(
                        tap: u32,
                        place: u32,
                        options: u32,
                        events_of_interest: u64,
                        callback: extern "C" fn(*mut c_void, u32, *mut c_void, *mut c_void) -> *mut c_void,
                        refcon: *mut c_void,
                    ) -> *mut c_void;

                    fn CFRunLoopAddSource(rl: *mut c_void, source: *mut c_void, mode: *mut c_void);
                    fn CFRunLoopRun();
                    fn CFRunLoopGetCurrent() -> *mut c_void;
                    fn CFMachPortCreateRunLoopSource(allocator: *mut c_void, port: *mut c_void, order: i32) -> *mut c_void;
                    fn kCFRunLoopCommonModes() -> *mut c_void;
                }

                let callback_box = Box::new(callback);
                let callback_ptr = Box::into_raw(Box::new(callback_box)) as *mut c_void;

                let event_tap = CGEventTapCreate(
                    0, // kCGSessionEventTap
                    0, // kCGHeadInsertEventTap
                    0, // kCGEventTapOptionDefault
                    1 << 10, // kCGEventMaskForAllEvents
                    event_tap_callback,
                    callback_ptr,
                );

                if event_tap.is_null() {
                    eprintln!("❌ 无法创建全局快捷键监听 - 可能缺少辅助功能权限");
                    return;
                }

                let run_loop_source = CFMachPortCreateRunLoopSource(ptr::null_mut(), event_tap, 0);
                let run_loop = CFRunLoopGetCurrent();

                CFRunLoopAddSource(run_loop, run_loop_source, kCFRunLoopCommonModes());

                println!("✅ 全局快捷键监听已启动");
                CFRunLoopRun(); // 进入事件循环
            }
        });

        Ok(())
    }

    pub fn stop_monitoring(&mut self) {
        self.sender = None;
        println!("🛑 全局快捷键监听已停止");
    }
}

// 简化的按键监听函数，用于概念验证
pub fn setup_simple_hotkey_listener() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚠️  注意：当前为简化实现，仅作为功能框架");
    println!("🔧 完整的全局按键监听需要更复杂的系统集成");
    Ok(())
}