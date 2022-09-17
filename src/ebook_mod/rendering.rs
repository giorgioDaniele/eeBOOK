use crate::*;
use druid::text::RichText;
use druid::text::RichTextBuilder;
use druid::Color;
use regex::Regex;
use std::collections::HashMap;

pub fn parse_calibre(
    input: &str,
    page: usize,
    formatting_info: &mut HashMap<(usize, usize), FormattingInfo>,
) -> String {

    

    let mut output = String::new();
    let mut lines_removed = 0;


    for (line_number, line) in input.lines().enumerate() {
        
        //DEFINZIONE DELL'ESPRESSIONE REGOLARE
        //CAPACE DI CATTURARE (QUASI) TUTTI
        //I TAG POSSIBILI

        //IMMEDIATA SOSTITUZIONE DI TUTTI I TAG
        //SPECIFICI PER IL RITORNO A CAPO
    
        let rx = Regex::new(r"<(.*?)>").unwrap();

        let line = line.replace("<br/>", "");

        let rx_a = Regex::new(r"(<!DOCTYPE.*)").unwrap();
        let rx_b = Regex::new(r#"("http.*)"#).unwrap();

        let processed  = rx.replace_all(&line, "");
        let processed2 = rx_a.replace_all(&processed, "");
        let processed3 = rx_b.replace_all(&processed2, "");

        let processed_line = processed3.trim();

        // A QUESTO PUNTO, IN processed_line C'E' IL TESTO
        // PURO, ESTRATTO DA QUELLO HTML, CONTENTENTE
        // IL TITOLO, I PARAGRAFI, ECCETERA
        // QUESTO TESTO NON E' PERO' ARRICCHITO,
        // OCCORRE QUINDI UN NUOVO PROCESSO
        // DI PARSING DEL TESTO HTML ORIGINALE
        // PER IDENTIFICARE IL FORMATO E LO STILE
        // DELLE DIVERSE RIGHE DA STAMPARE
        // NEL PANNELLO SOTTOSTANTE LA TOOLBAR


        if !processed_line.is_empty() {
            // TOLTE LE RIGHE VUOTE
            for captures in rx.captures_iter(&line) {
                // CATTURA TUTTI I MATCH (E RAGGRUPALI PER TIPO), INFATTI Captures NASCONDE UNA MAPPA
                for capture in captures.iter().flatten() {
                    // APPIATTENDO AD UNA SEMPLICE LISTA, ITERA SU TUTTI I MATCHES

                    if let Some(capture) = match capture.as_str() {
                        // DA IMPLEMENTARE
                        match_found if match_found.contains("<img") => None,
                        match_found if match_found.contains("<body") => None,
                        match_found if match_found.contains("<p>") => None,
                        // IMPLEMENTATI
                        match_found if match_found.contains("<title") => {
                            Some(FormattingInfo::Title)
                        }
                        match_found if match_found.contains("<h1") => Some(FormattingInfo::Heading),
                        match_found if match_found.contains("<h2") => {
                            Some(FormattingInfo::Heading2)
                        }
                        match_found if match_found.contains("<i") => Some(FormattingInfo::Italic),
                        match_found if match_found.contains("<b") => Some(FormattingInfo::Bold),
                        _ => None,
                    } {
                    
                        // L'IMPOSTAZIONE DI FORMATTAZIONE LETTA
                        // DAL FILE HTML (ALLA PAGINA n, RIGA m)
                        // E' MEMORIZZATA ALL'INTENO DI UNA MAPPA
                        // CHE TRACCIA, PAGINA PER PAGINA, RIGA PER RIGA,
                        // LE IMPOSTAZIONI DI FORMATTAZIONE DEL TESTO

                        formatting_info.insert((page, line_number - lines_removed), capture);
                    }
                }
            }
        }

        if processed_line.is_empty() {
            lines_removed += 1;
        } else {
            output.push_str(processed_line);
            output.push('\n');
        }
    }
    output
}

pub fn create_rich_page(
    input: &str,
    page: usize,
    formatting_info: &HashMap<(usize, usize), FormattingInfo>,
) -> RichText {

    let mut builder = RichTextBuilder::new();
    for (line_number, line) in input.lines().enumerate() {
        if let Some(format) = formatting_info.get(&(page, line_number)) {
            match format {
                FormattingInfo::Title => {
                    builder
                        .push(line)
                        .add_attr(druid::text::Attribute::FontSize(
                            druid::KeyOrValue::Concrete(70.0),
                        ))
                        .add_attr(druid::text::Attribute::FontFamily(
                            druid::FontFamily::SYSTEM_UI,
                        ))
                        .add_attr(druid::text::Attribute::weight(
                            druid::FontWeight::EXTRA_BOLD,
                        ))
                        .add_attr(druid::text::Attribute::TextColor(
                            druid::KeyOrValue::Concrete(Color::WHITE),
                        ));
                    builder.push("\n");
                    builder.push("\n");
                    builder.push("\n");
                }


                FormattingInfo::Heading => {
                    builder
                        .push(line)
                        .add_attr(druid::text::Attribute::FontSize(
                            druid::KeyOrValue::Concrete(30.0),
                        ))
                        .add_attr(druid::text::Attribute::FontFamily(
                            druid::FontFamily::SYSTEM_UI,
                        ))
                        .add_attr(druid::text::Attribute::weight(druid::FontWeight::BOLD))
                        .add_attr(druid::text::Attribute::TextColor(
                            druid::KeyOrValue::Concrete(Color::WHITE),
                        ));
                    builder.push("\n");
                    builder.push("\n");
                }


                FormattingInfo::Heading2 => {
                    builder
                        .push(line)
                        .add_attr(druid::text::Attribute::FontSize(
                            druid::KeyOrValue::Concrete(20.0),
                        ))
                        .add_attr(druid::text::Attribute::FontFamily(
                            druid::FontFamily::SYSTEM_UI,
                        ))
                        .add_attr(druid::text::Attribute::weight(druid::FontWeight::BOLD))
                        .add_attr(druid::text::Attribute::TextColor(
                            druid::KeyOrValue::Concrete(Color::WHITE),
                        ));
                    builder.push("\n");
                    builder.push("\n");
                }

                FormattingInfo::Bold => {
                    builder
                        .push(line)
                        .add_attr(druid::text::Attribute::FontSize(
                            druid::KeyOrValue::Concrete(20.0),
                        ))
                        .add_attr(druid::text::Attribute::FontFamily(
                            druid::FontFamily::SYSTEM_UI,
                        ))
                        .add_attr(druid::text::Attribute::weight(druid::FontWeight::BOLD))
                        .add_attr(druid::text::Attribute::text_color(Color::WHITE));
                    builder.push("\n");
                }

                FormattingInfo::Italic => {
                    builder
                        .push(line)
                        .add_attr(druid::text::Attribute::FontSize(
                            druid::KeyOrValue::Concrete(20.0),
                        ))
                        .add_attr(druid::text::Attribute::FontFamily(
                            druid::FontFamily::SYSTEM_UI,
                        ))
                        .add_attr(druid::text::Attribute::weight(druid::FontWeight::MEDIUM))
                        .add_attr(druid::text::Attribute::text_color(Color::WHITE));
                    builder.push("\n");
                }

                FormattingInfo::Ignore => {
                    builder.push("");
                }
            }
        } 
        else {
            builder
                .push(line)
                .add_attr(druid::text::Attribute::FontSize(
                    druid::KeyOrValue::Concrete(20.0),
                ))
                .add_attr(druid::text::Attribute::FontFamily(
                    druid::FontFamily::SYSTEM_UI,
                ))
                .add_attr(druid::text::Attribute::weight(druid::FontWeight::MEDIUM))
                .add_attr(druid::text::Attribute::text_color(Color::WHITE));
            builder.push("\n");
        }
    }

    builder.build()
}
