// game1.rs
use crate::card::Card;
use crate::difficulty::{DifficultySelection, GameDifficulty};
use crate::util;
use eframe::egui;
use rand::seq::SliceRandom;
use std::time::{Duration, Instant};

/// 神经衰弱游戏状态
#[derive(PartialEq)]
enum MemoryGameState {
    DifficultySelection, // 难度选择
    GamePlaying,         // 游戏进行中
    GameOver,            // 游戏结束状态
}

/// 神经衰弱游戏结构体
pub struct MemoryGame {
    state: MemoryGameState,
    difficulty_selection: DifficultySelection,
    game_cards: Vec<Option<Card>>,
    flipped_cards: Vec<usize>,
    matched_pairs: usize,
    moves_count: usize,
    game_started: bool,
    main_menu_cards: Vec<Card>,
    check_timer: Option<Instant>,
    hovered_card: Option<usize>,
    game_timer: Option<Instant>,
    time_remaining: Duration,
    game_won: bool,
}

impl MemoryGame {
    /// 创建新的神经衰弱游戏实例
    pub fn new() -> Self {
        Self {
            state: MemoryGameState::DifficultySelection,
            difficulty_selection: DifficultySelection::new(),
            game_cards: Vec::new(),
            flipped_cards: Vec::new(),
            matched_pairs: 0,
            moves_count: 0,
            game_started: false,
            main_menu_cards: Vec::new(),
            check_timer: None,
            hovered_card: None,
            game_timer: None,
            time_remaining: Duration::from_secs(0),
            game_won: false,
        }
    }

    /// 设置主菜单卡片
    pub fn set_main_menu_cards(&mut self, cards: Vec<Card>) {
        self.main_menu_cards = cards;
    }

    /// 显示游戏界面，返回是否要返回主菜单
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> bool {
        let mut return_to_menu = false;

        // 处理过渡动画
        if self.difficulty_selection.is_in_transition() {
            if self.difficulty_selection.show_transition_animation(ui) {
                self.state = MemoryGameState::GamePlaying;
                self.initialize_game(ctx);
            }
            return return_to_menu;
        }

        // 检查是否需要处理匹配结果
        if let Some(timer) = self.check_timer {
            if timer.elapsed() >= Duration::from_millis(1000) {
                self.process_match_result();
                self.check_timer = None;
            }
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
                self.state = MemoryGameState::GameOver;
                self.game_won = false;
                self.game_timer = None;
            }
        }

        match self.state {
            MemoryGameState::DifficultySelection => {
                let rules = vec![
                    "匹配相同数字且同颜色的牌对",
                    "红桃和方片为红色，黑桃和梅花为黑色",
                    "简单难度: 8对牌，60秒时间",
                    "中等难度: 12对牌，90秒时间", 
                    "困难难度: 18对牌，120秒时间",
                    "在时间内匹配所有牌对即可获胜",
                ];
                let (menu_return, _) = 
                    self.difficulty_selection.show(ui, "神经衰弱游戏", &rules);
                return_to_menu = menu_return;
            }
            MemoryGameState::GamePlaying => {
                return_to_menu = self.show_game_playing(ui, ctx);
            }
            MemoryGameState::GameOver => {
                return_to_menu = self.show_game_over(ui);
            }
        }

        return_to_menu
    }

    /// 获取总游戏时间（根据难度）
    fn get_total_time(&self) -> Duration {
        match self.difficulty_selection.selected_difficulty {
            Some(GameDifficulty::Easy) => Duration::from_secs(60),   // 简单60秒
            Some(GameDifficulty::Medium) => Duration::from_secs(90), // 中等90秒
            Some(GameDifficulty::Hard) => Duration::from_secs(120),  // 困难120秒
            None => Duration::from_secs(60),
        }
    }

    /// 显示游戏结束界面
    fn show_game_over(&mut self, ui: &mut egui::Ui) -> bool {
        let mut return_to_menu = false;

        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 2.0 - 100.0);
            
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 150.0);
                ui.vertical(|ui| {
                    if self.game_won {
                        ui.colored_label(egui::Color32::GOLD, "恭喜你赢了！");
                        ui.label(format!("总共移动次数: {}", self.moves_count));
                    } else {
                        ui.colored_label(egui::Color32::RED, "时间到！游戏失败");
                        ui.label(format!("完成进度: {}/{} 对", self.matched_pairs, self.get_total_pairs()));
                    }
                    
                    ui.add_space(20.0);
                    
                    ui.horizontal(|ui| {
                        if self.centered_button(ui, "重新开始", 120.0, 40.0).clicked() {
                            self.reset_game_state();
                            self.state = MemoryGameState::DifficultySelection;
                        }
                        
                        ui.add_space(20.0);
                        
                        if self.centered_button(ui, "返回主菜单", 120.0, 40.0).clicked() {
                            return_to_menu = true;
                            self.reset_game_state();
                            self.state = MemoryGameState::DifficultySelection;
                        }
                    });
                });
            });
        });

        return_to_menu
    }

    /// 获取总对数
    fn get_total_pairs(&self) -> usize {
        match self.difficulty_selection.selected_difficulty {
            Some(GameDifficulty::Easy) => 8,    // 8对
            Some(GameDifficulty::Medium) => 12, // 12对
            Some(GameDifficulty::Hard) => 18,   // 18对
            None => 8,
        }
    }

    /// 显示游戏进行界面
    fn show_game_playing(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> bool {
        let mut return_to_menu = false;

        if !self.game_started {
            self.game_started = true;
        }

        // 使用滚动区域来适应不同屏幕大小
        egui::ScrollArea::vertical()
            .max_height(ui.available_height())
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    
                    ui.horizontal(|ui| {
                        ui.add_space(ui.available_width() / 2.0 - 150.0);
                        ui.vertical(|ui| {
                            ui.heading("神经衰弱游戏");
                            ui.add_space(10.0);
                            
                            if let Some(difficulty) = self.difficulty_selection.selected_difficulty {
                                let (difficulty_text, total_pairs) = match difficulty {
                                    GameDifficulty::Easy => ("简单难度", 8),    // 8对
                                    GameDifficulty::Medium => ("中等难度", 12), // 12对
                                    GameDifficulty::Hard => ("困难难度", 18),   // 18对
                                };
                                ui.colored_label(egui::Color32::LIGHT_BLUE, difficulty_text);
                                ui.label(format!("进度: {}/{} 对", self.matched_pairs, total_pairs));
                                ui.label(format!("移动次数: {}", self.moves_count));
                                ui.label(format!("剩余卡片: {} 张", self.get_remaining_cards_count()));
                                
                                // 显示倒计时
                                let seconds_remaining = self.time_remaining.as_secs();
                                let color = if seconds_remaining <= 10 {
                                    egui::Color32::RED
                                } else if seconds_remaining <= 30 {
                                    egui::Color32::YELLOW
                                } else {
                                    egui::Color32::BLACK
                                };
                                ui.colored_label(color, format!("剩余时间: {}秒", seconds_remaining));
                            }
                            
                            ui.add_space(10.0);
                        });
                    });

                    ui.add_space(20.0);

                    // 游戏卡片网格
                    ui.horizontal(|ui| {
                        let grid_size = match self.difficulty_selection.selected_difficulty {
                            Some(GameDifficulty::Easy) => (4, 4),   // 4×4网格
                            Some(GameDifficulty::Medium) => (4, 6), // 4×6网格
                            Some(GameDifficulty::Hard) => (4, 9),   // 困难难度改为4×9网格
                            None => (4, 4),
                        };

                        let grid_width = (grid_size.1 as f32) * 78.0;
                        ui.add_space((ui.available_width() - grid_width) / 2.0);

                        egui::Grid::new("memory_game_grid")
                            .spacing(egui::vec2(8.0, 8.0))
                            .show(ui, |ui| {
                                for i in 0..grid_size.0 {
                                    for j in 0..grid_size.1 {
                                        let index = i * grid_size.1 + j;
                                        if index < self.game_cards.len() {
                                            if let Some(card) = &mut self.game_cards[index] {
                                                let is_hovered = self.hovered_card == Some(index);
                                                
                                                let base_size = egui::vec2(70.0, 100.0);
                                                let display_size = if is_hovered && !card.is_face_up && self.flipped_cards.len() < 2 && self.check_timer.is_none() {
                                                    base_size * 1.05
                                                } else {
                                                    base_size
                                                };
                                                
                                                let response = card.show(ui, display_size);
                                                
                                                if response.hovered() && !card.is_face_up && self.flipped_cards.len() < 2 && self.check_timer.is_none() {
                                                    self.hovered_card = Some(index);
                                                } else if self.hovered_card == Some(index) && !response.hovered() {
                                                    self.hovered_card = None;
                                                }
                                                
                                                if response.clicked() {
                                                    if self.check_timer.is_some() && !self.flipped_cards.contains(&index)  {
                                                        card.is_face_up = false;
                                                    } else if self.flipped_cards.len() < 2 
                                                        && card.is_face_up 
                                                        && !self.flipped_cards.contains(&index) 
                                                    {
                                                        self.flipped_cards.push(index);
                                                        
                                                        if self.flipped_cards.len() == 2 {
                                                            self.moves_count += 1;
                                                            self.check_timer = Some(Instant::now());
                                                        }
                                                    }
                                                }
                                            } else {
                                                ui.allocate_space(egui::vec2(70.0, 100.0));
                                            }
                                        } else {
                                            ui.allocate_space(egui::vec2(70.0, 100.0));
                                        }
                                    }
                                    ui.end_row();
                                }
                            });
                    });

                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        let buttons_width = 120.0 * 3.0 + 20.0 * 2.0;
                        ui.add_space((ui.available_width() - buttons_width) / 2.0);
                        
                        if self.centered_button(ui, "重新开始", 120.0, 40.0).clicked() {
                            self.initialize_game(ctx);
                        }
                        
                        ui.add_space(20.0);
                        
                        if self.centered_button(ui, "选择难度", 120.0, 40.0).clicked() {
                            self.state = MemoryGameState::DifficultySelection;
                            self.reset_game_state();
                        }
                        
                        ui.add_space(20.0);
                        
                        if self.centered_button(ui, "返回主菜单", 120.0, 40.0).clicked() {
                            return_to_menu = true;
                            self.reset_game_state();
                            self.state = MemoryGameState::DifficultySelection;
                        }
                    });

                    let total_pairs = self.get_total_pairs();
                    if self.matched_pairs >= total_pairs {
                        self.game_won = true;
                        self.state = MemoryGameState::GameOver;
                        self.game_timer = None;
                    }

                    ui.add_space(20.0);
                });
            });

        return_to_menu
    }

    /// 获取剩余卡片数量
    fn get_remaining_cards_count(&self) -> usize {
        self.game_cards.iter().filter(|card| card.is_some()).count()
    }

    /// 创建文字居中的按钮
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

    /// 处理匹配结果
    fn process_match_result(&mut self) {
        if self.flipped_cards.len() == 2 {
            if let (Some(Some(card1)), Some(Some(card2))) = (
                self.game_cards.get(self.flipped_cards[0]),
                self.game_cards.get(self.flipped_cards[1])
            ) {
                if card1.rank == card2.rank && Self::is_same_color(card1.suit, card2.suit) {
                    self.matched_pairs += 1;
                    
                    for &index in &self.flipped_cards {
                        if index < self.game_cards.len() {
                            self.game_cards[index] = None;
                        }
                    }
                    
                    self.flipped_cards.clear();
                } else {
                    let flipped_copy = self.flipped_cards.clone();
                    self.flipped_cards.clear();
                    
                    for &index in &flipped_copy {
                        if let Some(Some(card)) = self.game_cards.get_mut(index) {
                            card.is_face_up = false;
                        }
                    }
                }
            }
        }
    }

    /// 检查两张卡片是否同颜色
    fn is_same_color(suit1: u8, suit2: u8) -> bool {
        let is_red1 = suit1 == 2 || suit1 == 3;
        let is_red2 = suit2 == 2 || suit2 == 3;
        is_red1 == is_red2
    }

    /// 初始化游戏
    fn initialize_game(&mut self, ctx: &egui::Context) {
        self.game_cards.clear();
        self.flipped_cards.clear();
        self.matched_pairs = 0;
        self.moves_count = 0;
        self.check_timer = None;
        self.hovered_card = None;
        self.game_won = false;
        
        let pairs_count = match self.difficulty_selection.selected_difficulty {
            Some(GameDifficulty::Easy) => 8,    // 8对
            Some(GameDifficulty::Medium) => 12, // 12对
            Some(GameDifficulty::Hard) => 18,   // 18对
            None => 8,
        };
        
        let total_cards = pairs_count * 2;
        self.game_cards = vec![None; total_cards];
        
        let mut all_possible_pairs = Vec::new();
        
        for rank in 1..=13 {
            let red_suits = vec![2, 3];
            for i in 0..red_suits.len() {
                for j in (i + 1)..red_suits.len() {
                    all_possible_pairs.push((rank, red_suits[i], red_suits[j]));
                }
            }
            
            let black_suits = vec![1, 4];
            for i in 0..black_suits.len() {
                for j in (i + 1)..black_suits.len() {
                    all_possible_pairs.push((rank, black_suits[i], black_suits[j]));
                }
            }
        }
        
        let mut rng = rand::rng();
        all_possible_pairs.shuffle(&mut rng);
        
        for (pair_id, &(rank, suit1, suit2)) in all_possible_pairs.iter().take(pairs_count).enumerate() {
            let face_path1 = util::get_card_image_path(rank, suit1);
            let face_path2 = util::get_card_image_path(rank, suit2);
            
            if let Ok(card1) = Card::new(ctx, pair_id * 2, rank, suit1, "assets/card_back/default.png", &face_path1) {
                self.game_cards[pair_id * 2] = Some(card1);
            }
            
            if let Ok(card2) = Card::new(ctx, pair_id * 2 + 1, rank, suit2, "assets/card_back/default.png", &face_path2) {
                self.game_cards[pair_id * 2 + 1] = Some(card2);
            }
        }
        
        self.shuffle_cards();
        
        // 启动游戏计时器
        self.game_timer = Some(Instant::now());
        self.time_remaining = self.get_total_time();
    }

    /// 洗牌
    fn shuffle_cards(&mut self) {
        let mut rng = rand::rng();
        
        let mut cards: Vec<Card> = self.game_cards
            .iter_mut()
            .filter_map(|card| card.take())
            .collect();
        
        cards.shuffle(&mut rng);
        
        let mut card_iter = cards.into_iter();
        for slot in &mut self.game_cards {
            *slot = card_iter.next();
        }
    }

    /// 重置游戏状态
    fn reset_game_state(&mut self) {
        self.game_cards.clear();
        self.flipped_cards.clear();
        self.matched_pairs = 0;
        self.moves_count = 0;
        self.game_started = false;
        self.difficulty_selection.reset();
        self.check_timer = None;
        self.hovered_card = None;
        self.game_timer = None;
        self.time_remaining = Duration::from_secs(0);
        self.game_won = false;
    }
}

impl Default for MemoryGame {
    fn default() -> Self {
        Self::new()
    }
}