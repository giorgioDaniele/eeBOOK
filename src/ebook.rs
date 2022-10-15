// IMPORTAZIONI ESTERNE
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use druid::text::Attribute;
use druid::{ArcStr, Command, AppDelegate, Handled, commands, Target, Env, DelegateCtx, Color, FontFamily, FontStyle, FontWeight};
use druid::{Data, Lens, text::RichText, FileInfo};
use epub::doc::EpubDoc;
use walkdir::WalkDir;
use zip_extensions::zip_extract;

// IMPORTAZIONI INTERNE
use crate::chapter::Chapter;
use crate::constants::{EMPTY_STRING, IMAGE_LOADING, JUMP_BY_BUTTON, JUMP_BY_BAR_SEARCH, JUMP_BY_OCR_SEARCH, HELPER};
use crate::constants::GO_NEXT;
use crate::constants::EPUB_LOADING;
use crate::constants::ITALIAN; 
use crate::constants::ENGLISH;
use crate::search::searchPageFromImage;

// TIPO ENUMERATIVO CHE RAPPRESENTA GLI STATI DELL'APPLICAZIONE
#[allow(non_snake_case)]
#[derive(Clone, Data, PartialEq, PartialOrd)]
pub enum ViewState {
    ReadMode,
    EditMode,
    HelpMode,
    Idle, 
}


// STRUTTURA DATI ATTORNO ALLA QUALE REALIZZARE LE FUNZIONI DI APERTURA E SALVATAGGIO DEI
// FILES APERTI
pub struct Explorer;



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
    RTFpages:         
    Vec<RichText>,

    #[data(ignore)]
    RTFchaps:         
    Vec<RichText>,

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
    barText:              String,
    imgFromText:          String,

    // HELPER
    helper:               RichText,

    //LISTA DI IMMAGINI RICONOSCIUTE
    #[data(ignore)]
    imgListPath:          Vec<String>,

    imgViewedPath:        String,

}


// FUNZIONI/METODI ASSOCIATI
#[allow(non_snake_case)]
fn updateHTML(bookState: & mut BookState, chapter: i32) {

    saveHTML(bookState, chapter);

    if let Some(chapterMod) = bookState.chaps.get_mut((chapter - 1) as usize) {

        let html = &bookState.currentHTML;
        chapterMod.update(html.as_str());
        
        // SOSTITUZIONE DEL CAPITOLO IN FORMATO RTF
        bookState.RTFchaps.remove((chapter - 1) as usize);
        // INSERIMENTO DI QUELLO NUOVO (SEMPRE IN FORMATO RTF)
        bookState.RTFchaps.insert((chapter - 1) as usize, chapterMod.createRTFChap());
        

    }
    update(bookState, bookState.currentChap, bookState.currentPage, ViewState::EditMode);
}
#[allow(non_snake_case)]
fn saveHTML(bookState: & mut BookState, chapter: i32) {

    // RIMOZIONE DEL CODICE VECCHIO
    bookState.HTMLs.remove((chapter - 1) as usize);
    // AGGIORNAMENTO VETTORE CON IL CODICE NUOVO (MODIFICATO DALL'UTENTE)
    bookState.HTMLs.insert((chapter - 1) as usize, String::clone(&bookState.currentHTML)); 
}
#[allow(non_snake_case)]
fn update(bookState: & mut BookState, chapter: i32, page: i32, mode: ViewState) {

    if mode == ViewState::ReadMode {

        // RECUPERO PAGINA RTF
        if let Some(rtfPage) = bookState.RTFpages.get((page - 1) as usize) {
            bookState.currentRTFpage = RichText::clone(rtfPage);

        }
        if let Some(chapIndex) = bookState.pageBelongs.get(&page) {
            // RECUPERO CAPITOLO RTF
            bookState.currentChap    = i32::clone(chapIndex);
            if let Some(rtfChap) = bookState.RTFchaps.get((chapIndex - 1) as usize) {
                bookState.currentRTFchap = RichText::clone(rtfChap);
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
        if let Some(rtfChap) = bookState.RTFchaps.get((chapter - 1) as usize) {
            bookState.currentRTFchap = RichText::clone(rtfChap); 
        }
        // RECUPERO PAGINA RTF
        if let Some(pagesInChapter) = bookState.chapterContains.get(&bookState.currentChap) {
            if let Some(index) = pagesInChapter.get(0) {
                bookState.currentPage  = i32::clone(index);
                if let Some(rtfPage) = bookState.RTFpages.get((index - 1) as usize) {
                    bookState.currentRTFpage = RichText::clone(rtfPage);
                }
            }
        }
        bookState.cursor  = bookState.currentChap.to_string();
    }
}
#[allow(non_snake_case)]
fn loading(bookState: & mut BookState, fileInfo: & FileInfo) {

    if bookState.getCurrentObjType() == EPUB_LOADING {

        if !bookState.RTFpages.is_empty() {
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
            // RICERCA NEL TESTO
            //let  tessOutput = tessOutput.replace("\n", "").replace(" ", "");
            searchPageFromImage(&mut Box::new(bookState),&tessOutput);
            }
        }
    }
    
}
#[allow(non_snake_case)]
fn checkLimits(value: i32, lowerBound: i32, higherBound: i32) -> bool {
    value >= lowerBound && value <= higherBound 
}
#[allow(non_snake_case)]
pub fn react(bookState: & mut BookState) {
    updateHTML(bookState, bookState.currentChap);
}


// IMPLEMENTAZIONE DELLA STRUTTURA DATI PRINCIPALE
#[allow(non_snake_case)]
impl BookState {

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
        RTFpages:        Vec::new(),
        RTFchaps:        Vec::new(),
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
        barText:        String::from(EMPTY_STRING),
        imgFromText:    String::from(EMPTY_STRING),
        imgViewedPath:  String::from(EMPTY_STRING),


        helper:         RichText::new(HELPER.into())
        .with_attribute(0..18, Attribute::text_color(Color::rgb(0.50, 0.0, 1.0)))
        .with_attribute(0..18, Attribute::size(58.0))
        .with_attribute(0..18, Attribute::font_family(FontFamily::MONOSPACE))
        .with_attribute(128..150, Attribute::style(FontStyle::Italic))
        .with_attribute(19.., Attribute::size(26.0))
        .with_attribute(849.., Attribute::weight(FontWeight::BOLD)),

        //LISTA PATH IMMAGINI RICONOSCIUTE
        imgListPath:    Vec::new(),
    }    
}
    pub fn setup(self: & mut Self, nChpaters: i32) {
    
    self.epubOpen    = true;
    self.currentPage = 1;
    self.currentChap = 1;
    self.totChaps    = nChpaters;

    let  mut pageCounter = 0;

    for (chapIndex, chap) in self.chaps.iter().enumerate() {

        // INSERIMENTO CAPITOLO FORMATO RTF
        self.RTFchaps.push(chap.createRTFChap());

        // INSERIMENTO PAGINE FORMATO RTF
        let mut rtfPages = chap.createRTFPages();

        // VETTORE CONTENENTE I NUMERI DI PAGINA
        let mut pageIndices = Vec::new();

        // ITERA PER LE PAGINA APPENA CREATE
        // E INCREMENTE IL CONTATORE pageCounter (CHE RIPORTA
        // IL NUMERO DI PAGINA ATTUALMENTE RAGGIUNTO)
        (1..=rtfPages.len()).for_each(|_| {
            pageCounter += 1;
            pageIndices.push(pageCounter);
            // DATO IL NUMERO DI PAGINA, SI MEMORIZZA IL CAPITOLO CORRISPONDENTE (NUMERAZIONE A PARTIRE DA 1)
            self.pageBelongs.insert(pageCounter, chapIndex as i32 + 1);
        });

        // DATO IL CAPITOLO CORRISPONDENTE, SI MEMORIZZA L'ELENCO DI PAGINE CONTENUTE (NUMERAZIONE A PARTIRE DA 1)
        self.chapterContains.insert(chapIndex as i32 + 1, pageIndices);
        self.RTFpages.append(&mut rtfPages);  
                
    }
    
    self.totPages    = self.RTFpages.len() as i32;
    self.cursor      = self.currentPage.to_string();
    self.currentView = ViewState::ReadMode;

    // RECUPERO HTML
    if let Some(html) = self.HTMLs.get(self.currentChap as usize - 1) {
        self.currentHTML = String::clone(html);
    }
    // RECUPERO PAGINA RTF
    if let Some(rtfPage) = self.RTFpages.get(self.currentPage as usize - 1) {
        self.currentRTFpage = RichText::clone(rtfPage);
    }
    // RECUPERO CAPITOLO RTF
    if let Some(rtfChap) = self.RTFchaps.get(self.currentChap as usize - 1) {
        self.currentRTFchap = RichText::clone(rtfChap);
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
    self.RTFpages.clear();
    self.RTFchaps.clear();
    self.cover.clear();
    
    self.currentRTFpage =  RichText::new(ArcStr::from(EMPTY_STRING));
    self.currentRTFchap =  RichText::new(ArcStr::from(EMPTY_STRING));
    self.currentHTML    =  String::from(EMPTY_STRING);

    self.cursor         =  String::from(EMPTY_STRING);

    self.epubOpen       = false;

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
    pub fn setObjectType(self: & mut Self, obj: u8) {
        self.objectType = obj;
    }
    pub fn getCurrentView(self: & Self) -> ViewState {
        ViewState::clone(&self.currentView)
    }
    pub fn setPath(self: & mut Self, path: String) {
        self.path = path;
    }
    pub fn setLanguage(self: & mut Self, lang: String) {
        self.language = lang;
    }
    pub fn getLanguage(self: & Self)->&String {
        &self.language
    }

    pub fn getPath(self: & mut Self) -> & str {
        &self.path
    }
    pub fn getUltimateView(self: & Self) -> ViewState {
        ViewState::clone(&self.ultimateView)
    }
    pub fn setCurrentView(self: & mut Self, view: ViewState) {
        self.ultimateView = ViewState::clone(& self.currentView);
        self.currentView  = view;
    }
    pub fn getEpubOpen(self: & Self) -> bool {
        self.epubOpen
    }
    pub fn getCurrentPage(self: &  Self) -> i32 {
        self.currentPage
    }
    pub fn setCurrentPage(self: & mut Self, page: i32) {
        self.currentPage = page;
    }
    pub fn getCurrentChap(self: & Self) -> i32 {
        self.currentChap
    }
    pub fn getTotalPages(self: & Self) -> i32 {
        self.totPages
    }
    pub fn getTotalChaps(self: & Self) -> i32 {
        self.totChaps
    }
    pub fn getCurrentObjType(self: & Self) -> u8 {
        self.objectType
    }
    pub fn getCurrID(self: & Self) -> u8 {
        self.currID
    }
    pub fn getUltID(self: & Self) -> u8 {
        self.ultimeID
    }
    pub fn getCoverPixels(self: & Self) -> Vec<u8> {
        Vec::clone(&self.cover)
    }
    pub fn getBarText(self: & Self) -> String {
        String::clone(&self.barText)
    }
    pub fn setCursor(self: & mut Self, cursor: i32) {
        self.cursor = cursor.to_string();
    }
    pub fn getPlainPages(self: & Self) -> Vec<Vec<(String, i32)>> {
        let mut plainPages = Vec::new();
        let mut pageCounter = 0;
        for chap in self.chaps.iter() {
            plainPages.push(chap.createPlainPages(&mut pageCounter));
        }
        plainPages
    }
    pub fn getChapters(self: & Self) -> & Vec<Chapter> {
        &self.chaps
    }
    pub fn getHTMLs (self: & Self) ->  & Vec<String> {
        &self.HTMLs
    }


    pub fn getImagesPath (self: & Self) ->  & Vec<String> {
        &self.imgListPath
    }

    pub fn getPathImageViewed(self: & Self) -> String {
        String::clone(&self.imgViewedPath)
    }
    pub fn setPathImageViewed(self: & mut Self, path: String) {
        self.imgViewedPath = path;
    }

}


// IMPLEMENTAZIONE DE
#[allow(non_snake_case)]
impl AppDelegate<BookState> for Explorer {

    fn command(&mut self, _: &mut DelegateCtx, _: Target, cmd: &Command, bookState: &mut BookState, _ : & Env) -> Handled {

        // SALVATAGGIO EPUB
        if let Some(fileInfo) = cmd.get(commands::SAVE_FILE_AS) {

            saveHTML(bookState, bookState.getCurrentChap());

            let epubPath = bookState.getPath();
            if let Some(epubModPath) = fileInfo.path.to_str() {
                // COPIA EPUB IN FORMATO ZIP
                if let Ok(_) = fs::copy(epubPath, epubModPath.replace(".epub", ".zip")) {
                    // CREAZIONE PERCORSO PER LA CARTALLA DI ESTRAZIONE
                    if let Ok(folderPath) = PathBuf::from_str(&epubModPath.replace(".epub", "")) {
                        if let Ok(archivePath) = PathBuf::from_str(&epubModPath.replace(".epub", ".zip")) {
                            // EFFETTUA L'ESTRAZIONE
                            if let Ok(_) = zip_extract(&archivePath, &folderPath) {
                                // ELIMINAZIONE ARCHIVIO
                                if let Ok(_) = fs::remove_file(epubModPath.replace(".epub", ".zip")) {

                                    // MODIFICA DEI FILES HMTL ATTRAVERSO LA NAVIGAZIONE RICORSIVA DELLA CARTELLA
                                    let htmls = bookState.getHTMLs();
                                    for source in 
                                        WalkDir::new(folderPath).into_iter()
                                            .filter_map(|e| e.ok()) {
                                                
                                        let sourcePath = source.path().to_str().unwrap();
                                        for (index,  chap) in bookState.getChapters().iter().enumerate() {
                                            if sourcePath.contains(chap.getPath()) {
                                                if let Some(html) = htmls.get(index) {
                                                    if let Ok(_) = fs::write(sourcePath, html) {
                                                        // MODIFICA EFFETTUATA
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    // RICOMPRESSIONE DELLA CARTELLA
                                    // CREAZIONE PERCORSO PER LA CARTALLA DI ESTRAZIONE
                                    if let Ok(folderPath) = PathBuf::from_str(&epubModPath.replace(".epub", "")) {
                                        if let Ok(archivePath) = PathBuf::from_str(&epubModPath.replace(".epub", ".zip")) {
                                            if let Ok(_) = zip_extensions::zip_create_from_directory(&archivePath, &folderPath) {
                                                if let Ok(_) = fs::rename(
                                                    epubModPath.replace(".epub", ".zip"), 
                                                epubModPath.replace(".zip", ".epub")) {
                                                    // EPUB GENERATO

                                                    // ELIMINAZIONE CARTELLA TEMPORANEA
                                                    if let Ok(_) = fs::remove_dir_all(epubModPath.replace(".epub", "")) {
                                                        // ELIMINAZIONE EFFETTUATA
                                                    }
                                                }
                                            }
                                        }
                                    }
          
                                }
                            }
                        }
                    }
                }   
            }
            return Handled::Yes;
        }

        // APERTURA FILES
        if let Some(fileInfo) = cmd.get(commands::OPEN_FILE) {

            // CARICAMENTO EPUB
            if bookState.getCurrentObjType() == EPUB_LOADING {
               loading(bookState, fileInfo);
            }
            // CARICAMENTO JPEG, PNG, JPG
            if bookState.getCurrentObjType() == IMAGE_LOADING {
                loading(bookState, fileInfo);
                bookState.imgListPath.push(fileInfo.path.to_str().unwrap().to_string().to_owned());

                println!("LISTA: {:?}", bookState.imgListPath);
            }   
            return Handled::Yes;
        }
        Handled::No
    }
}

