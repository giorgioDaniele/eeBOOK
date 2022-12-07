use std::{path::PathBuf, fs, str::FromStr};

// IMPORTAZIONI ESTERNE
use druid::{AppDelegate, commands, DelegateCtx, Env, Handled, Target, Command};
use walkdir::WalkDir;
use zip_extensions::zip_extract;

// IMPORTAZIONI INTERNE
use crate::{ebook::{BookState, saveHTML, loading}, constants::{EPUB_LOADING, IMAGE_LOADING}};

// STRUTTURA DATI ATTORNO ALLA QUALE REALIZZARE LE FUNZIONI DI APERTURA E SALVATAGGIO DEI
// FILES APERTI
pub struct Explorer;

#[allow(non_snake_case)]
impl AppDelegate<BookState> for Explorer {

    fn command(&mut self, ctx: &mut DelegateCtx, _: Target, cmd: &Command, bookState: &mut BookState, _ : & Env) -> Handled {

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
                if let Some(path) = fileInfo.path().to_str() {
                    bookState.getImagesPath().push(String::from(path));
                }
                loading(bookState, fileInfo);
            }   
            return Handled::Yes;
        }
        Handled::No
    }
}

