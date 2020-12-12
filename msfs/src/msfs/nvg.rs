use crate::sys;
use uom::si::{angle::radian, f32::Angle};

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

/// A NanoVG render context.
pub struct Context {
    ctx: *mut sys::NVGcontext,
}

impl Context {
    /// Create a NanoVG render context from an `FsContext`.
    pub fn create(fs_ctx: sys::FsContext) -> Option<Self> {
        let uninit = std::mem::MaybeUninit::<sys::NVGparams>::zeroed();
        let mut params = unsafe { uninit.assume_init() };
        params.userPtr = fs_ctx;
        params.edgeAntiAlias = 1;

        let ctx = unsafe { sys::nvgCreateInternal(&mut params) };
        if ctx.is_null() {
            None
        } else {
            Some(Self { ctx })
        }
    }

    /// Draw a frame.
    pub fn draw_frame<F: Fn(&Frame) -> Result>(&self, width: usize, height: usize, dpr: f32, f: F) {
        unsafe {
            sys::nvgBeginFrame(self.ctx, width as f32, height as f32, dpr);
        }

        let frame = Frame { ctx: self.ctx };

        match f(&frame) {
            Ok(()) => unsafe {
                sys::nvgEndFrame(self.ctx);
            },
            Err(_) => unsafe {
                sys::nvgCancelFrame(self.ctx);
            },
        }
    }

    /// NanoVG allows you to load .ttf files and use the font to render text.
    ///
    /// The appearance of the text can be defined by setting the current text style
    /// and by specifying the fill color. Common text and font settings such as
    /// font size, letter spacing and text align are supported. Font blur allows you
    /// to create simple text effects such as drop shadows.
    ///
    /// At render time the font face can be set based on the font handles or name.
    ///
    /// Font measure functions return values in local space, the calculations are
    /// carried in the same resolution as the final rendering. This is done because
    /// the text glyph positions are snapped to the nearest pixels sharp rendering.
    ///
    /// The local space means that values are not rotated or scale as per the current
    /// transformation. For example if you set font size to 12, which would mean that
    /// line height is 16, then regardless of the current scaling and rotation, the
    /// returned line height is always 16. Some measures may vary because of the scaling
    /// since aforementioned pixel snapping.
    ///
    /// While this may sound a little odd, the setup allows you to always render the
    /// same way regardless of scaling.
    ///
    /// Note: currently only solid color fill is supported for text.
    pub fn create_font(
        &self,
        name: &str,
        filename: &str,
    ) -> std::result::Result<Font, Box<dyn std::error::Error>> {
        let name = std::ffi::CString::new(name).unwrap();
        let filename = std::ffi::CString::new(filename).unwrap();
        let handle = unsafe { sys::nvgCreateFont(self.ctx, name.as_ptr(), filename.as_ptr()) };
        match handle {
            -1 => panic!(),
            _ => Ok(Font { handle }),
        }
    }
}

/// Methods to draw on a frame. See `Context::draw_frame`.
pub struct Frame {
    ctx: *mut sys::NVGcontext,
}

impl Frame {
    pub fn draw_path<F: Fn(&Path) -> Result>(&self, f: F) -> Result {
        unsafe {
            sys::nvgBeginPath(self.ctx);
        }

        let path = Path { ctx: self.ctx };

        f(&path)
    }
}

/// A path.
pub struct Path {
    ctx: *mut sys::NVGcontext,
}

impl Path {
    /// Starts new sub-path with specified point as first point.
    pub fn move_to(&self, x: f32, y: f32) {
        unsafe {
            sys::nvgMoveTo(self.ctx, x, y);
        }
    }

    /// Adds line segment from the last point in the path to the specified point.
    pub fn line_to(&self, x: f32, y: f32) {
        unsafe {
            sys::nvgLineTo(self.ctx, x, y);
        }
    }

    /// Adds cubic bezier segment from last point in the path via two control points to the specified point.
    pub fn bezier_to(&self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        unsafe {
            sys::nvgBezierTo(self.ctx, c1x, c1y, c2x, c2y, x, y);
        }
    }

    /// Adds quadratic bezier segment from last point in the path via a control point to the
    /// specified point.
    pub fn quad_to(&self, cx: f32, cy: f32, x: f32, y: f32) {
        unsafe {
            sys::nvgQuadTo(self.ctx, cx, cy, x, y);
        }
    }

    /// Adds an arc segment at the corner defined by the last path point, and two specified points.
    pub fn arc_to(&self, x1: f32, y1: f32, x2: f32, y2: f32, radius: f32) {
        unsafe {
            sys::nvgArcTo(self.ctx, x1, y1, x2, y2, radius);
        }
    }

    /// Closes current sub-path with a line segment.
    pub fn close_path(&self) {
        unsafe {
            sys::nvgClosePath(self.ctx);
        }
    }

    /// Creates a new circle arc shaped sub-path. The arc center is at (`cx`,`cy`), the arc radius
    /// is `r`, and the arc is drawn from angle `a0` to `a1`, and swept in direction `dir`.
    pub fn arc(&self, cx: f32, cy: f32, r: f32, a0: Angle, a1: Angle, dir: Direction) {
        unsafe {
            sys::nvgArc(
                self.ctx,
                cx,
                cy,
                r,
                a0.get::<radian>(),
                a1.get::<radian>(),
                dir as i32,
            );
        }
    }

    /// Creates a new oval arc shaped sub-path. The arc center is at (`cx`, `cy`), the arc radius
    /// is (`rx`, `ry`), and the arc is draw from angle a0 to a1, and swept in direction `dir`.
    #[allow(clippy::too_many_arguments)]
    pub fn elliptical_arc(
        &self,
        cx: f32,
        cy: f32,
        rx: f32,
        ry: f32,
        a0: Angle,
        a1: Angle,
        dir: Direction,
    ) {
        unsafe {
            sys::nvgEllipticalArc(
                self.ctx,
                cx,
                cy,
                rx,
                ry,
                a0.get::<radian>(),
                a1.get::<radian>(),
                dir as i32,
            );
        }
    }

    /// Creates new rectangle shaped sub-path.
    pub fn rect(&self, x: f32, y: f32, w: f32, h: f32) {
        unsafe {
            sys::nvgRect(self.ctx, x, y, w, h);
        }
    }

    /// Creates a new rounded rectangle sub-path with rounded corners
    #[allow(clippy::many_single_char_names)]
    pub fn rounded_rect(&self, x: f32, y: f32, w: f32, h: f32, r: f32) {
        unsafe {
            sys::nvgRoundedRect(self.ctx, x, y, w, h, r);
        }
    }

    /// Creates new rounded rectangle shaped sub-path with varying radii for each corner.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::many_single_char_names)]
    pub fn rounded_rect_varying(
        &self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rad_top_left: f32,
        rad_top_right: f32,
        rad_bottom_right: f32,
        rad_bottom_left: f32,
    ) {
        unsafe {
            sys::nvgRoundedRectVarying(
                self.ctx,
                x,
                y,
                w,
                h,
                rad_top_left,
                rad_top_right,
                rad_bottom_right,
                rad_bottom_left,
            );
        }
    }

    /// Creates a new ellipse shaped sub-path.
    pub fn ellipse(&self, cx: f32, cy: f32, rx: f32, ry: f32) {
        unsafe {
            sys::nvgEllipse(self.ctx, cx, cy, rx, ry);
        }
    }

    /// Creates a new circle shaped path.
    pub fn circle(&self, cx: f32, cy: f32, r: f32) {
        unsafe {
            sys::nvgCircle(self.ctx, cx, cy, r);
        }
    }

    // TODO: fill
}

/// Winding direction
#[derive(Debug)]
#[repr(u32)]
pub enum Direction {
    /// Winding for holes.
    Clockwise = sys::NVGwinding_NVG_CW,
    /// Winding for solid shapes.
    CounterClockwise = sys::NVGwinding_NVG_CCW,
}

/// A font handle
pub struct Font {
    handle: std::os::raw::c_int,
}
