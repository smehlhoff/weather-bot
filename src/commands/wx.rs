use chrono::prelude::*;
use plotters::backend::BitMapBackend;
use plotters::drawing::IntoDrawingArea;
use plotters::prelude::*;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::fs;
use tokio::fs::File;

use crate::commands::uv;

use crate::lib::{config, error::Error, utils};

#[derive(Deserialize, Debug)]
pub struct CurrentWeather {
    pub location: Location,
    pub current: WeatherData,
}

#[derive(Deserialize, Debug)]
pub struct Location {
    pub name: String,
    pub region: String,
    pub lat: String,
    pub lon: String,
}

#[derive(Deserialize, Debug)]
pub struct WeatherData {
    pub temperature: i32,
    pub weather_descriptions: Vec<String>,
    pub wind_speed: i32,
    pub wind_degree: i32,
    pub wind_dir: String,
    pub pressure: i32,
    pub precip: f64,
    pub humidity: i32,
    pub cloudcover: i32,
    pub feelslike: i32,
    pub visibility: i32,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct CurrentForecast {
    creationDate: chrono::DateTime<Utc>,
    time: ForecastTime,
    data: ForecastData,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct ForecastTime {
    startPeriodName: Vec<String>,
    tempLabel: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct ForecastData {
    temperature: Vec<String>,
    text: Vec<String>,
}

pub async fn fetch_current(zip_code: i32) -> Result<CurrentWeather, Error> {
    let config = config::Config::load_config()?;
    let url = format!(
        "http://api.weatherstack.com/current?access_key={}&query={}&units=f",
        config.weatherstack, zip_code
    );
    let resp = reqwest::get(&url).await?.json().await;

    match resp {
        Ok(data) => {
            let resp: CurrentWeather = data;
            Ok(resp)
        }
        Err(_) => Err(Error::NotFound("The zip code provided does not match a location".into())),
    }
}

pub async fn fetch_forecast(lat: f64, lon: f64) -> Result<CurrentForecast, Error> {
    let config = config::Config::load_config()?;
    let url = format!(
        "https://forecast.weather.gov/MapClick.php?lat={lat}&lon={lon}&unit=0&lg=english&FcstType=json");
    let client = reqwest::ClientBuilder::new().user_agent(config.user_agent).build()?;
    let resp = client.get(&url).send().await?.json().await;

    match resp {
        Ok(data) => {
            let resp: CurrentForecast = data;
            Ok(resp)
        }
        Err(_) => Err(Error::NotFound("The zip code provided does not match a location".into())),
    }
}

async fn parse_current(zip_code: i32) -> String {
    match fetch_current(zip_code).await {
        Ok(data) => {
            let (city, state) = (data.location.name, data.location.region);
            let lat = data.location.lat.parse::<f64>().unwrap();
            let lon = data.location.lon.parse::<f64>().unwrap();
            #[allow(clippy::unreadable_literal)]
            let pressure = f64::from(data.current.pressure) * 0.0295301;

            format!(
                "```
Current Weather => {}, {} (lat: {:.2}, lon: {:.2})

Temperature:        {}\u{b0}
Wind Speed:         {} MPH
Wind Direction:     {} ({}\u{b0})
Pressure:           {:.2} Hg
Precipitation:      {} IN.
Humidity:           {}%
Cloud Cover:        {}%
Feels Like:         {}\u{b0}
Visbility:          {} MI.
```",
                city,
                state,
                lat,
                lon,
                data.current.temperature,
                data.current.wind_speed,
                data.current.wind_dir,
                data.current.wind_degree,
                pressure,
                data.current.precip,
                data.current.humidity,
                data.current.cloudcover,
                data.current.feelslike,
                data.current.visibility,
            )
        }
        Err(e) => format!("`There was an error retrieving data: {e}`"),
    }
}

#[command]
#[aliases("current")]
pub async fn wx_current(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let args: Vec<&str> = args.message().split(' ').collect();

    for arg in args {
        match utils::check_zip_code(arg) {
            Ok(zip_code) => {
                let data = parse_current(zip_code).await;
                msg.channel_id.say(&ctx.http, data).await?
            }
            Err(e) => msg.channel_id.say(&ctx.http, format!("`{e}`")).await?,
        };
    }

    Ok(())
}

pub async fn parse_forecast(zip_code: i32) -> String {
    match uv::fetch_location(zip_code).await {
        Ok((city, state, lat, lon)) => match fetch_forecast(lat, lon).await {
            Ok(data) => {
                let mut forecast = String::new();
                let time =
                    Local.from_utc_datetime(&data.creationDate.naive_local()).format("%I:%M %p");

                for i in 0..5 {
                    forecast.push_str(&format!(
                        "\n\n{} ({}: {})\n-----------------------\n\n{}",
                        data.time.startPeriodName[i],
                        data.time.tempLabel[i].to_lowercase(),
                        data.data.temperature[i],
                        data.data.text[i]
                    ));
                }

                format!(
                    "```Weather Forecast => {}, {} (lat: {:.2}, lon: {:.2}) {}\n\nLast updated at {}```",
                    city, state, lat, lon, forecast, time
                )
            }
            Err(e) => format!("`There was an error retrieving data: {e}`"),
        },
        Err(e) => format!("`There was an error retrieving data: {e}`"),
    }
}

#[command]
#[aliases("forecast")]
pub async fn wx_forecast(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let args: Vec<&str> = args.message().split(' ').collect();

    for arg in args {
        match utils::check_zip_code(arg) {
            Ok(zip_code) => {
                let data = parse_forecast(zip_code).await;
                msg.channel_id.say(&ctx.http, data).await?
            }
            Err(e) => msg.channel_id.say(&ctx.http, format!("`{e}`")).await?,
        };
    }

    Ok(())
}

fn create_forecast_graph(city: &str, state: &str, label: &str, temps: &[i32]) -> String {
    fs::create_dir_all("./images").expect("Error creating ./images directory");

    let timestamp: DateTime<Utc> = Utc::now();
    let file_name = format!("./images/{}_forecast_graph.png", timestamp.format("%y_%m_%d_%H%M%S"));

    let min = temps.iter().min().unwrap();
    let max = temps.iter().max().unwrap();

    let (temp_highs, temp_lows) = if label.contains("Low") {
        let highs: Vec<i32> = temps.iter().skip(1).step_by(2).map(|x| x.to_owned()).collect();
        let lows: Vec<i32> = temps.iter().step_by(2).map(|x| x.to_owned()).collect();

        (highs, lows)
    } else {
        let highs: Vec<i32> = temps.iter().step_by(2).map(|x| x.to_owned()).collect();
        let lows: Vec<i32> = temps.iter().skip(1).step_by(2).map(|x| x.to_owned()).collect();

        (highs, lows)
    };

    let root_area = BitMapBackend::new(&file_name, (1024, 768)).into_drawing_area();

    root_area.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root_area)
        .margin(30)
        .set_label_area_size(LabelAreaPosition::Left, 64)
        .set_label_area_size(LabelAreaPosition::Bottom, 64)
        .caption(format!("Forecasted Temperatures for {city}, {state}"), ("sans-serif", 36))
        .build_cartesian_2d(0..6, (min - 10)..(max + 10))
        .unwrap();

    chart
        .configure_mesh()
        .x_desc("Day")
        .y_desc("Temperature")
        .label_style(("sans-serif", 24))
        .draw()
        .unwrap();
    chart
        .draw_series(
            LineSeries::new(
                temp_highs.iter().enumerate().map(|(i, temp)| {
                    let x = i as i32;
                    let y = *temp;
                    (x, y)
                }),
                &RED,
            )
            .point_size(2),
        )
        .unwrap()
        .label("High")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));
    chart
        .draw_series(
            LineSeries::new(
                temp_lows.iter().enumerate().map(|(i, temp)| {
                    let x = i as i32;
                    let y = *temp;
                    (x, y)
                }),
                &BLUE,
            )
            .point_size(2),
        )
        .unwrap()
        .label("Low")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));
    chart
        .configure_series_labels()
        .label_font(("sans-serif", 18))
        .position(SeriesLabelPosition::MiddleRight)
        .border_style(BLACK)
        .background_style(WHITE.mix(0.8))
        .draw()
        .unwrap();

    file_name.to_string()
}

#[command]
#[aliases("graph")]
pub async fn wx_graph(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let args: Vec<&str> = args.message().split(' ').collect();

    for arg in args {
        match utils::check_zip_code(arg) {
            Ok(zip_code) => match uv::fetch_location(zip_code).await {
                Ok((city, state, lat, lon)) => match fetch_forecast(lat, lon).await {
                    Ok(data) => {
                        let temps: Vec<i32> = data
                            .data
                            .temperature
                            .iter()
                            .map(|x| x.parse::<i32>().unwrap())
                            .collect();
                        let file_name =
                            create_forecast_graph(&city, &state, &data.time.tempLabel[0], &temps);
                        let file = match File::open(file_name).await {
                            Ok(f) => f,
                            Err(e) => {
                                msg.channel_id.say(&ctx.http, format!("`{e}`")).await?;
                                return Ok(());
                            }
                        };
                        let file = vec![(&file, "forecast_graph.png")];

                        msg.channel_id.send_files(&ctx.http, file, |m| m.content("")).await?
                    }
                    Err(e) => msg.channel_id.say(&ctx.http, format!("`{e}`")).await?,
                },
                Err(e) => msg.channel_id.say(&ctx.http, format!("`{e}`")).await?,
            },
            Err(e) => msg.channel_id.say(&ctx.http, format!("`{e}`")).await?,
        };
    }

    Ok(())
}
