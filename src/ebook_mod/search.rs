use crate::*;

use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;



pub fn search_chapter (data: & mut BookState) {

    

    // CREAZIONE DEL CANALE DI COMUNICAZIONE
    let (transmit, receive) = channel::<(usize, bool)>();
    let turn = Arc::new((Mutex::new(0), Condvar::new()));

    let mut thread = Vec::new();
    for (index, page) in data.parsed_pages.iter().enumerate() {

        let transmit = Sender::clone(&transmit);
        let turn = Arc::clone(&turn);

        let string_to_analyze = page.clone();
        let search_text = data.bar_text.clone();

        thread.push(std::thread::spawn(move || {

            let result = string_to_analyze.contains(&search_text);

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
            data.current_page_i32 = (page + 1) as i32;
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
    data.jump_to_page(JUMP_BY_SEARCH);



}
