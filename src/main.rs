use anyhow::anyhow;
use ics::components::Property;
use ics::{Event, ICalendar, Standard, TimeZone};
use rocket::response::status::NotFound;
use rocket::Config;
use table_extract::Row;

#[macro_use]
extern crate rocket;

const URL: &str = "https://docs.google.com/spreadsheets/d/e/2PACX-1vRM5GLi6AJKszq5NmdvB4CG1t4NngoJzLigvQ81Q3IKWbwONE2t4bRGdQfBgFVT_KsCPkElmyL_Kkbv/pubhtml/sheet?headers=true&gid=442327001";

#[launch]
fn rocket() -> _ {
    let config = Config {
        address: "0.0.0.0".parse().unwrap(),
        port: 8000,
        ..Default::default()
    };
    rocket::custom(config).mount("/", routes![index])
}

#[get("/")]
async fn index() -> Result<String, NotFound<String>> {
    let ical = create_calendar().await;
    return ical.map_err(|e| NotFound(format!("Error: {}", e.to_string())));
}

async fn create_calendar() -> anyhow::Result<String> {
    let body = reqwest::get(URL).await?.text().await?;

    let mut calendar = ICalendar::new("2.0", "steinbock-kalender");
    calendar.add_timezone(TimeZone::standard(
        "Europe/Vienna",
        Standard::new("19961027T030000", "+0100", "+0200"),
    ));

    let table = table_extract::Table::find_first(body.as_str())
        .ok_or(anyhow!("Failed to find table in html"))?;
    // skip first because of table header
    for row in table.iter().skip(1) {
        let (date, section_name) = decode_table(row)?;

        let date_timestamp = create_timestamp(date);
        let timestamp = format!("{}T124650Z", date_timestamp);

        let uid = create_uid(section_name.clone(), timestamp.clone());
        let mut event = Event::new(uid, timestamp);

        event.push(Property::new(
            "SUMMARY",
            format!("Steinbock schraubt: {}", section_name),
        ));
        event.push(Property::new("DTSTART;VALUE=DATE", date_timestamp.clone()));
        event.push(Property::new("DTEND;VALUE=DATE", date_timestamp));

        calendar.add_event(event);
    }

    Ok(calendar.to_string())
}

fn create_timestamp(date: String) -> String {
    date.rsplit(".")
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join("")
}

fn create_uid(section_name: String, timestamp: String) -> String {
    let uid = format!("{}{}", timestamp, section_name)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("");
    uid
}

fn decode_table(row: Row) -> anyhow::Result<(String, String)> {
    let date = row
        .iter()
        .as_slice()
        .get(0)
        .ok_or(anyhow!("Date not in table"))?;
    let sec = row
        .iter()
        .as_slice()
        .get(1)
        .ok_or(anyhow!("Section not in table"))?;
    println!("{} at {} umgeschraubt", date, sec,);

    Ok((date.to_string(), sec.to_string()))
}
