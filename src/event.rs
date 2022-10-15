use druid::{Event, widget::Controller, Widget, EventCtx, Env};

use crate::ebook::{BookState, react};

#[allow(non_snake_case)]
pub struct EditModeEvent<F: Fn(&Event) -> bool> {
    pub filter: F,

}


#[allow(non_snake_case)]
impl<W: Widget<BookState>, F: Fn(&Event) -> bool> Controller<BookState, W> for EditModeEvent<F> {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut BookState, env: &Env) {
        if (self.filter)(event) {
            react(data);
        }
        // Always pass on the event!
        child.event(ctx, event, data, env)
    }
}
