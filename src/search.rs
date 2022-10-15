use std::fs;
use std::ops::ControlFlow::Break;
use std::path::{Path, PathBuf};
use std::sync::{mpsc::{channel, Sender}, Condvar, Mutex, Arc};
use std::thread::Thread;
use druid::piet::TextStorage;
use druid::platform_menus::win::file::new;

use crate::{ebook::{BookState}, constants::JUMP_BY_OCR_SEARCH};
use crate::constants::{EMPTY_STRING, JUMP_BY_BAR_SEARCH};

#[allow(non_snake_case)]
pub fn searchPageFromText (bookState: & mut BookState, key: & String) {


    let plainPages = bookState.getPlainPages();
    let mut thread     = Vec::new();

    let turn = Arc::new((Mutex::new(0), Condvar::new()));
    let (transmitter, 
        receiver) = channel::<Vec<(i32, usize)>>();

    for (chapNumber, pages) in plainPages.into_iter().enumerate() {
        
        let transmitter = Sender::clone(&transmitter);
        let turn = Arc::clone(&turn);

        let textToSearch = String::clone(key);

        thread.push(std::thread::spawn(move || {

            // ATTIVITA DEL THREAD
            let mut result = Vec::new();
            for (page, number) in pages.into_iter() {
                textToSearch.lines().enumerate().for_each(|(n,line)| {
                    if page.contains(line) {
                        println!("MATCH ALLA PAGINA {}, RIGA {}", number, n);
                        result.push((number, n))
                    }
                });
            }

            if let Ok(mut current_turn) = turn.0.lock() {
                while current_turn.ne(&chapNumber) {
                    current_turn = turn.1.wait(current_turn).unwrap();
                }
                // INVIO DEL RISULTATO
                transmitter.send(result).unwrap();
            }
        }));
    }
    drop(transmitter);

    let mut jump = 0;

    for result in receiver.iter() { 

        result.iter()
        .map(|elem| elem.0)
        .for_each(|nPage| {
            println!("PAGINA {}", nPage);
            if jump == 0 {
                bookState.setCurrentPage(nPage);
                bookState.jump(JUMP_BY_BAR_SEARCH);
                jump = 1;
            }
        });
        println!("");
        
        if let Ok(mut current_turn) = turn.0.lock() {
            *current_turn = *current_turn + 1;
            turn.1.notify_all();
        }   
    }

}

/*
#[allow(non_snake_case)]
pub fn reverseOCR (bookState: & mut BookState) {

    // CATTURA STRINGA NORMALE
    let plainPages = bookState.getPlainPages();
    if let Some(pages) = plainPages.get((bookState.getCurrentChap() - 1) as usize) {

        if let Some(text) =  pages.iter()
            .filter(|(_, nPage)| nPage.eq(&bookState.getCurrentPage()))
            .map(|(page, _)| String::clone(page))
            .last() {

            let mut language = bookState.getLanguage();
            let mut threads = Vec::new();

            let language = match language.as_str() {
                ITALIAN =>  String::from("ita"),
                ENGLISH =>  String::from("eng"),
                _             =>  String::from("eng"),
            };

            let imagesPath = bookState.getImagesPath().clone();
            let mut hits = 0;
            let mut bestPath = String::new();

            imagesPath.iter().enumerate()
                .for_each(|(id, path)| {

                    std::process::Command::new("tesseract")
                        .arg(path)
                        .arg("text")
                        .arg("--oem")
                        .arg("1")
                        .arg("-l")
                        .arg(language.to_owned())
                        .output()
                        .expect("[ERROR]: tesseract has failed");

                    if let Ok(tessOutput) = fs::read_to_string("text.txt") {
                        // ESECUZIONE DI TESSERACT
                        if let Ok(_) = fs::remove_file("text.txt") {
                            // RICERCA NEL TESTO
                            //let  tessOutput = tessOutput.replace("\n", "").replace(" ", "");

                            let mut currHits = 0;

                            println!("TESTO: {}", tessOutput);
                            tessOutput.lines().filter(|line| !line.is_empty()).for_each(|line|

                                if text.contains(line) {
                                    currHits += 1;
                                }
                            );
                            if currHits >= hits {
                                hits = currHits;
                                bestPath = String::clone(path);
                            }

                        }
                    }

                });
            if hits == 0 {
                println!("SUCA");
            }else {
                println!("BEST: {} - HITS: {}", bestPath, hits);
            }

        }

    }


}*/


#[allow(non_snake_case)]
pub fn reverseOCR (bookState: & mut BookState) -> String{

    // CATTURA STRINGA NORMALE
    let plainPages = bookState.getPlainPages();
    if let Some(pages) = plainPages.get((bookState.getCurrentChap() - 1) as usize) {

        let bestHit = Arc::new(Mutex::new((0, String::new())));

        if let Some(text) =  pages.iter()
            .filter(|(_, nPage)| nPage.eq(&bookState.getCurrentPage()))
            .map(|(page, _)| String::clone(page))
            .last() {

            let mut language = bookState.getLanguage();
            let mut threads = Vec::new();



            let language = match language.as_str() {
                ITALIAN =>  String::from("ita"),
                ENGLISH =>  String::from("eng"),
                _             =>  String::from("eng"),
            };

            let imagesPath = bookState.getImagesPath().clone();

            imagesPath.into_iter().enumerate()
                .for_each(|(id, path)| {

                    let bestHit = Arc::clone(&bestHit);
                    let text = String::clone(&text);
                    let language = String::clone(&language);
                    let path = String::clone(&path);

                    threads.push(std::thread::spawn(move || {
                        let mut title = id.to_string();

                        std::process::Command::new("tesseract")
                            .arg(path.to_owned())
                            .arg(title.clone())
                            .arg("--oem")
                            .arg("1")
                            .arg("-l")
                            .arg(language)
                            .output()
                            .expect("[ERROR]: tesseract has failed");
                        let pathText = PathBuf::from(title.clone() + ".txt");
                        if let Ok(tessOutput) = fs::read_to_string(pathText.clone()) {

                            // ESECUZIONE DI TESSERACT

                            if let Ok(_) = fs::remove_file(pathText.clone()) {

                                let mut currHits = 0;
                                tessOutput.lines()
                                    .into_iter().filter(|line| !line.is_empty()).for_each(|line|
                                    if text.contains(line) {
                                        currHits += 1;
                                    }
                                );
                                if let Ok(mut currBestHit) = bestHit.lock() {
                                    if currHits >= currBestHit.0 {
                                        currBestHit.0 = currHits;
                                        currBestHit.1 = String::clone(&path);
                                    }
                                }

                            }
                        }
                    }));
                });
            threads.into_iter().for_each(|thread| { if let Ok(_) = thread.join(){}});

            if let Ok(currBestHit) = bestHit.lock() {

                if currBestHit.0!=0{
                    //RITORNA IL PATH DELL'IMMAGINE
                    return currBestHit.1.clone();
                    println!("BEST HIT ALLA PAGINA {}: {}", currBestHit.1, currBestHit.0);
                }

            }
        }
    }
    return EMPTY_STRING.to_string();
}


#[allow(non_snake_case)]
pub fn searchPageFromImage (bookState: & mut BookState, key: & String) {


    let plainPages = bookState.getPlainPages();
    let mut threads     = Vec::new();

    let bestHit = Arc::new(Mutex::new((0, 0)));


    for (_, pages) in plainPages.into_iter().enumerate() {
        
        let bestHit = Arc::clone(&bestHit);
        let textToSearch = String::clone(key);

        threads.push(std::thread::spawn(move || {

            // ATTIVITA DEL THREAD
            for (page, number) in pages.into_iter() {

                // ANALISI DELLA PAGINA
                let mut hits = 0;
                textToSearch.lines().for_each(|line| 
                    if page.contains(line) {
                        hits += 1;
                    }
                );
                if let Ok(mut currBestHit) = bestHit.lock() {
                    if hits >= currBestHit.0 {
                        currBestHit.0 = hits;
                        currBestHit.1 = number;
                    }
                }
            }
        }));
    }
    for (ID,thread) in threads.into_iter().enumerate() {
        if let Ok(_) = thread.join() {}
    }
    if let Ok(currBestHit) = bestHit.lock() {
        println!("BEST HIT ALLA PAGINA {}: {}", currBestHit.1, currBestHit.0);
        bookState.setCurrentPage(currBestHit.1);
    }
    bookState.jump(JUMP_BY_OCR_SEARCH);
    println!("FINE");


}