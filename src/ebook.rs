// IMPORTAZIONI ESTERNE
use std::collections::HashMap;
use std::fs;
use std::ops::Range;
use druid::text::Attribute;
use druid::{ArcStr, Color, FontFamily, FontStyle, FontWeight};
use druid::{Data, Lens, text::RichText, FileInfo};
use epub::doc::EpubDoc;

// IMPORTAZIONI INTERNE
use crate::chapter::{Chapter, FormatInfo, render};
use crate::constants::{EMPTY_STRING, IMAGE_LOADING, JUMP_BY_BUTTON, JUMP_BY_BAR_SEARCH, JUMP_BY_OCR_SEARCH, HELPER, COLORS, COLORS_AVAILABLE};
use crate::constants::GO_NEXT;
use crate::constants::EPUB_LOADING;
use crate::constants::ITALIAN; 
use crate::constants::ENGLISH;
use crate::search::directOCR;

// TIPO ENUMERATIVO CHE RAPPRESENTA GLI STATI DELL'APPLICAZIONE
#[allow(non_snake_case)]
#[derive(Clone, Data, PartialEq, PartialOrd)]
pub enum ViewState {
    ReadMode,
    EditMode,
    HelpMode,
    Idle, 
}



// STRUTTURA DATI PRINCIPALE
#[allow(non_snake_case)]
#[derive(Clone, Data, Lens)]
pub struct BookState {

    path:          String,
    language:      String,

    currentPage:   i32,
    previousPage:  i32,
    currentChap:   i32,

    totPages:      i32,
    totChaps:      i32,

    cursor:         String,
    
    currentView:   ViewState,
    ultimateView:  ViewState,

    epubOpen:      bool,

    #[data(ignore)]
    chaps:         
    Vec<Chapter>,

    #[data(ignore)]
    pages:         
    Vec<Vec<(String, FormatInfo)>>,

    #[data(ignore)]
    chapters:         
    Vec<Vec<(String, FormatInfo)>>,

    #[data(ignore)]
    HTMLs:         
    Vec<String>,

    #[data(ignore)]
    chapterContains:
    HashMap<i32, Vec<i32>>,

    #[data(ignore)]
    pageBelongs:
    HashMap<i32, i32>,

    // DATI VOLATILI
    currentRTFpage:     RichText,
    currentRTFchap:     RichText,
    currentHTML:        String,

    objectType:         u8,

    // COPERTINA
    #[data(eq)]
    cover:                 Vec<u8>,
    ultimeID:              u8,
    currID:                u8,

    // RICERCA
    numberOfSearch:       u8,
    barText:              String,
    imgFromText:          String,

    // HELPER
    helper:               RichText,

    // DATI IMMAGINI RICONOSCIUTE
    #[data(ignore)]
    imgListPath:          Vec<String>,
    imgViewedPath:        String,

    // DATI TIPOGRAFICI
    fontSize:             f64,

    paperWordsEvaluetion: u32, 
    digitalPageEvaluetion: u32,
    msg: RichText,
}


// FUNZIONI/METODI ASSOCIATI
#[allow(non_snake_case)]
fn updateHTML(bookState: & mut BookState, chapter: i32) {

    saveHTML(bookState, chapter);

    if let Some(chapterMod) = bookState.chaps.get_mut((chapter - 1) as usize) {

        let html = &bookState.currentHTML;
        chapterMod.update(html.as_str());
        
        // SOSTITUZIONE DEL CAPITOLO IN FORMATO RTF
        bookState.chapters.remove((chapter - 1) as usize);
        // INSERIMENTO DI QUELLO NUOVO (SEMPRE IN FORMATO RTF)
        bookState.chapters.insert((chapter - 1) as usize, chapterMod.createChapter());
        

    }
    update(bookState, bookState.currentChap, bookState.currentPage, ViewState::EditMode);
}
#[allow(non_snake_case)]
pub fn saveHTML(bookState: & mut BookState, chapter: i32) {

    // RIMOZIONE DEL CODICE VECCHIO
    bookState.HTMLs.remove((chapter - 1) as usize);
    // AGGIORNAMENTO VETTORE CON IL CODICE NUOVO (MODIFICATO DALL'UTENTE)
    bookState.HTMLs.insert((chapter - 1) as usize, String::clone(&bookState.currentHTML)); 
}
#[allow(non_snake_case)]
fn update(bookState: & mut BookState, chapter: i32, page: i32, mode: ViewState) {

    if mode == ViewState::ReadMode {

        // RECUPERO PAGINA RTF
        if let Some(page) = bookState.pages.get((page - 1) as usize) {

            bookState.currentRTFpage = render(page, bookState.fontSize);
        }  
        if let Some(chapIndex) = bookState.pageBelongs.get(&page) {
            // RECUPERO CAPITOLO RTF
            bookState.currentChap    = i32::clone(chapIndex);
            if let Some(chapter) = bookState.chapters.get((chapIndex - 1) as usize) {
                bookState.currentRTFchap = render(chapter, bookState.fontSize);
            }
            // RECUPERO HTML CORRENTE
            if let Some(html) = bookState.HTMLs.get((chapIndex - 1) as usize) {
                bookState.currentHTML = String::clone(html);
            }
        }
        bookState.cursor  = bookState.currentPage.to_string();
    }

    if  mode == ViewState::EditMode {

        // RECUPERO PAGINA HTML CORRENTE
        if let Some(html) = bookState.HTMLs.get((chapter - 1) as usize) {
            bookState.currentHTML = String::clone(html);
        }
        // RECUPERO CAPITOLO RTF
        if let Some(chapter) = bookState.chapters.get((chapter - 1) as usize) {
            bookState.currentRTFchap = render(chapter, bookState.fontSize);
        }
        // RECUPERO PAGINA RTF
        if let Some(pagesInChapter) = bookState.chapterContains.get(&bookState.currentChap) {
            if let Some(index) = pagesInChapter.get(0) {
                bookState.currentPage  = i32::clone(index);
                if let Some(page) = bookState.pages.get((index - 1) as usize) {
                    bookState.currentRTFpage = render(page, bookState.fontSize);
                }
            }
        }
        bookState.cursor  = bookState.currentChap.to_string();
    }
}
#[allow(non_snake_case)]
pub fn loading(bookState: & mut BookState, fileInfo: & FileInfo) {

    if bookState.getCurrentObjType() == EPUB_LOADING {

        if !bookState.pages.is_empty() {
            bookState.clear();
        }
    

        let mut nChapters : usize = 0;
        if let Ok(mut epub) = EpubDoc::new(fileInfo.path()) {

            // PATH DEL LIBRO/ARCHIVIO EPUB APERTO
            bookState.setPath(fileInfo.path.to_str().unwrap().to_owned());
            // LINGUA DEL LIBRO
            if let Some(language) = epub.mdata("language") {
                bookState.setLanguage(language);
            }
            
            // COPERTINA (SE ESISTE)
            if let Ok(cover) = epub.get_cover() {
                bookState.cover = cover;
                bookState.ultimeID = bookState.currID;
                bookState.currID += 1;
            }else {
                bookState.currID += 1;
            }

            while epub.go_next().is_ok() {

                if let Ok(html) = epub.get_current_str() {

                    // CATTURA DEL CODICE HTML SORGENTE
                    bookState.HTMLs.push(String::clone(&html));
                    nChapters += 1;

                    // CREAZIONE DEL CAPITOLO
                    let mut chapter    : Chapter     = Chapter::new();

                    if let Ok (path) = epub.get_current_path() {
                        if let Some(path) = path.to_str() {
                            // POPOLAMENTO DEL CAPITOLO
                            chapter.fill(html, nChapters, path.to_string());
                            bookState.chaps.push(chapter);
                        }
                    }
                }
            }
        // INIZIALIZZAZIONE
        }bookState.setup(bookState.chaps.len() as i32);
    }
    if bookState.getCurrentObjType() == IMAGE_LOADING {
        let language : String;

        match bookState.language.as_str() {
            ITALIAN => language = String::from("ita"),
            ENGLISH => language = String::from("eng"),
            _             => language = String::from("eng"),
        }   
        std::process::Command::new("tesseract")
        .arg(fileInfo.path())
        .arg("text")
        .arg("--oem")
        .arg("1")
        .arg("-l")
        .arg(language)
        .output()
        .expect("[ERROR]: tesseract has failed");

        if let Ok(tessOutput) = fs::read_to_string("text.txt") {
            // ESECUZIONE DI TESSERACT
            if let Ok(_) = fs::remove_file("text.txt") {
            directOCR(&mut Box::new(bookState),&tessOutput);
            }
        }
    }
    
}
#[allow(non_snake_case)]
fn checkLimits(value: i32, lowerBound: i32, higherBound: i32) -> bool {
    value >= lowerBound && value <= higherBound 
}
#[allow(non_snake_case)]
pub fn reactToHTMLModification(bookState: & mut BookState) {
    updateHTML(bookState, bookState.currentChap);
}
#[allow(non_snake_case)]
pub fn reactToZoom(bookState: & mut BookState) {
    update(bookState, bookState.getCurrentChap(), bookState.getCurrentPage(), bookState.getCurrentView());
}
#[allow(non_snake_case)]
pub fn reactToSearch(bookState: & mut BookState, indexMatch: & Vec<Range<usize>>) {

    bookState.numberOfSearch += 1;
    bookState.numberOfSearch =  bookState.numberOfSearch % COLORS_AVAILABLE as u8;
    let color  = COLORS.get(bookState.numberOfSearch as usize).unwrap();
    let color = Color::clone(color);

    if bookState.currentView == ViewState::ReadMode {
        for range in indexMatch {
            let start = range.start;
            let end = range.end;
            bookState.currentRTFpage.
            add_attribute(start..end, 
                Attribute::TextColor(druid::KeyOrValue::Concrete(Color::clone(&color))));
        }
    }
    if bookState.currentView == ViewState::EditMode {
        
        for (index, range) in indexMatch.iter().enumerate() {
            let start = range.start;
            let end = range.end;
            bookState.currentRTFchap.
            add_attribute(start..end, 
                Attribute::TextColor(druid::KeyOrValue::Concrete(Color::clone(&color))));
        }
    }

   
}
#[allow(non_snake_case)]
pub fn reactToBarSearchRefresh(bookState: & mut BookState) {
    update(bookState, bookState.getCurrentChap(), bookState.getCurrentPage(), bookState.getCurrentView());
}


#[allow(non_snake_case)]
impl BookState { 

    // METODO COSTRUTTORE E DISTRUTTORE
    pub fn new() -> Self {

    BookState {

        path:            String::from(EMPTY_STRING),
        language:        String::from(EMPTY_STRING),

        chaps:           Vec::new(),
        currentPage:     0,
        previousPage:    0,
        currentChap:     0,
        cursor:          String::from(EMPTY_STRING),
        currentView:     ViewState::Idle,
        ultimateView:    ViewState::Idle,
        epubOpen:        false,
        pages:           Vec::new(),
        chapters:        Vec::new(),
        HTMLs:           Vec::new(),

        currentRTFpage:  RichText::new(ArcStr::from(EMPTY_STRING)),
        currentRTFchap:  RichText::new(ArcStr::from(EMPTY_STRING)),
        currentHTML:     String::from(EMPTY_STRING),

        totPages:       0,
        totChaps:       0,

        chapterContains: HashMap::new(),
        pageBelongs:     HashMap::new(),
        objectType:      EPUB_LOADING,

        // COPERTINA
        cover:           Vec::new(),
        currID:          0,
        ultimeID:        0,

        // RICERCA
        numberOfSearch: 0,
        barText:        String::from(EMPTY_STRING),
        imgFromText:    String::from(EMPTY_STRING),

        helper:         RichText::new(HELPER.into())
        .with_attribute(0..18, Attribute::text_color(Color::rgb(0.50, 0.0, 1.0)))
        .with_attribute(0..18, Attribute::size(58.0))
        .with_attribute(0..18, Attribute::font_family(FontFamily::MONOSPACE))
        .with_attribute(123..144, Attribute::style(FontStyle::Italic))
        .with_attribute(19.., Attribute::size(24.0))
        .with_attribute(898.., Attribute::weight(FontWeight::BOLD)),

        fontSize:       1.0,
        imgListPath:    Vec::new(), 
        imgViewedPath:  String::from(EMPTY_STRING),

        paperWordsEvaluetion: 0,    
        digitalPageEvaluetion: 0,
        msg:    RichText::new(HELPER.into()),

    }
}
    pub fn clear(self: & mut Self) {

    self.path.clear();

    self.currentChap = 0;
    self.currentPage = 0;
    self.totChaps = 0;
    self.totPages = 0;

    self.chaps.clear();

    self.HTMLs.clear();
    self.pages.clear();
    self.chapters.clear();
    self.cover.clear();
    self.imgListPath.clear();

    self.currentRTFpage =  RichText::new(ArcStr::from(EMPTY_STRING));
    self.currentRTFchap =  RichText::new(ArcStr::from(EMPTY_STRING));
    self.currentHTML    =  String::from(EMPTY_STRING);

    self.cursor         =  String::from(EMPTY_STRING);

    self.epubOpen       = false;
    self.numberOfSearch = 0;


}
    pub fn setup(self: & mut Self, nChpaters: i32) {
        
        self.epubOpen    = true;
        self.currentPage = 1;
        self.currentChap = 1;
        self.totChaps    = nChpaters;

        let  mut pageCounter = 0;

        for (chapIndex, chap) in self.chaps.iter().enumerate() {

            self.chapters.push(chap.createChapter());

            let mut pages = chap.createPages();

            // VETTORE CONTENENTE I NUMERI DI PAGINA
            let mut pageIndices = Vec::new();

            // ITERA PER LE PAGINA APPENA CREATE
            // E INCREMENTE IL CONTATORE pageCounter (CHE RIPORTA
            // IL NUMERO DI PAGINA ATTUALMENTE RAGGIUNTO)
            (1..=pages.len()).for_each(|_| {
                pageCounter += 1;
                pageIndices.push(pageCounter);
                // DATO IL NUMERO DI PAGINA, SI MEMORIZZA IL CAPITOLO CORRISPONDENTE (NUMERAZIONE A PARTIRE DA 1)
                self.pageBelongs.insert(pageCounter, chapIndex as i32 + 1);
            });

            // DATO IL CAPITOLO CORRISPONDENTE, SI MEMORIZZA L'ELENCO DI PAGINE CONTENUTE (NUMERAZIONE A PARTIRE DA 1)
            self.chapterContains.insert(chapIndex as i32 + 1, pageIndices);
            self.pages.append(&mut pages);  
                    
        }
        
        self.totPages    = self.pages.len() as i32;
        self.cursor      = self.currentPage.to_string();
        self.currentView = ViewState::ReadMode;

        // RECUPERO HTML
        if let Some(html) = self.HTMLs.get(self.currentChap as usize - 1) {
            self.currentHTML = String::clone(html);
        }
        // RECUPERO PAGINA RTF
        if let Some(page) = self.pages.get(self.currentPage as usize - 1) {
            self.currentRTFpage = render(page, self.fontSize);
        }
        // RECUPERO CAPITOLO RTF
        if let Some(chapter) = self.chapters.get(self.currentChap as usize - 1) {
            self.currentRTFchap = render(chapter, self.fontSize);
        }

    }
    
    // METODI GETTER
    pub fn getBarText(self: & Self) -> String {
        String::clone(&self.barText)
    }
    pub fn getChapters(self: & Self) -> & Vec<Chapter> {
        &self.chaps
    }   
    pub fn getCoverPixels(self: & Self) -> Vec<u8> {
        Vec::clone(&self.cover)
    }
    pub fn getCurrID(self: & Self) -> u8 {
        self.currID
    }
    pub fn getCurrentChap(self: & Self) -> i32 {
        self.currentChap
    }
    pub fn getCurrentObjType(self: & Self) -> u8 {
        self.objectType
    }
    pub fn getCurrentPage(self: &  Self) -> i32 {
        self.currentPage
    }
    pub fn getCurrentView(self: & Self) -> ViewState {
        ViewState::clone(&self.currentView)
    }
    pub fn getEpubOpen(self: & Self) -> bool {
        self.epubOpen
    }
    pub fn getFontSize(self: & Self) -> f64 {
        self.fontSize
    }
    pub fn getHTMLs (self: & Self) ->  & Vec<String> {
        &self.HTMLs
    }
    pub fn getImagesPath(self: & mut Self) -> & mut Vec<String> {
        & mut self.imgListPath
    }
    pub fn getImagesPathSize(self: & Self) -> usize {
        self.imgListPath.len()
    }
    pub fn getLanguage(self: & Self) -> &String {
        &self.language
    }
    pub fn getPath(self: & mut Self) -> & str {
        &self.path
    }
    pub fn getPlainPages(self: & Self) -> Vec<Vec<(String, i32)>> {
        let mut plainPages = Vec::new();
        let mut pageCounter = 0;
        for chap in self.chaps.iter() {
            plainPages.push(chap.createPlainPages(&mut pageCounter));
        }
        plainPages
    }
    pub fn getTotalChaps(self: & Self) -> i32 {
        self.totChaps
    }
    pub fn getTotalPages(self: & Self) -> i32 {
        self.totPages
    }
    pub fn getUltID(self: & Self) -> u8 {
        self.ultimeID
    }
    pub fn getUltimateView(self: & Self) -> ViewState {
        ViewState::clone(&self.ultimateView)
    }
    pub fn getPathImageViewed(self: & Self) -> & String {
        &self.imgViewedPath
    }
    pub fn getPlainChapters(self: & Self) -> Vec<String> {
        
        let mut chapters = Vec::new();
        for chap in self.chaps.iter() {
            chapters.push(chap.createPlainChapter());
        }
        chapters
    }
    pub fn getPaperWordsEvaluetion(self: & Self) -> u32 {
        self.paperWordsEvaluetion
    }    
    pub fn getDigitalPageEvaluetion(self: & Self) -> u32 {
        self.digitalPageEvaluetion
    }
    // NAVIGAZIONE DEL LIBRO
    pub fn jump(self: & mut Self, jumpType: u8) {

        if jumpType == JUMP_BY_BUTTON {

            // SALTO DA UNA PAGINA ALL'ALTRA
            if self.currentView == ViewState::ReadMode {
                // VERIFICA CHE L'INPUT SIA UN NUMERO
                if self.cursor.chars().all(|char| char.is_digit(10)) {
                    if let Ok(pageNumber) = self.cursor.parse::<i32>() {
                        if checkLimits(pageNumber, 1, self.totPages) {
                            if pageNumber == 1 {
                                self.currentPage = pageNumber;
                            }else {
                                self.currentPage = pageNumber;
                            }
                            update(self, self.currentChap, self.currentPage, ViewState::ReadMode);
                        }       
                    }
                }
            }
            // SALTO DA UN CAPITOLO ALL'ALTRO
            if self.currentView == ViewState::EditMode {
            // VERIFICA CHE L'INPUT SIA UN NUMERO
                if self.cursor.chars().all(|char| char.is_digit(10)) {
                    if let Ok(chapNumber) = self.cursor.parse::<i32>() {
                        if checkLimits(chapNumber, 1, self.totChaps) {
                            if chapNumber == 1 {
                                self.currentChap = 1;
                            }else {
                                self.currentChap = chapNumber;
                            }
                            update(self, self.currentChap, self.currentPage, ViewState::EditMode);
                        }       
                    }
                }
            }

        }if jumpType == JUMP_BY_BAR_SEARCH {
            update(self, self.currentChap, self.currentPage, ViewState::ReadMode);
        }if jumpType == JUMP_BY_OCR_SEARCH {
            update(self, self.currentChap, self.currentPage, ViewState::ReadMode);
        }

    }
    pub fn scroll(self: & mut Self, direction: u8) {

    match self.epubOpen {

        true => {
            if self.currentView == ViewState::ReadMode {
                if direction == GO_NEXT {
                    // MEMORIZZA HTML CAPITOLO PRECEDENTE
                    saveHTML(self, self.currentChap);
                    self.previousPage = self.currentPage;
                    self.currentPage += 1;
                    // UPDATE
                    update(self, self.currentChap, self.currentPage, ViewState::ReadMode);
                }else {
                    // MEMORIZZA HTML CAPITOLO PRECEDENTE
                    saveHTML(self, self.currentChap);
                    self.previousPage = self.currentPage;
                    self.currentPage -= 1;
                    // UPDATE
                    update(self, self.currentChap, self.currentPage, ViewState::ReadMode);
                }
            }
            else {
                if direction == GO_NEXT {
                    saveHTML(self, self.currentChap);
                    self.currentChap += 1;
                    // UPDATE
                    update(self, self.currentChap, self.currentPage, ViewState::EditMode);
                }else {
                    saveHTML(self, self.currentChap);
                    self.currentChap -= 1;
                    // UPDATE
                    update(self, self.currentChap, self.currentPage, ViewState::EditMode);
                }
            }
        },
        false => {
            panic!("[ERROR]: no book has been open");
        },
    }
}

    // METODI SETTER
    pub fn setCurrentPage(self: & mut Self, page: i32) {
        self.currentPage = page;
    }
    pub fn setCurrentView(self: & mut Self, view: ViewState) {
        self.ultimateView = ViewState::clone(& self.currentView);
        self.currentView  = view;
    }
    pub fn setCursor(self: & mut Self, cursor: i32) {
        self.cursor = cursor.to_string();
    }
    pub fn setLanguage(self: & mut Self, lang: String) {
        self.language = lang;
    }
    pub fn setObjectType(self: & mut Self, obj: u8) {
        self.objectType = obj;
    }
    pub fn setPath(self: & mut Self, path: String) {
        self.path = path;
    } 
    pub fn setPathImageViewed(self: & mut Self, path: String) {
        self.imgViewedPath = path;
    }
    pub fn setFontSize(self: & mut Self, fontSize: f64) {
        self.fontSize = fontSize
    } 
    pub fn setPaperWordsEvaluetion(self: & mut Self, value: u32){
        self.paperWordsEvaluetion = value;
    }    
    pub fn setDigitalPageEvaluetion(self: &mut Self, value: u32) {
        self.digitalPageEvaluetion = value;
    }
    pub fn setMsg(self: &mut Self, value: RichText) {
        self.msg = value;
    }
}

