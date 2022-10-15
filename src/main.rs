// IMPORTAZIONI ESTERNE
use druid::widget::Flex;
use druid::{WindowDesc, AppLauncher};
// IMPORTAZIONI INTERNE
use ebook::{BookState, Explorer};


mod constants;
mod chapter;
mod ebook;
mod toolbar;
mod screen;
mod search;
mod event;


fn main() {

    #[allow(non_snake_case)]
    let GUI = Flex::column()
    .with_child(toolbar::toolbar())
    .with_flex_child(screen::screen(), 1.0);

    let center = druid::Point::new(0 as f64, 0 as f64);

    // CREAZIONE DELLA FINESTRA
    let main_window = 
        WindowDesc::new(GUI)
        .title("eeBOOK")
        .set_position(center)
        .set_window_state(druid::WindowState::Maximized);

    let state = BookState::new();

    // LANCIO DELL'APPLICAZIONE
    AppLauncher::with_window(main_window)
        .delegate(Explorer)
        .log_to_console()
        .launch(state)
        .expect("[ERROR]: eeBook launch has failed\n");

}