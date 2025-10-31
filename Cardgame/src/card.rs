use eframe::egui::{self, TextureHandle, Vec2};
use image::ImageError;

#[derive(Clone)]
pub struct Card {
    pub id: usize,
    pub is_face_up: bool,
    pub rank: u8, // 1~13
    pub suit: u8, // 1~4
    pub back_tex: TextureHandle,
    pub face_tex: TextureHandle,
}

impl Card {
    /// 创建卡片
    pub fn new(
        ctx: &egui::Context,
        id: usize,
        rank: u8,
        suit: u8,
        back_path: &str,
        face_path: &str,
    ) -> Result<Self, ImageError> {
        assert!((1..=13).contains(&rank), "Rank must be between 1 and 13");
        assert!((1..=4).contains(&suit), "Suit must be between 1 and 4");
        
        let back_tex = Self::load_texture_from_file(ctx, back_path)?;
        let face_tex = Self::load_texture_from_file(ctx, face_path)?;
        
        Ok(Self {
            id,
            is_face_up: false,
            rank,
            suit,
            back_tex,
            face_tex,
        })
    }

    /// 从文件加载纹理
    fn load_texture_from_file(ctx: &egui::Context, path: &str) -> Result<TextureHandle, ImageError> {
        let image = image::open(path)?;
        let image = image.to_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let pixels = image.into_raw();
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
        Ok(ctx.load_texture(path, color_image, Default::default()))
    }

    /// 显示卡片并处理点击事件
    pub fn show(&mut self, ui: &mut egui::Ui, size: Vec2) -> egui::Response {
        let texture = if self.is_face_up {
            &self.face_tex
        } else {
            &self.back_tex
        };

        let image_button = egui::ImageButton::new(egui::Image::new(texture).max_size(size))
            .frame(false)
            .sense(egui::Sense::click());

        let response = ui.add(image_button);

        if response.clicked() {
            
           if!self.is_face_up{self.is_face_up = !self.is_face_up;}
        }

        response
    }
 


}