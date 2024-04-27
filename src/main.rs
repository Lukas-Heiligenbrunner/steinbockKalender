use ics::{Event, ICalendar, Standard, TimeZone};
use ics::components::Property;
use rocket::Config;
use rocket::response::status::NotFound;

#[macro_use] extern crate rocket;

const URL: &str = "https://docs.google.com/spreadsheets/d/e/2PACX-1vRM5GLi6AJKszq5NmdvB4CG1t4NngoJzLigvQ81Q3IKWbwONE2t4bRGdQfBgFVT_KsCPkElmyL_Kkbv/pubhtml/sheet?headers=true&gid=442327001";


#[launch]
fn rocket() -> _ {
    let config = Config {
        address: "0.0.0.0".parse().unwrap(),
        port: 8000,
        ..Default::default()
    };

    rocket::custom(config)
        .mount("/", routes![index])
}

#[get("/")]
async fn index() -> Result<String, NotFound<String>> {

    let ical = get_sheet().await;
    return ical.map_err(|e| NotFound(format!("Error: {}", e.to_string())));
}

async fn get_sheet() -> Result<String, Box<dyn std::error::Error>> {
    let body = reqwest::get(URL).await?.text().await?;

    let mut calendar = ICalendar::new("2.0", "steinbock-kalender");
    calendar.add_timezone(TimeZone::standard("Europe/Vienna", Standard::new("19961027T030000", "+0100", "+0200")));

    let table = table_extract::Table::find_first(body.as_str()).unwrap();
    for row in table.iter().skip(1) {
        let date = row.iter().as_slice().get(0).map_or_else(|| "Datum missing", |v| { v.as_str() });
        let sec = row.iter().as_slice().get(1).map_or_else(|| "sec missing", |v| { v.as_str() });
        println!(
            "{} at {} umgeschraubt",
            date,
            sec,
        );

        let section_name = sec.to_string();

        let dtstampdate = date.rsplit(".").map(|x| { x.to_string() }).collect::<Vec<String>>().join("");
        let dtstamp = format!("{}T124650Z", dtstampdate);

        let uid = format!("{}{}", dtstamp, section_name).split_whitespace().collect::<Vec<_>>().join("");
        let mut event = Event::new(uid, dtstamp);

        event.push(Property::new("SUMMARY", format!("Steinbock schraubt: {}", sec)));
        event.push(Property::new("DTSTART;VALUE=DATE", dtstampdate.clone()));
        event.push(Property::new("DTEND;VALUE=DATE", dtstampdate));

        calendar.add_event(event);
    }

    Ok(calendar.to_string())
}
