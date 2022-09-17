use crate::*;

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use druid::FontFamily;
use druid::FontStyle;
use druid::FontWeight;
use druid::text::Attribute;
use druid::text::RichText;
use druid::commands;
use druid::AppDelegate;
use druid::Command;
use druid::DelegateCtx;
use druid::Env;
use druid::Handled;
use druid::Target;
use druid::ArcStr;
use epub::doc::EpubDoc;
use epub_builder::EpubBuilder;
use epub_builder::EpubContent;
use epub_builder::ReferenceType;
use epub_builder::ZipLibrary;
use image::GenericImageView;
use image::Rgba;



// COSTANTE CHE RAPPRESENTA L'HELPER DELL'APPLICAZIONE (GRAZIE CLAUDIO)
const TEXT: &str = "\neeBOOK Reader©\n
Welcome to our Epub Reader. This application, developed in Rust language in collaboration with the Politecnico di Torino, allows you to read your digital books in epub format. In addition to reading and searching for chapters, the application authorizes the user to edit their books and save them.
The implementation of OCR recognition is under development.\n\n
Hoping that the app is to your liking, a greeting from the developers
Giorgio Daniele Luppina
Claudio Di Maida";





/// METODI ASSOCIATI PER LA CREAZIONE DEL FILE EPUB
/// 
/// rewind_epub_cursor PERMETTE DI RIPOSIZIONARE
/// IL LETTORE DELL'ARCHIVIO EPUB ALL'INIZIO DELL'ARCHIVIO STESSO.
/// QUESTA OPERAZIONE E' DI INIZIALIZZAZIONE
/// 
/// create_epub PERMETTE DI CREARE UN FILE IN FORMATO .epub
/// ATTRAVERSO IL CARICAMENTO DELLE PAGINE HTML CHE LO COSTITUISCONO
/// E I METADATI CHE LO CARATTERIZZANO



pub fn rewind_epub_cursor(epub_doc: & mut EpubDoc<BufReader<File>>) {
    while epub_doc.go_prev().is_ok() {}
}
pub fn create_epub(epub: &mut Vec<u8>, meta: & BookMetadata, html: & Vec<String>) -> Result<String, String> {
    

    if let Ok(mut builder) = EpubBuilder::new(ZipLibrary::new().unwrap()) {

        builder.metadata("author", meta.author.clone()).unwrap();
        builder.metadata("title",  meta.title.clone()).unwrap();
        builder.stylesheet(meta.stylesheet.as_bytes()).unwrap();

        let mut contents = Vec::new();
     
        
        for (index, html_source_code) in html.iter().enumerate() {
                if let Some(current_chapter_name) = meta.titles.get(index) {
                    contents.push(
                        EpubContent::new(current_chapter_name, html_source_code.as_bytes())
                            .title(current_chapter_name)
                            .reftype(ReferenceType::Text),
                    );
                }
        }
    
        for content in contents {
            builder.add_content(content);
        }

        builder.inline_toc();
        builder.generate(epub);
        return Ok("DONE".to_string())
    }
    Err("ERROR".to_string())


}


/// L'IMPLEMENTAZIONE DI Delegate E' QUEL MECCANISMO CHE PERMETTE
/// ALLA LIBRERIA Druid DI DIALOGARE CON IL SOTTOSITEMA DEL FILE SYSTEM
/// DEL SISTEMA OPERATIVO. NEL CASO D'USO, SI CHIEDE AL SISTEMA OPERATIVO
/// L'APERTURA E IL SALVATAGGIO SU DISCO


/// LA LETTURA E L'ACQUISIZIONE DEL FILE EPUB E' SVOLTA ATTRAVERSO
/// LA PROGRAMMAZIONE MULTITHREAD
/// ATTRAVERSO IL PARSING IN PARALLELO DI UN  NUMERO THREAD PARI
/// AL NUMERO DI FILE HTML DA PROCESSARE



impl AppDelegate<BookState> for Delegate {

    fn command(&mut self, ctx: &mut DelegateCtx, _target: Target, cmd: &Command, data: &mut BookState, _env: &Env) -> Handled {

        // SALVATAGGIO EPUB
        if let Some(file_info) = cmd.get(commands::SAVE_FILE_AS) {

            data.save_current_modified_page();
            let mut epub = Vec::new();
            let mut file = File::create(file_info.path()).unwrap();
            if let Ok(_result) = create_epub(&mut epub, &data.metadata, &data.raw_pages_modified) {
                if let Ok(_) = file.write_all(&epub) {
                    println!("DONE");
                }
            }
            return Handled::Yes;
        }

        // APERTURA EPUB
        if let Some(file_info) = cmd.get(commands::OPEN_FILE) {

            if !data.parsed_pages.is_empty() {
                data.clear_all();
            }

            let mut page = 0;

            if let Ok(mut epub_doc) = EpubDoc::new(file_info.path()) {

                // CATTURA DEI METADATI DEL FILE EPUB APERTO
                data.fill_metadata(&mut epub_doc);

                // CATTURA DEI PIXEL DELLA COPERTINA E LORO CARICAMENTO
                if let Ok(cover) = epub_doc.get_cover() {
                    data.cover_pixels = cover;
                    data.book_has_cover = true;
                    data.set_cover();
                }
                else {
                    data.book_has_cover = false;
                    data.set_default_cover();   
                }

                let (
                    transmit, 
                    receive) = 
                    channel::<(HashMap<(usize, usize), FormattingInfo>, String, usize)>
                    ();

                let turn = Arc::new((Mutex::new(1), Condvar::new()));
                
                rewind_epub_cursor(&mut epub_doc);
                while epub_doc.go_next().is_ok() {


                    if let Ok(current_html_page) = epub_doc.get_current_str() {

                        let transmit = Sender::clone(&transmit);
                        let turn = Arc::clone(&turn);

                        // CLONAZIONE NECESSARIA PER L'INSERIMENTO DELLA STESSA RISORSA IN STRUTTURE DATI DIFFERENTI
                        let html_first_source  = current_html_page.clone();
                        let html_second_source = current_html_page.clone();
                        let html_third_source = current_html_page.clone();

                        // AGGIORNAMENTO DEL NUMERO DI PAGINA HTML CORRENTE DA PROCESSARE
                        page += 1;

                        data.raw_pages.push(html_first_source);
                        data.raw_pages_modified.push(html_third_source);

                        // CREAZIONE DEL THREAD
                        std::thread::spawn(move || {

                            let mut partial_map = HashMap::new(); 
                            let parsed_string = 
                                parse_calibre(&html_second_source, page as usize, &mut partial_map);
                                if let Ok(mut current_turn) = turn.0.lock() {

                                    // MECCANISMO DI ATTESA ESCOGITATO PER GARANTIRE LA TRASMISSIONE IN ORDINE DELLA PAGINE PROCESSATE
                                    while current_turn.ne(&page) {
                                        current_turn = turn.1.wait(current_turn).unwrap();
                                    }
                                    transmit.send((partial_map, parsed_string, page)).unwrap();
                                }
                        });

                    }
                }
                drop(transmit);

                // RECUPERO DELLE RISORSE PROCESSATE DAI THREAD WORKERS
                for (partial_map, parsed_string, _n_page) in &receive {
                    for (key, value) in partial_map {
                                data.formatting_info.insert(key, value);
                    }
                    data.parsed_pages.push(parsed_string);
                    if let Ok(mut current_turn) = turn.0.lock() {
                        *current_turn = *current_turn + 1;
                        turn.1.notify_all();
                    }   
                }
                
            }
            else {
                panic!("[ERROR]: can not open the book");
            }

            // INIZIALIZZAZIONE GENERALE DI TUTTI GLI ATTRIBUTI CARATTERISITICI DEL LIBRO APERTO
            data.first_setup(1, page as i32);
            return Handled::Yes;
        }
        Handled::No
    }
}

impl BookState {

pub fn new() -> Self {

        BookState {

            // TROVI LA DESCRIZIONE DEI CAMPI NEL main.rs
            current_page_i32:    0,
            total_pages_i32:     0,
            current_page_string: String::from(ZERO_STRING),
            total_page_string:   String::from(ZERO_STRING),
            metadata:            BookMetadata::default(),

            epub_is_open:        false,
            epub_filepath:       String::from(EMPTY_STRING),

            raw_pages:           Vec::new(),
            raw_pages_modified:  Vec::new(),
            parsed_pages:        Vec::new(),
            formatting_info:     HashMap::new(),
        
            current_text_page:      String::from(EMPTY_STRING),
            current_html_page:      String::from(EMPTY_STRING),
            current_rich_text_page: RichText::new(ArcStr::from(EMPTY_STRING)),
        
            current_view: IDLE,
            ultimate_view: IDLE,

            cover_pixels:   Vec::new(),
            book_has_cover: false,
            width_cover:    0,
            height_cover:   0,


            rich_text_help: RichText::new(TEXT.into())
            .with_attribute(0..16, Attribute::text_color(Color::rgb(1.0, 0.2, 0.1)))
            .with_attribute(0..16, Attribute::size(28.0))
            .with_attribute(0..16, Attribute::font_family(FontFamily::MONOSPACE))
            .with_attribute(115..138, Attribute::style(FontStyle::Italic))
            .with_attribute(17.., Attribute::size(16.0))
            .with_attribute(447.., Attribute::weight(FontWeight::BOLD)),

            
            
        }
    }
pub fn next_page(self: &mut Self) {

        if self.epub_is_open {

            self.current_page_i32 = self.current_page_string.parse::<i32>().unwrap();

            // SALVATAGGIO DELLA PAGINA EVENTUALMENTE MODIFICATA
            self.save_current_modified_page();
            self.current_page_i32 += 1;

            if self.current_page_i32 >= self.total_pages_i32 {
                self.current_page_i32 = self.total_pages_i32;
            }

            self.current_page_string = self.current_page_i32.to_string();
            let index = (self.current_page_i32 - 1) as usize;

            if let Some(html) = self.raw_pages.get(index)  {
                self.current_html_page = html.to_string();
            }

            if let Some(text) = self.parsed_pages.get(index) {
                self.current_text_page = text.clone();
                self.current_rich_text_page = create_rich_page(&self.current_text_page, self.current_page_i32 as usize, &self.formatting_info);
            }
        }
        else { //NON FARE NULLA
        }

    }
pub fn previous_page(self: &mut Self) {

        if self.epub_is_open {

            self.current_page_i32 = self.current_page_string.parse::<i32>().unwrap();
            // SALVATAGGIO DELLA PAGINA EVENTUALMENTE MODIFICATA
            self.save_current_modified_page();
            self.current_page_i32 -= 1;

            if self.current_page_i32 <= 1 {
                self.current_page_i32 = 1;
            }

            self.current_page_string = self.current_page_i32.to_string();
            let index = (self.current_page_i32 - 1) as usize;

            if let Some(html) = self.raw_pages.get(index) {
                self.current_html_page = html.to_string();
            }

            if let Some(text) = self.parsed_pages.get(index) {
        
                self.current_text_page = text.clone();
                self.current_rich_text_page = create_rich_page(&self.current_text_page, self.current_page_i32 as usize, &self.formatting_info);
            }
        }
        else { //NON FARE NULLA
        }

}
pub fn jump_to_page(self: &mut Self) {

        if self.epub_is_open {

            // SE L'INPUT DELL'UTENTE (SOLO NUMERI)
            if self.current_page_string.bytes().all(|ch| ch.is_ascii_digit()) {

                let page_to_jump_to = self.current_page_string.parse::<i32>().unwrap();

                if page_to_jump_to <= self.total_pages_i32 && page_to_jump_to >= 0 {

                    // SALVATAGGIO DELLA PAGINA EVENTUALMENTE MODIFICATA
                    self.save_current_modified_page();
                    self.current_page_i32 = page_to_jump_to;

                    self.current_page_string = self.current_page_i32.to_string();
                    let index = (self.current_page_i32 - 1) as usize;

                    if let Some(html) = self.raw_pages.get(index)  {
                        self.current_html_page = html.to_string();
                    }
                    if let Some(text) = self.parsed_pages.get(index) {
                        self.current_text_page = text.clone();
                        self.current_rich_text_page = create_rich_page(&self.current_text_page, self.current_page_i32 as usize, &self.formatting_info);
                    }
                }
                // CONTROLLO SULL'INTERNVALLO DI PAGINE CARICATE
                if page_to_jump_to > self.total_pages_i32 {
                    self.current_page_string = self.current_page_i32.to_string();
                }
                if page_to_jump_to < 0 {
                    self.current_page_string = self.current_page_i32.to_string();
                }
            }
            else {
                self.current_page_string = self.current_page_i32.to_string();
            }
        }
        else {// NON FARE NIENTE
        }

    }
pub fn clear_all(self: &mut Self) {

        // AZZERAMENTO DELLO STATO DELL'APPLICAZIONE
        self.epub_filepath          = String::from(EMPTY_STRING);

        self.current_page_i32       = 1;
        self.total_pages_i32        = 0;

        self.current_page_string    = String::from(ZERO_STRING);
        self.total_page_string      = String::from(ZERO_STRING);
        self.current_text_page      = String::from(EMPTY_STRING);

        self.raw_pages.clear();
        self.raw_pages_modified.clear();
        self.metadata.titles.clear();
        self.parsed_pages.clear();
        self.formatting_info.clear();
        self.cover_pixels.clear();
        self.book_has_cover = false;
        self.width_cover    = 0;
        self.height_cover   = 0;

        self.epub_is_open = false;
        self.current_rich_text_page = RichText::new(ArcStr::from(EMPTY_STRING));
        self.current_view = IDLE;

     

}
pub fn first_setup (self: & mut Self, current_page: i32, total_pages: i32) {

        self.epub_is_open = true;

        self.current_page_i32    = current_page;
        self.total_pages_i32     = total_pages;
        self.current_page_string = current_page.to_string();
        self.total_page_string   = total_pages.to_string();

        self.current_page_string = self.current_page_i32.to_string();
        let index = (self.current_page_i32 - 1) as usize;

        if let Some(html) = self.raw_pages.get(index) {
            self.current_html_page = html.to_string();
        }

        if let Some(text) = self.parsed_pages.get(index) {
            self.current_text_page = text.clone();
            self.current_rich_text_page = create_rich_page(&self.current_text_page, self.current_page_i32 as usize, &self.formatting_info);
        }
        self.current_view = READ_MODE;
}
pub fn set_cover(self: & mut Self) {

        let mut pixels = Vec::new();
        for texel in self.cover_pixels.iter() {
            pixels.push(texel.clone());
        }

        let mut image_data = Vec::new();

        if let Ok(mut file) = fs::File::create("cover.jpeg") {
            if let Ok(_) = file.write_all(&pixels) {
                if let Ok(reader) = image::io::Reader::open("cover.jpeg") {

                    reader
                    .decode()
                    .unwrap()
                    .pixels()
                    .map(|texel| texel.2)
                    .collect::<Vec<Rgba<u8>>>()
                    .iter()
                    .for_each(|pixel| {

                        let red = pixel.0[0];
                        image_data.push(red);
                        let green = pixel.0[1];
                        image_data.push(green);
                        let blue = pixel.0[2];
                        image_data.push(blue);
                        let alpha = pixel.0[3];
                        image_data.push(alpha);

                    });
                }
            }
        }  
        self.cover_pixels.clear();
        self.cover_pixels = image_data;
        if let Ok(reader) = image::io::Reader::open("cover.jpeg") {
            if let Ok(dimension) = reader.into_dimensions() {
                self.width_cover =  dimension.0;
                self.height_cover = dimension.1;
            }
        }
        self.metadata.cover_image = String::from("cover.jpeg");
        

}
pub fn set_default_cover(self: & mut Self) {

    let mut image_data = Vec::new();

    if let Ok(reader) = image::io::Reader::open("no_cover.jpeg") {
        reader
        .decode()
        .unwrap()
        .pixels()
        .map(|texel| texel.2)
        .collect::<Vec<Rgba<u8>>>()
        .iter()
        .for_each(|pixel| {

                    let red = pixel.0[0];
                    image_data.push(red);
                    let green = pixel.0[1];
                    image_data.push(green);
                    let blue = pixel.0[2];
                    image_data.push(blue);
                    let alpha = pixel.0[3];
                    image_data.push(alpha);

        });
    }
    self.cover_pixels = image_data;
    if let Ok(reader) = image::io::Reader::open("no_cover.jpeg") {
        if let Ok(dimension) = reader.into_dimensions() {
            self.width_cover =  dimension.0;
            self.height_cover = dimension.1;
        }
    }
    self.metadata.cover_image = String::from("no_cover.jpeg");

}
pub fn swap_view(self: & mut Self, view: u8){

    self.ultimate_view = self.current_view;
    self.current_view = view;

}
pub fn fill_metadata(self: & mut BookState, epub_doc: & mut EpubDoc<BufReader<File>>) {

    if let Some(creator) = epub_doc.mdata("creator") {
        self.metadata.author = creator;
    }
    if let Some(title) = epub_doc.mdata("title") {
        self.metadata.title = title;
    }

    if let Some(description) = epub_doc.mdata("description") {
        self.metadata.description = description;
    }
    if let Some(language) = epub_doc.mdata("language") {
        self.metadata.language = language;
    }

    rewind_epub_cursor(epub_doc);       
    while epub_doc.go_next().is_ok() {

        if let Ok(name) = epub_doc.get_current_path() {
            if let Some(name) = name.to_str() {
                self.metadata.titles.push(name.to_owned());
            }
        }
    }
    self.metadata.generator = "eeBOOK Reader".to_string();

}
pub fn save_current_modified_page(self: & mut BookState) {
    self.raw_pages_modified.insert(self.current_page_i32 as usize, String::from(self.current_html_page.clone()));
}

}