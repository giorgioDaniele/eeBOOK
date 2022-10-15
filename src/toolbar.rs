// IMPORTAZIONI ESTERNE
use druid::{widget::{Flex, Button, ControllerHost, Padding, Click, DisabledIf, TextBox, Container, Label}, FileSpec, FileDialogOptions, WidgetExt, WindowDesc};
use druid::widget::ViewSwitcher;
use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImageView, GrayImage, RgbImage};
use druid::Screen;
// IMPORTAZIONI INTERNE
use crate::{ebook::{BookState, ViewState}, constants::{GO_PREV, GO_NEXT, JUMP_BY_BUTTON, EPUB_LOADING, PURPLE, ROUNDED_VALUE, BLACK, EMPTY_STRING, IMAGE_LOADING, BIG_SPACER, TINY_SPACER}, search::searchPageFromText};
use crate::search::reverseOCR;


#[allow(non_snake_case)]
fn fileButton () -> Padding<BookState, ControllerHost<Button<BookState>, Click<BookState>>> {

        // FORMATI DELLE RISORSE DA POTER APRIRE
        let EPUB = FileSpec::new("EPUB Format", &["epub"]);

        // PERMETTE ALL'UTENTE DI SCEGLIERE UN PERCORSO DI APERTURA
        // DI UNA RISORSA CON ESTENSIONE EPUB
        let openEpubDialogOptions = 
            FileDialogOptions::new()
            .clone()
            .allowed_types(vec![EPUB]);

            Button::<BookState>::new("Open Book")
            .on_click(move |ctx, bookState, _| {
                bookState.setObjectType(EPUB_LOADING);
                ctx.submit_command(druid::commands::SHOW_OPEN_PANEL.with(openEpubDialogOptions.clone()));
        })
        .padding(2.0)
}
#[allow(non_snake_case)]
fn saveButton () -> DisabledIf<BookState, Padding<BookState, ControllerHost<Button<BookState>, Click<BookState>>>> {
    
    // FORMATI DELLE RISORSE DA POTER APRIRE
    let EPUB = FileSpec::new("EPUB Format", &["epub"]);

    // PERMETTE ALL'UTENTE DI SCEGLIERE UN PERCORSO DI SALVATAGGIO
    // DI UNA RISORSA CON ESTENSIONE EPUB
    let saveEpubDialogOptions = 
            FileDialogOptions::new()
            .allowed_types(vec![EPUB])
            .default_type(EPUB);

    Button::<BookState>::new("Save Book")
            .on_click(move |ctx, _, _| {
                    ctx.submit_command(druid::commands::SHOW_SAVE_PANEL.with(saveEpubDialogOptions.clone()));
        })
        .padding(2.0)
        .disabled_if(|bookState, _|  
                bookState.getCurrentView()  == ViewState::ReadMode ||  
                bookState.getCurrentView() == ViewState::Idle || 
                bookState.getCurrentView() == ViewState::HelpMode)
    
}
#[allow(non_snake_case)]
fn readEditButton () -> DisabledIf<BookState, Padding<BookState, ControllerHost<Button<BookState>, Click<BookState>>>> {

    Button::dynamic(|bookState: &BookState, _| {
        match bookState.getCurrentView() {
            ViewState::ReadMode  =>  format!("Edit"),
            ViewState::EditMode  =>  format!("Read"),
            ViewState::Idle      =>  format!("Read"),
            ViewState::HelpMode  =>  format!("Read"),
        }
        })
        .on_click(|_, bookState, _| {

            match bookState.getEpubOpen() {
                
                true    => {
                    match bookState.getCurrentView() {
                        ViewState::ReadMode => {
                            bookState.setCurrentView(ViewState::EditMode);
                            bookState.setCursor(bookState.getCurrentChap());
                        }
                        ViewState::EditMode  => {
                            bookState.setCurrentView(ViewState::ReadMode);
                            bookState.setCursor(bookState.getCurrentPage());
                        }
                        ViewState::HelpMode      => {
                            match bookState.getUltimateView() {
                                ViewState::ReadMode => {
                                    bookState.setCurrentView(ViewState::ReadMode);
                                    bookState.setCursor(bookState.getCurrentPage());
                                }
                                ViewState::EditMode => {
                                    bookState.setCurrentView(ViewState::EditMode);
                                    bookState.setCursor(bookState.getCurrentChap()); 
                                }
                                ViewState::HelpMode  => panic!("[ERROR]: no a transition on READ/EDIT Button clicking"),
                                ViewState::Idle      => bookState.setCurrentView(ViewState::Idle),
                            }
                        }
                        ViewState::Idle => panic!("[ERROR]: no a transition on READ/EDIT Button clicking"),
                    }
                },
                false   => {}
            }
        })
    .padding(2.0)
    .disabled_if(|bookState, _| 
        bookState.getCurrentView() == ViewState::Idle || 
        (bookState.getCurrentView() == ViewState::HelpMode && bookState.getEpubOpen() == false))
}
#[allow(non_snake_case)]
fn previousPage () -> DisabledIf<BookState, Padding<BookState, ControllerHost<Button<BookState>, Click<BookState>>>> {

    Button::new("◁")
            .on_click(|_, bookState: &mut BookState, _| {
                if bookState.getCurrentView()== ViewState::ReadMode || bookState.getCurrentView() == ViewState::EditMode {
                    bookState.scroll(GO_PREV);
                }
        })
        .padding(2.0)
        .disabled_if(|BookState, _| 
            BookState.getCurrentPage() == 1 || BookState.getCurrentView() == ViewState::Idle || BookState.getCurrentView() == ViewState::HelpMode)
}
#[allow(non_snake_case)]
fn currentPageBox () -> Container<BookState> {

    TextBox::new()
            .with_text_alignment(druid::TextAlignment::Center)              
            .with_placeholder("0")                                         
            .fix_width(40.0)                                              
            .fix_height(25.0)                                              
            .lens(BookState::cursor)                         
            .border(PURPLE, 2.0)   
            .rounded(ROUNDED_VALUE)
            .background(BLACK)
}
#[allow(non_snake_case)]
fn totalPagesLabel () -> Label<BookState>  {

    Label::dynamic(|bookState: &BookState, _| {
    
        match bookState.getCurrentView() {
            ViewState::ReadMode =>  format!("of {}", bookState.getTotalPages()),
            ViewState::EditMode =>  format!("of {}", bookState.getTotalChaps()),
            ViewState::Idle      =>  format!("of {}", bookState.getTotalChaps()),
            ViewState::HelpMode     => {

                match bookState.getUltimateView() {
                    ViewState::ReadMode => format!("of {}", bookState.getTotalChaps()),
                    ViewState::EditMode => format!("of {}", bookState.getTotalChaps()),
                    ViewState::HelpMode => format!("of {}", bookState.getTotalPages()),
                    ViewState::Idle      => format!("of {}", bookState.getTotalPages()),
                }
            }
        }
    })

}
#[allow(non_snake_case)]
fn nextPage () -> DisabledIf<BookState, Padding<BookState, ControllerHost<Button<BookState>, Click<BookState>>>> {

    Button::new("▷")
            .on_click(|_, bookState: & mut BookState, _| {
                if bookState.getCurrentView() == ViewState::ReadMode || bookState.getCurrentView() == ViewState::EditMode {
                    bookState.scroll(GO_NEXT);
                }
        })
        .padding(2.0)
        .disabled_if(|bookState, _| 
            bookState.getCurrentPage() == bookState.getTotalPages()|| 
            bookState.getCurrentView() == ViewState::Idle || 
            bookState.getCurrentView() == ViewState::HelpMode)
}
#[allow(non_snake_case)]
fn jumpButton () -> DisabledIf<BookState, Padding<BookState, ControllerHost<Button<BookState>, Click<BookState>>>>  {

    Button::dynamic(|bookState: &BookState, _| {

        match bookState.getCurrentView() {

            ViewState::ReadMode  =>  format!("Go to Page"),
            ViewState::EditMode  =>  format!("Go to Chapter"),
            ViewState::Idle      =>  format!("Go to Page"),
            ViewState::HelpMode  =>  {                
                
                match bookState.getUltimateView() {

                ViewState::ReadMode => format!("Go to Page"),
                ViewState::EditMode => format!("Go to Chapter"),
                ViewState::HelpMode => panic!("[ERROR]: such a no state"),
                ViewState::Idle =>     format!("Go to Page"),
                }
            },
        }
    })
        .on_click(|_, bookState: &mut BookState, _| 
            if bookState.getCurrentView() == ViewState::ReadMode || bookState.getCurrentView() == ViewState::EditMode {
                bookState.jump(JUMP_BY_BUTTON)
            })
        .padding(2.0)
        .disabled_if(|bookState, _| 
            bookState.getCurrentView() == ViewState::Idle || bookState.getCurrentView() == ViewState::HelpMode)
}
#[allow(non_snake_case)]
fn textSearchBar () -> Container<BookState> {
    
    TextBox::new()
    .with_text_alignment(druid::TextAlignment::Center)
    .with_placeholder(EMPTY_STRING)
    .fix_width(200.0)
    .lens(BookState::barText)
    .border(PURPLE, 2.0)
    .rounded(ROUNDED_VALUE)
    .background(BLACK)
}
#[allow(non_snake_case)]
fn textSearchButton () -> DisabledIf<BookState, Padding<BookState, ControllerHost<Button<BookState>, Click<BookState>>>>{

    Button::new("🔍")
        .on_click(|_, bookState, _| {
            searchPageFromText(bookState, &bookState.getBarText());
        })
        .padding(2.0)
        .disabled_if(|bookState: & BookState, _| 
            bookState.getCurrentView() != ViewState::ReadMode || 
            bookState.getBarText().is_empty() || 
            bookState.getCurrentView() == ViewState::EditMode || 
            bookState.getCurrentView() == ViewState::Idle
            || bookState.getCurrentView() == ViewState::HelpMode)
}
#[allow(non_snake_case)]
fn ocrSearchButton () -> DisabledIf<BookState, Padding<BookState, ControllerHost<Button<BookState>, Click<BookState>>>> {

    let JPEG = FileSpec::new("JPEG Format", &["jpeg"]);
    let JPG  = FileSpec::new("JPG Format", &["jpg"]);
    let PNG  = FileSpec::new("PNG Format", &["png"]);

    let openImageDialogOptions = FileDialogOptions::new()
    .clone()
    .allowed_types(vec![JPEG, JPG, PNG])
    .name_label("Source")
    .button_text("Import");

    Button::new("📷")
        .on_click(move |ctx, bookState : & mut BookState, _env| {
            bookState.setObjectType(IMAGE_LOADING);
            ctx.submit_command(druid::commands::SHOW_OPEN_PANEL.with(openImageDialogOptions.clone()));
        })
        .padding(2.0).disabled_if(|bookState, _| 
            bookState.getCurrentView() == ViewState::EditMode || 
            bookState.getCurrentView()  == ViewState::HelpMode || 
            bookState.getCurrentView()  == ViewState::Idle)
}

#[allow(non_snake_case)]
fn ocrReverseButton () -> DisabledIf<BookState, Padding<BookState, ControllerHost<Button<BookState>, Click<BookState>>>> {

    Button::new("🖼️")
        .on_click(move |ctx, bookState : & mut BookState, _env| {

            let path = reverseOCR(bookState);
            bookState.setPathImageViewed(path.clone());

            let mut imgWidth = 0;
            let mut imgHeight = 0;
            if let Ok(reader) = image::io::Reader::open(bookState.getPathImageViewed()) {
                let (width, height) = reader.into_dimensions().unwrap();
                imgWidth = width;
                imgHeight = height;
            }

            if let Ok(reader) = image::io::Reader::open(bookState.getPathImageViewed()) {
                if let Ok(dynamicImage) = reader.decode() {
                    let mut imgData = Vec::new();
                    dynamicImage.pixels()
                        .map(|pictureElement| pictureElement.2)
                        .for_each(|pixel| {
                            imgData.push(pixel.0[0]); // RED
                            imgData.push(pixel.0[1]); // GREEN
                            imgData.push(pixel.0[2]); // BLUE
                            imgData.push(pixel.0[3]); // ALPHA
                        });
                    let image_data = druid::ImageBuf::from_raw(
                        imgData,
                        druid::piet::ImageFormat::RgbaPremul,
                        imgWidth as usize,
                        imgHeight as usize);
                    let img = druid::widget::Image::new(image_data);
                    let ui = Flex::<BookState>::column().with_child(img).boxed();

                    let rect= Screen::get_display_rect();
                    let center = druid::Point::new((rect.x1 as u32 - imgWidth) as f64 /2 as f64, 0 as f64);

                    ctx.new_window(WindowDesc::new(ui)
                        .title("Immagine trovata")
                        .set_position(center)
                        .window_size((imgWidth as f64, imgHeight as f64+10.0) ));
                }
            }

        })
        .padding(2.0).disabled_if(|bookState, _|

            bookState.getImagesPath().is_empty() ||
            bookState.getCurrentView() == ViewState::EditMode ||
            bookState.getCurrentView()  == ViewState::HelpMode ||
            bookState.getCurrentView()  == ViewState::Idle)
}


#[allow(non_snake_case)]
fn helperButton () -> ControllerHost<Button<BookState>, Click<BookState>> {

    Button::new("Help")
        .on_click(|_, bookState : & mut BookState, _| {


            match bookState.getCurrentView() {

                ViewState::ReadMode => bookState.setCurrentView(ViewState::HelpMode),
                ViewState::EditMode => bookState.setCurrentView(ViewState::HelpMode),
                ViewState::HelpMode => {
                    match bookState.getUltimateView() {
                        ViewState::ReadMode => bookState.setCurrentView(ViewState::ReadMode),
                        ViewState::EditMode => bookState.setCurrentView(ViewState::EditMode),
                        ViewState::HelpMode => panic!("[ERROR]: no such a state"),
                        ViewState::Idle => bookState.setCurrentView(ViewState::Idle),
                    }
                },
                ViewState::Idle => bookState.setCurrentView(ViewState::HelpMode),
            }
        })
    
}


#[allow(non_snake_case)]
pub fn toolbar () -> Flex<BookState> {

    Flex::column()
    .with_child(Flex::row()
        .with_child(fileButton())
        .with_child(saveButton())
        .with_child(readEditButton())
        .with_spacer(BIG_SPACER)
        .with_child(helperButton())
        .with_spacer(30.0)
        .with_child(previousPage())
        .with_default_spacer()
        .with_child(currentPageBox())
        .with_spacer(TINY_SPACER)
        .with_child(totalPagesLabel())
        .with_default_spacer()
        .with_child(nextPage())
        .with_spacer(BIG_SPACER)
        .with_child(jumpButton())
        .with_spacer(BIG_SPACER)
        .with_child(textSearchBar())
        .with_default_spacer()
        .with_child(textSearchButton())
        .with_default_spacer()
        .with_child(ocrSearchButton())
        .with_default_spacer()
        .with_child(ocrReverseButton())
        .align_left()
        .border(BLACK, 2.0)
        .rounded(ROUNDED_VALUE)
        .background(PURPLE), 
    )

}