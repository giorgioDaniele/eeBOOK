use std::{sync::{Mutex, Arc}, fs, path::PathBuf};
use std::ops::Index;
use regex::Regex;
use crate::{ebook::{BookState, reactToSearch}, constants::{JUMP_BY_OCR_SEARCH, EMPTY_STRING}};

#[allow(non_snake_case)]
pub fn looksForMatchesInPage (bookState: & mut BookState, key: & String) {

    
    let plainPages = bookState.getPlainPages();
    let mut indexMatches = Vec::new();

    let currentChapterNumber = bookState.getCurrentChap();
    if let Some(currentChapterContent) = plainPages.get((currentChapterNumber - 1) as usize) {
        let currentPageNumber = bookState.getCurrentPage();
        if let Some((text, _)) = currentChapterContent.iter()
        .find(|(_, nPage)| nPage.eq(&currentPageNumber)) {

            if let Ok(regex) = Regex::new(key) {
                regex.captures_iter(text).for_each(|capture| {
                    if let Some(matchesFound) = capture.get(0) {
                        indexMatches.push(matchesFound.range());
                    }
                })
            }
         }
    }
    reactToSearch(bookState, &indexMatches);
}

#[allow(non_snake_case)]
pub fn looksForMatchesInChapter(bookState: & mut BookState, key: & String) {

    let mut indexMatches = Vec::new();
    let chapters = bookState.getPlainChapters();

    if let Some(chapterText) =
         chapters.get((bookState.getCurrentChap() - 1) as usize) {
        if let Ok(regex) = Regex::new(key) {
                regex.captures_iter(chapterText).for_each(|capture| {
                    if let Some(matchesFound) = capture.get(0) {
                        indexMatches.push(matchesFound.range());
                    }
                })
        }
    }
    reactToSearch(bookState, &indexMatches);
}

#[allow(non_snake_case)]
pub fn directOCR (bookState: & mut BookState, key: & String) {


    let plainPages = bookState.getPlainPages();
    let mut threads     = Vec::new();

    let bestHit = Arc::new(Mutex::new((0, 0)));

    let keyWordsCount = key.split(" ").count();

    if bookState.getPaperWordsEvaluetion() == 0 {
        bookState.setPaperWordsEvaluetion(keyWordsCount as u32);
    }else {
        let previousValue = bookState.getPaperWordsEvaluetion();
        //if (previousValue - 100) > keyWordsCount as u32 {
          //  bookState.setPaperWordsEvaluetion(previousValue)
        //}else {
            let average = (previousValue + keyWordsCount as u32) / 2;
            bookState.setPaperWordsEvaluetion(average as u32)
        //}
    }

    for (_, pages) in plainPages.into_iter().enumerate() {
        
        let bestHit = Arc::clone(&bestHit);
        let textToSearch = String::clone(key);

        threads.push(std::thread::spawn(move || {

            // ATTIVITA DEL THREAD
            for (page, number) in pages.into_iter() {

                // ANALISI DELLA PAGINA
                let mut hits = 0;
                textToSearch.lines().filter(|line| !line.is_empty()).for_each(|line| 
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

#[allow(non_snake_case)]
pub fn reverseOCR (bookState: & mut BookState) -> String {



    let plainPages = bookState.getPlainPages();

    let currentPage = bookState.getCurrentPage();
    let mut wordsCount = 0;

    let currentChap = bookState.getCurrentChap();

/*
    let mut addPageOnCount = 5 + currentChap/2 as i32;

    plainPages.iter().flatten()
    .filter(|(_, nPage)| nPage <= &(currentPage)).for_each(|(page, nPage)| {
        wordsCount += page.split(" ").count();
    });

    bookState.setDigitalPageEvaluetion((wordsCount as u32 / bookState.getPaperWordsEvaluetion() as u32) + addPageOnCount as u32);

    println!("PAPER VALUE: {}, DIGITAL VALUE: {}", bookState.getPaperWordsEvaluetion(), bookState.getDigitalPageEvaluetion());
*/

    let mut wordsCount2 = 0;
    let mut nPageCount=0;


    for (indexChap, Chap) in plainPages.iter().enumerate(){

        if indexChap <= currentChap as usize{

            Chap.iter().filter(|(wordPage, nPage)| nPage<=&currentPage).for_each(|(word, nPage)| wordsCount2+=word.split(" ").count());
            nPageCount+= wordsCount2 / bookState.getPaperWordsEvaluetion() as usize;
            if wordsCount2 % bookState.getPaperWordsEvaluetion() as usize !=0 {
                nPageCount+=1;
            }
            wordsCount2=0;
        }
    }
    println!("DIGITAL VALUE PAGE: {}", nPageCount);

    bookState.setDigitalPageEvaluetion(nPageCount as u32);

    println!("PAPER VALUE: {}, DIGITAL VALUE: {}", bookState.getPaperWordsEvaluetion(), bookState.getDigitalPageEvaluetion());

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

                if currBestHit.0 !=0 {
                    return String::from(&currBestHit.1)
                } 
            }
        }
    }
    return String::from(EMPTY_STRING)
}


