// game3.rs - 德州扑克游戏（标准52张牌版）
use eframe::egui;
use crate::card::Card;
use crate::difficulty::{DifficultySelection, GameDifficulty};
use crate::util::get_card_image_path;
use std::collections::VecDeque;
use std::time::Instant;
use rand::seq::SliceRandom;
use rand::thread_rng;

/// 德州扑克游戏状态
pub struct TexasHoldemGame {
    // 游戏难度设置
    difficulty_selection: DifficultySelection,
    
    // 游戏核心状态（可能包含玩家状态、回合信息等）
    game_state: TexasHoldemState,
    
    // 玩家手牌
    player_hand: Vec<Card>,
    
    // AI手牌
    ai_hand: Vec<Card>,
    
    // 公共牌（桌面上的牌）
    community_cards: Vec<Card>,
    
    // 牌堆，使用VecDeque便于从前面抽牌
    deck: VecDeque<Card>,
    
    // 玩家筹码数量
    player_chips: i32,
    
    // AI筹码数量
    ai_chips: i32,
    
    // 底池总金额
    pot: u32,
    
    // 当前回合的最低跟注额
    current_bet: u32,
    
    // 当前游戏阶段（预翻牌、翻牌、转牌、河牌等）
    game_phase: GamePhase,
    
    // 显示给玩家的消息（如游戏结果、提示信息等）
    message: String,
    
    // 游戏是否结束的标志
    game_over: bool,
    
    // egui上下文，用于界面渲染和更新
    ctx: Option<egui::Context>,
    
    // 游戏是否正在初始化的标志
    game_initializing: bool,
    
    // 是否显示AI的牌（用于调试或游戏结束时）
    show_ai_cards: bool,
    
    // 是否正在等待AI做出决策
    waiting_for_ai: bool,
    
    // AI思考计时器，用于模拟AI思考时间
    ai_thinking_timer: Option<Instant>,
    
    // 牌背纹理句柄，用于渲染卡牌背面
    back_card_texture: Option<egui::TextureHandle>,
    
    // 标记玩家在当前回合是否已经行动
    player_acted: bool,
    
    // 标记AI在当前回合是否已经行动
    ai_acted: bool,
    
    // 标记双方是否都选择了过牌
    both_checked: bool,
    
    // 标记是否是第一轮下注
    first_round: bool,
    
    // 标记玩家是否已经使用过特殊行动
    has_used_special_action: bool,
}

/// 德州扑克游戏阶段
#[derive(PartialEq, Clone, Copy)]
enum GamePhase {
    PreFlop,
    Flop,
    Turn,
    River,
    Showdown,
}

/// 德州扑克游戏状态
#[derive(PartialEq, Clone, Copy)]
enum TexasHoldemState {
    DifficultySelection,
    Initializing,
    Playing,
}

/// 卡片标识
#[derive(Clone, Copy, Debug)]
struct CardId {
    rank: u8,
    suit: u8,
}

impl TexasHoldemGame {
    pub fn new() -> Self {
        Self {
            difficulty_selection: DifficultySelection::new(),
            game_state: TexasHoldemState::DifficultySelection,
            player_hand: Vec::new(),
            ai_hand: Vec::new(),
            community_cards: Vec::new(),
            deck: VecDeque::new(),
            player_chips: 200,
            ai_chips: 100,
            pot: 0,
            current_bet: 0,
            game_phase: GamePhase::PreFlop,
            message: "欢迎来到德州扑克！".to_string(),
            game_over: false,
            ctx: None,
            game_initializing: false,
            show_ai_cards: false,
            waiting_for_ai: false,
            ai_thinking_timer: None,
            back_card_texture: None,
            player_acted: false,
            ai_acted: false,
            both_checked: false,
            first_round: true,
            has_used_special_action: false,
        }
    }

    /// 设置主菜单卡片（只用于获取背面纹理）
    pub fn set_main_menu_cards(&mut self, cards: Vec<Card>) {
        if let Some(card) = cards.first() {
            self.back_card_texture = Some(card.back_tex.clone());
        }
    }

    /// 显示游戏界面
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> bool {
        if self.ctx.is_none() {
            self.ctx = Some(ctx.clone());
        }

        if self.waiting_for_ai {
            if let Some(timer) = self.ai_thinking_timer {
                if timer.elapsed().as_millis() > 500 {
                    self.waiting_for_ai = false;
                    self.ai_thinking_timer = None;
                    self.perform_ai_action();
                }
            }
        }

        let mut return_to_menu = false;

        match self.game_state {
            TexasHoldemState::DifficultySelection => {
                let rules = vec![
                    "每位玩家发2张底牌，然后依次进行5张公共牌的发牌",
                    "游戏分为四个下注回合：翻牌前、翻牌、转牌、河牌",
                    "通过组合7张牌（2张底牌+5张公共牌）形成最好的5张牌组合",
                    "牌型大小：同花顺 > 四条 > 葫芦 > 同花 > 顺子 > 三条 > 两对 > 一对 > 高牌",
                    "固定下注规则：每阶段固定下注额，逐阶段递增",
                    "支持负数筹码，结算后筹码≤0判负",
                    "每小局游戏中可以使用一次过牌或弃牌",
                    "使用标准52张扑克牌（无鬼牌）",
                    "简单难度：AI无脑下注，中等难度：AI根据手牌决定，困难难度：AI会诈唬",
                ];

                let (menu_return, game_start) = 
                    self.difficulty_selection.show(ui, "德州扑克", &rules);

                if menu_return {
                    return_to_menu = true;
                    self.reset_to_main_menu();
                }

                if game_start {
                    self.game_state = TexasHoldemState::Initializing;
                    self.game_initializing = true;
                    self.difficulty_selection.transition_timer = Some(Instant::now());
                }
            }
            TexasHoldemState::Initializing => {
                if self.difficulty_selection.show_transition_animation(ui) {
                    self.start_game_fast();
                    self.game_state = TexasHoldemState::Playing;
                    self.game_initializing = false;
                }
            }
            TexasHoldemState::Playing => {
                self.show_game_ui(ui);
            }
        }

        return_to_menu
    }

    /// 显示游戏主界面
    fn show_game_ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            // 顶部信息区域
            self.show_top_info(ui);
            
            // AI手牌区域
            self.show_ai_hand(ui);
            
            ui.add_space(10.0);

            // 公共牌区域
            self.show_community_cards(ui);
            
            ui.add_space(10.0);

            // 玩家手牌区域
            self.show_player_hand(ui);
            
            ui.add_space(20.0);

            // 游戏消息区域
            self.show_game_message(ui);
            
            ui.add_space(20.0);

            // 操作按钮区域 ，根据游戏状态显示不同的按钮
            self.show_action_buttons(ui);

            // 游戏进行中始终显示选择难度按钮
            if !self.game_over {
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.add_space(ui.available_width() / 2.0 - 75.0);
                    if self.centered_button(ui, "选择难度", 150.0, 35.0).clicked() {
                        self.reset_to_difficulty_selection();
                    }
                });
            }
        });
    }

    /// 显示顶部信息
    fn show_top_info(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // 左侧筹码信息
            ui.vertical(|ui| {
                // 根据筹码正负显示不同颜色
                let player_color = if self.player_chips >= 0 {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::RED
                };
                let ai_color = if self.ai_chips >= 0 {
                    egui::Color32::RED
                } else {
                    egui::Color32::GREEN
                };

                ui.colored_label(player_color, format!("你的筹码: {}", self.player_chips));
                ui.colored_label(ai_color, format!("AI筹码: {}", self.ai_chips));
                ui.colored_label(egui::Color32::GOLD, format!("底池: {}", self.pot));
                ui.colored_label(egui::Color32::LIGHT_BLUE, format!("当前下注: {}", self.current_bet));
                
                // 显示特殊行动使用状态
                if self.has_used_special_action {
                    ui.colored_label(egui::Color32::ORANGE, "本局已使用过特殊行动");
                } else {
                    ui.colored_label(egui::Color32::LIGHT_GREEN, "本局还可使用过牌/弃牌");
                }
            });
            
            // 右侧游戏状态信息
            ui.add_space(ui.available_width() - 300.0);
            ui.vertical(|ui| {
                let phase_text = match self.game_phase {
                    GamePhase::PreFlop => "翻牌前",
                    GamePhase::Flop => "翻牌圈",
                    GamePhase::Turn => "转牌圈", 
                    GamePhase::River => "河牌圈",
                    GamePhase::Showdown => "摊牌",
                };
                ui.colored_label(egui::Color32::BLACK, format!("阶段: {}", phase_text));

                if let Some(difficulty) = self.difficulty_selection.selected_difficulty {
                    let difficulty_text = match difficulty {
                        GameDifficulty::Easy => "简单难度",
                        GameDifficulty::Medium => "中等难度",
                        GameDifficulty::Hard => "困难难度",
                    };
                    ui.colored_label(egui::Color32::DARK_GRAY, difficulty_text);
                }
            });
        });

        // 游戏结束提示
        if self.game_over {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 150.0);
                if self.player_chips <= 0 {
                    ui.colored_label(egui::Color32::RED, "游戏结束！你的筹码≤0，你输了！");
                } else if self.ai_chips <= 0 {
                    ui.colored_label(egui::Color32::GREEN, "游戏结束！AI筹码≤0，你赢了！");
                }
            });
        }
    }

    /// 显示AI手牌
    fn show_ai_hand(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(ui.available_width() / 2.0 - 50.0);
            ui.label("AI手牌:");
        });
        ui.horizontal(|ui| {
            ui.add_space(ui.available_width() / 2.0 - 100.0);
            for card in &mut self.ai_hand {
                // 摊牌阶段或游戏结束时显示AI手牌
                card.is_face_up = self.show_ai_cards || self.game_phase == GamePhase::Showdown || self.game_over;
                card.show(ui, egui::vec2(80.0, 120.0));
            }
        });
    }

    /// 显示公共牌
    fn show_community_cards(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(ui.available_width() / 2.0 - 40.0);
            ui.label("公共牌:");
        });
        ui.horizontal(|ui| {
            let cards_to_reveal = match self.game_phase {
                GamePhase::PreFlop => 0,
                GamePhase::Flop => 3,
                GamePhase::Turn => 4,
                GamePhase::River => 5,
                GamePhase::Showdown => 5,
            };

            let total_width = 80.0 * 5.0 + 10.0 * 4.0;
            ui.add_space((ui.available_width() - total_width) / 2.0);

            for i in 0..5 {
                if i < self.community_cards.len() {
                    let mut temp_card = self.community_cards[i].clone();
                    temp_card.is_face_up = i < cards_to_reveal;
                    temp_card.show(ui, egui::vec2(80.0, 120.0));
                } else {
                    if let Some(back_card) = self.create_back_card_fast() {
                        let mut temp_card = back_card;
                        temp_card.is_face_up = false;
                        temp_card.show(ui, egui::vec2(80.0, 120.0));
                    }
                }
            }
        });
    }

    /// 显示玩家手牌
    fn show_player_hand(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(ui.available_width() / 2.0 - 40.0);
            ui.label("你的手牌:");
        });
        ui.horizontal(|ui| {
            ui.add_space(ui.available_width() / 2.0 - 110.0);
            for card in &mut self.player_hand {
                card.is_face_up = true;
                card.show(ui, egui::vec2(100.0, 150.0));
            }
        });
    }

    /// 显示游戏消息
    fn show_game_message(&mut self, ui: &mut egui::Ui) {
        if self.waiting_for_ai {
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 80.0);
                ui.colored_label(egui::Color32::BLACK, "AI正在思考中...");
            });
        } else {
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - (self.message.len() as f32 * 4.0));
                ui.colored_label(egui::Color32::BLACK, &self.message);
            });
        }
    }

    /// 显示操作按钮
    fn show_action_buttons(&mut self, ui: &mut egui::Ui) {
        // 根据游戏状态决定显示什么按钮
        if self.game_over {
            // 游戏结束状态：显示选择难度按钮
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 75.0);
                if self.centered_button(ui, "选择难度", 150.0, 40.0).clicked() {
                    self.reset_to_difficulty_selection();
                }
            });
        } else if self.game_phase == GamePhase::Showdown {
            // 摊牌阶段：显示下一局按钮
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 75.0);
                if self.centered_button(ui, "下一局", 150.0, 40.0).clicked() {
                    self.start_next_round();
                }
            });
        } else if !self.waiting_for_ai {
            // 正常游戏阶段：显示游戏操作按钮
            self.show_game_action_buttons(ui);
        } else {
            // AI思考中居中显示等待信息
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 75.0);
                ui.colored_label(egui::Color32::BLUE, "等待AI行动...");
            });
        }
    }

    /// 显示游戏操作按钮
    fn show_game_action_buttons(&mut self, ui: &mut egui::Ui) {
        let fixed_bet = self.get_fixed_bet_amount();

        if self.player_acted {
            // 玩家已经行动
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 75.0);
                ui.colored_label(egui::Color32::BLUE, "等待AI行动...");
            });
        } else {
            // 玩家需要行动
            ui.horizontal(|ui| {
             
                let can_use_special = !self.first_round && !self.has_used_special_action;
                let buttons_count = if can_use_special { 3 } else { 1 };
                let total_width = 120.0 * buttons_count as f32 + 20.0 * (buttons_count - 1) as f32;
                ui.add_space((ui.available_width() - total_width) / 2.0);

                // 下注按钮
                if self.centered_button(ui, &format!("下注 ({})", fixed_bet), 120.0, 40.0).clicked() {
                    self.place_bet(fixed_bet);
                    self.player_acted = true;
                    self.start_ai_thinking();
                }

                // 只有在不是第一回合且未使用过特殊行动时才显示过牌和弃牌
                if can_use_special {
                    ui.add_space(20.0);
                    
                    // 过牌按钮
                    if self.centered_button(ui, "过牌", 120.0, 40.0).clicked() {
                        self.player_check();
                        self.player_acted = true;
                        self.has_used_special_action = true;
                        self.start_ai_thinking();
                    }
                    
                    ui.add_space(20.0);
                    
                    // 弃牌按钮
                    if self.centered_button(ui, "弃牌", 120.0, 40.0).clicked() {
                        self.player_fold();
                        self.has_used_special_action = true;
                    }
                }
            });
        }
    }

    /// 获取固定下注金额
    fn get_fixed_bet_amount(&self) -> u32 {
        match self.game_phase {
            GamePhase::PreFlop => 10,
            GamePhase::Flop => 20,
            GamePhase::Turn => 30,
            GamePhase::River => 40,
            GamePhase::Showdown => 0,
        }
    }

    /// 创建居中的按钮
    fn centered_button(&self, ui: &mut egui::Ui, text: &str, width: f32, height: f32) -> egui::Response {
        ui.add_sized(
            egui::vec2(width, height),
            egui::Button::new(
                egui::RichText::new(text)
                    .text_style(egui::TextStyle::Button)
                    .color(egui::Color32::BLACK)
                    .size(14.0)
            )
        )
    }

    /// 快速创建背面卡片
    fn create_back_card_fast(&self) -> Option<Card> {
        if let Some(back_tex) = &self.back_card_texture {
            Some(Card {
                id: 999,
                is_face_up: false,
                rank: 1,
                suit: 1,
                back_tex: back_tex.clone(),
                face_tex: back_tex.clone(),
            })
        } else {
            None
        }
    }

    /// 开始AI思考
    fn start_ai_thinking(&mut self) {
        self.waiting_for_ai = true;
        self.ai_thinking_timer = Some(Instant::now());
    }

    /// 执行AI行动
    fn perform_ai_action(&mut self) {
        if self.ai_acted {
            return;
        }

        let action = self.calculate_ai_action();
        
        match action {
            AiAction::Fold => {
                self.message.push_str("\nAI选择弃牌");
                self.show_ai_cards = true;
                self.player_chips += self.pot as i32;
                self.pot = 0;
                self.ai_acted = true;
                self.check_round_end();
            }
            AiAction::Check => {
                self.message.push_str("\nAI选择过牌");
                self.ai_acted = true;
                self.both_checked = self.player_acted && self.ai_acted;
                self.check_round_end();
            }
            AiAction::Bet(amount) => {
                // AI也允许负数下注
                self.ai_chips -= amount as i32;
                self.pot += amount;
                self.current_bet = amount;
                self.message.push_str(&format!("\nAI下注 {} 筹码", amount));
                self.ai_acted = true;
                self.check_round_end();
            }
        }
    }

    /// AI行为逻辑
    fn calculate_ai_action(&self) -> AiAction {
        use rand::Rng;
        let mut rng = rand::rng();
        
        let fixed_bet = self.get_fixed_bet_amount();
        
        if let Some(difficulty) = self.difficulty_selection.selected_difficulty {
            match difficulty {
                GameDifficulty::Easy => {
                    // 简单AI：总是下注
                    AiAction::Bet(fixed_bet)
                }
                GameDifficulty::Medium => {
                    // 中等AI：总是下注，不会同意过牌
                    AiAction::Bet(fixed_bet)
                }
                GameDifficulty::Hard => {
                    // 困难AI：50%概率同意过牌，否则下注
                    if self.player_acted && rng.random_bool(0.5) {
                        AiAction::Check
                    } else {
                        AiAction::Bet(fixed_bet)
                    }
                }
            }
        } else {
            // 默认总是下注
            AiAction::Bet(fixed_bet)
        }
    }

    /// 玩家下注，允许负数
    fn place_bet(&mut self, amount: u32) {

        self.player_chips -= amount as i32;
        self.pot += amount;
        self.current_bet = amount;
        self.message = format!("你下注了 {} 筹码", amount);
        
        // 检查是否立即输掉游戏
        if self.player_chips <= 0 {
            self.message.push_str(&format!("\n你的筹码为{}，游戏结束！", self.player_chips));
            self.game_over = true;
        }
    }

    /// 玩家过牌
    fn player_check(&mut self) {
        self.message = "你选择了过牌".to_string();
        self.both_checked = self.player_acted && self.ai_acted;
    }

    /// 玩家弃牌
    fn player_fold(&mut self) {
        self.message = "你选择了弃牌，游戏结束！".to_string();
        self.show_ai_cards = true;
        self.ai_chips += self.pot as i32;
        self.pot = 0;
        self.check_game_end();
    }

    /// 检查回合是否结束
    fn check_round_end(&mut self) {
        if self.player_acted && self.ai_acted {
            if self.both_checked {
                self.message.push_str("\n双方都选择过牌，进入下一阶段");
                self.advance_phase();
            } else {
                self.advance_phase();
            }
        }
    }

    /// 快速启动游戏
    fn start_game_fast(&mut self) {
        self.initialize_deck_fast();
        self.deal_cards();
        
        self.message = "游戏开始！第一回合由你先下注。".to_string();
        self.game_over = false;
        self.show_ai_cards = false;
        self.waiting_for_ai = false;
        self.player_acted = false;
        self.ai_acted = false;
        self.first_round = true;
        self.has_used_special_action = false;
    }

    /// 快速初始化牌堆 - 只生成9张牌
    fn initialize_deck_fast(&mut self) {
        self.deck.clear();

        let  card_ids = self.generate_random_card_ids(9);
        
        for (id, card_id) in card_ids.iter().enumerate() {
            if let Ok(card) = self.create_card_fast(id, card_id.rank, card_id.suit) {
                self.deck.push_back(card);
            }
        }
    }

    /// 生成随机牌标识 - 只使用标准52张
    fn generate_random_card_ids(&self, count: usize) -> Vec<CardId> {
        let mut all_possible_cards = Vec::new();
        
        // 只生成标准52张牌：4种花色 × 13个点数
        for suit in 1..=4 {
            for rank in 1..=13 {
                all_possible_cards.push(CardId { rank, suit });
            }
        }
        
        let mut rng = rand::rng();
        all_possible_cards.shuffle(&mut rng);
        
        all_possible_cards.into_iter().take(count).collect()
    }

    /// 创建卡片 - 直接通过 get_card_image_path 加载纹理
    fn create_card_fast(&mut self, id: usize, rank: u8, suit: u8) -> Result<Card, image::ImageError> {
        if let Some(ctx) = &self.ctx {
            let face_path = get_card_image_path(rank, suit);
            
            // 使用默认背面纹理
            let back_path = "assets/card_back/default.png";
            
            Card::new(ctx, id, rank, suit, back_path, &face_path)
        } else {
            Err(image::ImageError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Context not available",
            )))
        }
    }

    /// 发牌
    fn deal_cards(&mut self) {
        self.player_hand.clear();
        self.ai_hand.clear();
        self.community_cards.clear();

        for _ in 0..2 {
            if let Some(card) = self.deck.pop_front() {
                self.player_hand.push(card);
            }
        }

        for _ in 0..2 {
            if let Some(card) = self.deck.pop_front() {
                self.ai_hand.push(card);
            }
        }

        for _ in 0..5 {
            if let Some(card) = self.deck.pop_front() {
                self.community_cards.push(card);
            }
        }
    }

    /// 进入下一阶段
    fn advance_phase(&mut self) {
        self.player_acted = false;
        self.ai_acted = false;
        self.both_checked = false;
        self.current_bet = 0;
        self.first_round = false;

        self.game_phase = match self.game_phase {
            GamePhase::PreFlop => {
                self.message.push_str("\n翻牌阶段！显示3张公共牌。");
                GamePhase::Flop
            }
            GamePhase::Flop => {
                self.message.push_str("\n转牌阶段！显示第4张公共牌。");
                GamePhase::Turn
            }
            GamePhase::Turn => {
                self.message.push_str("\n河牌阶段！显示第5张公共牌。");
                GamePhase::River
            }
            GamePhase::River => {
                self.message.push_str("\n进入摊牌阶段！");
                self.show_ai_cards = true;
                self.evaluate_showdown();
                GamePhase::Showdown
            }
            GamePhase::Showdown => GamePhase::Showdown,
        };
    }

    /// 评估摊牌
    fn evaluate_showdown(&mut self) {
        let player_result = self.evaluate_best_hand(&self.player_hand, &self.community_cards);
        let ai_result = self.evaluate_best_hand(&self.ai_hand, &self.community_cards);

        let comparison = self.compare_hands(&player_result, &ai_result);

        match comparison {
            std::cmp::Ordering::Greater => {
                self.message.push_str(&format!("\n你赢了！{} > {}", 
                    player_result.hand_strength.to_string(), 
                    ai_result.hand_strength.to_string()));
                self.player_chips += self.pot as i32;
            }
            std::cmp::Ordering::Less => {
                self.message.push_str(&format!("\nAI赢了！{} > {}", 
                    ai_result.hand_strength.to_string(), 
                    player_result.hand_strength.to_string()));
                self.ai_chips += self.pot as i32;
            }
            std::cmp::Ordering::Equal => {
                self.message.push_str(&format!("\n平局！双方都是{}", 
                    player_result.hand_strength.to_string()));
                let half_pot = self.pot / 2;
                self.player_chips += half_pot as i32;
                self.ai_chips += half_pot as i32;
            }
        }

        self.pot = 0;
        self.check_game_end();
    }

    /// 检查游戏是否结束 
    fn check_game_end(&mut self) {
        if self.player_chips <= 0 || self.ai_chips <= 0 {
            self.game_over = true;
        }
    }

    /// 开始下一局游戏
    fn start_next_round(&mut self) {
        self.player_hand.clear();
        self.ai_hand.clear();
        self.community_cards.clear();
        self.deck.clear();
        self.pot = 0;
        self.current_bet = 0;
        self.game_phase = GamePhase::PreFlop;
        self.show_ai_cards = false;
        self.waiting_for_ai = false;
        self.ai_thinking_timer = None;
        self.player_acted = false;
        self.ai_acted = false;
        self.both_checked = false;
        self.game_over = false;
        // 重置特殊行动使用状态 - 每一小局都可以重新使用
        self.has_used_special_action = false;
        self.first_round = true;
        
        self.initialize_deck_fast();
        self.deal_cards();
        
        self.message = format!("新一局开始！你的筹码: {}, AI筹码: {}", self.player_chips, self.ai_chips);
    }

    /// 重置到难度选择界面
    fn reset_to_difficulty_selection(&mut self) {
        self.difficulty_selection.reset();
        self.game_state = TexasHoldemState::DifficultySelection;
        self.game_initializing = false;
        self.reset_game_state();
    }

    /// 重置到主菜单
    fn reset_to_main_menu(&mut self) {
        self.difficulty_selection.reset();
        self.game_state = TexasHoldemState::DifficultySelection;
        self.game_initializing = false;
        self.reset_game_state();
    }

    /// 重置游戏状态
    fn reset_game_state(&mut self) {
        self.player_hand.clear();
        self.ai_hand.clear();
        self.community_cards.clear();
        self.deck.clear();
        self.player_chips = 200;
        self.ai_chips = 100;
        self.pot = 0;
        self.current_bet = 0;
        self.game_phase = GamePhase::PreFlop;
        self.message = "欢迎来到德州扑克！".to_string();
        self.game_over = false;
        self.show_ai_cards = false;
        self.waiting_for_ai = false;
        self.ai_thinking_timer = None;
        self.player_acted = false;
        self.ai_acted = false;
        self.both_checked = false;
        self.first_round = true;
        self.has_used_special_action = false;
    }

    /// 评估最佳手牌
    fn evaluate_best_hand(&self, hand: &[Card], community: &[Card]) -> HandResult {
        let all_cards: Vec<Card> = hand.iter().chain(community.iter()).cloned().collect();
        self.real_hand_evaluation(&all_cards)
    }

    /// 手牌评估实现
    fn real_hand_evaluation(&self, cards: &[Card]) -> HandResult {
        if cards.len() < 5 {
            return HandResult {
                hand_strength: HandStrength::HighCard,
                high_cards: self.get_sorted_ranks(cards),
            };
        }

        // 检查所有可能的5张牌组合
        let mut best_result = HandResult {
            hand_strength: HandStrength::HighCard,
            high_cards: vec![],
        };

        let combinations = self.generate_combinations(cards, 5);
        
        for combo in combinations {
            let result = self.evaluate_five_card_hand(&combo);
            if self.compare_hand_results(&result, &best_result) == std::cmp::Ordering::Greater {
                best_result = result;
            }
        }

        best_result
    }

    /// 生成所有可能的组合
    fn generate_combinations(&self, cards: &[Card], k: usize) -> Vec<Vec<Card>> {
        let mut result = Vec::new();
        let mut combination = Vec::new();
        self.combinations_recursive(cards, k, 0, &mut combination, &mut result);
        result
    }

    
    /// 递归生成组合
    fn combinations_recursive(
        &self,
        cards: &[Card],
        k: usize,
        start: usize,
        current: &mut Vec<Card>,
        result: &mut Vec<Vec<Card>>,
    ) {
        if current.len() == k {
            result.push(current.clone());
            return;
        }

        for i in start..cards.len() {
            current.push(cards[i].clone());
            self.combinations_recursive(cards, k, i + 1, current, result);
            current.pop();
        }
    }

    /// 评估5张牌的手牌
    fn evaluate_five_card_hand(&self, cards: &[Card]) -> HandResult {
        let mut ranks: Vec<u8> = cards.iter().map(|c| c.rank).collect();
        let suits: Vec<u8> = cards.iter().map(|c| c.suit).collect();
        
        ranks.sort_by(|a, b| b.cmp(a)); // 降序排列
        
        let is_flush = suits.iter().all(|&s| s == suits[0]);
        let is_straight = self.is_straight(&ranks);
        
        // 检查同花顺
        if is_flush && is_straight {
            return HandResult {
                hand_strength: HandStrength::StraightFlush,
                high_cards: vec![ranks[0]],
            };
        }
        
        // 检查四条
        if let Some(quad_rank) = self.has_n_of_a_kind(&ranks, 4) {
            let kicker = *ranks.iter().find(|&&r| r != quad_rank).unwrap_or(&1);
            return HandResult {
                hand_strength: HandStrength::FourOfAKind,
                high_cards: vec![quad_rank, kicker],
            };
        }
        
        // 检查葫芦
        if let (Some(three_rank), Some(two_rank)) = (self.has_n_of_a_kind(&ranks, 3), self.has_n_of_a_kind(&ranks, 2)) {
            if three_rank != two_rank {
                return HandResult {
                    hand_strength: HandStrength::FullHouse,
                    high_cards: vec![three_rank, two_rank],
                };
            }
        }
        
        // 检查同花
        if is_flush {
            return HandResult {
                hand_strength: HandStrength::Flush,
                high_cards: ranks,
            };
        }
        
        // 检查顺子
        if is_straight {
            return HandResult {
                hand_strength: HandStrength::Straight,
                high_cards: vec![ranks[0]],
            };
        }
        
        // 检查三条
        if let Some(three_rank) = self.has_n_of_a_kind(&ranks, 3) {
            let mut kickers: Vec<u8> = ranks.iter().filter(|&&r| r != three_rank).cloned().collect();
            kickers.sort_by(|a, b| b.cmp(a));
            kickers.truncate(2);
            kickers.insert(0, three_rank);
            return HandResult {
                hand_strength: HandStrength::ThreeOfAKind,
                high_cards: kickers,
            };
        }
        
        // 检查两对
        if let Some(pairs) = self.get_pairs(&ranks) {
            if pairs.len() >= 2 {
                let mut high_cards = vec![pairs[0], pairs[1]];
                let kicker = *ranks.iter().find(|&&r| r != pairs[0] && r != pairs[1]).unwrap_or(&1);
                high_cards.push(kicker);
                return HandResult {
                    hand_strength: HandStrength::TwoPair,
                    high_cards,
                };
            }
        }
        
        // 检查一对
        if let Some(pair_rank) = self.has_n_of_a_kind(&ranks, 2) {
            let mut kickers: Vec<u8> = ranks.iter().filter(|&&r| r != pair_rank).cloned().collect();
            kickers.sort_by(|a, b| b.cmp(a));
            kickers.truncate(3);
            kickers.insert(0, pair_rank);
            return HandResult {
                hand_strength: HandStrength::OnePair,
                high_cards: kickers,
            };
        }
        
        // 高牌
        HandResult {
            hand_strength: HandStrength::HighCard,
            high_cards: ranks,
        }
    }

    /// 检查是否是顺子
    fn is_straight(&self, ranks: &[u8]) -> bool {
        let mut sorted_ranks = ranks.to_vec();
        sorted_ranks.sort();
        sorted_ranks.dedup();
        
        if sorted_ranks.len() < 5 {
            return false;
        }
        
        // 检查普通顺子
        for i in 0..=sorted_ranks.len() - 5 {
            if sorted_ranks[i + 4] - sorted_ranks[i] == 4 {
                return true;
            }
        }
        
        // 检查A-2-3-4-5顺子
        if sorted_ranks.contains(&1) && sorted_ranks.contains(&2) && sorted_ranks.contains(&3) && 
           sorted_ranks.contains(&4) && sorted_ranks.contains(&5) {
            return true;
        }
        
        false
    }

    /// 检查是否有N张相同点数的牌
    fn has_n_of_a_kind(&self, ranks: &[u8], n: usize) -> Option<u8> {
        let mut count_map = std::collections::HashMap::new();
        for &rank in ranks {
            *count_map.entry(rank).or_insert(0) += 1;
        }
        
        for (&rank, &count) in &count_map {
            if count == n {
                return Some(rank);
            }
        }
        None
    }

    /// 获取所有对子
    fn get_pairs(&self, ranks: &[u8]) -> Option<Vec<u8>> {
        let mut count_map = std::collections::HashMap::new();
        for &rank in ranks {
            *count_map.entry(rank).or_insert(0) += 1;
        }
        
        let mut pairs: Vec<u8> = count_map
            .iter()
            .filter(|&(_, &count)| count >= 2)
            .map(|(&rank, _)| rank)
            .collect();
        
        pairs.sort_by(|a, b| b.cmp(a));
        
        if pairs.is_empty() {
            None
        } else {
            Some(pairs)
        }
    }

    /// 获取排序后的点数
    fn get_sorted_ranks(&self, cards: &[Card]) -> Vec<u8> {
        let mut ranks: Vec<u8> = cards.iter().map(|c| c.rank).collect();
        ranks.sort_by(|a, b| b.cmp(a));
        ranks
    }

    /// 比较两手牌的结果
    fn compare_hand_results(&self, hand1: &HandResult, hand2: &HandResult) -> std::cmp::Ordering {
        // 先比较手牌强度
        let strength_cmp = hand1.hand_strength.to_u8().cmp(&hand2.hand_strength.to_u8());
        if strength_cmp != std::cmp::Ordering::Equal {
            return strength_cmp;
        }
        
        // 相同手牌强度时，比较关键牌
        for (h1, h2) in hand1.high_cards.iter().zip(hand2.high_cards.iter()) {
            let card_cmp = h1.cmp(h2);
            if card_cmp != std::cmp::Ordering::Equal {
                return card_cmp;
            }
        }
        
        // 如果所有关键牌都相同，比较剩余牌
        let min_len = hand1.high_cards.len().min(hand2.high_cards.len());
        if hand1.high_cards.len() > min_len || hand2.high_cards.len() > min_len {
          
            return std::cmp::Ordering::Equal;
        }
        
        std::cmp::Ordering::Equal
    }

    /// 比较两手牌
    fn compare_hands(&self, hand1: &HandResult, hand2: &HandResult) -> std::cmp::Ordering {
        self.compare_hand_results(hand1, hand2)
    }
}

/// AI行动枚举
enum AiAction {
    Fold,
    Check,
    Bet(u32),
}

/// 手牌结果
struct HandResult {
    hand_strength: HandStrength,
    high_cards: Vec<u8>,
}

/// 手牌强度枚举
#[derive(PartialEq, Clone, Copy, Debug)]
enum HandStrength {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
}

impl HandStrength {
    fn to_u8(&self) -> u8 {
        match self {
            HandStrength::HighCard => 0,
            HandStrength::OnePair => 1,
            HandStrength::TwoPair => 2,
            HandStrength::ThreeOfAKind => 3,
            HandStrength::Straight => 4,
            HandStrength::Flush => 5,
            HandStrength::FullHouse => 6,
            HandStrength::FourOfAKind => 7,
            HandStrength::StraightFlush => 8,
        }
    }
    
    fn to_string(&self) -> String {
        match self {
            HandStrength::HighCard => "高牌".to_string(),
            HandStrength::OnePair => "一对".to_string(),
            HandStrength::TwoPair => "两对".to_string(),
            HandStrength::ThreeOfAKind => "三条".to_string(),
            HandStrength::Straight => "顺子".to_string(),
            HandStrength::Flush => "同花".to_string(),
            HandStrength::FullHouse => "葫芦".to_string(),
            HandStrength::FourOfAKind => "四条".to_string(),
            HandStrength::StraightFlush => "同花顺".to_string(),
        }
    }
}

impl Default for TexasHoldemGame {
    fn default() -> Self {
        Self::new()
    }
}