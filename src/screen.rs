// IMPORTAZIONI ESTERNE
use druid::{widget::{Label, LineBreaking, RawLabel, Scroll, Split, TextBox, ViewSwitcher}, TextAlignment, WidgetExt, Event};

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
                                        Err(_) => panic!(
                                            "[ERROR]: fatal error on loading the cover image"
                                        ),
                                    }
                                } else {
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
                    },
                )
                .split_point(0.4)
                .draggable(false)
                .solid_bar(true),
            ),

            &ViewState::EditMode => Box::new(
                Split::columns(
                    {
                        
                    
                        // SCHERMATA SINISTRA - CODICE HTML DELLA PAGINA VISUALIZZATA
                        TextBox::multiline()
                            .lens(BookState::currentHTML)
                            .padding(BIG_SPACER / 2.0)
                            .controller(EditModeEvent {
                                filter: |event| matches!(event, Event::KeyDown(_) | Event::KeyUp(_)),
                            })
                            .expand()

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
