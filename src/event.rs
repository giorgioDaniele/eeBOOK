use druid::{Event, widget::Controller, Widget, EventCtx, Env, Data};

use crate::ebook::{BookState, reactToHTMLModification, reactToZoom};

#[allow(non_snake_case)]
pub struct EditModeEvent<F: Fn(&Event) -> bool> {
    pub filter: F,

}
#[allow(non_snake_case)]
impl<W: Widget<BookState>, F: Fn(&Event) -> bool> Controller<BookState, W> for EditModeEvent<F> {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut BookState, env: &Env) {
        if (self.filter)(event) {
            // AGGIORNA HTML
            reactToHTMLModification(data);
        }
        child.event(ctx, event, data, env)
    }
}

#[allow(non_snake_case)]
pub struct ZoomEvent;
#[allow(non_snake_case)]
impl<W: Widget<BookState>> Controller<BookState, W> for ZoomEvent {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut BookState,
        env: &Env,
    ) 

    {   
        let previousFont = data.getFontSize();
        child.event(ctx, event, data, env);
        if !data.getFontSize().same(&previousFont) {
            // AGGIORNA DIMENSIONE DEL TESTO
            reactToZoom(data);
        }
    }
}

