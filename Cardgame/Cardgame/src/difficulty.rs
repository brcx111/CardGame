// difficulty.rs
use eframe::egui;
use std::time::Instant;

#[derive(PartialEq, Clone, Copy)]
pub enum GameDifficulty {
    Easy,    // 简单难度
    Medium,  // 中等难度
    Hard,    // 困难难度
}

/// 难度选择界面
pub struct DifficultySelection {
    pub selected_difficulty: Option<GameDifficulty>,
    pub transition_timer: Option<Instant>,
    pub transition_progress: f32,
    pub transition_complete: bool,
}

impl DifficultySelection {
    pub fn new() -> Self {
        Self {
            selected_difficulty: None,
            transition_timer: None,
            transition_progress: 0.0,
            transition_complete: false,
        }
    }

    /// 显示难度选择界面
    pub fn show(
        &mut self, 
        ui: &mut egui::Ui, 
        game_name: &str, 
        rules: &[&str]
    ) -> (bool, bool) {
        let mut return_to_menu = false;
        let mut start_game = false;

        // 重置过渡完成状态
        self.transition_complete = false;

        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 200.0);
                ui.vertical(|ui| {
                    ui.heading(game_name);
                    ui.add_space(10.0);
                    ui.label("请选择游戏难度开始游戏");
                    
                    // 规则说明
                    ui.add_space(20.0);
                    ui.colored_label(egui::Color32::LIGHT_BLUE, "游戏规则说明:");
                    for rule in rules {
                        ui.label(format!("• {}", rule));
                    }
                });
            });

            ui.add_space(30.0);

            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 100.0);
                ui.vertical(|ui| {
                    if self.centered_button(ui, "简单难度", 200.0, 60.0).clicked() {
                        self.selected_difficulty = Some(GameDifficulty::Easy);
                        self.transition_timer = Some(Instant::now());
                        start_game = true;
                    }
                    ui.add_space(20.0);

                    if self.centered_button(ui, "中等难度", 200.0, 60.0).clicked() {
                        self.selected_difficulty = Some(GameDifficulty::Medium);
                        self.transition_timer = Some(Instant::now());
                        start_game = true;
                    }
                    ui.add_space(20.0);

                    if self.centered_button(ui, "困难难度", 200.0, 60.0).clicked() {
                        self.selected_difficulty = Some(GameDifficulty::Hard);
                        self.transition_timer = Some(Instant::now());
                        start_game = true;
                    }
                });
            });

            ui.add_space(40.0);

            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 75.0);
                if self.centered_button(ui, "返回主菜单", 150.0, 40.0).clicked() {
                    return_to_menu = true;
                }
            });

            ui.add_space(20.0);
        });

        (return_to_menu, start_game)
    }

    /// 显示过渡动画并返回是否完成
    pub fn show_transition_animation(&mut self, ui: &mut egui::Ui) -> bool {
        if let Some(timer) = self.transition_timer {
            let elapsed = timer.elapsed().as_millis() as f32;
            self.transition_progress = (elapsed / 800.0).min(1.0);
            
            let alpha = (self.transition_progress * 255.0) as u8;
            let darken_color = egui::Color32::from_rgba_premultiplied(0, 0, 0, alpha);
            ui.painter().rect_filled(ui.available_rect_before_wrap(), 0.0, darken_color);

            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 2.0 - 50.0);
                
                ui.horizontal(|ui| {
                    ui.add_space(ui.available_width() / 2.0 - 100.0);
                    ui.vertical(|ui| {
                        let time = Instant::now().elapsed().as_secs_f32();
                        let dot_count = 3;
                        let mut loading_text = "加载中".to_string();
                        
                        for i in 0..dot_count {
                            let phase = (time * 2.0 + i as f32 * 0.5) % 1.0;
                            if phase < 0.5 {
                                loading_text.push('●');
                            } else {
                                loading_text.push('○');
                            }
                        }
                        
                        ui.colored_label(egui::Color32::WHITE, loading_text);
                        
                        if let Some(difficulty) = self.selected_difficulty {
                            let difficulty_text = match difficulty {
                                GameDifficulty::Easy => "简单难度",
                                GameDifficulty::Medium => "中等难度", 
                                GameDifficulty::Hard => "困难难度",
                            };
                            ui.colored_label(egui::Color32::LIGHT_BLUE, difficulty_text);
                        }
                    });
                });
            });

            if self.transition_progress >= 1.0 {
                self.transition_timer = None;
                self.transition_progress = 0.0;
                self.transition_complete = true;
                return true;
            }
        }
        false
    }

    /// 检查过渡动画是否正在进行
    pub fn is_in_transition(&self) -> bool {
        self.transition_timer.is_some()
    }



    /// 重置状态
    pub fn reset(&mut self) {
        self.selected_difficulty = None;
        self.transition_timer = None;
        self.transition_progress = 0.0;
        self.transition_complete = false;
    }

    /// 创建居中的按钮
    fn centered_button(&self, ui: &mut egui::Ui, text: &str, width: f32, height: f32) -> egui::Response {
        ui.add_sized(
            egui::vec2(width, height),
            egui::Button::new(
                egui::RichText::new(text)
                    .text_style(egui::TextStyle::Button)
                    .color(egui::Color32::BLACK)
            )
        )
    }
}

impl Default for DifficultySelection {
    fn default() -> Self {
        Self::new()
    }
}