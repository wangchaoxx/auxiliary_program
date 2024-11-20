use eframe::egui;
use enigo::{Enigo, MouseControllable};
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyManager,
};
use std::sync::{Arc, Mutex};
use std::thread;
struct RecoilConfig {
    weapons: std::collections::HashMap<String, Vec<RecoilStep>>,
}
#[derive(Clone)]
struct RecoilStep {
    x: f64,
    y: f64,
}
struct RecoilApp {
    // 武器配置
    config: RecoilConfig,

    // 当前选择的武器
    current_weapon: String,

    // 灵敏度
    sensitivity: f64,

    // 压枪状态
    is_shooting: Arc<Mutex<bool>>,

    // 状态文本
    status: String,

    // 检测是否按住鼠标左键
    is_left_button_pressed: Arc<Mutex<bool>>,
}

impl Default for RecoilConfig {
    fn default() -> Self {
        let mut weapons = std::collections::HashMap::new();

        // 默认武器配置
        weapons.insert(
            "AK47".to_string(),
            vec![
                RecoilStep { x: -0.5, y: -4.0 },
                RecoilStep { x: -0.3, y: -5.0 },
                RecoilStep { x: 0.2, y: -6.0 },
                RecoilStep { x: 0.5, y: -7.0 },
                RecoilStep { x: 0.8, y: -8.0 },
            ],
        );

        weapons.insert(
            "M416".to_string(),
            vec![
                RecoilStep { x: -0.4, y: -3.5 },
                RecoilStep { x: -0.2, y: -4.5 },
                RecoilStep { x: 0.3, y: -5.5 },
                RecoilStep { x: 0.6, y: -6.5 },
            ],
        );

        RecoilConfig { weapons }
    }
}
impl Default for RecoilApp {
    fn default() -> Self {
        Self {
            config: RecoilConfig::default(),
            current_weapon: "AK47".to_string(),
            sensitivity: 1.0,
            is_shooting: Arc::new(Mutex::new(false)),
            status: "等待中".to_string(),
        }
    }
}
impl RecoilApp {
    fn start_recoil(&mut self) {
        let is_shooting = Arc::clone(&self.is_shooting);
        let weapon_steps = self
            .config
            .weapons
            .get(&self.current_weapon)
            .cloned()
            .unwrap_or_default();

        let sensitivity = self.sensitivity;

        thread::spawn(move || {
            let mut enigo = Enigo::new();

            // 设置为正在压枪
            *is_shooting.lock().unwrap() = true;

            for step in weapon_steps {
                // 检查是否仍在压枪
                if !*is_shooting.lock().unwrap() {
                    break;
                }

                // 根据灵敏度调整移动
                let x_move = (step.x * sensitivity) as i32;
                let y_move = (step.y * sensitivity) as i32;

                // 移动鼠标
                enigo.mouse_move_relative(x_move, y_move);

                // 控制频率
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            // 结束压枪
            *is_shooting.lock().unwrap() = false;
        });

        self.status = "压枪中...".to_string();
    }

    fn stop_recoil(&mut self) {
        *self.is_shooting.lock().unwrap() = false;
        self.status = "等待中".to_string();
    }
}
impl eframe::App for RecoilApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 设置自定义字体
        // let font_id = egui::FontId::new(16.0, egui::FontFamily::Proportional);

        // ctx.fonts()
        //     .add_font(egui::FontData::from_static(include_bytes!(
        //         "/System/Library/Fonts/Hiragino Sans GB.ttc"
        //     )));

        // 设置自定义字体
        let mut fonts = egui::FontDefinitions::default();

        // 加载自定义中文字体（macOS 上的 Hiragino Sans GB 字体）
        fonts.font_data.insert(
            "my_chinese_font".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "/System/Library/Fonts/Hiragino Sans GB.ttc"
            )),
        );

        // 设置该字体为默认字体
        fonts.families.insert(
            egui::FontFamily::Proportional,
            vec!["my_chinese_font".to_owned()],
        );
        ctx.set_fonts(fonts);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("压枪宏控制器");

            // 武器选择
            ui.label("选择武器");
            egui::ComboBox::from_label("")
                .selected_text(&self.current_weapon)
                .show_ui(ui, |ui| {
                    for weapon in self.config.weapons.keys() {
                        ui.selectable_value(&mut self.current_weapon, weapon.clone(), weapon);
                    }
                });

            // 灵敏度调节
            ui.label("压枪灵敏度");
            ui.add(egui::Slider::new(&mut self.sensitivity, 0.1..=2.0));

            // 状态显示
            ui.label(&self.status);

            // 控制按钮
            ui.horizontal(|ui| {
                if ui.button("开始").clicked() {
                    self.start_recoil();
                }

                if ui.button("停止").clicked() {
                    self.stop_recoil();
                }
            });
        });
    }
}
fn setup_hotkeys(_is_shooting: Arc<Mutex<bool>>) -> global_hotkey::Result<GlobalHotKeyManager> {
    let hotkey_manager = GlobalHotKeyManager::new()?;

    // F1开始
    let start_hotkey = HotKey::new(Some(Modifiers::SHIFT), Code::F1);
    hotkey_manager.register(start_hotkey)?;

    // F2停止
    let stop_hotkey = HotKey::new(Some(Modifiers::SHIFT), Code::F2);
    hotkey_manager.register(stop_hotkey)?;

    // // 监听鼠标左键状态
    // std::thread::spawn(move || loop {
    //     let left_button_state = unsafe { GetAsyncKeyState(VK_LBUTTON) };
    //     *is_left_button_pressed.lock().unwrap() = left_button_state & 0x8000 != 0;
    //     std::thread::sleep(std::time::Duration::from_millis(10));
    // });

    Ok(hotkey_manager)
}
fn main() -> Result<(), eframe::Error> {
    // 初始化日志
    env_logger::init();

    // 创建应用状态
    let app = RecoilApp::default();
    let is_shooting = Arc::clone(&app.is_shooting);

    // 设置热键
    let _hotkey_manager = setup_hotkeys(is_shooting).expect("Failed to setup hotkeys");

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(400.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Auxiliary program",
        options,
        Box::new(|_cc| Box::<RecoilApp>::new(app)),
    )
}
