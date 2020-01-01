use std::str;
use curl::easy::Easy;
use std::error::Error;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::Write;
use std::fs::OpenOptions;
use std::env;

fn is_valid_supla_url(_url: &str) -> bool {
    //println!("is_valid_supla_url");
    //TODO
    
    return true;
}

fn download_file(url: &str) -> Option<()> {
    println!("url to download: {}", url);
    let vec: Vec<&str> = url.split('/').collect();
    let filename = &vec.last().unwrap();
    println!("filename: {}", filename);

    let file = OpenOptions::new().write(true).create_new(true).open(filename);
    let mut file = match file {
        Ok(f) => f,
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::AlreadyExists => {
                    println!("Error: file {} already exists", filename);
                },
                _ => println!("Error: {}", e)
            }
            return None;
        }
    };
    
    let mut easy = Easy::new();
    easy.url(url).unwrap();
    let mut transfer = easy.transfer();
    transfer.write_function(|data| {
        file.write_all(&data).unwrap();
        Ok(data.len())
    }).unwrap();
    let result = transfer.perform();
    match result {
        Ok(_) => {
            return Some(());
        },
        Err(e) => {
            println!("Error saving file: {:?}", e);
            return None;
        }
    }
}

fn build_xml_str(s1: &mut String, s2: &str) {
    s1.push_str(s2);
}



fn handle_audiomediafile(reader: &mut quick_xml::Reader<&[u8]>) -> Option<String>{
    let mut buf = Vec::new();
    match reader.read_event(&mut buf) {
        Ok(Event::Text(e)) => {
            let url = e.unescape_and_decode(&reader).unwrap();
            return Some(url);
        },
        _ => None
    }
}

fn handle_passthroughvariables(reader: &mut quick_xml::Reader<&[u8]>) {
    //TODO: get episode name from XML variables
    let mut buf = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Empty(ref e)) => {
                //println!("Start: {:?}", e);
                for a in e.attributes() {
                    let _a = a.unwrap();
                    //let key = String::from_utf8(a.key.to_vec()).unwrap();
                    //let value = String::from_utf8(a.value.to_vec()).unwrap();
                    //println!("attr key: {} value: {}", key, value);
                }
            },
            Ok(Event::End(ref _e)) => {
                //println!("End: {:?}", e);
                break;
            }
            _ => ()
        }
    }
}

fn get_mp3_url_from_xml(s: &str) -> Option<String> {
    let mut mp3url = None;
    let mut buf = Vec::new();
    let mut reader = Reader::from_str(s);
    reader.trim_text(true);

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                //println!("Start: {}", e.unescape_and_decode(&reader).unwrap());
                match e.name() {
                     b"AudioMediaFile" => { 
                         //println!(": {:?}", e.name());
                         let url = handle_audiomediafile(&mut reader).unwrap();
                         mp3url = Some(url);
                     },
                     b"PassthroughVariables" => {
                         handle_passthroughvariables(&mut reader);
                     }
                     _ => ()
                }
            },
            // unescape and decode the text event using the reader encoding
            //Ok(Event::Text(e)) => txt.push(e.unescape_and_decode(&reader).unwrap()),
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (), // There are several other `Event`s we do not consider here
        }
    }

    return mp3url;
}

fn get_xml(url: &str, s: &mut String) -> Result<(), ()> {  
    let mut easy = Easy::new();
    easy.url(url).unwrap();
    let mut transfer = easy.transfer();
    transfer.write_function(|data| {
        let datas = str::from_utf8(&data).unwrap();
        
        build_xml_str(s, &datas);
        
        Ok(data.len())
    }).unwrap();
    let result = transfer.perform();

    match result {
        Ok(_) => {
            println!("Transfer ok, url: {}", url);
            return Ok(());
        },
        Err(e) => {
            println!("Transfer error: {}", e.description());
            return Err(());
        }
    }
}

fn generate_xml_url(siteurl: &str) -> String {
    println!("get_xml_url: {}", siteurl);

    let url_prefix = "https://gatling.nelonenmedia.fi/media-xml-cache?id=";
    let url_postfix = "&v=3/";

    let v: Vec<&str> = siteurl.split('/').collect();
    let podcast_id = &v.last().unwrap().to_string();
    println!("podcast_id: {}", podcast_id);

    let podcast_url = format!("{}{}{}", url_prefix, podcast_id, url_postfix);

    return podcast_url;
}

fn download_from_supla(supla_url: &str) -> Result<(), ()> {
    println!("download_from_supla: {}", supla_url);

    let xml_url = generate_xml_url(supla_url);

    let mut xml_str = String::new();
    let result = get_xml(&xml_url, &mut xml_str);
    match result {
        Ok(_) => println!("XML downloaded"),
        Err(_) => {
            println!("Can't download XML: {}", xml_url);
            return Err(());
        }
    }

    let mp3url = get_mp3_url_from_xml(&xml_str);
    match mp3url {
        Some(url) => {
            let result = download_file(&url);
            match result {
                Some(_) =>  Ok(()),
                None => Err(())
            }
        },
        None => {
            println!("Can't find MP3 url, maybe invalid podcast URL?");
            return Err(())
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("invalid arg: Use \"downloader http://your-url\"");
        return;
    }
    let siteurl = &args[1];

    let valid_url = is_valid_supla_url(&siteurl);
    if valid_url == false {
        println!("Invalid url: {}", siteurl);
        return;
    }

    let result = download_from_supla(&siteurl);
    match result {
        Ok(()) => {
            println!("Download ok");
        },
        Err(()) => {
            println!("Download failed");
        }
    }
}
