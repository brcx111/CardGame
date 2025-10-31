// game2.rs
use crate::card::Card;
use crate::difficulty::{DifficultySelection, GameDifficulty};
use crate::util;
use eframe::egui;
use rand::Rng;
use std::time::{Duration, Instant};

/// 猜数字游戏状态
#[derive(PartialEq)]
enum GuessNumberState {
    DifficultySelection, // 难度选择
    GamePlaying,         // 游戏进行中
    GameOver,           // 游戏结束
}

/// 猜数字游戏结构体
pub struct GuessNumberGame {
    state: GuessNumberState,
    difficulty_selection: DifficultySelection,
    target_number: Vec<u8>,           // 目标数字
    guesses: Vec<(Vec<u8>, String)>,  // 猜测记录 (猜测数字, 结果)
    current_guess: Vec<u8>,           // 当前猜测数字
    max_attempts: usize,              // 最大尝试次数
    attempts: usize,                  // 当前尝试次数
    game_won: bool,                   // 是否获胜
    digit_count: usize,               // 数字位数
    game_cards: Vec<Card>,            // 游戏卡片
    flipped_cards: Vec<usize>,        // 已翻开的卡片
    game_timer: Option<Instant>,      // 游戏计时器
    time_remaining: Duration,         // 剩余时间
}

impl GuessNumberGame {
    /// 创建新的猜数字游戏实例
    pub fn new() -> Self {
        Self {
            state: GuessNumberState::DifficultySelection,
            difficulty_selection: DifficultySelection::new(),
            target_number: Vec::new(),
            guesses: Vec::new(),
            current_guess: Vec::new(),
            max_attempts: 20,
            attempts: 0,
            game_won: false,
            digit_count: 4,
            game_cards: Vec::new(),
            flipped_cards: Vec::new(),
            game_timer: None,
            time_remaining: Duration::from_secs(0),
        }
    }

    /// 显示游戏界面，返回是否要返回主菜单
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> bool {
        let mut return_to_menu = false;

        // 处理过渡动画
        if self.difficulty_selection.is_in_transition() {
            if self.difficulty_selection.show_transition_animation(ui) {
                self.state = GuessNumberState::GamePlaying;
                self.initialize_game(ctx);
            }
            return return_to_menu;
        }

        // 更新游戏计时器
        if let Some(timer) = self.game_timer {
            let elapsed = timer.elapsed();
            let total_time = self.get_total_time();
            
            if elapsed < total_time {
                self.time_remaining = total_time - elapsed;
            } else {
                // 时间到，游戏结束
                self.time_remaining = Duration::from_secs(0);
                self.state = GuessNumberState::GameOver;
                self.game_won = false;
                self.game_timer = None;
                // 翻开所有卡片
                for card in &mut self.game_cards {
                    card.is_face_up = true;
                }
            }
        }

        // 使用 CentralPanel 确保内容始终居中
        egui::CentralPanel::default().show(ui.ctx(), |ui| {
            match self.state {
                GuessNumberState::DifficultySelection => {
                    let rules = vec![
                        "根据提示猜测4位数字（0-9，数字可以重复）",
                        "A表示数字和位置都正确，B表示数字正确但位置错误",
                        "简单难度: 20次尝试，180秒时间",
                        "中等难度: 15次尝试，120秒时间",
                        "困难难度: 10次尝试，90秒时间",
                    ];
                    let (menu_return, _) = 
                        self.difficulty_selection.show(ui, "猜数字游戏", &rules);
                    return_to_menu = menu_return;
                }
                GuessNumberState::GamePlaying => {
                    return_to_menu = self.show_game_playing(ui, ctx);
                }
                GuessNumberState::GameOver => {
                    return_to_menu = self.show_game_over(ui);
                }
            }
        });

        return_to_menu
    }

    /// 获取总游戏时间（根据难度）
    fn get_total_time(&self) -> Duration {
        match self.difficulty_selection.selected_difficulty {
            Some(GameDifficulty::Easy) => Duration::from_secs(180),   // 简单180秒
            Some(GameDifficulty::Medium) => Duration::from_secs(120), // 中等120秒
            Some(GameDifficulty::Hard) => Duration::from_secs(90),    // 困难90秒
            None => Duration::from_secs(180),
        }
    }

    /// 获取最大尝试次数（根据难度）
    fn get_max_attempts(&self) -> usize {
        match self.difficulty_selection.selected_difficulty {
            Some(GameDifficulty::Easy) => 20,    // 简单20次
            Some(GameDifficulty::Medium) => 15,  // 中等15次
            Some(GameDifficulty::Hard) => 10,    // 困难10次
            None => 20,
        }
    }

    /// 显示游戏进行界面
    fn show_game_playing(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> bool {
        let mut return_to_menu = false;

        // 初始化游戏
        if self.game_cards.is_empty() {
            self.initialize_game(ctx);
        }

        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            
            // 游戏标题和状态信息 
            ui.heading("猜数字游戏");
            ui.add_space(10.0);
            
            if let Some(difficulty) = self.difficulty_selection.selected_difficulty {
                let difficulty_text = match difficulty {
                    GameDifficulty::Easy => "简单难度",
                    GameDifficulty::Medium => "中等难度",
                    GameDifficulty::Hard => "困难难度",
                };
                ui.colored_label(egui::Color32::LIGHT_BLUE, difficulty_text);
                
                // 游戏状态信息 - 确保倒计时显示
                ui.horizontal(|ui| {
                    ui.label(format!("剩余尝试次数: {}/{}", self.max_attempts - self.attempts, self.max_attempts));
                    
                    // 显示倒计时 - 确保正确显示
                    let seconds_remaining = self.time_remaining.as_secs();
                    let minutes = seconds_remaining / 60;
                    let seconds = seconds_remaining % 60;
                    let time_text = if minutes > 0 {
                        format!("{}分{}秒", minutes, seconds)
                    } else {
                        format!("{}秒", seconds)
                    };
                    
                    let color = if seconds_remaining <= 10 {
                        egui::Color32::RED
                    } else if seconds_remaining <= 30 {
                        egui::Color32::YELLOW
                    } else {
                        egui::Color32::BLACK
                    };
                    ui.colored_label(color, format!("剩余时间: {}", time_text));
                });
            }
            
            ui.add_space(30.0);

            // 数字卡片显示区域 - 居中显示
            ui.label("目标数字卡片:");
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                let total_width = self.digit_count as f32 * 80.0 + (self.digit_count - 1) as f32 * 10.0;
                ui.add_space(ui.available_width() / 2.0 - total_width / 2.0);
                
                for i in 0..self.digit_count {
                    if i < self.game_cards.len() {
                        let card = &mut self.game_cards[i];
                        let response = card.show(ui, egui::vec2(70.0, 100.0));
                        
                        if response.clicked() && card.is_face_up {
                            card.is_face_up = false;
                        }
                    }
                    
                    if i < self.digit_count - 1 {
                        ui.add_space(10.0);
                    }
                }
            });

            ui.add_space(30.0);

            
            // 数字输入区域，居中显示
            ui.label("请输入您的猜测 (0-9，数字可以重复):");
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                let total_width = self.digit_count as f32 * 80.0;
                ui.add_space(ui.available_width() / 2.0 - total_width / 2.0);
                
                for i in 0..self.digit_count {
                    ui.vertical(|ui| {
                        ui.label(format!("第{}位", i + 1));
                        egui::ComboBox::from_id_salt(format!("digit_{}", i))
                            .width(60.0)
                            .selected_text(format!("{}", self.current_guess[i]))
                            .show_ui(ui, |ui| {
                                for digit in 0..=9 {
                                    ui.selectable_value(&mut self.current_guess[i], digit, format!("{}", digit));
                                }
                            });
                    });
                    
                    if i < self.digit_count - 1 {
                        ui.add_space(10.0);
                    }
                }
            });


            ui.add_space(30.0);

            // 控制按钮
            let button_width = 200.0;
            let button_height = 40.0;
            
            if self.centered_button(ui, "提交猜测", button_width, button_height).clicked() {
                self.submit_guess();
            }
            ui.add_space(15.0);

            if self.centered_button(ui, "重新开始", button_width, button_height).clicked() {
                self.reset_game_state();
                self.state = GuessNumberState::DifficultySelection;
            }
            ui.add_space(15.0);

            if self.centered_button(ui, "返回主菜单", button_width, button_height).clicked() {
                return_to_menu = true;
                self.reset_game_state();
                self.state = GuessNumberState::DifficultySelection;
            }

            ui.add_space(30.0);

            // 猜测历史记录 - 使用可滚动区域
            if !self.guesses.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.heading("猜测记录");
                    ui.add_space(10.0);
                    
                    // 使用可滚动区域显示猜测记录
                    egui::ScrollArea::vertical()
                        .max_height(200.0) 
                        .show(ui, |ui| {
                     
                            ui.horizontal(|ui| {
                                ui.label("次数");
                                ui.add_space(40.0);
                                ui.label("猜测数字");
                                ui.add_space(40.0);
                                ui.label("结果");
                            });
                            
                            ui.separator();
                            
                            // 猜测记录
                            for (index, (guess, result)) in self.guesses.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{}", index + 1));
                                    ui.add_space(30.0);
                                    ui.label(guess.iter().map(|d| d.to_string()).collect::<String>());
                                    ui.add_space(30.0);
                                    ui.colored_label(egui::Color32::LIGHT_BLUE, result.clone());
                                });
                            }
                        });
                });
            }

            ui.add_space(20.0);
        });

        return_to_menu
    }

    /// 显示游戏结束界面
    fn show_game_over(&mut self, ui: &mut egui::Ui) -> bool {
        let mut return_to_menu = false;

        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            
            // 游戏结果标题
            if self.game_won {
                ui.colored_label(egui::Color32::GOLD, "🎉 恭喜你猜对了！ 🎉");
            } else {
                ui.colored_label(egui::Color32::RED, "💀 游戏结束！ 💀");
            }
            
            ui.add_space(20.0);
            
            // 显示目标数字卡片 - 游戏结束界面也显示卡片
            ui.label("目标数字:");
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                let total_width = self.digit_count as f32 * 80.0 + (self.digit_count - 1) as f32 * 10.0;
                ui.add_space(ui.available_width() / 2.0 - total_width / 2.0);
                
                for i in 0..self.digit_count {
                    if i < self.game_cards.len() {
                        let card = &mut self.game_cards[i];
                        // 确保卡片是翻开状态
                        card.is_face_up = true;
                        card.show(ui, egui::vec2(70.0, 100.0));
                    }
                    
                    if i < self.digit_count - 1 {
                        ui.add_space(10.0);
                    }
                }
            });

            ui.add_space(10.0);
            ui.label(format!("目标数字: {}", 
                self.target_number.iter().map(|d| {
                    if *d == 10 { "0".to_string() } else { d.to_string() }
                }).collect::<String>()));
            
            if self.game_won {
                ui.label(format!("总共尝试次数: {}", self.attempts));
            } else {
                if self.attempts >= self.max_attempts {
                    ui.label("尝试次数用完了！");
                } else {
                    ui.label("时间到了！");
                }
            }
            
            ui.add_space(20.0);

            // 显示所有猜测记录
            if !self.guesses.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.heading("猜测记录");
                    ui.add_space(10.0);
                    // 使用可滚动区域显示猜测记录
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                // 猜测记录 
                                for (index, (guess, result)) in self.guesses.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.add_space(ui.available_width() / 2.0 - 70.0); 
                                        ui.label(format!("{}: ", index + 1));
                                        ui.label(guess.iter().map(|d| d.to_string()).collect::<String>());
                                        ui.label(" → ");
                                        ui.colored_label(egui::Color32::LIGHT_BLUE, result.clone());
                                    });
                                }
                            });
                        });
                });
                ui.add_space(30.0);
            }
            
            // 按钮
            let button_width = 200.0;
            let button_height = 40.0;
            
            if self.centered_button(ui, "再玩一次", button_width, button_height).clicked() {
                self.reset_game_state();
                self.state = GuessNumberState::DifficultySelection;
            }
            ui.add_space(15.0);

            if self.centered_button(ui, "返回主菜单", button_width, button_height).clicked() {
                return_to_menu = true;
                self.reset_game_state();
                self.state = GuessNumberState::DifficultySelection;
            }

            ui.add_space(20.0);
        });

        return_to_menu
    }

    /// 初始化游戏
    fn initialize_game(&mut self, ctx: &egui::Context) {
        self.generate_new_number();
        self.max_attempts = self.get_max_attempts();
        self.attempts = 0;
        self.game_won = false;
        self.game_cards.clear();
        self.flipped_cards.clear();
        
        // 创建数字卡片
        for (i, &digit) in self.target_number.iter().enumerate() {
            let suit = (i % 4) as u8 + 1; // 循环使用四种花色
            let card_value = if digit == 0 { 10 } else { digit }; // 0用10表示
            let face_path = util::get_card_image_path(card_value, suit);
            if let Ok(card) = Card::new(ctx, i as usize, card_value, suit, "assets/card_back/default.png", &face_path) {
                self.game_cards.push(card);
            }
        }
        
        // 启动游戏计时器
        self.game_timer = Some(Instant::now());
        self.time_remaining = self.get_total_time();
    }

    /// 生成新的目标数字
    fn generate_new_number(&mut self) {
        let mut rng = rand::rng();
        
        // 生成随机数字
        self.target_number = (0..self.digit_count)
            .map(|_| rng.random_range(0..=9))
            .collect();
        
        // 重置猜测状态
        self.guesses.clear();
        self.current_guess = vec![0; self.digit_count];
    }

    /// 提交猜测
    fn submit_guess(&mut self) {
        self.attempts += 1;

        // 计算A和B的数量
        let mut a_count = 0;
        let mut b_count = 0;
        
    
        let mut target_used = vec![false; self.digit_count];
        let mut guess_used = vec![false; self.digit_count];
        
        // 先计算A
        for i in 0..self.digit_count {
            if self.current_guess[i] == self.target_number[i] {
                a_count += 1;
                target_used[i] = true;
                guess_used[i] = true;
            }
        }
        
        // 再计算B（数字正确但位置错误）
        for i in 0..self.digit_count {
            if !guess_used[i] {
                for j in 0..self.digit_count {
                    if !target_used[j] && self.current_guess[i] == self.target_number[j] {
                        b_count += 1;
                        target_used[j] = true;
                        break;
                    }
                }
            }
        }

        // 保存猜测记录
        let result = format!("{}A{}B", a_count, b_count);
        self.guesses.push((self.current_guess.clone(), result));

        // 检查是否获胜
        if a_count == self.digit_count {
            self.game_won = true;
            self.state = GuessNumberState::GameOver;
            self.game_timer = None;
            // 翻开所有卡片
            for card in &mut self.game_cards {
                card.is_face_up = true;
            }
        } else if self.attempts >= self.max_attempts {
            self.state = GuessNumberState::GameOver;
            self.game_timer = None;
            // 翻开所有卡片
            for card in &mut self.game_cards {
                card.is_face_up = true;
            }
        }

        // 重置当前猜测
        self.current_guess = vec![0; self.digit_count];
    }

    /// 重置游戏状态
    fn reset_game_state(&mut self) {
        self.target_number.clear();
        self.guesses.clear();
        self.current_guess.clear();
        self.attempts = 0;
        self.game_won = false;
        self.game_cards.clear();
        self.flipped_cards.clear();
        self.difficulty_selection.reset();
        self.game_timer = None;
        self.time_remaining = Duration::from_secs(0);
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

impl Default for GuessNumberGame {
    fn default() -> Self {
        Self::new()
    }
}