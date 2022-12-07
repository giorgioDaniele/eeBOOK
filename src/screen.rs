use core::panic;

// IMPORTAZIONI ESTERNE
use druid::{widget::{Label, LineBreaking, RawLabel, Scroll, Split, TextBox, ViewSwitcher}, TextAlignment, WidgetExt, Event, FontDescriptor, FontStyle, FontWeight};
use image::GenericImageView;

// IMPORTAZIONI INTERNE
use crate::{constants::{BIG_SPACER, EMPTY_STRING}, ebook::{BookState, ViewState}, event::{EditModeEvent}};


#[allow(non_snake_case)]
pub fn screen() -> ViewSwitcher<BookState, ViewState> {


    ViewSwitcher::new(
        |bookState: &BookState, _| ViewState::clone(&bookState.getCurrentView()),
        |selector, _, _| match selector {

            &ViewState::Idle => Box::new(Label::new(EMPTY_STRING)),
            &ViewState::ReadMode => Box::new(
                Split::columns(
                    {
                        // AGGIORNAMENTO DINAMICO DELLA COPERTINA
                        ViewSwitcher::new(
                            |bookState: &BookState, _| bookState.getCurrID(),
                            |currID, bookState, _| {
                                
                                // ROBUSTO AD EVENTUALE OVERFLOW, DOVUTO ALL'APERTURA DI PIU' di 256 LIBRI CONSECUTIVAMENTE
                                if currID != &bookState.getUltID() {
                                    match druid::ImageBuf::from_data(&bookState.getCoverPixels()) {
                                        Ok(image) => Box::new(druid::widget::Image::new(image)),
                                        Err(_) => {
                                            let mut imgWidth = 0;
                                            let mut imgHeight = 0;

                                            if let Ok(reader) = image::io::Reader::open("no_image.jpg") {
                                                let (width, height) = reader.into_dimensions().unwrap();
                                                imgWidth = width;
                                                imgHeight = height;
                                            }

                                            if let Ok(reader) = image::io::Reader::open("no_image.jpg") {
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
                                                        Box::new(druid::widget::Image::new(image_data).expand())
                                                    } else {
                                                        Box::new(Label::new(""))
                                                    } 
                                            }else {
                                                Box::new(Label::new(""))
                                            }
                                    }
                                } 
                                }else {
                                    panic!("[ERROR]: this is must be unreachable");
                                }
                            },
                        )
                    },
                    {

                        // SCHERMATA DESTRA - TESTO DEL LIBRO APERTO (SULLA PAGINA CORRENTE)
                        let mut textToDisplay = RawLabel::new();
                        textToDisplay.set_line_break_mode(LineBreaking::WordWrap); // RITORNO A CAPO AUTOMATICO AL RAGGIUNGIMENTO DEL BORDO DELLA FINESTRA
                        textToDisplay.set_text_alignment(TextAlignment::Justified); // VISUALIZZAZIONE DEL TESTO GIUSTIFICATO

                        Scroll::new(
                            textToDisplay
                                .lens(BookState::currentRTFpage)
                                .padding(BIG_SPACER),
                        )
                        .vertical()
                        .expand()
                    }
                )
                .split_point(0.4)
                .draggable(false)
                .solid_bar(true)
            ),

            &ViewState::EditMode => Box::new(
                Split::columns(
                    {
                        
                    
                        // SCHERMATA SINISTRA - CODICE HTML DELLA PAGINA VISUALIZZATA
                        let htmlInterface = TextBox::multiline()
                        .with_font(FontDescriptor::default()
                            .with_style(FontStyle::Regular)
                            .with_weight(FontWeight::NORMAL))
                            .with_text_size(20.0)
                            .lens(BookState::currentHTML)
                            .padding(BIG_SPACER / 2.0)
                            .controller(EditModeEvent {
                                filter: |event| matches!(event, Event::KeyDown(_) | Event::KeyUp(_)),
                            })
                            .expand();
                        htmlInterface

                    },
                    {
                        // SCHERMATA DESTRA - TESTO RENDERIZZATO CORRISPONDENTE ALL'HTML
                        let mut textToDisplay = RawLabel::new();
                        textToDisplay.set_line_break_mode(LineBreaking::WordWrap);
                        textToDisplay.set_text_alignment(TextAlignment::Justified);
                        Scroll::new(
                            textToDisplay
                                .lens(BookState::currentRTFchap)
                                .padding(BIG_SPACER),
                        )
                        .vertical()
                    },
                )
                .draggable(false)
                .solid_bar(true),
            ),

            &ViewState::HelpMode => Box::new({
                let mut textToDisplay = RawLabel::new();
                textToDisplay.set_line_break_mode(LineBreaking::WordWrap);
                textToDisplay.set_text_alignment(TextAlignment::Justified);

                Scroll::new(textToDisplay)
                    .lens(BookState::helper)
                    .padding(BIG_SPACER)
                    .expand()
            }),
        },
    )
}
