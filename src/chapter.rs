
use druid::{text::{RichText, RichTextBuilder, Attribute}, Color, FontWeight, KeyOrValue::Concrete, FontStyle};
use crate::constants::ELEMENTS_IN_PAGE;



#[allow(non_snake_case)]
#[derive(Clone)]
pub struct Chapter {

    path:          String,
    id:            usize,
    title:         String,
    heading:       String, 
    subHeading:    String,
    paragraphs:    Vec<String>,
    nParagraphs:   usize
}



#[allow(non_snake_case)]
impl Chapter {

    pub fn new() -> Chapter {

        Chapter { 
            path:          String::new(), 
            id:            0,
            title:         String::new(),
            heading:       String::new(),
            subHeading:    String::new(),
            paragraphs:    Vec::new(),
            nParagraphs:   0 as usize, 
        }
    }
    pub fn fill (self: & mut Self, html: String, id: usize, path: String) {
        
        self.id = id;
    
        if let Ok(DOM) = tl::parse(&html, tl::ParserOptions::default()) {

            let PARSER = DOM.parser();
            let nodes  = DOM.nodes();
            for node in nodes {
                    
                if let Some(tag) = node.as_tag() {

                    if tag.name().as_utf8_str().eq("head") {
                        let text = node.inner_text(PARSER);
                        self.addTitle(&text);
                    }
                    if tag.name().as_utf8_str().eq("h1") {
                        let text = node.inner_text(PARSER);
                        self.addHeading(&text);
                    }
                    if tag.name().as_utf8_str().eq("h2") {
                        let text = node.inner_text(PARSER);
                        self.addSubHeading(&text);
                    }
                    if tag.name().as_utf8_str().eq("p") {
                        let text = node.inner_text(PARSER);
                        self.paragraphs.push(String::from(text));
                    }
                }
            }
        }
        self.nParagraphs = self.paragraphs.len();
        if path.as_str().trim().contains("%20") {
                self.addPath(path.trim()
                        .replace("%20", " ")
                        .replace("/", "\\").to_owned());
            }else {
                self.addPath(path.trim().replace("/", "\\").to_owned());
            }
    }
    pub fn update(self: & mut Self, html: & str) {
        let html = String::from(html);
        let path = String::clone(&self.path);
        self.clear();
        self.fill(html, self.id, path);
    }
    fn clear(self: & mut Self) {
        self.title.clear();
        self.heading.clear();
        self.subHeading.clear();
        self.paragraphs.clear();
    }
    pub fn addPath(self: & mut Self, path: String) {
        self.path = path;
    }
    pub fn getPath(self: & Self) -> & String {
        &self.path
    }
    pub fn addTitle(self: & mut Self, title: & str) {
        self.title   = String::from(title);
    } 
    pub fn addHeading(self: & mut Self, heading: & str) {
        self.heading   = String::from(heading);
    }
    pub fn addSubHeading(self: & mut Self, subHeading: & str) {
        self.subHeading   = String::from(subHeading);
    }
    pub fn createRTFPages(self: & Self) -> Vec<RichText> {

        let mut pages = Vec::new();

        let mut builder  = RichTextBuilder::new();
        let mut elemCounter = 0;

        if !self.title.is_empty() {
            builder.push(&self.title)
            .add_attr(Attribute::TextColor(Concrete(Color::WHITE)))
            .add_attr(Attribute::FontSize(Concrete(40.0)))
            .add_attr(Attribute::Weight(FontWeight::BOLD));
            elemCounter += 1;

        }if !self.heading.is_empty() {
            builder.push(&self.heading)
            .add_attr(Attribute::TextColor(Concrete(Color::WHITE)))
            .add_attr(Attribute::FontSize(Concrete(30.0)))
            .add_attr(Attribute::Weight(FontWeight::BOLD))
            .add_attr(Attribute::Style(FontStyle::Italic));
            builder.push("\n");
            elemCounter += 1;

        }if !self.subHeading.is_empty() {
            builder.push(&self.subHeading)
            .add_attr(Attribute::TextColor(Concrete(Color::WHITE)))
            .add_attr(Attribute::FontSize(Concrete(20.0)))
            .add_attr(Attribute::Weight(FontWeight::BOLD))
            .add_attr(Attribute::Style(FontStyle::Italic));
            builder.push("\n");
            elemCounter += 1;
        }

        (1..=ELEMENTS_IN_PAGE - elemCounter).into_iter().for_each(|i| {
            if let Some(paragraph) = self.paragraphs.get(i - 1) {
                builder.push(paragraph)
                .add_attr(Attribute::TextColor(Concrete(Color::WHITE)))
                .add_attr(Attribute::FontSize(Concrete(20.0)))
                .add_attr(Attribute::Weight(FontWeight::NORMAL));
                builder.push("\n\n");
            }
        });

        pages.push(builder.build());
        // PRIMA PAGINA COMPLETATA

        let elementsFilled = elemCounter;

        // RIEMPIMENTO DELLE RIMANENTI
        let mut builder  = RichTextBuilder::new();
        let mut elemCounter = 0;

        for paragraph in self.paragraphs.iter().skip(ELEMENTS_IN_PAGE - elementsFilled) {
 
            if elemCounter.eq(&ELEMENTS_IN_PAGE) {
                pages.push(builder.build());
                elemCounter = 0;
                builder = RichTextBuilder::new();
            }
            else {
                builder.push(paragraph)
                .add_attr(Attribute::TextColor(Concrete(Color::WHITE)))
                .add_attr(Attribute::FontSize(Concrete(20.0)))
                .add_attr(Attribute::Weight(FontWeight::NORMAL));
                builder.push("\n\n");
                elemCounter += 1;
            }
        }
        if elemCounter != 0 {
            pages.push(builder.build());
        }
        pages

    }
    pub fn createRTFChap(self: & Self) -> RichText {

        
        let mut builder  = RichTextBuilder::new();

        if !self.title.is_empty() {
            builder.push(&self.title)
            .add_attr(Attribute::TextColor(Concrete(Color::WHITE)))
            .add_attr(Attribute::FontSize(Concrete(40.0)))
            .add_attr(Attribute::Weight(FontWeight::BOLD));

        }if !self.heading.is_empty() {
            builder.push(&self.heading)
            .add_attr(Attribute::TextColor(Concrete(Color::WHITE)))
            .add_attr(Attribute::FontSize(Concrete(30.0)))
            .add_attr(Attribute::Style(FontStyle::Italic));
            builder.push("\n");

        }if !self.subHeading.is_empty() {
            builder.push(&self.subHeading)
            .add_attr(Attribute::TextColor(Concrete(Color::WHITE)))
            .add_attr(Attribute::FontSize(Concrete(20.0)))
            .add_attr(Attribute::Style(FontStyle::Italic));
            builder.push("\n");
      
        }

        (1..=self.nParagraphs).into_iter().for_each(|i| {
            if let Some(paragraph) = self.paragraphs.get(i - 1) {
                builder.push(paragraph)
                .add_attr(Attribute::TextColor(Concrete(Color::WHITE)))
                .add_attr(Attribute::FontSize(Concrete(20.0)))
                .add_attr(Attribute::Weight(FontWeight::NORMAL));
                builder.push("\n\n");
            }
        });

        builder.build()
    }
    pub fn createPlainPages(self: & Self, pageCounter: & mut i32) -> Vec<(String, i32)> {

        let mut pages = Vec::new();

        let mut currPage  = String::new();
        let mut elemCounter = 0;

        if !self.title.is_empty() {
            currPage.push_str(&self.title);
            elemCounter += 1;
            

        }if !self.heading.is_empty() {
            currPage.push_str(&self.heading);
            elemCounter += 1;
            currPage.push_str("\n");

        }if !self.subHeading.is_empty() {
            currPage.push_str(&self.subHeading);
            elemCounter += 1;
            currPage.push_str("\n");
        }

        (1..=ELEMENTS_IN_PAGE - elemCounter).into_iter().for_each(|i| {
            if let Some(paragraph) = self.paragraphs.get(i - 1) {
                currPage.push_str(paragraph);
                currPage.push_str("\n\n");
            }
        });

        *pageCounter += 1;

        pages.push((currPage.to_string(), i32::clone(pageCounter)));
        // PRIMA PAGINA COMPLETATA

        let elementsFilled = elemCounter;

        // RIEMPIMENTO DELLE RIMANENTI
        let mut currPage  = String::new();
        let mut elemCounter = 0;

        for paragraph in self.paragraphs.iter().skip(ELEMENTS_IN_PAGE - elementsFilled) {
 
            if elemCounter.eq(&ELEMENTS_IN_PAGE) {
                *pageCounter += 1;
                pages.push((currPage.to_string(), i32::clone(pageCounter)));
                elemCounter = 0;
                currPage.clear();
            }
            else {
                currPage.push_str(paragraph);
                currPage.push_str("\n\n");
                elemCounter += 1;
            }
        }
        if elemCounter != 0 {
            *pageCounter += 1;
            pages.push((currPage.to_string(), i32::clone(pageCounter)));
        }        
        pages
        

    }

}

