    use druid::Color;


    pub const  ELEMENTS_IN_PAGE           : usize = 10;

    // DIREZIONE DI SCORRIMENTO DELLE PAGINE
    pub const  GO_NEXT                    : u8 = 0;
    pub const  GO_PREV                    : u8 = 1; 


    // ELEMENTI DI STILE
    pub const  PURPLE             : druid::Color  = Color::rgb8(100, 32, 240);
    pub const  BLACK              : druid::Color  = Color::rgb8(0, 0, 0);
    pub const  TINY_SPACER        : f64 = 2.0;
    pub const  BIG_SPACER         : f64 = 30.0;
    pub const  ROUNDED_VALUE      : f64 = 5.0;


    // TIPOLOGIE DI SALTO NEL TESTO
    pub const  JUMP_BY_BUTTON         : u8 = 0;
    pub const  JUMP_BY_BAR_SEARCH     : u8 = 1;
    pub const  JUMP_BY_OCR_SEARCH     : u8 = 2;

    // TIPOLOGIA DI FILE CARICATI
    pub const  EPUB_LOADING       : u8 = 0;
    pub const  IMAGE_LOADING      : u8 = 1;

    // SUPPORTO LINGUE
    pub const  ITALIAN            : &str = "it";
    pub const  ENGLISH            : &str = "en";

    pub const  EMPTY_STRING       : &str = "";

    
    pub const HELPER: &str = 
        "\n  eeBook Reader©\n

        Welcome to our Epub Reader. This application, developed in Rust language in collaboration with the Politecnico di Torino, allows you to read your digital 
        books in epub format. In addition to reading and searching for chapters, the application authorizes the user to edit their books and save them.
        Also, you can search for the first occurrence of a given string.
        The implementation of OCR involves the management of two buttons:
            - 📷 ➜ Starting from a chosen photo of the book you have taken, you can go directly to the page in digital version;
            - 🖼️ ➜ Starting from a digital page it is possible to view the photo relating to the book page if this has been previously recognized.\n\n
        Hoping that the app is to your liking, a greeting from the developers
    
        Giorgio Daniele Luppina
        Claudio Di Maida";


