use crate::Canvas;

use heapless::FnvIndexMap;
use blitstr_ref as blitstr;
use blitstr::GlyphStyle;
use graphics_server::*;

pub(crate) trait LayoutApi {
    type Layout;

    fn init(gfx: &graphics_server::Gfx, trng: &trng::Trng,
        base_trust: u8, status_canvas: &Canvas,
        canvases: &mut FnvIndexMap<Gid, Canvas, {crate::MAX_CANVASES}>) -> Result<Self::Layout, xous::Error>;
    fn clear(&self, gfx: &graphics_server::Gfx, canvases: &mut FnvIndexMap<Gid, Canvas, {crate::MAX_CANVASES}>) -> Result<(), xous::Error>;
    // for Chats, this resizes the height of the input area; for menus, it resizes the overall height
    fn resize_height(&mut self, gfx: &graphics_server::Gfx, new_height: i16, status_canvas: &Canvas, canvases: &mut FnvIndexMap<Gid, Canvas, {crate::MAX_CANVASES}>) -> Result<Point, xous::Error>;
    fn get_input_canvas(&self) -> Option<Gid> { None }
    fn get_prediction_canvas(&self) -> Option<Gid> { None }
    fn get_content_canvas(&self) -> Gid; // layouts always have a content canvas
}

#[derive(Debug, Copy, Clone)]
// GIDs of canvases that are used the "Chat" layout.
pub(crate) struct ChatLayout {
    // a set of GIDs to track the elements of the chat layout
    pub content: Gid,
    pub predictive: Gid,
    pub input: Gid,

    // my internal bookkeeping records. Allow input area to grow into content area
    min_content_height: i16,
    min_input_height: i16,
    screensize: Point,
    small_height: i16,
    regular_height: i16,
}
impl LayoutApi for ChatLayout {
    type Layout = ChatLayout;
    // pass in the status canvas so we can size around it, but we can't draw on it
    fn init(gfx: &graphics_server::Gfx, trng: &trng::Trng, base_trust: u8,
        status_canvas: &Canvas, canvases: &mut FnvIndexMap<Gid, Canvas, {crate::MAX_CANVASES}>) -> Result<ChatLayout, xous::Error> {
        let screensize = gfx.screen_size().expect("Couldn't get screen size");
        // get the height of various text regions to compute the layout
        let small_height: i16 = gfx.glyph_height_hint(GlyphStyle::Small).expect("couldn't get glyph height") as i16;
        let regular_height: i16 = gfx.glyph_height_hint(GlyphStyle::Regular).expect("couldn't get glyph height") as i16;
        let margin = 4;

        // allocate canvases in structures, and record their GID for future reference
        let predictive_canvas = Canvas::new(
            Rectangle::new_coords(0, screensize.y - regular_height - margin*2, screensize.x, screensize.y),
            base_trust,
            &trng, None
        ).expect("couldn't create predictive text canvas");
        canvases.insert(predictive_canvas.gid(), predictive_canvas).expect("couldn't store predictive canvas");

        let min_input_height = regular_height + margin*2;
        let input_canvas = Canvas::new(
            Rectangle::new_v_stack(predictive_canvas.clip_rect(), -min_input_height),
         base_trust, &trng, None
        ).expect("couldn't create input text canvas");
        canvases.insert(input_canvas.gid(), input_canvas).expect("couldn't store input canvas");

        let content_canvas = Canvas::new(
            Rectangle::new_v_span(status_canvas.clip_rect(), input_canvas.clip_rect()),
            base_trust / 2, &trng, None
        ).expect("couldn't create content canvas");
        canvases.insert(content_canvas.gid(), content_canvas).expect("can't store content canvas");

        Ok(ChatLayout {
            content: content_canvas.gid(),
            predictive: predictive_canvas.gid(),
            input: input_canvas.gid(),
            min_content_height: 64,
            min_input_height,
            screensize,
            small_height,
            regular_height,
        })
    }
    fn clear(&self, gfx: &graphics_server::Gfx, canvases: &mut FnvIndexMap<Gid, Canvas, {crate::MAX_CANVASES}>) -> Result<(), xous::Error> {
        let input_canvas = canvases.get(&self.input).expect("couldn't find input canvas");
        let content_canvas = canvases.get(&self.content).expect("couldn't find content canvas");
        let predictive_canvas = canvases.get(&self.predictive).expect("couldn't find predictive canvas");

        let mut rect = content_canvas.clip_rect();
        rect.style = DrawStyle {fill_color: Some(PixelColor::Light), stroke_color: None, stroke_width: 0,};
        gfx.draw_rectangle(rect).expect("can't clear canvas");

        let mut rect = predictive_canvas.clip_rect();
        rect.style = DrawStyle {fill_color: Some(PixelColor::Light), stroke_color: None, stroke_width: 0,};
        gfx.draw_rectangle(rect).expect("can't clear canvas");

        let mut rect = input_canvas.clip_rect();
        rect.style = DrawStyle {fill_color: Some(PixelColor::Light), stroke_color: None, stroke_width: 0,};
        gfx.draw_rectangle(rect).expect("can't clear canvas");
        Ok(())
    }
    fn resize_height(&mut self, gfx: &graphics_server::Gfx, new_height: i16, status_canvas: &Canvas, canvases: &mut FnvIndexMap<Gid, Canvas, {crate::MAX_CANVASES}>) -> Result<Point, xous::Error> {
        let input_canvas = canvases.get(&self.input).expect("couldn't find input canvas");
        let predictive_canvas = canvases.get(&self.predictive).expect("couldn't find predictive canvas");

        let height: i16 = if new_height < self.min_input_height {
            self.min_input_height
        } else {
            new_height
        };
        let mut new_input_rect = Rectangle::new_v_stack(predictive_canvas.clip_rect(), -height);
        let mut new_content_rect = Rectangle::new_v_span(status_canvas.clip_rect(), new_input_rect);
        if (new_content_rect.br.y - new_content_rect.tl.y) > self.min_content_height {
            {
                let input_canvas_mut = canvases.get_mut(&self.input).expect("couldn't find input canvas");
                input_canvas_mut.set_clip(new_input_rect);
                new_input_rect.style = DrawStyle {fill_color: Some(PixelColor::Light), stroke_color: None, stroke_width: 0,};
                gfx.draw_rectangle(new_input_rect).expect("can't clear canvas");
                    }
            {
                let content_canvas_mut = canvases.get_mut(&self.content).expect("couldn't find content canvas");
                content_canvas_mut.set_clip(new_content_rect);
                new_content_rect.style = DrawStyle {fill_color: Some(PixelColor::Light), stroke_color: None, stroke_width: 0,};
                gfx.draw_rectangle(new_content_rect).expect("can't clear canvas");
            }
            // we resized to this new height
            Ok(new_content_rect.br)
        } else {
            // we didn't resize anything, height unchanged
            Ok(input_canvas.clip_rect().br)
        }
    }
    fn get_input_canvas(&self) -> Option<Gid> {
        Some(self.input)
    }
    fn get_prediction_canvas(&self) -> Option<Gid> {
        Some(self.predictive)
    }
    fn get_content_canvas(&self) -> Gid {
        self.content
    }
}

// remember GIDs of the canvases for menus
#[derive(Debug, Copy, Clone)]
pub(crate) struct MenuLayout {
    pub menu: Gid,
    menu_y_pad: i16,
    menu_x_pad: i16,
    menu_min_height: i16,
    screensize: Point,
    small_height: i16,
}
impl LayoutApi for MenuLayout {
    type Layout = MenuLayout;
    fn init(gfx: &graphics_server::Gfx, trng: &trng::Trng, base_trust: u8, _status_canvas: &Canvas, canvases: &mut FnvIndexMap<Gid, Canvas, {crate::MAX_CANVASES}>) -> Result<MenuLayout, xous::Error> {
        let screensize = gfx.screen_size().expect("Couldn't get screen size");
        // get the height of various text regions to compute the layout
        let small_height: i16 = gfx.glyph_height_hint(GlyphStyle::Small).expect("couldn't get glyph height") as i16;

        const MENU_Y_PAD: i16 = 100;
        const MENU_X_PAD: i16 = 35;
        // build for an initial size of 1 entry
        let menu_canvas = Canvas::new(
            Rectangle::new_coords(MENU_X_PAD, MENU_Y_PAD, screensize.x - MENU_X_PAD, MENU_Y_PAD + small_height),
            base_trust, &trng, None
        ).expect("couldn't create menu canvas");
        canvases.insert(menu_canvas.gid(), menu_canvas).expect("can't store menu canvas");

        Ok(MenuLayout {
            menu: menu_canvas.gid(),
            menu_y_pad: MENU_Y_PAD,
            menu_x_pad: MENU_X_PAD,
            menu_min_height: small_height,
            screensize,
            small_height,
        })
    }
    fn clear(&self, gfx: &graphics_server::Gfx, canvases: &mut FnvIndexMap<Gid, Canvas, {crate::MAX_CANVASES}>) -> Result<(), xous::Error> {
        let menu_canvas = canvases.get(&self.menu).expect("couldn't find menu canvas");

        let mut rect = menu_canvas.clip_rect();
        rect.style = DrawStyle {fill_color: Some(PixelColor::Dark), stroke_color: None, stroke_width: 0,};
        gfx.draw_rectangle(rect)
    }
    fn resize_height(&mut self, gfx: &graphics_server::Gfx, new_height: i16, _status_canvas: &Canvas, canvases: &mut FnvIndexMap<Gid, Canvas, {crate::MAX_CANVASES}>) -> Result<Point, xous::Error> {
        let menu_canvas = canvases.get_mut(&self.menu).expect("couldn't find menu canvas");

        let mut height: i16 = if new_height < self.menu_min_height {
            self.menu_min_height
        } else {
            new_height
        };
        if new_height > self.screensize.y - self.menu_y_pad {
            height = self.screensize.y - self.menu_y_pad;
        }
        let mut menu_clip_rect = Rectangle::new_coords(self.menu_x_pad, self.menu_y_pad, self.screensize.x - self.menu_x_pad, height);
        menu_clip_rect.style = DrawStyle {fill_color: Some(PixelColor::Dark), stroke_color: None, stroke_width: 0,};
        menu_canvas.set_clip(menu_clip_rect);
        gfx.draw_rectangle(menu_clip_rect).expect("can't clear menu");
        Ok(menu_clip_rect.br)
    }
    fn get_content_canvas(&self) -> Gid {
        self.menu
    }
}