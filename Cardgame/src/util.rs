  /// 根据rank和suit获取图片文件名
pub fn get_card_image_path(rank: u8, suit: u8) -> String {
        let suit_name = match suit {
            1 => "Spade",   // 黑桃
            2 => "Heart",   // 红桃
            3 => "Diamond", // 方片
            4 => "Club",    
            _ => "Spade",
        };

        let rank_name = match rank {
            1 => "A",
            2 => "2",
            3 => "3",
            4 => "4",
            5 => "5",
            6 => "6",
            7 => "7",
            8 => "8",
            9 => "9",
            10 => "10",
            11 => "J",
            12 => "Q",
            13 => "K",
            _ => "A",
        };

        format!("assets/card_face/{}{}.png", suit_name, rank_name)
    }