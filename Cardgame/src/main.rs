// 声明模块
mod card;
mod game1;
mod game2;
mod game3;
mod util;
mod difficulty;

// 导入依赖
use card::Card;
use eframe::egui;
use game1::MemoryGame;
use game2::GuessNumberGame;
use game3::TexasHoldemGame;
use std::sync::Arc;
use std::time::Instant;

/// 程序主入口点
fn main() -> eframe::Result {
    // 配置原生窗口选项
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("纸牌游戏"),
        ..Default::default()
    };

    // 启动原生GUI应用程序
    eframe::run_native(
        "纸牌游戏",
        native_options,
        Box::new(|cc| {
            // 在应用创建时加载中文字体
            setup_fonts(&cc.egui_ctx);
            // 返回应用程序实例
            Ok(Box::new(CardGameApp::new(cc)))
        }),
    )
}

/// 字体设置函数
fn setup_fonts(ctx: &egui::Context) {
    if let Ok(font_data) = std::fs::read("fonts/yahei.ttf") {
        let mut fonts = egui::FontDefinitions::default();
        
        fonts.font_data.insert(
            "yahei".to_owned(),
            Arc::new(egui::FontData::from_owned(font_data))
        );
        
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "yahei".to_owned());
        
        ctx.set_fonts(fonts);
    }
}

/// 应用程序状态枚举
#[derive(PartialEq, Clone, Copy)]
enum AppState {
    MainMenu,           // 主菜单状态
    MemoryGame,         // 神经衰弱游戏状态
    GuessNumberGame,    // 猜数字游戏状态
    TexasHoldem,        // 德州扑克游戏状态
}

/// 主应用程序结构体
struct CardGameApp {
    cards: Vec<Card>,               // 主菜单卡片
    app_state: AppState,            // 应用程序状态
    memory_game: MemoryGame,        // 神经衰弱游戏实例
    guess_number_game: GuessNumberGame, // 猜数字游戏实例
    texas_holdem_game: TexasHoldemGame, // 德州扑克游戏实例
    transition_timer: Option<Instant>, // 转场计时器
    transition_progress: f32,       // 转场进度
    target_game: AppState,          // 目标游戏
    background_texture: Option<egui::TextureHandle>, // 背景图片纹理
}

impl CardGameApp {
    /// 创建新的应用实例
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = &cc.egui_ctx;

        // 加载背景图片
        let background_texture = if let Ok(image_data) = std::fs::read("assets/back_ground.jpg") {
            if let Ok(image) = image::load_from_memory(&image_data) {
                let image_buffer = image.to_rgba8();
                let pixels: Vec<_> = image_buffer.pixels().flat_map(|p| p.0).collect();
                let size = [image.width() as usize, image.height() as usize];
                
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                Some(ctx.load_texture("background", color_image, Default::default()))
            } else {
                None
            }
        } else {
            None
        };

        // 初始化主菜单卡片
        
        let mut cards = Vec::new();
        
        // 创建三张卡片
        if let Ok(card) = Card::new(ctx, 0, 1, 1, "assets/card_back/default.png", "assets/card_face/JOKER-A.png") {
            cards.push(card);
        }
        
        if let Ok(card) = Card::new(ctx, 1, 1, 1, "assets/card_back/default.png", "assets/card_face/JOKER-B.png") {
            cards.push(card);
        }
        
        if let Ok(card) = Card::new(ctx, 2, 1, 1, "assets/card_back/default.png", "assets/card_face/JOKER-A.png") {
            cards.push(card);
        }

        // 初始化游戏实例并设置主菜单卡片用于纹理复用
        let mut memory_game = MemoryGame::new();
        memory_game.set_main_menu_cards(cards.clone());

        let mut texas_holdem_game = TexasHoldemGame::new();
        texas_holdem_game.set_main_menu_cards(cards.clone());

        Self {
            cards,
            app_state: AppState::MainMenu,
            memory_game,
            guess_number_game: GuessNumberGame::new(),
            texas_holdem_game,
            transition_timer: None,
            transition_progress: 0.0,
            target_game: AppState::MainMenu,
            background_texture,
        }
    }

    /// 显示主菜单界面
    fn show_main_menu(&mut self, ui: &mut egui::Ui) {
        // 处理转场动画
        if let Some(timer) = self.transition_timer {
            let elapsed = timer.elapsed().as_millis() as f32;
            self.transition_progress = (elapsed / 1000.0).min(1.0); // 1秒转场时间
            
            if self.transition_progress >= 1.0 {
                // 转场完成，进入目标游戏界面
                self.app_state = self.target_game;
                self.transition_timer = None;
                self.transition_progress = 0.0;
                
                // 将所有卡片翻回背面
                for card in &mut self.cards {
                    card.is_face_up = false;
                }
            }
        }

        // 在整个面板上绘制黑色覆盖层
        if let Some(_) = self.transition_timer {
            let alpha = (self.transition_progress * 255.0) as u8;
            let darken_color = egui::Color32::from_rgba_premultiplied(0, 0, 0, alpha);
            ui.painter().rect_filled(ui.available_rect_before_wrap(), 0.0, darken_color);
        }

        ui.vertical_centered_justified(|ui| {
            ui.add_space(ui.available_height() * 0.15);
            
            // 标题区域
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("纸牌游戏");
                ui.add_space(20.0);
                ui.label("请选择游戏");
                ui.add_space(40.0);
            });

            // 卡片显示区域
            ui.horizontal(|ui| {
                let available_width = ui.available_width();
                let total_cards_width = 150.0 * 3.0 + 30.0 * 2.0;
                let horizontal_padding = (available_width - total_cards_width) / 2.0;
                
                ui.add_space(horizontal_padding.max(0.0));
                
                for index in 0..self.cards.len() {
                    ui.vertical(|ui| {
                        ui.add_space(10.0);
                        
                        let card = &mut self.cards[index];
                        
                        // 基础卡片大小
                        let base_size = egui::vec2(150.0, 220.0);
                        
                        // 为卡片分配空间并检测悬停
                        let (_, card_rect) = ui.allocate_space(base_size);
                        let is_hovered = ui.rect_contains_pointer(card_rect) && self.transition_timer.is_none();
                        
                        // 根据悬停状态确定显示大小
                        let display_size = if is_hovered {
                            base_size * 1.1 // 悬停时放大10%
                        } else {
                            base_size
                        };
                        
                     
                        let center_offset = (display_size - base_size) * 0.5;
                        let card_pos = card_rect.min - center_offset;
                        
                        // 在正确位置显示卡片
                        ui.allocate_ui_at_rect(
                            egui::Rect::from_min_size(card_pos, display_size),
                            |ui| {
                                let response = card.show(ui, display_size);
                                
                                // 只在没有转场时处理点击
                                if self.transition_timer.is_none() && response.clicked() {
                                    // 开始转场动画
                                    self.transition_timer = Some(Instant::now());
                                    card.is_face_up = true; 
                                
                                    // 根据点击的卡片设置目标游戏
                                    self.target_game = match index {
                                        0 => AppState::MemoryGame,
                                        1 => AppState::GuessNumberGame,
                                        2 => AppState::TexasHoldem,
                                        _ => AppState::MainMenu,
                                    };
                                }
                            }
                        );
                        
                        ui.add_space(10.0);
                    });
                    
                    if index < self.cards.len() - 1 {
                        ui.add_space(30.0);
                    }
                }
                
                ui.add_space(horizontal_padding.max(0.0));
            });

            // 显示转场提示
            if self.transition_timer.is_some() {
                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    ui.add_space(ui.available_width() / 2.0 - 100.0);
                    // 在转场时使用白色文字确保可见性
                    let text_color = if self.transition_progress > 0.5 {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::BLACK
                    };
                    
                    let game_name = match self.target_game {
                        AppState::MemoryGame => "神经衰弱游戏",
                        AppState::GuessNumberGame => "猜数字游戏",
                        AppState::TexasHoldem => "德州扑克游戏",
                        _ => "游戏",
                    };
                    
                    ui.colored_label(text_color, format!("正在进入{}...", game_name));
                });
            }
            
            ui.add_space(ui.available_height() * 0.15);
        });
    }
}

impl eframe::App for CardGameApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                // 获取整个面板的矩形区域
                let panel_rect = ui.available_rect_before_wrap();
                
                // 绘制背景图片 - 拉伸填充整个窗口
                if let Some(texture) = &self.background_texture {
                    let image = egui::Image::new(texture)
                        .fit_to_exact_size(panel_rect.size());
                    ui.add(image);
                } else {
                    // 备用背景
                    ui.painter().rect_filled(panel_rect, 0.0, egui::Color32::from_rgb(240, 240, 240));
                }
                
                // 显示当前状态的界面
                match self.app_state {
                    AppState::MainMenu => {
                        self.show_main_menu(ui);
                    }
                    AppState::MemoryGame => {
                        if self.memory_game.show(ui, ctx) {
                            self.app_state = AppState::MainMenu;
                        }
                    }
                    AppState::GuessNumberGame => {
                        if self.guess_number_game.show(ui, ctx) {
                            self.app_state = AppState::MainMenu;
                        }
                    }
                    AppState::TexasHoldem => {
                        if self.texas_holdem_game.show(ui, ctx) {
                            self.app_state = AppState::MainMenu;
                        }
                    }
                }
            });
    }
}