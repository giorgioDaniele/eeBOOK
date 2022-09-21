use crate::*;

use druid::TextAlignment;
use druid::widget::Button;
use druid::widget::Flex;
use druid::widget::Image;
use druid::widget::Label;
use druid::widget::LineBreaking;
use druid::widget::RawLabel;
use druid::widget::Scroll;

use druid::widget::Split;
use druid::widget::TextBox;
use druid::widget::ViewSwitcher;
use druid::Env;
use druid::FileDialogOptions;
use druid::FileSpec;
use druid::ImageBuf;
use druid::Widget;
use druid::WidgetExt;

use super::search::search_chapter;





pub fn userinterface_builder() -> impl Widget<BookState> {

   
    let EPUB = FileSpec::new("EPUB Format", &["epub"]);
    let JPEG = FileSpec::new("JPEG Format", &["jpeg"]);
    let JPG  = FileSpec::new("JPG Format", &["jpg"]);
    let PNG  = FileSpec::new("PNG Format", &["png"]);

    let save_epub_dialog_options = FileDialogOptions::new()
        .allowed_types(vec![EPUB])
        .default_type(EPUB)
        .default_name(DEFAULT_SAVED_BOOK)
        .name_label("Target")
        .title("Choose a target for this lovely file")
        .button_text("Export");

    let open_epub_dialog_options = FileDialogOptions::new()
        .clone()
        .allowed_types(vec![EPUB])
        .name_label("Source")
        .button_text("Import");

    let open_image_dialog_options = FileDialogOptions::new()
        .clone()
        .allowed_types(vec![JPEG, JPG, PNG])
        .name_label("Source")
        .button_text("Import");

    // BOTTONE OCR
    let ocr_button = Button::<BookState>::new("Search by Image")
        .on_click(move |ctx, current, _env| {
            current.object_loaded = IMAGE_LOADING;
            ctx.submit_command(druid::commands::SHOW_OPEN_PANEL.with(open_image_dialog_options.clone()));
        })
        .padding(2.0).disabled_if(|data, _| 
            data.current_view == EDIT_MODE || 
            data.current_view == HELP_MODE || 
            data.current_view == IDLE);

    // BOTTONE DI APERTURE DEL FILE EXPLORER (APERTURA DI UN EPUB)
    let file_button = Button::<BookState>::new("Open Book")
        .on_click(move |ctx, current, _env| {
            current.current_view = IDLE;
            current.object_loaded = EPUB_LOADING;
            ctx.submit_command(druid::commands::SHOW_OPEN_PANEL.with(open_epub_dialog_options.clone()));
            
        })
        .padding(2.0);
    
    // BOTTONE DI APERTURE DEL FILE EXPLORER (SALVATAGGIO DI UN EPUB)
    let save_button = Button::<BookState>::new("Save Book")
        .on_click(move |ctx, current, _env| {
                ctx.submit_command(druid::commands::SHOW_SAVE_PANEL.with(save_epub_dialog_options.clone()));
        })
        .padding(2.0).disabled_if(|data, _| 
            data.current_view == READ_MODE || 
            data.current_view == IDLE ||
            data.current_view == HELP_MODE);
    
    // BOTTONE DI APERTURA DELLA MODALITA' EDIT
    let edit_button = Button::<BookState>::dynamic(|state: &BookState, _: &Env| {
        if state.current_view == READ_MODE || state.current_view == IDLE {
            let text = "Edit Book";
            format!("{}", text)
       }else {
            let text = "Read Book";
            format!("{}", text)
        }
    })
        .on_click(|_ctx, current, _env| {

            if current.epub_is_open {
                if current.current_view == READ_MODE {
                    current.current_view = EDIT_MODE;
                } else {
                    if current.current_view == EDIT_MODE {
                        current.current_view = READ_MODE;
                    }
                }
            }
            
        })
        .padding(2.0).disabled_if(|data, _| data.current_view == IDLE || data.current_view == HELP_MODE);
    
    // RICERCA NEL TESTO
    let search_text_button = Button::<BookState>::new("🔍")
        .on_click(|_ctx, current, _env| {
                search_chapter(current)
        })
        .padding(2.0).disabled_if(|data, _| 
            data.current_view != READ_MODE || 
            data.bar_text.is_empty() || 
            data.current_view == EDIT_MODE || 
            data.current_view == IDLE
            || data.current_view == HELP_MODE);

    // BOTTONE PER LA VISUALIZZAZIONE DELL'HELPER
    let help_button = Button::<BookState>::new("About")
        .on_click(|_ctx, current, _env| {

            if current.current_view == EDIT_MODE || current.current_view == READ_MODE || current.current_view == IDLE {
                current.swap_view(HELP_MODE);
            }else {
                if current.current_view == HELP_MODE {
                    if current.epub_is_open {
                        current.swap_view(READ_MODE);
                    }
                    else {
                        current.swap_view(IDLE);
                    }
                }
            }
        })
        .padding(2.0);
    
    // SCORRIMENTO IN AVANTI
    let next_page = Button::<BookState>::new("▷")
        .on_click(|_ctx, current: &mut BookState, _env| {
            if current.current_view == READ_MODE || current.current_view == EDIT_MODE {
                current.next_page()
            }
    })
        .padding(2.0).disabled_if(|data, _| 
            data.current_page_i32 == data.total_pages_i32 || data.current_view == IDLE || data.current_view == HELP_MODE);

    // SCORRIMENTO ALL'INDIETRO
    let previous_page = Button::<BookState>::new("◁")
        .on_click(|_ctx, current: &mut BookState, _env| {
            if current.current_view == READ_MODE || current.current_view == EDIT_MODE {
                current.previous_page()
            }
        })
        .padding(2.0).disabled_if(|data, _| 
            data.current_page_i32 == 1 || data.current_view == IDLE || data.current_view == HELP_MODE);
    
    // COMANDO DI SALTO
    let search_page = Button::<BookState>::new("Go to Chapter")
        .on_click(|_ctx, current: &mut BookState, _env| 
            if current.current_view == READ_MODE || current.current_view == EDIT_MODE {
                current.jump_to_page(JUMP_BY_NUMBER)
            })
        .padding(2.0).disabled_if(|data, _| data.current_view == IDLE || data.current_view == HELP_MODE);

    

    let toolbar = Flex::column().with_child(
        Flex::row()
        .with_child(file_button)
        .with_child(save_button)
        .with_child(edit_button)
        .with_child(help_button)
        .with_spacer(30.0)
        .with_child(previous_page)
        .with_default_spacer()
        .with_child(
                
                // CASELLA DI TESTO PER INSERIRE IL NUMERO DI PAGINA/VISUALIZZARE QUELLO CORRENTE
                TextBox::new()
                    .with_text_alignment(druid::TextAlignment::Center)
                    .with_placeholder("0")
                    .fix_width(40.0)
                    .fix_height(25.0)
                    .lens(BookState::current_page_string)
                    .border(PURPLE, 2.0)
                    .rounded(ROUNDED_VALUE)
                    .background(BLACK),

        )
        .with_spacer(TINY_SPACER)
        .with_child(
                
                // TESTO CHE RIPORTA LA LUNGHEZZA COMPLESSIVA DEL FILE EPUB
                Label::dynamic(|state: &BookState, _: &Env| {
                    format!("of {}", state.total_pages_i32)
                }),

        )
        .with_default_spacer()
        .with_child(next_page)
        .with_default_spacer()
        .with_child(search_page)
        .with_spacer(BIG_SPACER)
        .with_child(

                // CASELLA DI TESTO PER INSERIRE PAROLA DA CERCARE
                TextBox::new()
                .with_text_alignment(druid::TextAlignment::Center)
                .with_placeholder(EMPTY_STRING)
                .fix_width(200.0)
                .lens(BookState::bar_text)
                .border(PURPLE, 2.0)
                .rounded(ROUNDED_VALUE)
                .background(BLACK),

        )
        .with_default_spacer()
        .with_child(search_text_button)
        .with_default_spacer()
        .with_child(ocr_button)
        .align_left()
        .border(BLACK, 2.0)
        .rounded(ROUNDED_VALUE)
        .background(PURPLE), 
    );

    let screen = ViewSwitcher::new(
        |data: &BookState, _env| data.current_view,
        |selector, data, _env| match selector {

        
            &IDLE => Box::new(
            
                Label::new(EMPTY_STRING)
            ),

            &READ_MODE => Box::new(
                Split::columns(

                    {   

                        let image_data = ImageBuf::from_raw(
                                data.cover_pixels.clone(),
                                druid::piet::ImageFormat::RgbaPremul,
                                data.width_cover as usize,
                                data.height_cover as usize);
                        
                        let image_cover = Image::new(image_data);
                        image_cover.expand()

                    },

                    {
                        
                        // CREAZIONE DEL CASELLA DI TESTO (IN SOLA LETTURA) FORMATTATO
                        let mut text_to_display = RawLabel::new();
                        text_to_display.set_line_break_mode(LineBreaking::WordWrap);
                        text_to_display.set_text_alignment(TextAlignment::Justified);
                        Scroll::new(text_to_display
                            .lens(BookState::current_rich_text_page)
                            .padding(BIG_SPACER))
                            .vertical()
                    },

                )
                .split_point(0.4)
                .draggable(false)
                .solid_bar(true),
            ),

            &EDIT_MODE => Box::new(
                Split::columns(

                    {   

                        // CREAZIONE DI UNA FINESTRA INTERAGIBILE PER LA MODIFICA DEL TESTO HTML
                        TextBox::multiline()
                        .lens(BookState::current_html_page)
                        .padding(BIG_SPACER / 2.0)
                        .expand()

                    },

                    {
                        
                        // CREAZIONE DEL CASELLA DI TESTO (IN SOLA LETTURA) FORMATTATO
                        let mut text_to_display = RawLabel::new();
                        text_to_display.set_line_break_mode(LineBreaking::WordWrap);
                        text_to_display.set_text_alignment(TextAlignment::Justified);
                        Scroll::new(text_to_display
                            .lens(BookState::current_rich_text_page)
                            .padding(BIG_SPACER))
                            .vertical()
                    },

                )
                .draggable(false)
                .solid_bar(true),
            ),

            &HELP_MODE => Box::new({

                    // CREAZIONE DEL CASELLA DI TESTO DELL'HELPER
                    let mut text_to_displaynote = RawLabel::new();
                    text_to_displaynote.set_line_break_mode(LineBreaking::WordWrap);
                    text_to_displaynote.set_text_alignment(TextAlignment::Justified);
                    Scroll::new(text_to_displaynote
                        .lens(BookState::rich_text_help)
                        .padding(BIG_SPACER)).vertical()
                }
            ),

            _ => Box::new(Button::new("")),
        },
    );

    Flex::column()
        .with_child(toolbar)
        .with_flex_child(screen, 1.0)
}

