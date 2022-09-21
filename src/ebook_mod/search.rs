use crate::*;

use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;



pub fn search_chapter (data: & mut BookState, img: u8) {

    // CREAZIONE DEL CANALE DI COMUNICAZIONE
    let (transmit, receive) = channel::<(usize, bool)>();
    let turn = Arc::new((Mutex::new(0), Condvar::new()));

    let mut thread = Vec::new();
    let mut cause : u8 = JUMP_BY_OCR_SEARCH;
    for (index, page) in data.parsed_pages.iter().enumerate() {

        let transmit = Sender::clone(&transmit);
        let turn = Arc::clone(&turn);

        let string_to_analyze = page.clone();
        let mut search_text =   String::new();    //STRINGA DA CERCARE
        let mut search_text_1 = String::new();    //STRINGA DA CERCARE

        

        //VERIFICA SE SIAMO IN RICERCA TRAMITE IMMAGINE (img=1) O TRAMITE STRINGA (img=0)
        if img == JUMP_BY_OCR_SEARCH {
            search_text = data.ocr_text.lines().next().unwrap().trim().to_string();
            cause = JUMP_BY_OCR_SEARCH;
            //SE L'IMMAGINE CONTIENE PIU' RIGHE, VERIFICA ANCHE SE LA SECONDA RIGA FA PARTE DELLA PAGINA
            if data.ocr_text.clone().len()>1{
                    search_text_1=data.ocr_text.clone().lines().next().unwrap().trim().to_string();
            }
        }
        if img == JUMP_BY_SEARCH {
            search_text = data.bar_text.clone();
            cause = JUMP_BY_SEARCH;
        }

        let img_length= data.ocr_text.clone().len();
        thread.push(std::thread::spawn(move || {

            let mut result = string_to_analyze.contains(&search_text);
            //SE LA PRIMA RICERCA RITORNA true, VERIFICA SE, QUALORA CI SIANO PIU' STRINGHE NELL'IMMAGINE,
            // ANCHE LA SECONDA RICERCA RITORNA true
            if result {
             if img_length > 1 {
                result = string_to_analyze.contains(&search_text_1);
            }
            }
            if let Ok(mut current_turn) = turn.0.lock() {

                // MECCANISMO DI ATTESA ESCOGITATO PER GARANTIRE LA TRASMISSIONE IN ORDINE DELLA PAGINE PROCESSATE
                while current_turn.ne(&index) {
                    current_turn = turn.1.wait(current_turn).unwrap();
                }
                transmit.send((index, result)).unwrap();
            }
        }));

    }

    drop(transmit);

    // RECUPERO DELLE RISORSE PROCESSATE DAI THREAD WORKERS
    let mut first_hit = 0;
    for (page, result) in &receive {

        if result == true && first_hit == 0{
            data.current_page_i32 = page as i32 + 1;
            first_hit = 1;
        }

        if let Ok(mut current_turn) = turn.0.lock() {
            *current_turn = *current_turn + 1;
            turn.1.notify_all();
        }   
    }
    for tid in thread {
        tid.join().unwrap();
    }

    data.jump_to_page(cause);


}
