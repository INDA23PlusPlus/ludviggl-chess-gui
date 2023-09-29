
#![allow(warnings)]
#[macro_use]
extern crate lazy_static;

use ggez;
use simonsev_chess as chess;
use ggez::glam;
use std::sync::{ Mutex, Arc };

// The pixel offset of the first square in the board texture
const BOARD_OFFSET: u32 = 40;
// The offset between squares in the board texture
const SQUARE_OFFSET: u32 = 22;
// The offset of the piece within the square
const PIECE_OFFSET: u32 = 4;

lazy_static! {
    static ref GAME: Mutex<chess::Game> = Mutex::new(chess::Game::new());
}

struct Images {
    pawn:   ggez::graphics::Image,
    rook:   ggez::graphics::Image,
    knight: ggez::graphics::Image,
    bishop: ggez::graphics::Image,
    queen:  ggez::graphics::Image,
    king:   ggez::graphics::Image,
}

#[derive(Clone, Copy, Debug)]
enum InputState {
    PieceSelect,
    MoveSelect { from: (u8, u8), },
    CheckMate,
    Promotion { selection_index: usize, },
}

fn validate(square: (i32, i32)) -> Option<(u8, u8)> {
    let valid = square.0 >= 0 && square.0 < 8
        && square.1 >= 0 && square.1 < 8;
    if valid { Some((square.0 as u8, square.1 as u8)) }
        else { None }
}

fn square_str(square: (u8, u8)) -> String {
    format!("{}{}",
        match square.1 {
            0 => 'A',
            1 => 'B',
            2 => 'C',
            3 => 'D',
            4 => 'E',
            5 => 'F',
            6 => 'G',
            7 => 'H',
            _ => panic!()
        },
        (square.0 + 1).to_string()
    )
}

struct State {
    input_state: InputState,
    board_img: ggez::graphics::Image,
    white_img: Images,
    black_img: Images,
}

// STATE
impl State {

    pub fn new(ctx: &mut ggez::Context) -> ggez::GameResult<State> {

        Ok(Self {
            input_state: InputState::PieceSelect,
            board_img: ggez::graphics::Image::from_path(
                ctx,
                "/board_alt.png"
            ).unwrap(),
            white_img: Images {
                pawn:   ggez::graphics::Image::from_path(
                    ctx,
                    "/white_pawn.png"
                ).unwrap(),
                rook:   ggez::graphics::Image::from_path(
                    ctx,
                    "/white_rook.png"
                ).unwrap(),
                knight: ggez::graphics::Image::from_path(
                    ctx,
                    "/white_knight.png"
                ).unwrap(),
                bishop: ggez::graphics::Image::from_path(
                    ctx,
                    "/white_bishop.png"
                ).unwrap(),
                queen:  ggez::graphics::Image::from_path(
                    ctx,
                    "/white_queen.png"
                ).unwrap(),
                king:   ggez::graphics::Image::from_path(
                    ctx,
                    "/white_king.png"
                ).unwrap(),
            },
            black_img: Images {
                pawn:   ggez::graphics::Image::from_path(
                    ctx,
                    "/black_pawn.png"
                ).unwrap(),
                rook:   ggez::graphics::Image::from_path(
                    ctx,
                    "/black_rook.png"
                ).unwrap(),
                knight: ggez::graphics::Image::from_path(
                    ctx,
                    "/black_knight.png"
                ).unwrap(),
                bishop: ggez::graphics::Image::from_path(
                    ctx,
                    "/black_bishop.png"
                ).unwrap(),
                queen:  ggez::graphics::Image::from_path(
                    ctx,
                    "/black_queen.png"
                ).unwrap(),
                king:   ggez::graphics::Image::from_path(
                    ctx,
                    "/black_king.png"
                ).unwrap(),
            },
        })
    }

    fn forward(&self, ctx: &ggez::Context) -> (glam::Vec2, glam::Vec2) {

        let (scale, _, translation) = self.affine(&ctx)
            .to_scale_angle_translation();

        (scale, translation) 
    }

    fn back(&self, ctx: &ggez::Context) -> (glam::Vec2, glam::Vec2) {

        let (scale, translation) = self.forward(ctx);

        (1.0 / scale, -translation) 
    }

    fn affine(&self, ctx: &ggez::Context) -> glam::Affine2 {

        let s = self.board_img.width() as f32;
        let (w, h) = ctx.gfx.drawable_size();
        
        let scale = f32::min(w, h) / s;
        let scale = glam::Vec2::new(scale, scale);

        let offset = if w > h {
            glam::Vec2::new(w / 2.0 - h / 2.0, 0.0)
        } else {
            glam::Vec2::new(0.0, h / 2.0 - w / 2.0)
        };
        
        glam::Affine2::from_scale_angle_translation(
            scale, 0.0, offset
        )
    }

    fn square_from_pos(&self, ctx: &ggez::Context, x: f32, y: f32) -> (i32, i32) {

        let trans = self.back(ctx);
        let mut p = glam::Vec2::new(x, y);
        p += trans.1;
        p *= trans.0;
        p -= glam::Vec2::splat(BOARD_OFFSET as f32);
        p /= SQUARE_OFFSET as f32;
        p = glam::Vec2::new(p.y, p.x); // Rotate board

        if p.x < 0.0 || p.y < 0.0 {
            (-1, -1)
        } else {
            (p.x as i32, p.y as i32)
        }
    }

    fn piece_transform(
        &self,
        ctx: &ggez::Context,
        x: u32,
        y: u32,
        w: u32,
        h: u32
    ) -> (glam::Vec2, glam::Vec2) {

        let (scale, origin) = self.forward(&ctx);

        let (x, y) = (y, x); // Rotate board

        let offset = scale * glam::Vec2::new(
            (x * SQUARE_OFFSET + SQUARE_OFFSET / 2 - w / 2 + BOARD_OFFSET) as f32,
            (y * SQUARE_OFFSET + SQUARE_OFFSET / 2 - h / 2 + BOARD_OFFSET) as f32
        ); 

        (scale, origin + offset)
    }

    fn draw_square(
        &self,
        ctx: &mut ggez::Context,
        canvas: &mut ggez::graphics::Canvas,
        x: u8,
        y: u8
    ) {

        let (scale, origin) = self.forward(ctx);

        let (x, y) = (y, x); // Rotate board

        let offset = scale * glam::Vec2::new(
            (x as u32 * SQUARE_OFFSET + BOARD_OFFSET) as f32,
            (y as u32 * SQUARE_OFFSET + BOARD_OFFSET) as f32
        );
        
        use ggez::graphics::*;
        let rect = Mesh::new_rectangle(
            ctx,
            DrawMode::Fill(FillOptions::DEFAULT),
            Rect::new(0.0, 0.0, SQUARE_OFFSET as f32, SQUARE_OFFSET as f32),
            Color::from([0.3, 0.3, 0.9, 0.5])
        ).unwrap();

        rect.draw(
            canvas,
            DrawParam::new()
                .scale(scale)
                .dest(origin + offset)
        );
    }

    fn draw_promotion(
        &self,
        ctx: &mut ggez::Context,
        canvas: &mut ggez::graphics::Canvas
    ) {

        let mut prefixes: &mut [&str] = &mut [" "; 4];
        
        if let InputState::Promotion { selection_index } = self.input_state {
            
            prefixes[selection_index] = "*";
            let text = format!(
            "Select promotion:\n{} Rook\n{} Knight\n{} Bishop\n{} Queen",
            prefixes[0], prefixes[1],
            prefixes[2], prefixes[3]
            ); 

            self.draw_text(ctx, canvas, text);
        }
    }

    fn draw_text(
        &self,
        ctx: &mut ggez::Context,
        canvas: &mut ggez::graphics::Canvas,
        text: String
    ) {
        
        let (x, y) = ctx.gfx.drawable_size();
        let c = glam::Vec2::new(x, y) / 2.0;

        use ggez::graphics::*;

        let rect = Mesh::new_rectangle(
            ctx,
            DrawMode::Fill(FillOptions::DEFAULT),
            Rect::new(0.0, 0.0, x, y),
            Color::from([0.3, 0.3, 0.9, 0.9])
        ).unwrap();

        canvas.draw(
            &rect,
            DrawParam::new()
        );

        canvas.draw(
            Text::new(text)
                .set_font("Handjet")
                .set_layout(TextLayout::center())
                .set_scale(80.),
            DrawParam::new()
                .color(Color::from([0.9, 0.4, 0.4, 1.0]))
                .dest(c)
        );
    }
}

// EVENT HANDLER
impl ggez::event::EventHandler<ggez::GameError> for State {

    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {

        let game = GAME.lock().unwrap();

        let bgd_hex = 0x057137;

        // Fill background
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
        let transform = self.forward(&ctx);
        canvas.draw(
            &self.board_img,
            ggez::graphics::DrawParam::new()
                .scale(transform.0)
                .dest(transform.1)
        );

        // Draw highlights
        match self.input_state {
            InputState::MoveSelect { from, } => {
                self.draw_square(ctx, &mut canvas, from.0, from.1);    
            },
            _ => (),
        };

        // Draw pieces
        for r in game.get_board().iter() {
            for s in r.iter() {

                let s = s.clone();

                if !s.occupied { continue; }

                // Select color
                let images = if s.piece.white { &self.white_img } else { &self.black_img };
                
                // Select piece type
                use chess::PieceType::*;
                let img = match s.piece.piece_type {
                    Pawn   => &images.pawn,
                    Rook   => &images.rook,
                    Knight => &images.knight,
                    Bishop => &images.bishop,
                    Queen  => &images.queen,
                    King   => &images.king,
                    _ => panic!(),
                };

                let transform = self.piece_transform(
                    ctx,
                    s.x as u32,
                    s.y as u32,
                    img.width(), img.height()
                );

                canvas.draw(img, ggez::graphics::DrawParam::new()
                            .scale(transform.0)
                            .dest(transform.1)
                );
            }
        }

        if let InputState::CheckMate = self.input_state {

            let player_str = if game.white_turn {
                "White"
            } else {
                "Black"
            };


            self.draw_text(ctx, &mut canvas, format!("{} won!\nPress R to reset", player_str));
        }

        if let InputState::Promotion { .. } = self.input_state {
            
            self.draw_promotion(ctx, &mut canvas);
        }
        
        canvas.finish(ctx);
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        input: ggez::input::keyboard::KeyInput,
        repeated: bool
    ) -> ggez::GameResult {
        
        if repeated { return Ok(()); }
    
        use ggez::input::keyboard::KeyCode;
        if let Some(keycode) = input.keycode {
            match keycode {
                KeyCode::R => {
                    self.input_state = InputState::PieceSelect;
                    *GAME.lock().unwrap() = chess::Game::new();
                },
                KeyCode::Escape => {
                    ctx.request_quit();
                },
                KeyCode::P => self.input_state = InputState::Promotion { selection_index: 0 },

                _ => match &mut self.input_state {

                    InputState::Promotion { selection_index, } => {

                        match keycode {
                            KeyCode::Up => {
                                if *selection_index > 0 {
                                    *selection_index -= 1;
                                }
                            },
                            KeyCode::Down => {
                                if *selection_index < 3 {
                                    *selection_index += 1;
                                }
                            },
                            _ => (),
                        }
                    }, 
                    _ => (),
                },
                
            }
        }

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        button: ggez::input::mouse::MouseButton,
        x: f32,
        y: f32
    ) -> ggez::GameResult {

        let mut game = GAME.lock().unwrap();

        if !matches!(button, ggez::input::mouse::MouseButton::Left) {
            return Ok(());
        }

        if matches!(self.input_state, InputState::CheckMate) {
            return Ok(())
        }

        let square = self.square_from_pos(ctx, x, y);
        
        if let Some(pos) = validate(square) {
            
            match self.input_state.clone() {

                InputState::PieceSelect => {
                    
                    let square = &game.get_board()[square.0 as usize][square.1 as usize];
                    if !square.occupied { return Ok(()); }

                    if square.piece.white == game.white_turn {
                        self.input_state = InputState::MoveSelect { from: pos, };
                    }
                },

                InputState::MoveSelect { from, } => {

                    let from_str = square_str(from);
                    let to_str = square_str(pos);

                    game.input_move(
                        from_str,
                        to_str,
                    );
                    *game = game.clone().do_turn();
                    if game.mate {
                        self.input_state = InputState::CheckMate;
                    } else {
                        self.input_state = InputState::PieceSelect;
                    }
                },
                _ => (),
            }
        } else {
            self.input_state = InputState::PieceSelect;
        }

        Ok(()) 
    }
}

fn main() {

    let mut c = ggez::conf::Conf::new();
    
    c.backend = ggez::conf::Backend::Gl;
    c.window_mode.fullscreen_type(ggez::conf::FullscreenType::True);


    let (mut ctx, event_loop) = ggez::ContextBuilder::new("Chess", "Ludvig Gunne Lindström")
        .default_conf(c)
        .add_resource_path("./assets")
        .build()
        .unwrap();

    ctx.gfx.window().set_title("Chess");
    ctx.gfx.add_font(
        "Handjet",
        ggez::graphics::FontData::from_path(&ctx, "/Handjet-Medium.ttf").unwrap()
    );

    let mut state = State::new(&mut ctx).unwrap();

    ggez::event::run(ctx, event_loop, state);
}
