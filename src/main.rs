// curl --request POST \
//     --url https://www.kawa.net/works/ajax/romanize/romanize.cgi \
//     --header 'content-type: multipart/form-data' \
//     --form mode=japanese \
//     --form 'q=おはよう'

use clap::Parser;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

/// Get romaji pronunciation for Japanese input (hiragana/katakana)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("Get romaji pronunciation for {} ...", args.input);
    println!();

    let client: reqwest::Client = reqwest::Client::new();
    let form = [("mode", "japanese"), ("q", args.input.as_str())];
    let res = client.post("https://www.kawa.net/works/ajax/romanize/romanize.cgi").form(&form).send().await?;
    let text = res.text().await?;
    // println!("body = {:?}", text);

    let mut reader = Reader::from_str(&text);
    reader.trim_text(true);

    let mut romaji: Vec<String> = Vec::new();
    let mut txt = Vec::new();
    let mut buf = Vec::new();

    // The `Reader` does not implement `Iterator` because it outputs borrowed data (`Cow`s)
    loop {
        // NOTE: this is the generic case when we don't know about the input BufRead.
        // when the input is a &str or a &[u8], we don't actually need to use another
        // buffer, we could directly call `reader.read_event()`
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            // exits the loop when reaching end of file
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                if let Some(attr) = e.try_get_attribute("title").unwrap() {
                    romaji.push(String::from_utf8(attr.value.into_owned()).unwrap());
                }
                // println!("{:?}", e.try_get_attribute("title").unwrap());
                // romaji.push(e.attributes());
            }
            Ok(Event::Text(e)) => {
                // println!("{:?}", e);
                txt.push(e.unescape().unwrap().into_owned())
            },
            // There are several other `Event`s we do not consider here
            _ => (),
        }
        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    println!("{}", romaji.join(" "));
    println!("{}", txt.join(" "));

    Ok(())
}
