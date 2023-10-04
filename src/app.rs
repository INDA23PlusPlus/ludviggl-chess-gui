
use crate::logic;
use ggez::*;
use ggez::graphics::*;

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

    fn update(&mut self, ctx: &mut Context) -> GameResult { Ok(()) }

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

        Ok(())
    }
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
    
    let mut gui = Gui::new(&ctx, layer);

    event::run(ctx, event_loop, gui);
}
