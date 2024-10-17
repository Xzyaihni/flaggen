use sdl2::{
    pixels::{Color, PixelFormatEnum},
    render::{WindowCanvas, TextureAccess, Texture, TextureCreator},
    video::WindowContext,
    event::{WindowEvent, Event},
    keyboard::Keycode
};

use strum::EnumCount;

use nalgebra::Vector2;

use image::{RgbImage, Rgb};


#[derive(Debug)]
pub struct FlagBackground
{
    horizontal: bool,
    lines: Vec<Rgb<u8>>
}

pub fn random_color() -> Rgb<u8>
{
    let r = ||
    {
        fastrand::u8(0..=u8::MAX)
    };

    Rgb([r(), r(), r()])
}

impl FlagBackground
{
    pub fn random() -> Self
    {
        let amount = fastrand::usize(1..6);
        let lines = (0..amount).map(|_| random_color()).collect();

        FlagBackground{
            horizontal: fastrand::bool(),
            lines
        }
    }
}

#[derive(Debug, EnumCount)]
pub enum FlagForegroundShape
{
    Circle,
    Ring(f32),
    LeftTriangle
}

impl FlagForegroundShape
{
    pub fn random() -> Self
    {
        match fastrand::usize(0..Self::COUNT)
        {
            0 => Self::Circle,
            1 => Self::Ring(fastrand::f32() * 0.5 + 0.1),
            2 => Self::LeftTriangle,
            _ => unreachable!()
        }
    }
}

#[derive(Debug)]
pub struct FlagForeground
{
    color: Rgb<u8>,
    shape: FlagForegroundShape
}

impl FlagForeground
{
    pub fn random() -> Self
    {
        Self{
            color: random_color(),
            shape: FlagForegroundShape::random()
        }
    }

    fn draw_with_fn(
        image: &mut RgbImage,
        color: Rgb<u8>,
        mut f: impl FnMut(Vector2<i32>) -> bool
    )
    {
        image.enumerate_pixels_mut().for_each(|(x, y, pixel)|
        {
            let pos = Vector2::new(x as i32, y as i32);

            if f(pos)
            {
                *pixel = color;
            };
        })
    }

    pub fn draw_on(&self, image: &mut RgbImage)
    {
        let size: Vector2<i32> = Vector2::new(image.width(), image.height()).cast();
        let lower_size = Vector2::repeat(image.width().min(image.height()) as f32);

        match self.shape
        {
            FlagForegroundShape::Circle
            | FlagForegroundShape::Ring(_) =>
            {
                let radius = 0.8 / 2.0;

                Self::draw_with_fn(image, self.color, |pos|
                {
                    let pos = (pos - size / 2).map(|x| x as f32).component_div(&lower_size);

                    let distance = pos.magnitude();

                    match self.shape
                    {
                        FlagForegroundShape::Circle =>
                        {
                            distance <= radius
                        },
                        FlagForegroundShape::Ring(ring_width) =>
                        {
                            ((radius - ring_width / 2.0)..=radius).contains(&distance)
                        },
                        _ => unreachable!()
                    }
                });
            },
            FlagForegroundShape::LeftTriangle =>
            {
                Self::draw_with_fn(image, self.color, |pos|
                {
                    let pos = pos.map(|x| x as f32).component_div(&lower_size);

                    (pos.x + (pos.y - 0.5).abs()) < 0.5
                });
            }
        }
    }
}

pub fn create_flag(
    background: FlagBackground,
    foreground: Option<FlagForeground>,
    width: u32,
    height: u32
) -> RgbImage
{
    eprintln!("creating {width}x{height} image with {background:?} and {foreground:?}");

    let mut background = RgbImage::from_fn(width, height, |x, y|
    {
        let pos = if background.horizontal
        {
            x as f32 / width as f32
        } else
        {
            y as f32 / height as f32
        };

        let pos = pos * background.lines.len() as f32;

        background.lines[pos as usize]
    });

    if let Some(foreground) = foreground
    {
        foreground.draw_on(&mut background);
    }

    background
}

pub fn random_flag(width: u32, height: u32) -> RgbImage
{
    let background = FlagBackground::random();

    let mut has_foreground = fastrand::bool();

    let solid = background.lines.len() == 1;
    if solid
    {
        has_foreground = true;
    }

    let mut foreground = has_foreground.then(FlagForeground::random);

    if let (Some(foreground), true) = (foreground.as_mut(), solid)
    {
        foreground.shape = FlagForegroundShape::Circle;
    }

    create_flag(
        background,
        foreground,
        width,
        height
    )
}

fn main()
{
    let ctx = sdl2::init().unwrap();

    let video = ctx.video().unwrap();

    let window = video.window("flag generator!", 640, 360)
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let creator = canvas.texture_creator();

    let mut events = ctx.event_pump().unwrap();

    let mut texture = None;

    fn next_flag<'a>(
        canvas: &mut WindowCanvas,
        creator: &'a TextureCreator<WindowContext>,
        texture: &mut Option<Texture<'a>>
    )
    {
        let (width, height) = canvas.window().size();
        let flag = random_flag(width, height);

        *texture = Some(creator.create_texture(
            PixelFormatEnum::RGB24,
            TextureAccess::Static,
            width,
            height
        ).unwrap());

        texture.as_mut().unwrap().update(None, &flag, flag.width() as usize * 3).unwrap();

        canvas.copy(texture.as_ref().unwrap(), None, None).unwrap();

        canvas.present();

        let path = "flag.png";
        flag.save(path).unwrap();
    }

    next_flag(&mut canvas, &creator, &mut texture);

    for event in events.wait_iter()
    {
        match event
        {
            Event::Quit{..} => return,
            Event::KeyDown{keycode: Some(Keycode::Space), ..} =>
            {
                next_flag(&mut canvas, &creator, &mut texture);
            },
            Event::Window{win_event, ..} =>
            {
                match win_event
                {
                    WindowEvent::SizeChanged(_, _) =>
                    {
                        canvas.set_draw_color(Color::RGB(0, 0, 0));
                        canvas.clear();

                        canvas.copy(texture.as_ref().unwrap(), None, None).unwrap();

                        canvas.present();
                    },
                    _ => ()
                }
            },
            _ => ()
        }
    }
}
