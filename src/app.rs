
use crate::logic;
use ggez::*;
use ggez::graphics::*;
use ggez::input::*;
use glam::Vec2;

// The pixel offset of the first square in the board texture
const BOARD_OFFSET: u32 = 40;
// The offset between squares in the board texture
const SQUARE_OFFSET: u32 = 22;
// The width/height of the board texture
const BOARD_SIZE: u32 = 256;

struct Images {
    pawn:   Image,
    rook:   Image,
    knight: Image,
    bishop: Image,
    queen:  Image,
    king:   Image,
}

struct Gui {
    
    layer: logic::Layer,
    board: Image,
    black: Images,
    white: Images,
}

impl Gui {

    pub fn new(ctx: &Context, layer: logic::Layer) -> Self {
        
        Self {
            layer,
            board: Image::from_path(ctx, "/board_alt.png").unwrap(),
            black: Images {
                pawn:   Image::from_path(ctx, "/black_pawn.png").unwrap(),
                rook:   Image::from_path(ctx, "/black_rook.png").unwrap(),
                knight: Image::from_path(ctx, "/black_knight.png").unwrap(),
                bishop: Image::from_path(ctx, "/black_bishop.png").unwrap(),
                queen:  Image::from_path(ctx, "/black_queen.png").unwrap(),
                king:   Image::from_path(ctx, "/black_king.png").unwrap(),
            },
            white: Images {
                pawn:   Image::from_path(ctx, "/white_pawn.png").unwrap(),
                rook:   Image::from_path(ctx, "/white_rook.png").unwrap(),
                knight: Image::from_path(ctx, "/white_knight.png").unwrap(),
                bishop: Image::from_path(ctx, "/white_bishop.png").unwrap(),
                queen:  Image::from_path(ctx, "/white_queen.png").unwrap(),
                king:   Image::from_path(ctx, "/white_king.png").unwrap(),
            },
        }
    }
}

impl event::EventHandler<GameError> for  Gui {

    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.layer.update();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {

        // Fill background
        let bgd_hex = 0x057137;
        let mut canvas = ggez::graphics::Canvas::from_frame(
            ctx,
            ggez::graphics::Color::from_rgb(
                (((bgd_hex & 0xff0000) >> 16) & 0xff) as u8,
                (((bgd_hex & 0x00ff00) >>  8) & 0xff) as u8,
                (((bgd_hex & 0x0000ff) >>  0) & 0xff) as u8,
            )
        );
        canvas.set_sampler(ggez::graphics::Sampler::nearest_clamp());

        // Draw board
        let (offset, scale) = board_transform(ctx);
        let board_param = DrawParam::new()
            .dest(offset)
            .scale(scale);

        canvas.draw(
            &self.board,    
            board_param,
        );

        if let SelectMove { from, } = self.layer.get_state() {
            highlight_square(ctx, &mut canvas, from.0, from.1);
        }

        // Draw pieces
        for x in 0..8u8 {
            for y in 0..8u8 {

                match self.layer.get_piece_at(x, y) {
                    None => (),
                    Some(piece) => {

                        use logic::{ Player::*, Piece::*, };

                        let images = match piece.1 {
                            White => &self.white,
                            Black => &self.black,
                        };

                        let image = match piece.0 {
                            Pawn   => &images.pawn,
                            Rook   => &images.rook,
                            Knight => &images.knight,
                            Bishop => &images.bishop,
                            Queen  => &images.queen,
                            King   => &images.king,
                        };

                        let (offset, scale) = piece_transform(ctx, x, y, image);

                        let draw_param = DrawParam::new()
                            .dest(offset)
                            .scale(scale);

                        canvas.draw(image, draw_param);
                    }
                }
            }
        }

        use logic::State::*;
        match self.layer.get_state() {
            OpponentTurn => 
                draw_text(ctx, &mut canvas, "Opponents turn".to_string()),
            CheckMate(player) => {
                draw_text(ctx, &mut canvas, format!("{:?} won!", player));
            },
            _ => (),
        }

        canvas.finish(ctx).unwrap();

        Ok(())
    }
    
    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        button: mouse::MouseButton,
        x: f32,
        y: f32
    ) -> GameResult {

        use mouse::MouseButton::*;

        match button {
            Left => {
                
                let (x, y) = square_from_pos(ctx, x, y);

                let valid = x >= 0 && x < 8 &&
                    y >= 0 && y < 8;

                use logic::State::*;
                match self.layer.get_state().clone() {
                    SelectPiece => {
                        if valid {
                            self.layer.select_piece((x as u8, y as u8));
                        }
                    },
                    SelectMove { from: _, } => {
                        if valid {
                            self.layer.play_move((x as u8, y as u8));
                        }
                    },
                    _ => (),
                }
            },
            _ => (),
        }

        Ok(())
    }
}

fn highlight_square(ctx: &Context, canvas: &mut Canvas, x: u8, y: u8) {

    let rect = Mesh::new_rectangle(
        ctx,
        DrawMode::Fill(FillOptions::DEFAULT),
        Rect::new(0.0, 0.0, SQUARE_OFFSET as f32, SQUARE_OFFSET as f32),
        Color::from([0.3, 0.3, 0.9, 0.5])
    ).unwrap();

    let (offset, scale) = square_transform(ctx, x, y);
    let param = DrawParam::new()
        .dest(offset)
        .scale(scale);

    canvas.draw(&rect, param);
}

fn board_transform(ctx: &Context) -> (Vec2, Vec2) {

    let (w, h) = ctx.gfx.size();
    let s = if w > h { h } else { w };
    let t = s / (BOARD_SIZE as f32);
    let offset = if w > h {
        Vec2 {
            x: (w - h) / 2.,
            y: 0.,
        }
    } else {
        Vec2 {
            x: 0.,
            y: (h - w) / 2.,
        }
    };

    (offset, Vec2::splat(t))
}

fn square_transform(ctx: &Context, x: u8, y: u8) -> (Vec2, Vec2) {

    let (mut offset, scale) = board_transform(ctx);
    offset += BOARD_OFFSET as f32 * scale;
    offset += Vec2 {
        x: scale.x * SQUARE_OFFSET as f32 * x as f32,
        y: scale.y * SQUARE_OFFSET as f32 * y as f32,
    };

    (offset, scale)
}

fn square_from_pos(ctx: &Context, x: f32, y: f32) -> (i8, i8) {
    
    let (offset, scale) = board_transform(ctx);
    let mut pos = Vec2 { x, y, };

    pos -= offset;
    pos /= scale;
    pos -= Vec2::splat(BOARD_OFFSET as f32);
    pos /= SQUARE_OFFSET as f32;
    (pos.x as i8, pos.y as i8)
}

fn draw_text(ctx: &Context, canvas: &mut Canvas, text: String) {

    let (w, h) = ctx.gfx.size();
    let center = Vec2::new(w / 2., h / 2.);

    let rect = Mesh::new_rectangle(
        ctx,
        DrawMode::Fill(FillOptions::DEFAULT),
        Rect::new(0., 0., w, h),
        Color::from([0.3, 0.3, 0.9, 0.9]),
    ).unwrap();

    canvas.draw(&rect, DrawParam::new());

    let param = DrawParam::new()
        .color(Color::from([0.9, 0.4, 0.4, 1.0]))
        .dest(center);
    
    canvas.draw(
        Text::new(text)
            .set_font("Handjet")
            .set_layout(TextLayout::center())
            .set_scale(80.),
        param,
    );
}

fn piece_transform(
    ctx: &Context,
    x: u8,
    y: u8,
    image: &Image
) -> (Vec2, Vec2) {

    let w = image.width() as f32;
    let h = image.height() as f32;

    let (mut offset, scale) = square_transform(ctx, x, y); 

    offset += Vec2 {
        x: scale.x * (SQUARE_OFFSET as f32 - w) / 2.,
        y: scale.y * (SQUARE_OFFSET as f32 - h) / 2.,
    };

    (offset, scale)
}

pub fn run(layer: logic::Layer) {

    let mut config = conf::Conf::new();         

    config.backend = conf::Backend::Gl;
    
    let (mut ctx, event_loop) = ContextBuilder::new("Chess", "What is this")
        .default_conf(config)
        .add_resource_path("./assets")
        .build()
        .unwrap();

    ctx.gfx.window().set_title("Chess");
    ctx.gfx.add_font(
        "Handjet",
        FontData::from_path(&ctx, "/Handjet-Medium.ttf").unwrap()
    );
    
    let gui = Gui::new(&ctx, layer);

    event::run(ctx, event_loop, gui);
}
