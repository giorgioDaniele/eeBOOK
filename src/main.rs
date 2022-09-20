#![windows_subsystem = "windows"]

const PURPLE             : druid::Color = Color::rgb8(100, 32, 240);
const BLACK              : druid::Color  = Color::rgb8(0, 0, 0);
const DEFAULT_SAVED_BOOK : &str = "eeBOOK.epub";
const TINY_SPACER        : f64 = 2.0;
const BIG_SPACER         : f64 = 30.0;
const ROUNDED_VALUE      : f64 = 5.0;
const READ_MODE          : u8 = 0;
const EDIT_MODE          : u8 = 1;
const IDLE               : u8 = 2;
const HELP_MODE          : u8 = 3;
const EMPTY_STRING       : &str = "";
const ZERO_STRING        : &str =  "0";

mod ebook_mod;
use druid::text::RichText;
use druid::{AppLauncher, FileInfo};
use druid::Color;
use druid::Data;
use druid::Lens;
use druid::WindowDesc;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;


use crate::ebook_mod::rendering::*;
use crate::ebook_mod::widget::*;


/// IL MECCANISMO DI CREAZIONE DI UN INTERFACCIA GRAFICA E' BASATO
/// SULLA REALIZZAZIONE DI UNA STRUTTURA DATI ATTORNO ALLA QUALE
/// EDIFICARE I WIDGET CHE RAPPRESENTANO GLI ELEMENTANO INTERAGIBILI
/// DELL'APPLICAZIONE STESSA
/// LA STRUTTURA DATI UTILIZZATA E' LA STRUCT BookState


struct Delegate;
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum FormattingInfo {
    Title,
    Heading,
    Heading2,
    Bold,
    Italic,
    Ignore,
}

#[derive(Clone, Data, Default)]
pub struct BookMetadata {

    author:        String,
    title:         String,
    stylesheet:    String,
    description:   String,
    language:      String,
    generator:     String,
    cover_image:   String,
   
}

#[derive(Clone, Data, Lens)]

pub struct BookState {

    // MISURE DELL'ARCHIVIO EPUB APERTO
    current_page_i32:    i32,
    total_pages_i32:     i32,
    current_page_string: String,
    total_page_string:   String,
    metadata:            BookMetadata,

    epub_filepath:       String,
    epub_is_open:        bool,

    // INFORMAZIONI PER LA PRESENTAZIONE DELLA GUI
    current_view:           u8,
    ultimate_view:          u8, 
    // VETTORE CONTENENTE HTML PURO
    #[data(eq)]
    raw_pages:              Vec<String>,
    // VETTORE CONTENENTE HTML MODIFICATO DALL'UTENTE
    #[data(eq)]
    raw_pages_modified:     Vec<String>,
    // VETTORE CONTENTENTE IL TESTO FORMATTATO
    #[data(eq)]
    parsed_pages:           Vec<String>,
    // MAPPA CONTENTENTE LE IMPOSTAZIONI DI FORMATO DEL TESTO
    #[data(eq)]
    formatting_info:        HashMap<(usize, usize), FormattingInfo>,
    // INFORMAZIONI VOLATILI PER IL RENDENDERING DEL TESTO
    current_text_page:      String,
    current_rich_text_page: RichText,
    current_html_page:      String, 
    rich_text_help:         RichText,

    // INFORMAZIONI PER IL RENDERING DELLA COPERTINA DEL LIBRO
    #[data(eq)]
    cover_pixels:    Vec<u8>,
    book_has_cover:  bool,
    width_cover:     u32,
    height_cover:    u32,

    #[data(ignore)]
    epub_path:      FileInfo,
}


pub fn main() {

    let center = druid::Point::new(0 as f64, 0 as f64);

    let main_window = 
        WindowDesc::new(userinterface_builder())
        .title("eeBOOK")
        .set_position(center)
        .set_window_state(druid::WindowState::Maximized);

    let state = BookState::new();

    AppLauncher::with_window(main_window)
        .delegate(Delegate)
        .log_to_console()
        .launch(state)
        .expect("[ERROR]: eeBook launch has failed\n");
}
